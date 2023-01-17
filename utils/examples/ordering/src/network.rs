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

use crate::Data;
use stance_consensus::{NodeIndex, Recipient};
use stance_consensus_mock::{Hasher64, PartialMultisignature, Signature};
use codec::{Decode, Encode};
use log::error;
use std::net::SocketAddr;
use tokio::{
    io,
    net::UdpSocket,
    time::{sleep, Duration},
};

const MAX_UDP_DATAGRAM_BYTES: usize = 65536;

pub type NetworkData = stance_consensus::NetworkData<Hasher64, Data, Signature, PartialMultisignature>;

#[derive(Debug)]
pub struct Network {
    my_id: usize,
    addresses: Vec<SocketAddr>,
    socket: UdpSocket,
    /// Buffer for incoming data.
    ///
    /// It's allocated on the heap, because otherwise it overflows the stack when used inside a future.
    buffer: Box<[u8; MAX_UDP_DATAGRAM_BYTES]>,
}

impl Network {
    pub async fn new(
        my_id: NodeIndex,
        ports: &[usize],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let my_id = my_id.0;
        assert!(my_id < ports.len());

        let addresses = ports
            .iter()
            .map(|p| format!("127.0.0.1:{}", p).parse::<SocketAddr>())
            .collect::<Result<Vec<_>, _>>()?;

        let socket = Self::bind_socket(addresses[my_id]).await;
        Ok(Network {
            my_id,
            addresses,
            socket,
            buffer: Box::new([0; MAX_UDP_DATAGRAM_BYTES]),
        })
    }

    async fn bind_socket(address: SocketAddr) -> UdpSocket {
        loop {
            match UdpSocket::bind(address).await {
                Ok(socket) => {
                    return socket;
                }
                Err(e) => {
                    error!("{}", e);
                    error!("Waiting 10 seconds before the next attempt...");
                    sleep(Duration::from_secs(10)).await;
                }
            };
        }
    }

    fn send_to_peer(&self, data: NetworkData, recipient: usize) {
        if let Err(e) = self.try_send_to_peer(data, recipient) {
            error!("Sending failed, recipient: {:?}, error: {:?}", recipient, e);
        }
    }

    fn try_send_to_peer(&self, data: NetworkData, recipient: usize) -> io::Result<()> {
        let encoded = data.encode();
        assert!(encoded.len() <= MAX_UDP_DATAGRAM_BYTES);

        self.socket
            .try_send_to(&encoded, self.addresses[recipient])?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl stance_consensus::Network<NetworkData> for Network {
    fn send(&self, data: NetworkData, recipient: Recipient) {
        match recipient {
            Recipient::Everyone => {
                for r in 0..self.addresses.len() {
                    if r != self.my_id {
                        self.send_to_peer(data.clone(), r);
                    }
                }
            }
            Recipient::Node(r) => {
                if r.0 < self.addresses.len() {
                    self.send_to_peer(data, r.0);
                } else {
                    error!("Recipient unknown: {}", r.0);
                }
            }
        }
    }

    async fn next_event(&mut self) -> Option<NetworkData> {
        match self.socket.recv_from(self.buffer.as_mut()).await {
            Ok((_len, _addr)) => NetworkData::decode(&mut &self.buffer[..]).ok(),
            Err(e) => {
                error!("Couldn't receive datagram: {:?}", e);
                None
            }
        }
    }
}
