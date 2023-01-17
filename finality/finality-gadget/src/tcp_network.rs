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

use std::{io::Result as IoResult, net::ToSocketAddrs as _};

use stance_primitives::AuthorityId;
use codec::{Decode, Encode};
use log::info;
use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpListener, TcpStream, ToSocketAddrs,
};

use crate::{
    network::{Multiaddress, NetworkIdentity, PeerId},
    validator_network::{Dialer, Listener, Splittable},
};

impl Splittable for TcpStream {
    type Sender = OwnedWriteHalf;
    type Receiver = OwnedReadHalf;

    fn split(self) -> (Self::Sender, Self::Receiver) {
        let (receiver, sender) = self.into_split();
        (sender, receiver)
    }
}

#[async_trait::async_trait]
impl Listener for TcpListener {
    type Connection = TcpStream;
    type Error = std::io::Error;

    async fn accept(&mut self) -> Result<Self::Connection, Self::Error> {
        let stream = TcpListener::accept(self).await.map(|(stream, _)| stream)?;
        if stream.set_linger(None).is_err() {
            info!(target: "validator-network", "stream.set_linger(None) failed.");
        };
        Ok(stream)
    }
}

impl PeerId for AuthorityId {}

/// A representation of a single TCP address with an associated peer ID.
#[derive(Debug, Hash, Encode, Decode, Clone, PartialEq, Eq)]
pub struct TcpMultiaddress {
    peer_id: AuthorityId,
    address: String,
}

impl Multiaddress for TcpMultiaddress {
    type PeerId = AuthorityId;

    fn get_peer_id(&self) -> Option<Self::PeerId> {
        Some(self.peer_id.clone())
    }

    fn add_matching_peer_id(self, peer_id: Self::PeerId) -> Option<Self> {
        match self.peer_id == peer_id {
            true => Some(self),
            false => None,
        }
    }
}

#[derive(Clone)]
struct TcpDialer;

#[async_trait::async_trait]
impl Dialer<TcpMultiaddress> for TcpDialer {
    type Connection = TcpStream;
    type Error = std::io::Error;

    async fn connect(
        &mut self,
        addresses: Vec<TcpMultiaddress>,
    ) -> Result<Self::Connection, Self::Error> {
        let parsed_addresses: Vec<_> = addresses
            .into_iter()
            .filter_map(|address| address.address.to_socket_addrs().ok())
            .flatten()
            .collect();
        let stream = TcpStream::connect(&parsed_addresses[..]).await?;
        if stream.set_linger(None).is_err() {
            info!(target: "validator-network", "stream.set_linger(None) failed.");
        };
        Ok(stream)
    }
}

struct TcpNetworkIdentity {
    peer_id: AuthorityId,
    addresses: Vec<TcpMultiaddress>,
}

impl NetworkIdentity for TcpNetworkIdentity {
    type PeerId = AuthorityId;
    type Multiaddress = TcpMultiaddress;

    fn identity(&self) -> (Vec<Self::Multiaddress>, Self::PeerId) {
        (self.addresses.clone(), self.peer_id.clone())
    }
}

/// Create a new tcp network, including an identity that can be used for constructing
/// authentications for other peers.
#[allow(dead_code)]
pub async fn new_tcp_network<A: ToSocketAddrs>(
    listening_addresses: A,
    external_addresses: Vec<String>,
    peer_id: AuthorityId,
) -> IoResult<(
    impl Dialer<TcpMultiaddress>,
    impl Listener,
    impl NetworkIdentity,
)> {
    let listener = TcpListener::bind(listening_addresses).await?;
    let identity = TcpNetworkIdentity {
        addresses: external_addresses
            .into_iter()
            .map(|address| TcpMultiaddress {
                peer_id: peer_id.clone(),
                address,
            })
            .collect(),
        peer_id,
    };
    Ok((TcpDialer {}, listener, identity))
}
