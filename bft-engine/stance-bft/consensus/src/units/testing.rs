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
    creation::Creator as GenericCreator,
    units::{
        FullUnit as GenericFullUnit, PreUnit as GenericPreUnit,
        UncheckedSignedUnit as GenericUncheckedSignedUnit, Unit as GenericUnit,
    },
    Hasher, NodeCount, NodeIndex, Round, SessionId, Signed,
};
use stance_bft_mock::{Data, Hasher64, Keychain, Signature};

type Creator = GenericCreator<Hasher64>;
type PreUnit = GenericPreUnit<Hasher64>;
type Unit = GenericUnit<Hasher64>;
type FullUnit = GenericFullUnit<Hasher64, Data>;
type UncheckedSignedUnit = GenericUncheckedSignedUnit<Hasher64, Data, Signature>;

pub fn creator_set(n_members: NodeCount) -> Vec<Creator> {
    (0..n_members.0)
        .map(|i| Creator::new(NodeIndex(i), n_members))
        .collect()
}

pub fn create_units<'a, C: Iterator<Item = &'a Creator>>(
    creators: C,
    round: Round,
) -> Vec<(PreUnit, Vec<<Hasher64 as Hasher>::Hash>)> {
    creators
        .map(|c| c.create_unit(round).expect("Creation should succeed."))
        .collect()
}

pub fn preunit_to_unit(preunit: PreUnit, session_id: SessionId) -> Unit {
    FullUnit::new(preunit, Some(0), session_id).unit()
}

impl Creator {
    pub fn add_units(&mut self, units: &[Unit]) {
        for unit in units {
            self.add_unit(unit);
        }
    }
}

pub async fn preunit_to_unchecked_signed_unit(
    pu: PreUnit,
    session_id: SessionId,
    keychain: &Keychain,
) -> UncheckedSignedUnit {
    let full_unit = FullUnit::new(pu, Some(0), session_id);
    let signed_unit = Signed::sign(full_unit, keychain).await;
    signed_unit.into()
}
