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

use codec::{Decode, Encode};

use crate::{
    crypto::Signature,
    network::{Data, Multiaddress},
    NodeIndex, SessionId,
};

mod connections;
mod discovery;
mod service;
mod session;

use connections::Connections;
pub use discovery::{Discovery, DiscoveryMessage};
pub use service::{
    Config as ConnectionManagerConfig, Service as ConnectionManager, SessionCommand,
    IO as ConnectionIO,
};
pub use session::{Handler as SessionHandler, HandlerError as SessionHandlerError};

/// Data validators use to authenticate themselves for a single session
/// and disseminate their addresses.
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
pub struct AuthData<M: Multiaddress> {
    addresses: Vec<M>,
    node_id: NodeIndex,
    session_id: SessionId,
}

impl<M: Multiaddress> AuthData<M> {
    pub fn session(&self) -> SessionId {
        self.session_id
    }

    pub fn creator(&self) -> NodeIndex {
        self.node_id
    }

    pub fn addresses(&self) -> Vec<M> {
        self.addresses.clone()
    }
}

/// A full authentication, consisting of a signed AuthData.
pub type Authentication<M> = (AuthData<M>, Signature);

/// The data that should be sent to the network service.
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
pub enum NetworkData<D: Data, M: Multiaddress> {
    Meta(DiscoveryMessage<M>),
    Data(D, SessionId),
}
