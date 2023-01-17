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

use crate::NodeIndex;

use codec::{Decode, Encode};

/// A recipient of a message, either a specific node or everyone.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Decode, Encode)]
pub enum Recipient {
    Everyone,
    Node(NodeIndex),
}

/// Network represents an interface for sending and receiving NetworkData.
///
/// Note on Rate Control: it is assumed that Network implements a rate control mechanism guaranteeing
/// that no node is allowed to spam messages without limits. We do not specify details yet, but in
/// future releases we plan to publish recommended upper bounds for the amounts of bandwidth and
/// number of messages allowed per node per a unit of time. These bounds must be carefully crafted
/// based upon the number of nodes N and the configured delays between subsequent Dag rounds, so
/// that at the same time spammers are cut off but honest nodes are able function correctly within
/// these bounds.
///
/// Note on Network Reliability: it is not assumed that each message that Stance orders to send
/// reaches its intended recipient, there are some built-in reliability mechanisms within Stance
/// that will automatically detect certain failures and resend messages as needed. Clearly, the less
/// reliable the network is, the worse the performance of Stance will be (generally slower to
/// produce output). Also, not surprisingly if the percentage of dropped messages is too high
/// Stance might stop making progress, but from what we observe in tests, this happens only when
/// the reliability is extremely bad, i.e., drops below 50% (which means there is some significant
/// issue with the network).
///
/// We refer to the documentation [here](https;//setheum-labs.github.io/Stance/stance_api.md)
/// Section 3.1.2 for a discussion of the required guarantees of this trait's implementation.
#[async_trait::async_trait]
pub trait Network<D>: Send {
    /// Send a message to a single node or everyone, depending on the value of the recipient
    /// argument.
    ///
    /// Note on the implementation: this function should be implemented in a non-blocking manner.
    /// Otherwise, the performance might be affected negatively or the execution may end up in a deadlock.
    fn send(&self, data: D, recipient: Recipient);
    /// Receive a message from the network.
    async fn next_event(&mut self) -> Option<D>;
}
