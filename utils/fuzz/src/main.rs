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

use std::{
    io,
    io::{BufReader, BufWriter},
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "fuzz-helper",
    about = "data generator for the purpose of fuzzing"
)]
struct Opt {
    /// Verify data provided on stdin by calling member::run on it.
    #[structopt(short, long)]
    check_fuzz: bool,

    /// Generate data for a given number of members.
    /// When used with the 'check_fuzz' flag it verifies data assuming this number of members.
    #[structopt(default_value = "4")]
    members: usize,

    /// Generate a given number of batches.
    /// When used with the 'check_fuzz' flag it will verify if we are able to create at least this number of batches.
    #[structopt(default_value = "30")]
    batches: usize,
}

fn main() {
    let opt = Opt::from_args();
    if opt.check_fuzz {
        stance_fuzz::check_fuzz(BufReader::new(io::stdin()), opt.members, Some(opt.batches));
    } else {
        stance_fuzz::generate_fuzz(BufWriter::new(io::stdout()), opt.members, opt.batches);
    }
}
