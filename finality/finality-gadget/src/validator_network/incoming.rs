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

use std::fmt::{Display, Error as FmtError, Formatter};

use stance_primitives::AuthorityId;
use futures::channel::{mpsc, oneshot};
use log::{debug, info};

use crate::{
    crypto::AuthorityPen,
    validator_network::{
        protocol_negotiation::{protocol, ProtocolNegotiationError},
        protocols::ProtocolError,
        Data, Splittable,
    },
};

enum IncomingError {
    ProtocolNegotiationError(ProtocolNegotiationError),
    ProtocolError(ProtocolError),
}

impl Display for IncomingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        use IncomingError::*;
        match self {
            ProtocolNegotiationError(e) => write!(f, "protocol negotiation error: {}", e),
            ProtocolError(e) => write!(f, "protocol error: {}", e),
        }
    }
}

impl From<ProtocolNegotiationError> for IncomingError {
    fn from(e: ProtocolNegotiationError) -> Self {
        IncomingError::ProtocolNegotiationError(e)
    }
}

impl From<ProtocolError> for IncomingError {
    fn from(e: ProtocolError) -> Self {
        IncomingError::ProtocolError(e)
    }
}

async fn manage_incoming<D: Data, S: Splittable>(
    authority_pen: AuthorityPen,
    stream: S,
    result_for_parent: mpsc::UnboundedSender<(AuthorityId, oneshot::Sender<()>)>,
    data_for_user: mpsc::UnboundedSender<D>,
) -> Result<(), IncomingError> {
    debug!(target: "validator-network", "Performing incoming protocol negotiation.");
    let (stream, protocol) = protocol(stream).await?;
    debug!(target: "validator-network", "Negotiated protocol, running.");
    Ok(protocol
        .manage_incoming(stream, authority_pen, result_for_parent, data_for_user)
        .await?)
}

/// Manage an incoming connection. After the handshake it will send the recognized AuthorityId to
/// the parent, together with an exit channel for this process. When this channel is dropped the
/// process ends. Whenever data arrives on this connection it will be passed to the user. Any
/// failures in receiving data result in the process stopping, we assume the other side will
/// reestablish it if necessary.
pub async fn incoming<D: Data, S: Splittable>(
    authority_pen: AuthorityPen,
    stream: S,
    result_for_parent: mpsc::UnboundedSender<(AuthorityId, oneshot::Sender<()>)>,
    data_for_user: mpsc::UnboundedSender<D>,
) {
    if let Err(e) = manage_incoming(authority_pen, stream, result_for_parent, data_for_user).await {
        info!(target: "validator-network", "Incoming connection failed: {}", e);
    }
}
