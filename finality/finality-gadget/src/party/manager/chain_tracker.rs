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

use stance::SpawnHandle;
use futures::channel::oneshot;
use log::debug;
use sc_client_api::HeaderBackend;
use sp_consensus::SelectChain;
use sp_runtime::traits::Block;

use crate::{
    data_io::ChainTracker,
    party::{AuthoritySubtaskCommon, Task},
};

/// Runs the latest block refresher within a single session.
pub fn task<B, SC, C>(
    subtask_common: AuthoritySubtaskCommon,
    chain_tracker: ChainTracker<B, SC, C>,
) -> Task
where
    B: Block,
    C: HeaderBackend<B> + 'static,
    SC: SelectChain<B> + 'static,
{
    let AuthoritySubtaskCommon {
        spawn_handle,
        session_id,
    } = subtask_common;
    let (stop, exit) = oneshot::channel();
    let task = async move {
        debug!(target: "stance-party", "Running the chain refresh task for {:?}", session_id);
        chain_tracker.run(exit).await;
        debug!(target: "stance-party", "Chain refresh task stopped for {:?}", session_id);
    };

    let handle = spawn_handle.spawn_essential("stance/consensus_session_refresher", task);
    Task::new(handle, stop)
}
