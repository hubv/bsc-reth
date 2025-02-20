#![allow(dead_code)]

use reth_db::database::Database;
use reth_errors::ProviderResult;
use reth_primitives::{SealedBlock, StaticFileSegment, TransactionSignedNoHash, B256, U256};
use reth_provider::{
    ProviderFactory, StaticFileProviderFactory, StaticFileWriter, TransactionsProviderExt,
};
use std::sync::{
    mpsc::{Receiver, SendError, Sender},
    Arc,
};
use tokio::sync::oneshot;

use crate::{
    database::{DatabaseAction, DatabaseServiceHandle},
    tree::ExecutedBlock,
};

/// Writes finalized blocks to reth's static files.
///
/// This is meant to be a spawned service that listens for various incoming finalization operations,
/// and writing to or producing new static files.
///
/// This should be spawned in its own thread with [`std::thread::spawn`], since this performs
/// blocking file operations in an endless loop.
#[derive(Debug)]
pub struct StaticFileService<DB> {
    /// The db / static file provider to use
    provider: ProviderFactory<DB>,
    /// Handle for the database service
    database_handle: DatabaseServiceHandle,
    /// Incoming requests to write static files
    incoming: Receiver<StaticFileAction>,
}

impl<DB> StaticFileService<DB>
where
    DB: Database + 'static,
{
    /// Create a new static file service.
    pub const fn new(
        provider: ProviderFactory<DB>,
        incoming: Receiver<StaticFileAction>,
        database_handle: DatabaseServiceHandle,
    ) -> Self {
        Self { provider, database_handle, incoming }
    }

    // TODO: some things about this are a bit weird, and just to make the underlying static file
    // writes work - tx number, total difficulty inclusion. They require either additional in memory
    // data or a db lookup. Maybe we can use a db read here
    /// Writes the transactions to static files, to act as a log.
    ///
    /// This will then send a command to the db service, that it should update the checkpoints for
    /// headers and block bodies.
    fn log_transactions(
        &self,
        block: Arc<SealedBlock>,
        start_tx_number: u64,
        td: U256,
        sender: oneshot::Sender<()>,
    ) -> ProviderResult<()> {
        let provider = self.provider.static_file_provider();
        let mut header_writer = provider.get_writer(block.number, StaticFileSegment::Headers)?;
        let mut transactions_writer =
            provider.get_writer(block.number, StaticFileSegment::Transactions)?;
        let mut sidecars_writer = provider.get_writer(block.number, StaticFileSegment::Sidecars)?;

        // TODO: does to_compact require ownership?
        header_writer.append_header(block.header().clone(), td, block.hash())?;
        let no_hash_transactions =
            block.body.clone().into_iter().map(TransactionSignedNoHash::from);

        let mut tx_number = start_tx_number;
        for tx in no_hash_transactions {
            transactions_writer.append_transaction(tx_number, tx)?;
            tx_number += 1;
        }

        // increment block for transactions
        transactions_writer.increment_block(StaticFileSegment::Transactions, block.number)?;

        sidecars_writer.append_sidecars(
            block.sidecars.clone().unwrap_or_default(),
            block.number,
            block.hash(),
        )?;

        // finally commit
        transactions_writer.commit()?;
        header_writer.commit()?;
        sidecars_writer.commit()?;

        // TODO: do we care about the mpsc error here?
        // send a command to the db service to update the checkpoints for headers / bodies
        let _ = self
            .database_handle
            .send_action(DatabaseAction::UpdateTransactionMeta((block.number, sender)));

        Ok(())
    }

    /// Write execution-related block data to static files.
    ///
    /// This will then send a command to the db service, that it should write new data, and update
    /// the checkpoints for execution and beyond.
    fn write_execution_data(
        &self,
        blocks: Vec<ExecutedBlock>,
        sender: oneshot::Sender<B256>,
    ) -> ProviderResult<()> {
        if blocks.is_empty() {
            return Ok(())
        }
        let provider = self.provider.static_file_provider();

        // NOTE: checked non-empty above
        let first_block = blocks.first().unwrap().block();
        let last_block = blocks.last().unwrap().block();

        // get highest receipt, if it returns none, use zero (this is the first static file write)
        let mut current_receipt = provider
            .get_highest_static_file_tx(StaticFileSegment::Receipts)
            .map(|num| num + 1)
            .unwrap_or_default();
        let mut current_block = first_block.number;

        let mut receipts_writer =
            provider.get_writer(first_block.number, StaticFileSegment::Receipts)?;
        for receipts in blocks.iter().map(|block| block.execution_outcome().receipts.clone()) {
            debug_assert!(receipts.len() == 1);
            // TODO: should we also assert that the receipt is not None here, that means the
            // receipt is pruned
            for maybe_receipt in receipts.first().unwrap() {
                if let Some(receipt) = maybe_receipt {
                    receipts_writer.append_receipt(current_receipt, receipt.clone())?;
                }
                current_receipt += 1;
            }

            // increment the block
            receipts_writer.increment_block(StaticFileSegment::Receipts, current_block)?;
            current_block += 1;
        }

        // finally increment block and commit
        receipts_writer.commit()?;

        // TODO: do we care about the mpsc error here?
        // send a command to the db service to update the checkpoints for execution etc.
        let _ = self.database_handle.send_action(DatabaseAction::SaveBlocks((blocks, sender)));

        Ok(())
    }

    /// Removes the blocks above the given block number from static files. Also removes related
    /// receipt and header data.
    ///
    /// This is exclusive, i.e., it only removes blocks above `block_number`, and does not remove
    /// `block_number`.
    ///
    /// Returns the block hash for the lowest block removed from the database, which should be
    /// the hash for `block_number + 1`.
    ///
    /// This is meant to be called by the db service, as this should only be done after related data
    /// is removed from the database, and checkpoints are updated.
    ///
    /// Returns the hash of the lowest removed block.
    fn remove_blocks_above(
        &self,
        block_num: u64,
        sender: oneshot::Sender<()>,
    ) -> ProviderResult<()> {
        let sf_provider = self.provider.static_file_provider();
        let db_provider_rw = self.provider.provider_rw()?;

        // get the highest static file block for the total block range
        let highest_static_file_block = sf_provider
            .get_highest_static_file_block(StaticFileSegment::Headers)
            .expect("todo: error handling, headers should exist");

        // Get the total txs for the block range, so we have the correct number of columns for
        // receipts and transactions
        let tx_range = db_provider_rw
            .transaction_range_by_block_range(block_num..=highest_static_file_block)?;
        let total_txs = tx_range.end().saturating_sub(*tx_range.start());

        // get the writers
        let mut header_writer = sf_provider.get_writer(block_num, StaticFileSegment::Headers)?;
        let mut transactions_writer =
            sf_provider.get_writer(block_num, StaticFileSegment::Transactions)?;
        let mut receipts_writer = sf_provider.get_writer(block_num, StaticFileSegment::Receipts)?;
        let mut sidecars_writer = sf_provider.get_writer(block_num, StaticFileSegment::Sidecars)?;

        // finally actually truncate, these internally commit
        receipts_writer.prune_receipts(total_txs, block_num)?;
        transactions_writer.prune_transactions(total_txs, block_num)?;
        header_writer.prune_headers(highest_static_file_block.saturating_sub(block_num))?;

        // get the highest static file block of sidecars and prune
        let highest_static_file_block = sf_provider
            .get_highest_static_file_block(StaticFileSegment::Sidecars)
            .expect("todo: error handling, headers should exist");
        sidecars_writer.prune_sidecars(highest_static_file_block.saturating_sub(block_num))?;

        sf_provider.commit()?;

        Ok(())
    }
}

