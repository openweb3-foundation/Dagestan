// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

// This file is part of STANCE.

// Copyright (C) 2019-Present Setheum Labs.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use std::{collections::HashSet, fmt::Debug, marker::PhantomData, sync::Arc, time::Duration};

use stance::{DelayConfig, SpawnHandle};
use stance_primitives::KEY_TYPE;
use async_trait::async_trait;
use futures::channel::oneshot;
use log::{debug, trace, warn};
use sc_client_api::Backend;
use sp_consensus::SelectChain;
use sp_keystore::CryptoStore;
use sp_runtime::traits::{Block as BlockT, Header};

use crate::{
    crypto::{AuthorityPen, AuthorityVerifier, Keychain},
    data_io::{ChainTracker, DataStore, OrderedDataInterpreter},
    default_stance_config, mpsc,
    network::{split, ComponentNetworkMap, ManagerError, RequestBlocks, SessionManager},
    party::{backup::ABFTBackup, traits::NodeSessionManager},
    AuthorityId, JustificationNotification, Metrics, NodeIndex, SessionBoundaries, SessionId,
    SessionPeriod, UnitCreationDelay, VersionedNetworkData,
};

mod aggregator;
mod authority;
mod chain_tracker;
mod data_store;
mod member;
mod task;

pub use authority::{SubtaskCommon, Subtasks, Task as AuthorityTask};
pub use task::{Handle, Task};

pub struct NodeSessionManagerImpl<C, SC, B, RB, BE>
where
    B: BlockT,
    C: crate::ClientForStance<B, BE> + Send + Sync + 'static,
    BE: Backend<B> + 'static,
    SC: SelectChain<B> + 'static,
    RB: RequestBlocks<B>,
{
    client: Arc<C>,
    select_chain: SC,
    session_period: SessionPeriod,
    unit_creation_delay: UnitCreationDelay,
    authority_justification_tx: mpsc::UnboundedSender<JustificationNotification<B>>,
    block_requester: RB,
    metrics: Option<Metrics<<B::Header as Header>::Hash>>,
    spawn_handle: crate::SpawnHandle,
    session_manager: SessionManager<VersionedNetworkData<B>>,
    keystore: Arc<dyn CryptoStore>,
    _phantom: PhantomData<BE>,
}

impl<C, SC, B, RB, BE> NodeSessionManagerImpl<C, SC, B, RB, BE>
where
    B: BlockT,
    C: crate::ClientForStance<B, BE> + Send + Sync + 'static,
    BE: Backend<B> + 'static,
    SC: SelectChain<B> + 'static,
    RB: RequestBlocks<B>,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        client: Arc<C>,
        select_chain: SC,
        session_period: SessionPeriod,
        unit_creation_delay: UnitCreationDelay,
        authority_justification_tx: mpsc::UnboundedSender<JustificationNotification<B>>,
        block_requester: RB,
        metrics: Option<Metrics<<B::Header as Header>::Hash>>,
        spawn_handle: crate::SpawnHandle,
        session_manager: SessionManager<VersionedNetworkData<B>>,
        keystore: Arc<dyn CryptoStore>,
    ) -> Self {
        Self {
            client,
            select_chain,
            session_period,
            unit_creation_delay,
            authority_justification_tx,
            block_requester,
            metrics,
            spawn_handle,
            session_manager,
            keystore,
            _phantom: PhantomData,
        }
    }

    async fn spawn_subtasks(
        &self,
        session_id: SessionId,
        authorities: &[AuthorityId],
        node_id: NodeIndex,
        exit_rx: oneshot::Receiver<()>,
        backup: ABFTBackup,
    ) -> Subtasks {
        debug!(target: "afa", "Authority task {:?}", session_id);

        let authority_verifier = AuthorityVerifier::new(authorities.to_vec());
        let authority_pen =
            AuthorityPen::new(authorities[node_id.0].clone(), self.keystore.clone())
                .await
                .expect("The keys should sign successfully");
        let multikeychain =
            Keychain::new(node_id, authority_verifier.clone(), authority_pen.clone());

        let session_boundaries = SessionBoundaries::new(session_id, self.session_period);
        let (blocks_for_aggregator, blocks_from_interpreter) = mpsc::unbounded();

        let consensus_config = create_stance_config(
            authorities.len(),
            node_id,
            session_id,
            self.unit_creation_delay,
        );

        let (chain_tracker, data_provider) = ChainTracker::new(
            self.select_chain.clone(),
            self.client.clone(),
            session_boundaries.clone(),
            Default::default(),
            self.metrics.clone(),
        );

        let ordered_data_interpreter = OrderedDataInterpreter::<B, C>::new(
            blocks_for_aggregator,
            self.client.clone(),
            session_boundaries.clone(),
        );

        let subtask_common = SubtaskCommon {
            spawn_handle: self.spawn_handle.clone(),
            session_id: session_id.0,
        };
        let aggregator_io = aggregator::IO {
            blocks_from_interpreter,
            justifications_for_chain: self.authority_justification_tx.clone(),
        };

        let data_network = self
            .session_manager
            .start_validator_session(session_id, authority_verifier, node_id, authority_pen)
            .await
            .expect("Failed to start validator session!");

        let data_network = data_network.map();

        let (unfiltered_stance_network, rmc_network) =
            split(data_network, "stance_network", "rmc_network");
        let (data_store, stance_network) = DataStore::new(
            session_boundaries.clone(),
            self.client.clone(),
            self.block_requester.clone(),
            Default::default(),
            unfiltered_stance_network,
        );

        Subtasks::new(
            exit_rx,
            member::task(
                subtask_common.clone(),
                multikeychain.clone(),
                consensus_config,
                stance_network.into(),
                data_provider,
                ordered_data_interpreter,
                backup,
            ),
            aggregator::task(
                subtask_common.clone(),
                self.client.clone(),
                aggregator_io,
                session_boundaries,
                self.metrics.clone(),
                multikeychain,
                rmc_network,
            ),
            chain_tracker::task(subtask_common.clone(), chain_tracker),
            data_store::task(subtask_common, data_store),
        )
    }
}

