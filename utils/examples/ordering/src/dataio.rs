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

use stance_consensus_types::{
    DataProvider as DataProviderT, FinalizationHandler as FinalizationHandlerT, NodeIndex,
};
use async_trait::async_trait;
use codec::{Decode, Encode};
use futures::{channel::mpsc::unbounded, future::pending};
use log::{error, info};

type Receiver<T> = futures::channel::mpsc::UnboundedReceiver<T>;
type Sender<T> = futures::channel::mpsc::UnboundedSender<T>;

pub type Data = (NodeIndex, u32);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Decode, Encode)]
pub struct DataProvider {
    id: NodeIndex,
    counter: u32,
    n_data: u32,
    stalled: bool,
}

impl DataProvider {
    pub fn new(id: NodeIndex, counter: u32, n_data: u32, stalled: bool) -> Self {
        Self {
            id,
            counter,
            n_data,
            stalled,
        }
    }
}

#[async_trait]
impl DataProviderT<Data> for DataProvider {
    async fn get_data(&mut self) -> Option<Data> {
        if self.n_data == 0 {
            if self.stalled {
                info!("Awaiting DataProvider::get_data forever");
                pending::<()>().await;
            }
            info!("Providing None");
            None
        } else {
            let data = (self.id, self.counter);
            info!("Providing data: {}", self.counter);
            self.counter += 1;
            self.n_data -= 1;
            Some(data)
        }
    }
}

#[derive(Clone)]
pub struct FinalizationHandler {
    tx: Sender<Data>,
}

impl FinalizationHandlerT<Data> for FinalizationHandler {
    fn data_finalized(&mut self, d: Data) {
        if let Err(e) = self.tx.unbounded_send(d) {
            error!(target: "finalization-handler", "Error when sending data from FinalizationHandler {:?}.", e);
        }
    }
}

impl FinalizationHandler {
    pub fn new() -> (Self, Receiver<Data>) {
        let (tx, rx) = unbounded();
        (Self { tx }, rx)
    }
}
