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

use codec::{Decode, Encode};
use sp_runtime::{traits::Block, SaturatedConversion};

use crate::NumberFor;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct SessionBoundaries<B: Block> {
    first_block: NumberFor<B>,
    last_block: NumberFor<B>,
}

impl<B: Block> SessionBoundaries<B> {
    pub fn new(session_id: SessionId, period: SessionPeriod) -> Self {
        SessionBoundaries {
            first_block: first_block_of_session::<B>(session_id, period),
            last_block: last_block_of_session::<B>(session_id, period),
        }
    }

    pub fn first_block(&self) -> NumberFor<B> {
        self.first_block
    }

    pub fn last_block(&self) -> NumberFor<B> {
        self.last_block
    }
}

pub fn first_block_of_session<B: Block>(
    session_id: SessionId,
    period: SessionPeriod,
) -> NumberFor<B> {
    (session_id.0 * period.0).into()
}

pub fn last_block_of_session<B: Block>(
    session_id: SessionId,
    period: SessionPeriod,
) -> NumberFor<B> {
    ((session_id.0 + 1) * period.0 - 1).into()
}

pub fn session_id_from_block_num<B: Block>(num: NumberFor<B>, period: SessionPeriod) -> SessionId {
    SessionId(num.saturated_into::<u32>() / period.0)
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Encode, Decode)]
pub struct SessionId(pub u32);

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Encode, Decode)]
pub struct SessionPeriod(pub u32);
