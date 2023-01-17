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

//! A set of abstractions for dealing with `ReliableMulticast` in a more testable
//! and modular way.
//!
//! We expose the `Multicast` trait, mimicking the interface of `stance::ReliableMulticast`

use std::{
    fmt::{Debug, Display},
    hash::Hash as StdHash,
};

use stance_rmc::{MultiKeychain, ReliableMulticast, Signable, Signature};
use codec::{Codec, Decode, Encode};

/// A convenience trait for gathering all of the desired hash characteristics.
pub trait Hash: AsRef<[u8]> + StdHash + Eq + Clone + Codec + Debug + Display + Send + Sync {}

impl<T: AsRef<[u8]> + StdHash + Eq + Clone + Codec + Debug + Display + Send + Sync> Hash for T {}

/// A wrapper allowing block hashes to be signed.
#[derive(PartialEq, Eq, StdHash, Clone, Debug, Default, Encode, Decode)]
pub struct SignableHash<H: Hash> {
    hash: H,
}

impl<H: Hash> SignableHash<H> {
    pub fn new(hash: H) -> Self {
        Self { hash }
    }

    pub fn get_hash(&self) -> H {
        self.hash.clone()
    }
}

impl<H: Hash> Signable for SignableHash<H> {
    type Hash = H;
    fn hash(&self) -> Self::Hash {
        self.hash.clone()
    }
}

/// Anything that exposes the same interface as `stance::ReliableMulticast`.
///
/// The trait defines an associated type: `Signed`. For `ReliableMulticast`, this is simply
/// `stance::Multisigned` but the mocks are free to use the simplest matching implementation.
#[async_trait::async_trait]
pub trait Multicast<H: Hash, PMS>: Send + Sync {
    async fn start_multicast(&mut self, signable: SignableHash<H>);
    async fn next_signed_pair(&mut self) -> (H, PMS);
}

#[async_trait::async_trait]
impl<'a, H: Hash, MK: MultiKeychain<PartialMultisignature = SS>, SS: Signature + Send + Sync>
    Multicast<H, SS> for ReliableMulticast<'a, SignableHash<H>, MK>
{
    async fn start_multicast(&mut self, hash: SignableHash<H>) {
        self.start_rmc(hash).await;
    }

    async fn next_signed_pair(&mut self) -> (H, SS) {
        let ms = self.next_multisigned_hash().await.into_unchecked();
        (ms.as_signable().get_hash(), ms.signature())
    }
}
