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

use sp_blockchain::Error;
use sp_runtime::{traits::Block, Justification};

use crate::{
    finalization::BlockFinalizer,
    testing::mocks::{single_action_mock::SingleActionMock, TBlock, THash, TNumber},
};

type CallArgs = (THash, TNumber, Option<Justification>);

#[derive(Clone)]
pub(crate) struct MockedBlockFinalizer {
    mock: SingleActionMock<CallArgs>,
}

impl MockedBlockFinalizer {
    pub(crate) fn new() -> Self {
        Self {
            mock: Default::default(),
        }
    }

    pub(crate) async fn has_not_been_invoked(&self) -> bool {
        self.mock.has_not_been_invoked().await
    }

    pub(crate) async fn has_been_invoked_with(&self, block: TBlock) -> bool {
        self.mock
            .has_been_invoked_with(|(hash, number, _)| {
                block.hash() == hash && block.header.number == number
            })
            .await
    }
}

impl BlockFinalizer<TBlock> for MockedBlockFinalizer {
    fn finalize_block(
        &self,
        hash: THash,
        block_number: TNumber,
        justification: Option<Justification>,
    ) -> Result<(), Error> {
        self.mock.invoke_with((hash, block_number, justification));
        Ok(())
    }
}