impl<DB> StaticFileService<DB>
where
    DB: Database + 'static,
{
    /// This is the main loop, that will listen to static file actions, and write DB data to static
    /// files.
    pub fn run(self) {
        // If the receiver errors then senders have disconnected, so the loop should then end.
        while let Ok(action) = self.incoming.recv() {
            match action {
                StaticFileAction::LogTransactions((
                    block,
                    start_tx_number,
                    td,
                    response_sender,
                )) => {
                    self.log_transactions(block, start_tx_number, td, response_sender)
                        .expect("todo: handle errors");
                }
                StaticFileAction::RemoveBlocksAbove((block_num, response_sender)) => {
                    self.remove_blocks_above(block_num, response_sender)
                        .expect("todo: handle errors");
                }
                StaticFileAction::WriteExecutionData((blocks, response_sender)) => {
                    self.write_execution_data(blocks, response_sender)
                        .expect("todo: handle errors");
                }
            }
        }
    }
}

/// A signal to the static file service that some data should be copied from the DB to static files.
#[derive(Debug)]
pub enum StaticFileAction {
    /// The given block has been added to the canonical chain, its transactions and headers will be
    /// persisted for durability.
    ///
    /// This will then send a command to the db service, that it should update the checkpoints for
    /// headers and block bodies.
    LogTransactions((Arc<SealedBlock>, u64, U256, oneshot::Sender<()>)),

    /// Write execution-related block data to static files.
    ///
    /// This will then send a command to the db service, that it should write new data, and update
    /// the checkpoints for execution and beyond.
    WriteExecutionData((Vec<ExecutedBlock>, oneshot::Sender<B256>)),

    /// Removes the blocks above the given block number from static files. Also removes related
    /// receipt and header data.
    ///
    /// This is meant to be called by the db service, as this should only be done after related
    /// data is removed from the database, and checkpoints are updated.
    RemoveBlocksAbove((u64, oneshot::Sender<()>)),
}

/// A handle to the static file service
#[derive(Debug, Clone)]
pub struct StaticFileServiceHandle {
    /// The channel used to communicate with the static file service
    sender: Sender<StaticFileAction>,
}

impl StaticFileServiceHandle {
    /// Create a new [`StaticFileServiceHandle`] from a [`Sender<StaticFileAction>`].
    pub const fn new(sender: Sender<StaticFileAction>) -> Self {
        Self { sender }
    }

    /// Sends a specific [`StaticFileAction`] in the contained channel. The caller is responsible
    /// for creating any channels for the given action.
    pub fn send_action(&self, action: StaticFileAction) -> Result<(), SendError<StaticFileAction>> {
        self.sender.send(action)
    }
}
