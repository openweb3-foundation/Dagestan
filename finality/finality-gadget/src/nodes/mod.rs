// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

// This file is part of DAGESTAN.

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

mod nonvalidator_node;
mod validator_node;

use std::{future::Future, sync::Arc};

use stance_primitives::{AuthorityId, SessionAuthorityData};
use codec::Encode;
use log::warn;
pub use nonvalidator_node::run_nonvalidator_node;
use sc_client_api::Backend;
use sc_network::{ExHashT, NetworkService};
use sp_runtime::{
    traits::{Block, Header, NumberFor},
    RuntimeAppPublic,
};
pub use validator_node::run_validator_node;

use crate::{
    crypto::AuthorityVerifier,
    finalization::StanceFinalizer,
    justification::{
        StanceJustification, JustificationHandler, JustificationRequestSchedulerImpl, SessionInfo,
        SessionInfoProvider, Verifier,
    },
    last_block_of_session, mpsc,
    mpsc::UnboundedSender,
    session_id_from_block_num,
    session_map::ReadOnlySessionMap,
    JustificationNotification, Metrics, MillisecsPerBlock, SessionPeriod,
};

/// Max amount of tries we can not update a finalized block number before we will clear requests queue
const MAX_ATTEMPTS: u32 = 5;

struct JustificationVerifier {
    authority_verifier: AuthorityVerifier,
    emergency_signer: Option<AuthorityId>,
}

impl From<SessionAuthorityData> for JustificationVerifier {
    fn from(authority_data: SessionAuthorityData) -> Self {
        JustificationVerifier {
            authority_verifier: AuthorityVerifier::new(authority_data.authorities().to_vec()),
            emergency_signer: authority_data.emergency_finalizer().clone(),
        }
    }
}

impl<B: Block> Verifier<B> for JustificationVerifier {
    fn verify(&self, justification: &StanceJustification, hash: B::Hash) -> bool {
        use StanceJustification::*;
        let encoded_hash = hash.encode();
        match justification {
            CommitteeMultisignature(multisignature) => match self
                .authority_verifier
                .is_complete(&encoded_hash, multisignature)
            {
                true => true,
                false => {
                    warn!(target: "stance-justification", "Bad multisignature for block hash #{:?} {:?}", hash, multisignature);
                    false
                }
            },
            EmergencySignature(signature) => match &self.emergency_signer {
                Some(emergency_signer) => match emergency_signer.verify(&encoded_hash, signature) {
                    true => true,
                    false => {
                        warn!(target: "stance-justification", "Bad emergency signature for block hash #{:?} {:?}", hash, signature);
                        false
                    }
                },
                None => {
                    warn!(target: "stance-justification", "Received emergency signature for block with hash #{:?}, which has no emergency signer defined.", hash);
                    false
                }
            },
        }
    }
}

struct JustificationParams<B: Block, H: ExHashT, C> {
    pub network: Arc<NetworkService<B, H>>,
    pub client: Arc<C>,
    pub justification_rx: mpsc::UnboundedReceiver<JustificationNotification<B>>,
    pub metrics: Option<Metrics<<B::Header as Header>::Hash>>,
    pub session_period: SessionPeriod,
    pub millisecs_per_block: MillisecsPerBlock,
    pub session_map: ReadOnlySessionMap,
}

struct SessionInfoProviderImpl {
    session_authorities: ReadOnlySessionMap,
    session_period: SessionPeriod,
}

impl SessionInfoProviderImpl {
    fn new(session_authorities: ReadOnlySessionMap, session_period: SessionPeriod) -> Self {
        Self {
            session_authorities,
            session_period,
        }
    }
}

#[async_trait::async_trait]
impl<B: Block> SessionInfoProvider<B, JustificationVerifier> for SessionInfoProviderImpl {
    async fn for_block_num(&self, number: NumberFor<B>) -> SessionInfo<B, JustificationVerifier> {
        let current_session = session_id_from_block_num::<B>(number, self.session_period);
        let last_block_height = last_block_of_session::<B>(current_session, self.session_period);
        let verifier = self
            .session_authorities
            .get(current_session)
            .await
            .map(|authority_data| authority_data.into());

        SessionInfo {
            current_session,
            last_block_height,
            verifier,
        }
    }
}

fn setup_justification_handler<B, H, C, BE>(
    just_params: JustificationParams<B, H, C>,
) -> (
    UnboundedSender<JustificationNotification<B>>,
    impl Future<Output = ()>,
)
where
    B: Block,
    H: ExHashT,
    C: crate::ClientForStance<B, BE> + Send + Sync + 'static,
    C::Api: stance_primitives::StanceSessionApi<B>,
    BE: Backend<B> + 'static,
{
    let JustificationParams {
        network,
        client,
        justification_rx,
        metrics,
        session_period,
        millisecs_per_block,
        session_map,
    } = just_params;

    let handler = JustificationHandler::new(
        SessionInfoProviderImpl::new(session_map, session_period),
        network,
        client.clone(),
        StanceFinalizer::new(client),
        JustificationRequestSchedulerImpl::new(&session_period, &millisecs_per_block, MAX_ATTEMPTS),
        metrics,
        Default::default(),
    );

    let (authority_justification_tx, authority_justification_rx) = mpsc::unbounded();
    (authority_justification_tx, async move {
        handler
            .run(authority_justification_rx, justification_rx)
            .await;
    })
}
