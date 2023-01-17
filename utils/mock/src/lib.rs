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

//! Mock implementations of required traits. Do NOT use outside of testing!

mod crypto;
mod dataio;
mod hasher;
mod network;
mod spawner;

pub use crypto::{BadSigning, Keychain, PartialMultisignature, Signable, Signature};
pub use dataio::{Data, DataProvider, FinalizationHandler, Loader, Saver, StalledDataProvider};
pub use hasher::{Hash64, Hasher64};
pub use network::{
    Network, NetworkHook, NetworkReceiver, NetworkSender, Peer, ReconnectSender, Router,
};
pub use spawner::Spawner;
