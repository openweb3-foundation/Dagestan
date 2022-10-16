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

//! Implements the Stance BFT Consensus protocol as a "finality gadget". The [Member] struct
//! requires access to a network layer, a cryptographic primitive, and a data provider that
//! gives appropriate access to the set of available data that we need to make consensus on.

mod alerts;
mod config;
mod consensus;
mod creation;
mod extender;
mod member;
mod network;
mod runway;
mod terminal;
mod terminator;
mod units;

mod task_queue;
#[cfg(test)]
mod testing;

pub use stance_bft_types::{
    Data, DataProvider, FinalizationHandler, Hasher, IncompleteMultisignatureError, Index, Indexed,
    Keychain, MultiKeychain, Multisigned, Network, NodeCount, NodeIndex, NodeMap, NodeSubset,
    PartialMultisignature, PartiallyMultisigned, Recipient, Round, SessionId, Signable, Signature,
    SignatureError, SignatureSet, Signed, SpawnHandle, TaskHandle, UncheckedSigned,
};
pub use config::{default_config, exponential_slowdown, Config, DelayConfig};
pub use member::{run_session, LocalIO};
pub use network::NetworkData;
pub use terminator::{handle_task_termination, Terminator};

type Receiver<T> = futures::channel::mpsc::UnboundedReceiver<T>;
type Sender<T> = futures::channel::mpsc::UnboundedSender<T>;
