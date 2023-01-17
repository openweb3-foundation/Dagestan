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

use crate::crypto::{PartialMultisignature, Signature};
use stance_types::{
    Index, Keychain as KeychainT, MultiKeychain as MultiKeychainT, NodeCount, NodeIndex,
    PartialMultisignature as PartialMultisignatureT, SignatureSet,
};
use async_trait::async_trait;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Keychain {
    count: NodeCount,
    index: NodeIndex,
}

impl Keychain {
    pub fn new(count: NodeCount, index: NodeIndex) -> Self {
        Keychain { count, index }
    }

    pub fn new_vec(node_count: NodeCount) -> Vec<Self> {
        (0..node_count.0)
            .map(|i| Self::new(node_count, i.into()))
            .collect()
    }

    fn quorum(&self) -> usize {
        2 * self.count.0 / 3 + 1
    }
}

impl Index for Keychain {
    fn index(&self) -> NodeIndex {
        self.index
    }
}

#[async_trait]
impl KeychainT for Keychain {
    type Signature = Signature;

    fn node_count(&self) -> NodeCount {
        self.count
    }

    async fn sign(&self, msg: &[u8]) -> Self::Signature {
        Signature::new(msg.to_vec(), self.index)
    }

    fn verify(&self, msg: &[u8], sgn: &Self::Signature, index: NodeIndex) -> bool {
        index == sgn.index() && msg == sgn.msg()
    }
}

impl MultiKeychainT for Keychain {
    type PartialMultisignature = PartialMultisignature;

    fn bootstrap_multi(
        &self,
        signature: &Self::Signature,
        index: NodeIndex,
    ) -> Self::PartialMultisignature {
        SignatureSet::add_signature(SignatureSet::with_size(self.node_count()), signature, index)
    }

    fn is_complete(&self, msg: &[u8], partial: &Self::PartialMultisignature) -> bool {
        let signature_count = partial.iter().count();
        if signature_count < self.quorum() {
            return false;
        }
        partial.iter().all(|(i, sgn)| self.verify(msg, sgn, i))
    }
}
