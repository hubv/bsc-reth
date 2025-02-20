use super::{
    metrics::{StaticFileProviderMetrics, StaticFileProviderOperation},
    LoadedJarRef,
};
use crate::{
    to_range, BlockHashReader, BlockNumReader, HeaderProvider, ReceiptProvider,
    TransactionsProvider,
};
use reth_chainspec::ChainInfo;
use reth_db::static_file::{
    HeaderMask, ReceiptMask, SidecarMask, StaticFileCursor, TransactionMask,
};
use reth_db_api::models::CompactU256;
use reth_primitives::{
    Address, BlobSidecars, BlockHash, BlockHashOrNumber, BlockNumber, Header, Receipt,
    SealedHeader, TransactionMeta, TransactionSigned, TransactionSignedNoHash, TxHash, TxNumber,
    B256, U256,
};
use reth_storage_api::SidecarsProvider;
use reth_storage_errors::provider::{ProviderError, ProviderResult};
use std::{
    ops::{Deref, RangeBounds},
    sync::Arc,
};

/// Provider over a specific `NippyJar` and range.
#[derive(Debug)]
pub struct StaticFileJarProvider<'a> {
    /// Main static file segment
    jar: LoadedJarRef<'a>,
    /// Another kind of static file segment to help query data from the main one.
    auxiliary_jar: Option<Box<Self>>,
    metrics: Option<Arc<StaticFileProviderMetrics>>,
}

impl<'a> Deref for StaticFileJarProvider<'a> {
    type Target = LoadedJarRef<'a>;
    fn deref(&self) -> &Self::Target {
        &self.jar
    }
}

impl<'a> From<LoadedJarRef<'a>> for StaticFileJarProvider<'a> {
    fn from(value: LoadedJarRef<'a>) -> Self {
        StaticFileJarProvider { jar: value, auxiliary_jar: None, metrics: None }
    }
}

impl<'a> StaticFileJarProvider<'a> {
    /// Provides a cursor for more granular data access.
    pub fn cursor<'b>(&'b self) -> ProviderResult<StaticFileCursor<'a>>
    where
        'b: 'a,
    {
        let result = StaticFileCursor::new(self.value(), self.mmap_handle())?;

        if let Some(metrics) = &self.metrics {
            metrics.record_segment_operation(
                self.segment(),
                StaticFileProviderOperation::InitCursor,
                None,
            );
        }

        Ok(result)
    }

    /// Adds a new auxiliary static file to help query data from the main one
    pub fn with_auxiliary(mut self, auxiliary_jar: Self) -> Self {
        self.auxiliary_jar = Some(Box::new(auxiliary_jar));
        self
    }

    /// Enables metrics on the provider.
    pub fn with_metrics(mut self, metrics: Arc<StaticFileProviderMetrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }
}

impl<'a> HeaderProvider for StaticFileJarProvider<'a> {
    fn header(&self, block_hash: &BlockHash) -> ProviderResult<Option<Header>> {
        Ok(self
            .cursor()?
            .get_two::<HeaderMask<Header, BlockHash>>(block_hash.into())?
            .filter(|(_, hash)| hash == block_hash)
            .map(|(header, _)| header))
    }

    fn header_by_number(&self, num: BlockNumber) -> ProviderResult<Option<Header>> {
        self.cursor()?.get_one::<HeaderMask<Header>>(num.into())
    }

    fn header_td(&self, block_hash: &BlockHash) -> ProviderResult<Option<U256>> {
        Ok(self
            .cursor()?
            .get_two::<HeaderMask<CompactU256, BlockHash>>(block_hash.into())?
            .filter(|(_, hash)| hash == block_hash)
            .map(|(td, _)| td.into()))
    }

    fn header_td_by_number(&self, num: BlockNumber) -> ProviderResult<Option<U256>> {
        Ok(self.cursor()?.get_one::<HeaderMask<CompactU256>>(num.into())?.map(Into::into))
    }

    fn headers_range(&self, range: impl RangeBounds<BlockNumber>) -> ProviderResult<Vec<Header>> {
        let range = to_range(range);

        let mut cursor = self.cursor()?;
        let mut headers = Vec::with_capacity((range.end - range.start) as usize);

        for num in range.start..range.end {
            if let Some(header) = cursor.get_one::<HeaderMask<Header>>(num.into())? {
                headers.push(header);
            }
        }

        Ok(headers)
    }

