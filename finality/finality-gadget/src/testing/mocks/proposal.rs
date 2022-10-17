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

use sp_runtime::traits::Block as BlockT;
use substrate_test_runtime_client::runtime::{Block, Header};

use crate::data_io::{StanceData, UnvalidatedStanceProposal};

pub fn unvalidated_proposal_from_headers(headers: Vec<Header>) -> UnvalidatedStanceProposal<Block> {
    let num = headers.last().unwrap().number;
    let hashes = headers.into_iter().map(|header| header.hash()).collect();
    UnvalidatedStanceProposal::new(hashes, num)
}

pub fn stance_data_from_blocks(blocks: Vec<Block>) -> StanceData<Block> {
    let headers = blocks.into_iter().map(|b| b.header().clone()).collect();
    stance_data_from_headers(headers)
}

pub fn stance_data_from_headers(headers: Vec<Header>) -> StanceData<Block> {
    StanceData {
        head_proposal: unvalidated_proposal_from_headers(headers),
    }
}
