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

use super::*;

pub struct IO<'a, H: Hasher, D: Data, MK: MultiKeychain> {
    pub messages_for_network: Sender<(
        AlertMessage<H, D, MK::Signature, MK::PartialMultisignature>,
        Recipient,
    )>,
    pub messages_from_network:
        Receiver<AlertMessage<H, D, MK::Signature, MK::PartialMultisignature>>,
    pub notifications_for_units: Sender<ForkingNotification<H, D, MK::Signature>>,
    pub alerts_from_units: Receiver<Alert<H, D, MK::Signature>>,
    pub rmc: ReliableMulticast<'a, H::Hash, MK>,
    pub messages_from_rmc: Receiver<RmcMessage<H::Hash, MK::Signature, MK::PartialMultisignature>>,
    pub messages_for_rmc: Sender<RmcMessage<H::Hash, MK::Signature, MK::PartialMultisignature>>,
    pub alerter_index: NodeIndex,
}

impl<'a, H: Hasher, D: Data, MK: MultiKeychain> IO<'a, H, D, MK> {
    pub fn rmc_message_to_network(
        &mut self,
        message: RmcMessage<H::Hash, MK::Signature, MK::PartialMultisignature>,
        exiting: &mut bool,
    ) {
        self.send_message_for_network(
            AlertMessage::RmcMessage(self.alerter_index, message),
            Recipient::Everyone,
            exiting,
        );
    }

    pub fn send_notification_for_units(
        &mut self,
        notification: ForkingNotification<H, D, MK::Signature>,
        exiting: &mut bool,
    ) {
        if self
            .notifications_for_units
            .unbounded_send(notification)
            .is_err()
        {
            warn!(target: "Stance-alerter", "{:?} Channel with forking notifications should be open", self.alerter_index);
            *exiting = true;
        }
    }

    pub fn send_message_for_network(
        &mut self,
        message: AlertMessage<H, D, MK::Signature, MK::PartialMultisignature>,
        recipient: Recipient,
        exiting: &mut bool,
    ) {
        if self
            .messages_for_network
            .unbounded_send((message, recipient))
            .is_err()
        {
            warn!(target: "Stance-alerter", "{:?} Channel with notifications for network should be open", self.alerter_index);
            *exiting = true;
        }
    }
}
