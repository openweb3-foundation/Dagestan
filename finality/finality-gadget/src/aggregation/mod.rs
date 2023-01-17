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

use std::{fmt::Debug, hash::Hash, marker::PhantomData, time::Instant};

use stance_aggregator::NetworkError;
use stance::{Recipient, SignatureSet};
use sp_runtime::traits::Block;

use crate::{
    crypto::Signature,
    metrics::Checkpoint,
    network::{Data, DataNetwork, SendError},
    Metrics,
};

pub type RmcNetworkData<B> =
    stance_aggregator::RmcNetworkData<<B as Block>::Hash, Signature, SignatureSet<Signature>>;

pub struct NetworkWrapper<D: Data, N: DataNetwork<D>>(N, PhantomData<D>);

impl<D: Data, N: DataNetwork<D>> NetworkWrapper<D, N> {
    pub fn new(network: N) -> Self {
        Self(network, PhantomData)
    }
}

impl<H: Debug + Hash + Eq + Debug + Copy> stance_aggregator::Metrics<H> for Metrics<H> {
    fn report_aggregation_complete(&mut self, h: H) {
        self.report_block(h, Instant::now(), Checkpoint::Aggregating);
    }
}

#[async_trait::async_trait]
impl<T, D> stance_aggregator::ProtocolSink<D> for NetworkWrapper<D, T>
where
    T: DataNetwork<D>,
    D: Data,
{
    async fn next(&mut self) -> Option<D> {
        self.0.next().await
    }

    fn send(&self, data: D, recipient: Recipient) -> Result<(), NetworkError> {
        self.0.send(data, recipient).map_err(|e| match e {
            SendError::SendFailed => NetworkError::SendFail,
        })
    }
}
