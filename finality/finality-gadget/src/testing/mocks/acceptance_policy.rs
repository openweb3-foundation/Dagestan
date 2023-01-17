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

use std::{cell::RefCell, collections::VecDeque};

#[derive(Clone, Debug)]
pub(crate) enum AcceptancePolicy {
    Unavailable,
    AlwaysAccept,
    AlwaysReject,
    FromSequence(RefCell<VecDeque<bool>>),
}

impl AcceptancePolicy {
    pub(crate) fn accepts(&self) -> bool {
        use AcceptancePolicy::*;

        match &self {
            Unavailable => panic!("Policy is unavailable!"),
            AlwaysAccept => true,
            AlwaysReject => false,
            FromSequence(seq) => seq
                .borrow_mut()
                .pop_front()
                .expect("Not enough values provided!"),
        }
    }
}
