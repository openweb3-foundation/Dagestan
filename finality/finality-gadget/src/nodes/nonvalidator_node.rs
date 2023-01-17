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

use log::{debug, error};
use sc_client_api::Backend;
use sc_network::ExHashT;
use sp_consensus::SelectChain;
use sp_runtime::traits::Block;

use crate::{
    nodes::{setup_justification_handler, JustificationParams},
    session_map::{AuthorityProviderImpl, FinalityNotificatorImpl, SessionMapUpdater},
    StanceConfig,
};

pub async fn run_nonvalidator_node<B, H, C, BE, SC>(stance_config: StanceConfig<B, H, C, SC>)
where
    B: Block,
    H: ExHashT,
    C: crate::ClientForStance<B, BE> + Send + Sync + 'static,
    C::Api: stance_primitives::StanceSessionApi<B>,
    BE: Backend<B> + 'static,
    SC: SelectChain<B> + 'static,
{
    let StanceConfig {
        network,
        client,
        metrics,
        session_period,
        millisecs_per_block,
        justification_rx,
        spawn_handle,
        ..
    } = stance_config;
    let map_updater = SessionMapUpdater::<_, _, B>::new(
        AuthorityProviderImpl::new(client.clone()),
        FinalityNotificatorImpl::new(client.clone()),
    );
    let session_authorities = map_updater.readonly_session_map();
    spawn_handle.spawn("stance/updater", None, async move {
        debug!(target: "stance-party", "SessionMapUpdater has started.");
        map_updater.run(session_period).await
    });
    let (_, handler_task) = setup_justification_handler(JustificationParams {
        justification_rx,
        network,
        client,
        metrics,
        session_period,
        millisecs_per_block,
        session_map: session_authorities,
    });

    debug!(target: "stance-party", "JustificationHandler has started.");
    handler_task.await;
    error!(target: "stance-party", "JustificationHandler finished.");
}