    fn sealed_header(&self, number: BlockNumber) -> ProviderResult<Option<SealedHeader>> {
        Ok(self
            .cursor()?
            .get_two::<HeaderMask<Header, BlockHash>>(number.into())?
            .map(|(header, hash)| header.seal(hash)))
    }

    fn sealed_headers_while(
        &self,
        range: impl RangeBounds<BlockNumber>,
        mut predicate: impl FnMut(&SealedHeader) -> bool,
    ) -> ProviderResult<Vec<SealedHeader>> {
        let range = to_range(range);

        let mut cursor = self.cursor()?;
        let mut headers = Vec::with_capacity((range.end - range.start) as usize);

        for number in range.start..range.end {
            if let Some((header, hash)) =
                cursor.get_two::<HeaderMask<Header, BlockHash>>(number.into())?
            {
                let sealed = header.seal(hash);
                if !predicate(&sealed) {
                    break
                }
                headers.push(sealed);
            }
        }
        Ok(headers)
    }
}

impl<'a> BlockHashReader for StaticFileJarProvider<'a> {
    fn block_hash(&self, number: u64) -> ProviderResult<Option<B256>> {
        self.cursor()?.get_one::<HeaderMask<BlockHash>>(number.into())
    }

    fn canonical_hashes_range(
        &self,
        start: BlockNumber,
        end: BlockNumber,
    ) -> ProviderResult<Vec<B256>> {
        let mut cursor = self.cursor()?;
        let mut hashes = Vec::with_capacity((end - start) as usize);

        for number in start..end {
            if let Some(hash) = cursor.get_one::<HeaderMask<BlockHash>>(number.into())? {
                hashes.push(hash)
            }
        }
        Ok(hashes)
    }
}

impl<'a> BlockNumReader for StaticFileJarProvider<'a> {
    fn chain_info(&self) -> ProviderResult<ChainInfo> {
        // Information on live database
        Err(ProviderError::UnsupportedProvider)
    }

    fn best_block_number(&self) -> ProviderResult<BlockNumber> {
        // Information on live database
        Err(ProviderError::UnsupportedProvider)
    }

    fn last_block_number(&self) -> ProviderResult<BlockNumber> {
        // Information on live database
        Err(ProviderError::UnsupportedProvider)
    }

    fn block_number(&self, hash: B256) -> ProviderResult<Option<BlockNumber>> {
        let mut cursor = self.cursor()?;

        Ok(cursor
            .get_one::<HeaderMask<BlockHash>>((&hash).into())?
            .and_then(|res| (res == hash).then(|| cursor.number()).flatten()))
    }
}

impl<'a> TransactionsProvider for StaticFileJarProvider<'a> {
    fn transaction_id(&self, hash: TxHash) -> ProviderResult<Option<TxNumber>> {
        let mut cursor = self.cursor()?;

        Ok(cursor
            .get_one::<TransactionMask<TransactionSignedNoHash>>((&hash).into())?
            .and_then(|res| (res.hash() == hash).then(|| cursor.number()).flatten()))
    }

    fn transaction_by_id(&self, num: TxNumber) -> ProviderResult<Option<TransactionSigned>> {
        Ok(self
            .cursor()?
            .get_one::<TransactionMask<TransactionSignedNoHash>>(num.into())?
            .map(|tx| tx.with_hash()))
    }

    fn transaction_by_id_no_hash(
        &self,
        num: TxNumber,
    ) -> ProviderResult<Option<TransactionSignedNoHash>> {
        self.cursor()?.get_one::<TransactionMask<TransactionSignedNoHash>>(num.into())
    }

    fn transaction_by_hash(&self, hash: TxHash) -> ProviderResult<Option<TransactionSigned>> {
        Ok(self
            .cursor()?
            .get_one::<TransactionMask<TransactionSignedNoHash>>((&hash).into())?
            .map(|tx| tx.with_hash()))
    }

    fn transaction_by_hash_with_meta(
        &self,
        _hash: TxHash,
    ) -> ProviderResult<Option<(TransactionSigned, TransactionMeta)>> {
        // Information required on indexing table [`tables::TransactionBlocks`]
        Err(ProviderError::UnsupportedProvider)
    }

