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

use core::result::Result;
use std::{marker::PhantomData, sync::Arc};

use log::{debug, warn};
use sc_client_api::{Backend, Finalizer, HeaderBackend, LockImportRun};
use sp_api::{BlockId, NumberFor};
use sp_blockchain::Error;
use sp_runtime::{traits::Block, Justification};

pub trait BlockFinalizer<B: Block> {
    fn finalize_block(
        &self,
        hash: B::Hash,
        block_number: NumberFor<B>,
        justification: Option<Justification>,
    ) -> Result<(), Error>;
}

pub struct StanceFinalizer<B, BE, C>
where
    B: Block,
    BE: Backend<B>,
    C: HeaderBackend<B> + LockImportRun<B, BE> + Finalizer<B, BE>,
{
    client: Arc<C>,
    phantom: PhantomData<(B, BE)>,
}

impl<B, BE, C> StanceFinalizer<B, BE, C>
where
    B: Block,
    BE: Backend<B>,
    C: HeaderBackend<B> + LockImportRun<B, BE> + Finalizer<B, BE>,
{
    pub(crate) fn new(client: Arc<C>) -> Self {
        StanceFinalizer {
            client,
            phantom: PhantomData,
        }
    }
}

impl<B, BE, C> BlockFinalizer<B> for StanceFinalizer<B, BE, C>
where
    B: Block,
    BE: Backend<B>,
    C: HeaderBackend<B> + LockImportRun<B, BE> + Finalizer<B, BE>,
{
    fn finalize_block(
        &self,
        hash: B::Hash,
        block_number: NumberFor<B>,
        justification: Option<Justification>,
    ) -> Result<(), Error> {
        let status = self.client.info();
        if status.finalized_number >= block_number {
            warn!(target: "stance-finality", "trying to finalize a block with hash {} and number {}
               that is not greater than already finalized {}", hash, block_number, status.finalized_number);
        }

        debug!(target: "stance-finality", "Finalizing block with hash {:?} and number {:?}. Previous best: #{:?}.", hash, block_number, status.finalized_number);

        let update_res = self.client.lock_import_and_run(|import_op| {
            // NOTE: all other finalization logic should come here, inside the lock
            self.client
                .apply_finality(import_op, BlockId::Hash(hash), justification, true)
        });
        let status = self.client.info();
        debug!(target: "stance-finality", "Attempted to finalize block with hash {:?}. Current best: #{:?}.", hash, status.finalized_number);
        update_res
    }
}
