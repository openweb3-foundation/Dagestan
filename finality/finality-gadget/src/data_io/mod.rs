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

use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
};

use codec::{Decode, Encode};
use sp_runtime::traits::Block as BlockT;

mod chain_info;
mod data_interpreter;
mod data_provider;
mod data_store;
mod proposal;
mod status_provider;

pub use chain_info::ChainInfoProvider;
pub use data_interpreter::OrderedDataInterpreter;
pub use data_provider::ChainTracker;
pub use data_store::{DataStore, DataStoreConfig};
pub use proposal::UnvalidatedStanceProposal;

// Maximum number of blocks above the last finalized allowed in an Stance proposal.
pub const MAX_DATA_BRANCH_LEN: usize = 7;

/// The data ordered by the Stance consensus.
#[derive(Clone, Debug, Encode, Decode)]
pub struct StanceData<B: BlockT> {
    pub head_proposal: UnvalidatedStanceProposal<B>,
}

// Need to be implemented manually, as deriving does not work (`BlockT` is not `Hash`).
impl<B: BlockT> Hash for StanceData<B> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.head_proposal.hash(state);
    }
}

// Clippy does not allow deriving PartialEq when implementing Hash manually
impl<B: BlockT> PartialEq for StanceData<B> {
    fn eq(&self, other: &Self) -> bool {
        self.head_proposal.eq(&other.head_proposal)
    }
}

impl<B: BlockT> Eq for StanceData<B> {}

/// A trait allowing to check the data contained in an Stance network message, for the purpose of
/// data availability checks.
pub trait StanceNetworkMessage<B: BlockT>: Clone + Debug {
    fn included_data(&self) -> Vec<StanceData<B>>;
}

#[derive(Clone, Debug)]
pub struct ChainInfoCacheConfig {
    pub block_cache_capacity: usize,
}

impl Default for ChainInfoCacheConfig {
    fn default() -> ChainInfoCacheConfig {
        ChainInfoCacheConfig {
            block_cache_capacity: 2000,
        }
    }
}