    fn transaction_block(&self, _id: TxNumber) -> ProviderResult<Option<BlockNumber>> {
        // Information on indexing table [`tables::TransactionBlocks`]
        Err(ProviderError::UnsupportedProvider)
    }

    fn transactions_by_block(
        &self,
        _block_id: BlockHashOrNumber,
    ) -> ProviderResult<Option<Vec<TransactionSigned>>> {
        // Related to indexing tables. Live database should get the tx_range and call static file
        // provider with `transactions_by_tx_range` instead.
        Err(ProviderError::UnsupportedProvider)
    }

    fn transactions_by_block_range(
        &self,
        _range: impl RangeBounds<BlockNumber>,
    ) -> ProviderResult<Vec<Vec<TransactionSigned>>> {
        // Related to indexing tables. Live database should get the tx_range and call static file
        // provider with `transactions_by_tx_range` instead.
        Err(ProviderError::UnsupportedProvider)
    }

    fn transactions_by_tx_range(
        &self,
        range: impl RangeBounds<TxNumber>,
    ) -> ProviderResult<Vec<reth_primitives::TransactionSignedNoHash>> {
        let range = to_range(range);
        let mut cursor = self.cursor()?;
        let mut txes = Vec::with_capacity((range.end - range.start) as usize);

        for num in range {
            if let Some(tx) =
                cursor.get_one::<TransactionMask<TransactionSignedNoHash>>(num.into())?
            {
                txes.push(tx)
            }
        }
        Ok(txes)
    }

    fn senders_by_tx_range(
        &self,
        range: impl RangeBounds<TxNumber>,
    ) -> ProviderResult<Vec<Address>> {
        let txs = self.transactions_by_tx_range(range)?;
        TransactionSignedNoHash::recover_signers(&txs, txs.len())
            .ok_or(ProviderError::SenderRecoveryError)
    }

    fn transaction_sender(&self, num: TxNumber) -> ProviderResult<Option<Address>> {
        Ok(self
            .cursor()?
            .get_one::<TransactionMask<TransactionSignedNoHash>>(num.into())?
            .and_then(|tx| tx.recover_signer()))
    }
}

impl<'a> ReceiptProvider for StaticFileJarProvider<'a> {
    fn receipt(&self, num: TxNumber) -> ProviderResult<Option<Receipt>> {
        self.cursor()?.get_one::<ReceiptMask<Receipt>>(num.into())
    }

    fn receipt_by_hash(&self, hash: TxHash) -> ProviderResult<Option<Receipt>> {
        if let Some(tx_static_file) = &self.auxiliary_jar {
            if let Some(num) = tx_static_file.transaction_id(hash)? {
                return self.receipt(num)
            }
        }
        Ok(None)
    }

    fn receipts_by_block(&self, _block: BlockHashOrNumber) -> ProviderResult<Option<Vec<Receipt>>> {
        // Related to indexing tables. StaticFile should get the tx_range and call static file
        // provider with `receipt()` instead for each
        Err(ProviderError::UnsupportedProvider)
    }

    fn receipts_by_tx_range(
        &self,
        range: impl RangeBounds<TxNumber>,
    ) -> ProviderResult<Vec<Receipt>> {
        let range = to_range(range);
        let mut cursor = self.cursor()?;
        let mut receipts = Vec::with_capacity((range.end - range.start) as usize);

        for num in range {
            if let Some(tx) = cursor.get_one::<ReceiptMask<Receipt>>(num.into())? {
                receipts.push(tx)
            }
        }
        Ok(receipts)
    }
}

impl<'a> SidecarsProvider for StaticFileJarProvider<'a> {
    fn sidecars(&self, block_hash: &BlockHash) -> ProviderResult<Option<BlobSidecars>> {
        Ok(self
            .cursor()?
            .get_two::<SidecarMask<BlobSidecars, BlockHash>>(block_hash.into())?
            .filter(|(_, hash)| hash == block_hash)
            .map(|(sc, _)| sc))
    }

    fn sidecars_by_number(&self, num: BlockNumber) -> ProviderResult<Option<BlobSidecars>> {
        self.cursor()?.get_one::<SidecarMask<BlobSidecars>>(num.into())
    }
}
