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

pub(crate) use acceptance_policy::AcceptancePolicy;
pub(crate) use block_finalizer::MockedBlockFinalizer;
pub(crate) use block_request::MockedBlockRequester;
pub(crate) use header_backend::{create_block, Client};
pub(crate) use justification_handler_config::JustificationRequestSchedulerImpl;
pub(crate) use proposal::{
    stance_data_from_blocks, stance_data_from_headers, unvalidated_proposal_from_headers,
};
pub(crate) use session_info::{SessionInfoProviderImpl, VerifierWrapper};

pub(crate) type TBlock = substrate_test_runtime::Block;
pub(crate) type THeader = substrate_test_runtime::Header;
pub(crate) type THash = substrate_test_runtime::Hash;
pub(crate) type TNumber = substrate_test_runtime::BlockNumber;

mod acceptance_policy;
mod block_finalizer;
mod block_request;
mod header_backend;
mod justification_handler_config;
mod proposal;
mod session_info;
mod single_action_mock;
mod validator_network;
