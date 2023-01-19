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
pub use data_provider::{ChainTracker, DataProvider};
pub use data_store::{DataStore, DataStoreConfig};
pub use proposal::UnvalidatedDagestanProposal;

// Maximum number of blocks above the last finalized allowed in an AlephBFT proposal.
pub const MAX_DATA_BRANCH_LEN: usize = 7;

/// The data ordered by the Dagestan consensus.
#[derive(Clone, Debug, Encode, Decode)]
pub struct DagestanData<B: BlockT> {
    pub head_proposal: UnvalidatedDagestanProposal<B>,
}

// Need to be implemented manually, as deriving does not work (`BlockT` is not `Hash`).
impl<B: BlockT> Hash for DagestanData<B> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.head_proposal.hash(state);
    }
}

// Clippy does not allow deriving PartialEq when implementing Hash manually
impl<B: BlockT> PartialEq for DagestanData<B> {
    fn eq(&self, other: &Self) -> bool {
        self.head_proposal.eq(&other.head_proposal)
    }
}

impl<B: BlockT> Eq for DagestanData<B> {}

/// A trait allowing to check the data contained in an AlephBFT network message, for the purpose of
/// data availability checks.
pub trait DagestanNetworkMessage<B: BlockT>: Clone + Debug {
    fn included_data(&self) -> Vec<DagestanData<B>>;
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
