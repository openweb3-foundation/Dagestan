use sp_runtime::traits::Block as BlockT;
use substrate_test_runtime_client::runtime::{Block, Header};

use crate::data_io::{DagestanData, UnvalidatedDagestanProposal};

pub fn unvalidated_proposal_from_headers(headers: Vec<Header>) -> UnvalidatedDagestanProposal<Block> {
    let num = headers.last().unwrap().number;
    let hashes = headers.into_iter().map(|header| header.hash()).collect();
    UnvalidatedDagestanProposal::new(hashes, num)
}

pub fn dagestan_data_from_blocks(blocks: Vec<Block>) -> DagestanData<Block> {
    let headers = blocks.into_iter().map(|b| b.header().clone()).collect();
    dagestan_data_from_headers(headers)
}

pub fn dagestan_data_from_headers(headers: Vec<Header>) -> DagestanData<Block> {
    DagestanData {
        head_proposal: unvalidated_proposal_from_headers(headers),
    }
}
