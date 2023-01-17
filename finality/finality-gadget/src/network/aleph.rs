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

use std::marker::PhantomData;

use stance::{Network as StanceNetwork, NetworkData as StanceNetworkData, SignatureSet};
use log::warn;
use sp_runtime::traits::Block;

use crate::{
    crypto::Signature,
    data_io::{StanceData, StanceNetworkMessage},
    network::{Data, DataNetwork},
    Hasher,
};

pub type NetworkData<B> =
    StanceNetworkData<Hasher, StanceData<B>, Signature, SignatureSet<Signature>>;

impl<B: Block> StanceNetworkMessage<B> for NetworkData<B> {
    fn included_data(&self) -> Vec<StanceData<B>> {
        self.included_data()
    }
}

/// A wrapper needed only because of type system theoretical constraints. Sadness.
pub struct NetworkWrapper<D: Data, DN: DataNetwork<D>> {
    inner: DN,
    _phantom: PhantomData<D>,
}

impl<D: Data, DN: DataNetwork<D>> From<DN> for NetworkWrapper<D, DN> {
    fn from(inner: DN) -> Self {
        NetworkWrapper {
            inner,
            _phantom: PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<D: Data, DN: DataNetwork<D>> StanceNetwork<D> for NetworkWrapper<D, DN> {
    fn send(&self, data: D, recipient: stance::Recipient) {
        if self.inner.send(data, recipient).is_err() {
            warn!(target: "stance-network", "Error sending an Stance message to the network.");
        }
    }

    async fn next_event(&mut self) -> Option<D> {
        self.inner.next().await
    }
}
