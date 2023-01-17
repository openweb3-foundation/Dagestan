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

use std::{
    cmp::min,
    time::{Duration, Instant},
};

use crate::{MillisecsPerBlock, SessionPeriod};

pub enum SchedulerActions {
    ClearQueue,
    Request,
    Wait,
}

/// Bunch of methods for managing frequency of sending justification requests.
pub trait JustificationRequestScheduler {
    /// Decides whether we can request new justification.
    fn schedule_action(&mut self) -> SchedulerActions;
    /// Notice block finalization.
    fn on_block_finalized(&mut self);
    /// Notice request sending.
    fn on_request_sent(&mut self);
}

pub struct JustificationRequestSchedulerImpl {
    last_request_time: Instant,
    last_finalization_time: Instant,
    delay: Duration,
    attempt: u32,
    max_attemps: u32,
}

impl JustificationRequestSchedulerImpl {
    pub fn new(
        session_period: &SessionPeriod,
        millisecs_per_block: &MillisecsPerBlock,
        max_attemps: u32,
    ) -> Self {
        Self {
            last_request_time: Instant::now(),
            last_finalization_time: Instant::now(),
            ///Request justification during the session. Usually every two blocks,
            ///unless session period is peculiar small in which case we request it more often to ensure non-validators won't lag
            delay: Duration::from_millis(min(
                millisecs_per_block.0 * 2,
                millisecs_per_block.0 * session_period.0 as u64 / 10,
            )),
            attempt: 0,
            max_attemps,
        }
    }

    fn enough_time_elapsed(&self) -> bool {
        let now = Instant::now();

        now - self.last_finalization_time > self.delay
            && now - self.last_request_time > 2 * self.delay
    }
}

impl JustificationRequestScheduler for JustificationRequestSchedulerImpl {
    fn schedule_action(&mut self) -> SchedulerActions {
        let now = Instant::now();
        if self.enough_time_elapsed() {
            self.attempt += 1;

            if self.attempt == self.max_attemps {
                self.attempt = 0;
                return SchedulerActions::ClearQueue;
            }

            self.last_request_time = now;
            SchedulerActions::Request
        } else {
            SchedulerActions::Wait
        }
    }

    fn on_block_finalized(&mut self) {
        self.attempt = 0;
        self.last_finalization_time = Instant::now();
    }

    fn on_request_sent(&mut self) {
        self.last_request_time = Instant::now();
    }
}
