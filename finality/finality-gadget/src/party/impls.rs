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

use std::{marker::PhantomData, sync::Arc};

use sc_client_api::Backend;
use sp_runtime::traits::{Block as BlockT, NumberFor, SaturatedConversion};

use crate::{
    party::traits::{Block, ChainState, SessionInfo},
    ClientForStance, SessionId, SessionPeriod,
};

pub struct ChainStateImpl<B, BE, CFA>
where
    B: BlockT,
    BE: Backend<B>,
    CFA: ClientForStance<B, BE>,
{
    pub client: Arc<CFA>,
    pub _phantom: PhantomData<(B, BE)>,
}

impl<B, BE, CFA> ChainState<B> for ChainStateImpl<B, BE, CFA>
where
    B: BlockT,
    BE: Backend<B>,
    CFA: ClientForStance<B, BE>,
{
    fn best_block_number(&self) -> <B as Block>::Number {
        self.client.info().best_number
    }
    fn finalized_number(&self) -> <B as Block>::Number {
        self.client.info().finalized_number
    }
}

pub struct SessionInfoImpl {
    session_period: SessionPeriod,
}

impl SessionInfoImpl {
    pub fn new(session_period: SessionPeriod) -> Self {
        Self { session_period }
    }
}

impl<B: BlockT> SessionInfo<B> for SessionInfoImpl {
    fn session_id_from_block_num(&self, n: NumberFor<B>) -> SessionId {
        SessionId(n.saturated_into::<u32>() / self.session_period.0)
    }

    fn last_block_of_session(&self, session_id: SessionId) -> NumberFor<B> {
        ((session_id.0 + 1) * self.session_period.0 - 1).into()
    }

    fn first_block_of_session(&self, session_id: SessionId) -> NumberFor<B> {
        (session_id.0 * self.session_period.0).into()
    }
}
