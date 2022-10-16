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

use crate::{
    config::{Config as GeneralConfig, DelaySchedule},
    runway::NotificationOut,
    units::{PreUnit, Unit},
    Hasher, NodeCount, NodeIndex, Receiver, Round, Sender, Terminator,
};
use futures::{channel::oneshot, FutureExt, StreamExt};
use futures_timer::Delay;
use log::{debug, error, warn};
use std::{
    fmt::{Debug, Formatter},
    time::Duration,
};

mod creator;

pub use creator::Creator;

/// The configuration needed for the process creating new units.
#[derive(Clone)]
pub struct Config {
    node_id: NodeIndex,
    n_members: NodeCount,
    create_lag: DelaySchedule,
    max_round: Round,
}

impl Debug for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("node id", &self.node_id)
            .field("member count", &self.n_members)
            .field("max round", &self.max_round)
            .finish()
    }
}

impl From<GeneralConfig> for Config {
    fn from(conf: GeneralConfig) -> Self {
        Config {
            node_id: conf.node_ix,
            n_members: conf.n_members,
            create_lag: conf.delay_config.unit_creation_delay,
            max_round: conf.max_round,
        }
    }
}

pub struct IO<H: Hasher> {
    pub(crate) incoming_parents: Receiver<Unit<H>>,
    pub(crate) outgoing_units: Sender<NotificationOut<H>>,
}

async fn create_unit<H: Hasher>(
    round: Round,
    creator: &mut Creator<H>,
    create_lag: &DelaySchedule,
    mut can_create: bool,
    incoming_parents: &mut Receiver<Unit<H>>,
    mut exit: &mut oneshot::Receiver<()>,
) -> Result<(PreUnit<H>, Vec<H::Hash>), ()> {
    let mut delay = Delay::new(create_lag(round.into())).fuse();
    loop {
        if can_create {
            if let Some(result) = creator.create_unit(round) {
                return Ok(result);
            }
        }
        futures::select! {
            unit = incoming_parents.next() => match unit {
                Some(unit) => creator.add_unit(&unit),
                None => {
                    debug!(target: "StanceBFT-creator", "Incoming parent channel closed, exiting.");
                    return Err(());
                }
            },
            _ = &mut delay => {
                if can_create {
                    warn!(target: "StanceBFT-creator", "Delay passed despite us not waiting for it.");
                }
                can_create = true;
                delay = Delay::new(Duration::from_secs(30 * 60)).fuse();
            },
            _ = exit => {
                debug!(target: "StanceBFT-creator", "Received exit signal.");
                return Err(());
            },
        }
    }
}

/// A process responsible for creating new units. It receives all the units added locally to the Dag
/// via the `incoming_parents` channel. It creates units according to an internal strategy respecting
/// always the following constraints: if round is equal to 0, U has no parents, otherwise for a unit U of round r > 0
/// - all U's parents are from round (r-1),
/// - all U's parents are created by different nodes,
/// - one of U's parents is the (r-1)-round unit by U's creator,
/// - U has > floor(2*N/3) parents.
/// - U will appear in the channel only if all U's parents appeared there before
/// The currently implemented strategy creates the unit U according to a delay schedule and when enough
/// candidates for parents are available for all the above constraints to be satisfied.
///
/// We refer to the documentation https://setheum-labs.github.io/Stance/internals.html
/// Section 5.1 for a discussion of this component.
pub async fn run<H: Hasher>(
    conf: Config,
    io: IO<H>,
    mut starting_round: oneshot::Receiver<Option<Round>>,
    mut terminator: Terminator,
) {
    let Config {
        node_id,
        n_members,
        create_lag,
        max_round,
    } = conf;
    let mut creator = Creator::new(node_id, n_members);
    let IO {
        mut incoming_parents,
        outgoing_units,
    } = io;

    let starting_round = futures::select! {
        maybe_round =  starting_round => match maybe_round {
            Ok(Some(round)) => round,
            Ok(None) => {
                warn!(target: "StanceBFT-creator", "None starting round provided. Exiting.");
                return;
            }
            Err(e) => {
                error!(target: "StanceBFT-creator", "Starting round not provided: {}", e);
                return;
            }
        },
        _ = &mut terminator.get_exit() => {
            terminator.terminate_sync().await;
            return;
        },
    };

    debug!(target: "StanceBFT-creator", "Creator starting from round {}", starting_round);
    for round in starting_round..max_round {
        // Skip waiting if someone created a unit of a higher round.
        // In such a case at least 2/3 nodes created units from this round so we aren't skipping a
        // delay we should observe.
        let ignore_delay = creator.current_round() > round;
        let (unit, parent_hashes) = match create_unit(
            round,
            &mut creator,
            &create_lag,
            ignore_delay,
            &mut incoming_parents,
            terminator.get_exit(),
        )
        .await
        {
            Ok((u, ph)) => (u, ph),
            Err(_) => {
                terminator.terminate_sync().await;
                return;
            }
        };
        if let Err(e) =
            outgoing_units.unbounded_send(NotificationOut::CreatedPreUnit(unit, parent_hashes))
        {
            warn!(target: "StanceBFT-creator", "Notification send error: {}. Exiting.", e);
            return;
        }
    }

    warn!(target: "StanceBFT-creator", "Maximum round reached. Not creating another unit.");
}
