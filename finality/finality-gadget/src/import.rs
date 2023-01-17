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

use std::{collections::HashMap, marker::PhantomData, sync::Arc, time::Instant};

use stance_primitives::DAGESTAN_ENGINE_ID;
use futures::channel::mpsc::{TrySendError, UnboundedSender};
use log::{debug, warn};
use sc_client_api::backend::Backend;
use sc_consensus::{
    BlockCheckParams, BlockImport, BlockImportParams, ImportResult, JustificationImport,
};
use sp_api::TransactionFor;
use sp_consensus::Error as ConsensusError;
use sp_runtime::{
    traits::{Block as BlockT, Header, NumberFor},
    Justification,
};

use crate::{
    justification::{backwards_compatible_decode, DecodeError, JustificationNotification},
    metrics::{Checkpoint, Metrics},
};

pub struct StanceBlockImport<Block, Be, I>
where
    Block: BlockT,
    Be: Backend<Block>,
    I: crate::ClientForStance<Block, Be>,
{
    inner: Arc<I>,
    justification_tx: UnboundedSender<JustificationNotification<Block>>,
    metrics: Option<Metrics<<Block::Header as Header>::Hash>>,
    _phantom: PhantomData<Be>,
}

#[derive(Debug)]
enum SendJustificationError<Block>
where
    Block: BlockT,
{
    Send(TrySendError<JustificationNotification<Block>>),
    Consensus(Box<ConsensusError>),
    Decode(DecodeError),
}

impl<Block: BlockT> From<DecodeError> for SendJustificationError<Block> {
    fn from(decode_error: DecodeError) -> Self {
        Self::Decode(decode_error)
    }
}

impl<Block, Be, I> StanceBlockImport<Block, Be, I>
where
    Block: BlockT,
    Be: Backend<Block>,
    I: crate::ClientForStance<Block, Be>,
{
    pub fn new(
        inner: Arc<I>,
        justification_tx: UnboundedSender<JustificationNotification<Block>>,
        metrics: Option<Metrics<<Block::Header as Header>::Hash>>,
    ) -> StanceBlockImport<Block, Be, I> {
        StanceBlockImport {
            inner,
            justification_tx,
            metrics,
            _phantom: PhantomData,
        }
    }

    fn send_justification(
        &mut self,
        hash: Block::Hash,
        number: NumberFor<Block>,
        justification: Justification,
    ) -> Result<(), SendJustificationError<Block>> {
        debug!(target: "stance-justification", "Importing justification for block {:?}", number);
        if justification.0 != DAGESTAN_ENGINE_ID {
            return Err(SendJustificationError::Consensus(Box::new(
                ConsensusError::ClientImport("Stance can import only Stance justifications.".into()),
            )));
        }
        let justification_raw = justification.1;
        let stance_justification = backwards_compatible_decode(justification_raw)?;

        self.justification_tx
            .unbounded_send(JustificationNotification {
                hash,
                number,
                justification: stance_justification,
            })
            .map_err(SendJustificationError::Send)
    }
}

impl<Block, Be, I> Clone for StanceBlockImport<Block, Be, I>
where
    Block: BlockT,
    Be: Backend<Block>,
    I: crate::ClientForStance<Block, Be>,
{
    fn clone(&self) -> Self {
        StanceBlockImport {
            inner: self.inner.clone(),
            justification_tx: self.justification_tx.clone(),
            metrics: self.metrics.clone(),
            _phantom: PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<Block, Be, I> BlockImport<Block> for StanceBlockImport<Block, Be, I>
where
    Block: BlockT,
    Be: Backend<Block>,
    I: crate::ClientForStance<Block, Be> + Send,
    for<'a> &'a I:
        BlockImport<Block, Error = ConsensusError, Transaction = TransactionFor<I, Block>>,
    TransactionFor<I, Block>: Send + 'static,
{
    type Error = <I as BlockImport<Block>>::Error;
    type Transaction = TransactionFor<I, Block>;

    async fn check_block(
        &mut self,
        block: BlockCheckParams<Block>,
    ) -> Result<ImportResult, Self::Error> {
        self.inner.check_block(block).await
    }

    async fn import_block(
        &mut self,
        mut block: BlockImportParams<Block, Self::Transaction>,
        cache: HashMap<[u8; 4], Vec<u8>>,
    ) -> Result<ImportResult, Self::Error> {
        let number = *block.header.number();
        let post_hash = block.post_hash();
        if let Some(m) = &self.metrics {
            m.report_block(post_hash, Instant::now(), Checkpoint::Importing);
        };

        let justifications = block.justifications.take();

        debug!(target: "stance-justification", "Importing block {:?} {:?} {:?}", number, block.header.hash(), block.post_hash());
        let import_result = self.inner.import_block(block, cache).await;

        let imported_aux = match import_result {
            Ok(ImportResult::Imported(aux)) => aux,
            Ok(r) => return Ok(r),
            Err(e) => return Err(e),
        };

        if let Some(justification) =
            justifications.and_then(|just| just.into_justification(DAGESTAN_ENGINE_ID))
        {
            debug!(target: "stance-justification", "Got justification along imported block {:?}", number);

            if let Err(e) =
                self.send_justification(post_hash, number, (DAGESTAN_ENGINE_ID, justification))
            {
                warn!(target: "stance-justification", "Error while receiving justification for block {:?}: {:?}", post_hash, e);
            }
        }

        if let Some(m) = &self.metrics {
            m.report_block(post_hash, Instant::now(), Checkpoint::Imported);
        };

        Ok(ImportResult::Imported(imported_aux))
    }
}

#[async_trait::async_trait]
impl<Block, Be, I> JustificationImport<Block> for StanceBlockImport<Block, Be, I>
where
    Block: BlockT,
    Be: Backend<Block>,
    I: crate::ClientForStance<Block, Be>,
{
    type Error = ConsensusError;

    async fn on_start(&mut self) -> Vec<(Block::Hash, NumberFor<Block>)> {
        debug!(target: "stance-justification", "On start called");
        Vec::new()
    }

    async fn import_justification(
        &mut self,
        hash: Block::Hash,
        number: NumberFor<Block>,
        justification: Justification,
    ) -> Result<(), Self::Error> {
        debug!(target: "stance-justification", "import_justification called on {:?}", justification);
        self.send_justification(hash, number, justification)
            .map_err(|error| match error {
                SendJustificationError::Send(_) => ConsensusError::ClientImport(String::from(
                    "Could not send justification to ConsensusParty",
                )),
                SendJustificationError::Consensus(e) => *e,
                SendJustificationError::Decode(e) => {
                    warn!(target: "stance-justification", "Justification for block {:?} decoded incorrectly: {}", number, e);
                    ConsensusError::ClientImport(String::from("Could not decode justification"))
                }
            })
    }
}
