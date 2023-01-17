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

use std::{cmp::Ordering, fmt::Debug, hash::Hash as StdHash, marker::PhantomData};

use stance::Hasher;
use codec::{Decode, Encode};
use sp_runtime::traits::Hash;

#[derive(Debug, PartialEq, Eq, Clone, Copy, StdHash, Encode, Decode)]
pub struct OrdForHash<O: Eq + Copy + Clone + Send + Debug + StdHash + Encode + Decode + AsRef<[u8]>>
{
    inner: O,
}

impl<O: Eq + Copy + Clone + Send + Sync + Debug + StdHash + Encode + Decode + AsRef<[u8]>>
    PartialOrd for OrdForHash<O>
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<O: Eq + Copy + Clone + Send + Sync + Debug + StdHash + Encode + Decode + AsRef<[u8]>> Ord
    for OrdForHash<O>
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.as_ref().cmp(other.inner.as_ref())
    }
}

impl<O: Eq + Copy + Clone + Send + Sync + Debug + StdHash + Encode + Decode + AsRef<[u8]>>
    AsRef<[u8]> for OrdForHash<O>
{
    fn as_ref(&self) -> &[u8] {
        self.inner.as_ref()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Wrapper<H: Hash> {
    phantom: PhantomData<H>,
}

impl<H: Hash> Hasher for Wrapper<H> {
    type Hash = OrdForHash<H::Output>;

    fn hash(s: &[u8]) -> Self::Hash {
        Self::Hash {
            inner: <H as Hash>::hash(s),
        }
    }
}
