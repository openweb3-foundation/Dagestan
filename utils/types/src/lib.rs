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

//! Traits that need to be implemented by the user.

mod dataio;
mod network;
mod tasks;

pub use stance_crypto::{
    IncompleteMultisignatureError, Index, Indexed, Keychain, MultiKeychain, Multisigned, NodeCount,
    NodeIndex, NodeMap, NodeSubset, PartialMultisignature, PartiallyMultisigned, Signable,
    Signature, SignatureError, SignatureSet, Signed, UncheckedSigned,
};
pub use dataio::{DataProvider, FinalizationHandler};
pub use network::{Network, Recipient};
pub use tasks::{SpawnHandle, TaskHandle};

use codec::Codec;
use std::{fmt::Debug, hash::Hash as StdHash};

/// Data type that we want to order.
pub trait Data: Eq + Clone + Send + Sync + Debug + StdHash + Codec + 'static {}

impl<T> Data for T where T: Eq + Clone + Send + Sync + Debug + StdHash + Codec + 'static {}

/// A hasher, used for creating identifiers for blocks or units.
pub trait Hasher: Eq + Clone + Send + Sync + Debug + 'static {
    /// A hash, as an identifier for a block or unit.
    type Hash: AsRef<[u8]> + Eq + Ord + Copy + Clone + Send + Sync + Debug + StdHash + Codec;

    fn hash(s: &[u8]) -> Self::Hash;
}

/// The number of a session for which the consensus is run.
pub type SessionId = u64;

/// An asynchronous round of the protocol.
pub type Round = u16;
