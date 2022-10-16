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

use crate::{BlockNum, NetworkData};
use async_trait::async_trait;
use futures::channel::mpsc::UnboundedSender;
use log::debug;
use parking_lot::Mutex;
use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Formatter},
    sync::Arc,
};

pub type Data = BlockNum;

#[derive(Clone)]
pub struct DataStore {
    next_message_id: u32,
    current_block: Arc<Mutex<BlockNum>>,
    available_blocks: HashSet<BlockNum>,
    message_requirements: HashMap<u32, usize>,
    dependent_messages: HashMap<BlockNum, Vec<u32>>,
    pending_messages: HashMap<u32, NetworkData>,
    messages_for_member: UnboundedSender<NetworkData>,
}

impl Debug for DataStore {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataStore")
            .field("next message id", &self.next_message_id)
            .field("available block count", &self.available_blocks.len())
            .field(
                "message requirement count",
                &self.message_requirements.len(),
            )
            .field("dependent message count", &self.dependent_messages.len())
            .field("pending message count", &self.pending_messages.len())
            .finish()
    }
}

impl DataStore {
    pub fn new(
        current_block: Arc<Mutex<BlockNum>>,
        messages_for_member: UnboundedSender<NetworkData>,
    ) -> Self {
        let available_blocks = (0..=*current_block.lock()).collect();
        DataStore {
            next_message_id: 0,
            current_block,
            available_blocks,
            message_requirements: HashMap::new(),
            dependent_messages: HashMap::new(),
            pending_messages: HashMap::new(),
            messages_for_member,
        }
    }

    fn add_pending_message(&mut self, message: NetworkData, requirements: Vec<BlockNum>) {
        let message_id = self.next_message_id;
        // Whatever test you are running should end before this becomes a problem.
        self.next_message_id += 1;
        for block_num in requirements.iter() {
            self.dependent_messages
                .entry(*block_num)
                .or_insert_with(Vec::new)
                .push(message_id);
        }
        self.message_requirements
            .insert(message_id, requirements.len());
        self.pending_messages.insert(message_id, message);
    }

    pub fn add_message(&mut self, message: NetworkData) {
        let requirements: Vec<_> = message
            .included_data()
            .into_iter()
            .filter(|b| !self.available_blocks.contains(b))
            .collect();
        if requirements.is_empty() {
            self.messages_for_member
                .unbounded_send(message)
                .expect("member accept messages");
        } else {
            self.add_pending_message(message, requirements);
        }
    }

    fn push_messages(&mut self, num: BlockNum) {
        for message_id in self
            .dependent_messages
            .entry(num)
            .or_insert_with(Vec::new)
            .iter()
        {
            *self
                .message_requirements
                .get_mut(message_id)
                .expect("there are some requirements") -= 1;
            if self.message_requirements[message_id] == 0 {
                let message = self
                    .pending_messages
                    .remove(message_id)
                    .expect("there is a pending message");
                self.messages_for_member
                    .unbounded_send(message)
                    .expect("member accept messages");
                self.message_requirements.remove(message_id);
            }
        }
        self.dependent_messages.remove(&num);
    }

    pub fn add_block(&mut self, num: BlockNum) {
        debug!(target: "data-store", "Added block {:?}.", num);
        self.available_blocks.insert(num);
        self.push_messages(num);
        while self
            .available_blocks
            .contains(&(*self.current_block.lock() + 1))
        {
            *self.current_block.lock() += 1;
        }
    }
}

#[derive(Clone)]
pub struct DataProvider {
    current_block: Arc<Mutex<BlockNum>>,
}

#[async_trait]
impl stance_consensus::DataProvider<Data> for DataProvider {
    async fn get_data(&mut self) -> Option<Data> {
        Some(*self.current_block.lock())
    }
}

impl DataProvider {
    pub fn new() -> (Self, Arc<Mutex<BlockNum>>) {
        let current_block = Arc::new(Mutex::new(0));
        (
            DataProvider {
                current_block: current_block.clone(),
            },
            current_block,
        )
    }
}
