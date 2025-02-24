use crate::segments::{
    AccountHistory, ReceiptsByLogs, Segment, SenderRecovery, StaticFileSidecars, StorageHistory,
    TransactionLookup, UserReceipts,
};
use reth_db_api::database::Database;
use reth_provider::providers::StaticFileProvider;
use reth_prune_types::PruneModes;

use super::{StaticFileHeaders, StaticFileReceipts, StaticFileTransactions};

/// Collection of [Segment]. Thread-safe, allocated on the heap.
#[derive(Debug)]
pub struct SegmentSet<DB: Database> {
    inner: Vec<Box<dyn Segment<DB>>>,
}

impl<DB: Database> SegmentSet<DB> {
    /// Returns empty [`SegmentSet`] collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds new [Segment] to collection.
    pub fn segment<S: Segment<DB> + 'static>(mut self, segment: S) -> Self {
        self.inner.push(Box::new(segment));
        self
    }

    /// Adds new [Segment] to collection if it's [Some].
    pub fn segment_opt<S: Segment<DB> + 'static>(self, segment: Option<S>) -> Self {
        if let Some(segment) = segment {
            return self.segment(segment)
        }
        self
    }

    /// Consumes [`SegmentSet`] and returns a [Vec].
    pub fn into_vec(self) -> Vec<Box<dyn Segment<DB>>> {
        self.inner
    }

    /// Creates a [`SegmentSet`] from an existing components, such as [`StaticFileProvider`] and
    /// [`PruneModes`].
    pub fn from_components(
        static_file_provider: StaticFileProvider,
        prune_modes: PruneModes,
    ) -> Self {
        let PruneModes {
            sender_recovery,
            transaction_lookup,
            receipts,
            account_history,
            storage_history,
            receipts_log_filter,
        } = prune_modes;

        Self::default()
            // Static file headers
            .segment(StaticFileHeaders::new(static_file_provider.clone()))
            // Static file transactions
            .segment(StaticFileTransactions::new(static_file_provider.clone()))
            // Static file receipts
            .segment(StaticFileReceipts::new(static_file_provider.clone()))
            // Static file receipts
            .segment(StaticFileSidecars::new(static_file_provider))
            // Account history
            .segment_opt(account_history.map(AccountHistory::new))
            // Storage history
            .segment_opt(storage_history.map(StorageHistory::new))
            // User receipts
            .segment_opt(receipts.map(UserReceipts::new))
            // Receipts by logs
            .segment_opt(
                (!receipts_log_filter.is_empty())
                    .then(|| ReceiptsByLogs::new(receipts_log_filter.clone())),
            )
            // Transaction lookup
            .segment_opt(transaction_lookup.map(TransactionLookup::new))
            // Sender recovery
            .segment_opt(sender_recovery.map(SenderRecovery::new))
    }
}

impl<DB: Database> Default for SegmentSet<DB> {
    fn default() -> Self {
        Self { inner: Vec::new() }
    }
}
