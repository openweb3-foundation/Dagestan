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

use std::sync::{Arc, Mutex};

use crate::{
    justification::{StanceJustification, SessionInfo, SessionInfoProvider, Verifier},
    last_block_of_session, session_id_from_block_num,
    testing::mocks::{AcceptancePolicy, TBlock, THash, TNumber},
    SessionPeriod,
};

pub(crate) struct VerifierWrapper {
    acceptance_policy: Arc<Mutex<AcceptancePolicy>>,
}

impl Verifier<TBlock> for VerifierWrapper {
    fn verify(&self, _justification: &StanceJustification, _hash: THash) -> bool {
        self.acceptance_policy.lock().unwrap().accepts()
    }
}

pub(crate) struct SessionInfoProviderImpl {
    session_period: SessionPeriod,
    acceptance_policy: Arc<Mutex<AcceptancePolicy>>,
}

impl SessionInfoProviderImpl {
    pub(crate) fn new(session_period: SessionPeriod, acceptance_policy: AcceptancePolicy) -> Self {
        Self {
            session_period,
            acceptance_policy: Arc::new(Mutex::new(acceptance_policy)),
        }
    }
}

#[async_trait::async_trait]
impl SessionInfoProvider<TBlock, VerifierWrapper> for SessionInfoProviderImpl {
    async fn for_block_num(&self, number: TNumber) -> SessionInfo<TBlock, VerifierWrapper> {
        let current_session = session_id_from_block_num::<TBlock>(number, self.session_period);
        SessionInfo {
            current_session,
            last_block_height: last_block_of_session::<TBlock>(
                current_session,
                self.session_period,
            ),
            verifier: match &*self.acceptance_policy.lock().unwrap() {
                AcceptancePolicy::Unavailable => None,
                _ => Some(VerifierWrapper {
                    acceptance_policy: self.acceptance_policy.clone(),
                }),
            },
        }
    }
}
