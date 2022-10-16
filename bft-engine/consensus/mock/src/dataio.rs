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

use stance_bft_types::{DataProvider as DataProviderT, FinalizationHandler as FinalizationHandlerT};
use async_trait::async_trait;
use codec::{Decode, Encode};
use futures::{channel::mpsc::unbounded, future::pending};
use log::error;
use parking_lot::Mutex;
use std::{
    io::{Cursor, Write},
    sync::Arc,
};

type Receiver<T> = futures::channel::mpsc::UnboundedReceiver<T>;
type Sender<T> = futures::channel::mpsc::UnboundedSender<T>;

pub type Data = u32;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct DataProvider {
    counter: usize,
    n_data: Option<usize>,
}

impl DataProvider {
    pub fn new() -> Self {
        Self {
            counter: 0,
            n_data: None,
        }
    }

    pub fn new_finite(n_data: usize) -> Self {
        Self {
            counter: 0,
            n_data: Some(n_data),
        }
    }
}

#[async_trait]
impl DataProviderT<Data> for DataProvider {
    async fn get_data(&mut self) -> Option<Data> {
        self.counter += 1;
        if let Some(n_data) = self.n_data {
            if n_data < self.counter {
                return None;
            }
        }
        Some(self.counter as u32)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Decode, Encode)]
pub struct StalledDataProvider {}

impl StalledDataProvider {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl DataProviderT<Data> for StalledDataProvider {
    async fn get_data(&mut self) -> Option<Data> {
        pending().await
    }
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug, Default)]
pub struct Saver {
    data: Arc<Mutex<Vec<u8>>>,
}

impl Saver {
    pub fn new(data: Arc<Mutex<Vec<u8>>>) -> Self {
        Self { data }
    }
}

impl Write for Saver {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        self.data.lock().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }
}

pub type Loader = Cursor<Vec<u8>>;