#[derive(Debug)]
pub enum SessionManagerError {
    NotAuthority,
    ManagerError(ManagerError),
}

#[async_trait]
impl<C, SC, B, RB, BE> NodeSessionManager for NodeSessionManagerImpl<C, SC, B, RB, BE>
where
    B: BlockT,
    C: crate::ClientForStance<B, BE> + Send + Sync + 'static,
    C::Api: stance_primitives::StanceSessionApi<B>,
    BE: Backend<B> + 'static,
    SC: SelectChain<B> + 'static,
    RB: RequestBlocks<B>,
{
    type Error = SessionManagerError;

    async fn spawn_authority_task_for_session(
        &self,
        session: SessionId,
        node_id: NodeIndex,
        backup: ABFTBackup,
        authorities: &[AuthorityId],
    ) -> AuthorityTask {
        let (exit, exit_rx) = futures::channel::oneshot::channel();
        let subtasks = self
            .spawn_subtasks(session, authorities, node_id, exit_rx, backup)
            .await;

        AuthorityTask::new(
            self.spawn_handle
                .spawn_essential("stance/session_authority", async move {
                    if subtasks.wait_completion().await.is_err() {
                        warn!(target: "stance-party", "Authority subtasks failed.");
                    }
                }),
            node_id,
            exit,
        )
    }

    async fn early_start_validator_session(
        &self,
        session: SessionId,
        authorities: &[AuthorityId],
    ) -> Result<(), Self::Error> {
        let node_id = match self.node_idx(authorities).await {
            Some(id) => id,
            None => return Err(SessionManagerError::NotAuthority),
        };
        let authority_verifier = AuthorityVerifier::new(authorities.to_vec());
        let authority_pen =
            AuthorityPen::new(authorities[node_id.0].clone(), self.keystore.clone())
                .await
                .expect("The keys should sign successfully");
        self.session_manager
            .early_start_validator_session(session, authority_verifier, node_id, authority_pen)
            .map_err(SessionManagerError::ManagerError)
    }

    fn start_nonvalidator_session(
        &self,
        session: SessionId,
        authorities: &[AuthorityId],
    ) -> Result<(), Self::Error> {
        let authority_verifier = AuthorityVerifier::new(authorities.to_vec());

        self.session_manager
            .start_nonvalidator_session(session, authority_verifier)
            .map_err(SessionManagerError::ManagerError)
    }

    fn stop_session(&self, session: SessionId) -> Result<(), Self::Error> {
        self.session_manager
            .stop_session(session)
            .map_err(SessionManagerError::ManagerError)
    }

    async fn node_idx(&self, authorities: &[AuthorityId]) -> Option<NodeIndex> {
        let our_consensus_keys: HashSet<_> = self
            .keystore
            .keys(KEY_TYPE)
            .await
            .unwrap()
            .into_iter()
            .collect();
        trace!(target: "stance-data-store", "Found {:?} consensus keys in our local keystore {:?}", our_consensus_keys.len(), our_consensus_keys);
        authorities
            .iter()
            .position(|pkey| our_consensus_keys.contains(&pkey.into()))
            .map(|id| id.into())
    }
}

fn create_stance_config(
    n_members: usize,
    node_id: NodeIndex,
    session_id: SessionId,
    unit_creation_delay: UnitCreationDelay,
) -> stance::Config {
    let mut consensus_config = default_stance_config(n_members.into(), node_id, session_id.0 as u64);
    consensus_config.max_round = 7000;
    let unit_creation_delay = Arc::new(move |t| {
        if t == 0 {
            Duration::from_millis(2000)
        } else {
            exponential_slowdown(t, unit_creation_delay.0 as f64, 5000, 1.005)
        }
    });
    let delay_config = DelayConfig {
        tick_interval: Duration::from_millis(100),
        requests_interval: Duration::from_millis(3000),
        unit_rebroadcast_interval_min: Duration::from_millis(15000),
        unit_rebroadcast_interval_max: Duration::from_millis(20000),
        unit_creation_delay,
    };
    consensus_config.delay_config = delay_config;
    consensus_config
}

fn exponential_slowdown(
    t: usize,
    base_delay: f64,
    start_exp_delay: usize,
    exp_base: f64,
) -> Duration {
    // This gives:
    // base_delay, for t <= start_exp_delay,
    // base_delay * exp_base^(t - start_exp_delay), for t > start_exp_delay.
    let delay = if t < start_exp_delay {
        base_delay
    } else {
        let power = t - start_exp_delay;
        base_delay * exp_base.powf(power as f64)
    };
    let delay = delay.round() as u64;
    // the above will make it u64::MAX if it exceeds u64
    Duration::from_millis(delay)
}
