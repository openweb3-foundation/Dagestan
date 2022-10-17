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

use stance_types::{Index, NodeIndex, SignatureSet};
use codec::{Decode, Encode};
use std::hash::Hash;

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Encode, Decode)]
pub struct Signature {
    msg: Vec<u8>,
    index: NodeIndex,
}

impl Signature {
    pub fn new(msg: Vec<u8>, index: NodeIndex) -> Self {
        Self { msg, index }
    }

    pub fn msg(&self) -> &Vec<u8> {
        &self.msg
    }
}

impl Index for Signature {
    fn index(&self) -> NodeIndex {
        self.index
    }
}

pub type PartialMultisignature = SignatureSet<Signature>;
