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
use tokio::{
    io::{AsyncRead, AsyncWrite},
    time::{sleep, timeout, Duration},
};

use crate::validator_network::io::{receive_data, send_data};

const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_MISSED_HEARTBEATS: u32 = 4;

/// Represents the heartbeat message. Holds a single integer, so that it encodes into a nonempty
/// string of bytes.
#[derive(Debug, Clone, Encode, Decode)]
struct Heartbeat(u32);

/// Sends heartbeat messages at regular intervals, indefinitely.
/// Fails if the communication channel is closed.
pub async fn heartbeat_sender<S: AsyncWrite + Unpin + Send>(mut stream: S) {
    loop {
        // Random number so the message contains something.
        stream = match send_data(stream, Heartbeat(43)).await {
            Ok(stream) => stream,
            // If anything at all went wrong, the heartbeat is dead.
            Err(_) => return,
        };
        sleep(HEARTBEAT_TIMEOUT).await;
    }
}

/// Receives heartbeat messages indefinitely.
/// Fails if the communication channel is closed, or if no message is received
/// for too long.
pub async fn heartbeat_receiver<S: AsyncRead + Unpin + Send>(mut stream: S) {
    loop {
        stream = match timeout(
            HEARTBEAT_TIMEOUT * MAX_MISSED_HEARTBEATS,
            receive_data::<S, Heartbeat>(stream),
        )
        .await
        {
            Ok(Ok((stream, _))) => stream,
            // If anything at all went wrong the heartbeat is dead.
            _ => return,
        };
    }
}

#[cfg(test)]
mod tests {
    use tokio::{
        self,
        time::{timeout, Duration},
    };

    use super::{heartbeat_receiver, heartbeat_sender};
    use crate::validator_network::mock::MockSplittable;

    #[tokio::test]
    async fn sender_closed_on_broken_connection() {
        let (stream, _) = MockSplittable::new(4096);
        timeout(Duration::from_secs(10), heartbeat_sender(stream))
            .await
            .expect("should end immediately");
    }

    #[tokio::test]
    async fn receiver_closed_on_broken_connection() {
        let (stream, _) = MockSplittable::new(4096);
        timeout(Duration::from_secs(10), heartbeat_receiver(stream))
            .await
            .expect("should end immediately");
    }
}
