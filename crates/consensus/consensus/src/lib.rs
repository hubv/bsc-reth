//! Consensus protocol functions

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/paradigmxyz/reth/main/assets/reth-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/paradigmxyz/reth/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

use reth_primitives::{
    constants::MINIMUM_GAS_LIMIT, BlockHash, BlockNumber, BlockWithSenders, Bloom, GotExpected,
    GotExpectedBoxed, Header, InvalidTransactionError, Receipt, Request, SealedBlock, SealedHeader,
    B256, U256,
};

#[cfg(feature = "std")]
use std::fmt::Debug;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{fmt::Debug, vec::Vec};

/// A consensus implementation that does nothing.
pub mod noop;

#[cfg(any(test, feature = "test-utils"))]
/// test helpers for mocking consensus
pub mod test_utils;

/// Post execution input passed to [`Consensus::validate_block_post_execution`].
#[derive(Debug)]
pub struct PostExecutionInput<'a> {
    /// Receipts of the block.
    pub receipts: &'a [Receipt],
    /// EIP-7685 requests of the block.
    pub requests: &'a [Request],
}

impl<'a> PostExecutionInput<'a> {
    /// Creates a new instance of `PostExecutionInput`.
    pub const fn new(receipts: &'a [Receipt], requests: &'a [Request]) -> Self {
        Self { receipts, requests }
    }
}

/// Consensus is a protocol that chooses canonical chain.
#[auto_impl::auto_impl(&, Arc)]
pub trait Consensus: Debug + Send + Sync {
    /// Validate if header is correct and follows consensus specification.
    ///
    /// This is called on standalone header to check if all hashes are correct.
    fn validate_header(&self, header: &SealedHeader) -> Result<(), ConsensusError>;

    /// Validate that the header information regarding parent are correct.
    /// This checks the block number, timestamp, basefee and gas limit increment.
    ///
    /// This is called before properties that are not in the header itself (like total difficulty)
    /// have been computed.
    ///
    /// **This should not be called for the genesis block**.
    ///
    /// Note: Validating header against its parent does not include other Consensus validations.
    fn validate_header_against_parent(
        &self,
        header: &SealedHeader,
        parent: &SealedHeader,
    ) -> Result<(), ConsensusError>;

    /// Validates the given headers
    ///
    /// This ensures that the first header is valid on its own and all subsequent headers are valid
    /// on its own and valid against its parent.
    ///
    /// Note: this expects that the headers are in natural order (ascending block number)
    fn validate_header_range(&self, headers: &[SealedHeader]) -> Result<(), HeaderConsensusError> {
        if let Some((initial_header, remaining_headers)) = headers.split_first() {
            self.validate_header(initial_header)
                .map_err(|e| HeaderConsensusError(e, initial_header.clone()))?;
            let mut parent = initial_header;
            for child in remaining_headers {
                self.validate_header(child).map_err(|e| HeaderConsensusError(e, child.clone()))?;
                self.validate_header_against_parent(child, parent)
                    .map_err(|e| HeaderConsensusError(e, child.clone()))?;
                parent = child;
            }
        }
        Ok(())
    }

    /// Validate if the header is correct and follows the consensus specification, including
    /// computed properties (like total difficulty).
    ///
    /// Some consensus engines may want to do additional checks here.
    ///
    /// Note: validating headers with TD does not include other Consensus validation.
    fn validate_header_with_total_difficulty(
        &self,
        header: &Header,
        total_difficulty: U256,
    ) -> Result<(), ConsensusError>;

    /// Validate a block disregarding world state, i.e. things that can be checked before sender
    /// recovery and execution.
    ///
    /// See the Yellow Paper sections 4.3.2 "Holistic Validity", 4.3.4 "Block Header Validity", and
    /// 11.1 "Ommer Validation".
    ///
    /// **This should not be called for the genesis block**.
    ///
    /// Note: validating blocks does not include other validations of the Consensus
    fn validate_block_pre_execution(&self, block: &SealedBlock) -> Result<(), ConsensusError>;

    /// Validate a block considering world state, i.e. things that can not be checked before
    /// execution.
    ///
    /// See the Yellow Paper sections 4.3.2 "Holistic Validity".
    ///
    /// Note: validating blocks does not include other validations of the Consensus
    fn validate_block_post_execution(
        &self,
        block: &BlockWithSenders,
        input: PostExecutionInput<'_>,
    ) -> Result<(), ConsensusError>;
}

/// Consensus Errors
#[derive(thiserror_no_std::Error, Debug, PartialEq, Eq, Clone)]
pub enum ConsensusError {
    /// Error when the gas used in the header exceeds the gas limit.
    #[error("block used gas ({gas_used}) is greater than gas limit ({gas_limit})")]
    HeaderGasUsedExceedsGasLimit {
        /// The gas used in the block header.
        gas_used: u64,
        /// The gas limit in the block header.
        gas_limit: u64,
    },

    /// Error when block gas used doesn't match expected value
    #[error("block gas used mismatch: {gas}; gas spent by each transaction: {gas_spent_by_tx:?}")]
    BlockGasUsed {
        /// The gas diff.
        gas: GotExpected<u64>,
        /// Gas spent by each transaction
        gas_spent_by_tx: Vec<(u64, u64)>,
    },

    /// Error when the hash of block ommer is different from the expected hash.
    #[error("mismatched block ommer hash: {0}")]
    BodyOmmersHashDiff(GotExpectedBoxed<B256>),

    /// Error when the state root in the block is different from the expected state root.
    #[error("mismatched block state root: {0}")]
    BodyStateRootDiff(GotExpectedBoxed<B256>),

    /// Error when the transaction root in the block is different from the expected transaction
    /// root.
    #[error("mismatched block transaction root: {0}")]
    BodyTransactionRootDiff(GotExpectedBoxed<B256>),

    /// Error when the receipt root in the block is different from the expected receipt root.
    #[error("receipt root mismatch: {0}")]
    BodyReceiptRootDiff(GotExpectedBoxed<B256>),

    /// Error when header bloom filter is different from the expected bloom filter.
    #[error("header bloom filter mismatch: {0}")]
    BodyBloomLogDiff(GotExpectedBoxed<Bloom>),

    /// Error when the withdrawals root in the block is different from the expected withdrawals
    /// root.
    #[error("mismatched block withdrawals root: {0}")]
    BodyWithdrawalsRootDiff(GotExpectedBoxed<B256>),

    /// Error when the requests root in the block is different from the expected requests
    /// root.
    #[error("mismatched block requests root: {0}")]
    BodyRequestsRootDiff(GotExpectedBoxed<B256>),

    /// Error when a block with a specific hash and number is already known.
    #[error("block with [hash={hash}, number={number}] is already known")]
    BlockKnown {
        /// The hash of the known block.
        hash: BlockHash,
        /// The block number of the known block.
        number: BlockNumber,
    },

    /// Error when the parent hash of a block is not known.
    #[error("block parent [hash={hash}] is not known")]
    ParentUnknown {
        /// The hash of the unknown parent block.
        hash: BlockHash,
    },

    /// Error when the block number does not match the parent block number.
    #[error(
        "block number {block_number} does not match parent block number {parent_block_number}"
    )]
    ParentBlockNumberMismatch {
        /// The parent block number.
        parent_block_number: BlockNumber,
        /// The block number.
        block_number: BlockNumber,
    },

    /// Error when the parent hash does not match the expected parent hash.
    #[error("mismatched parent hash: {0}")]
    ParentHashMismatch(GotExpectedBoxed<B256>),

    /// Error when the block timestamp is not expected compared to the predicted timestamp.
    #[error("block timestamp {timestamp} is not expected compared to {predicted_timestamp}")]
    TimestampNotExpected {
        /// The block's timestamp.
        timestamp: u64,
        /// The predicted timestamp.
        predicted_timestamp: u64,
    },

    /// Error when the block timestamp is in the future compared to our clock time.
    #[error("block timestamp {timestamp} is in the future compared to our clock time {present_timestamp}")]
    TimestampIsInFuture {
        /// The block's timestamp.
        timestamp: u64,
        /// The current timestamp.
        present_timestamp: u64,
    },

    /// Error when the base fee is missing.
    #[error("base fee missing")]
    BaseFeeMissing,

    /// Error when there is a transaction signer recovery error.
    #[error("transaction signer recovery error")]
    TransactionSignerRecoveryError,

    /// Error when the extra data length exceeds the maximum allowed.
    #[error("extra data {len} exceeds max length")]
    ExtraDataExceedsMax {
        /// The length of the extra data.
        len: usize,
    },

    /// Error when the difficulty after a merge is not zero.
    #[error("difficulty after merge is not zero")]
    TheMergeDifficultyIsNotZero,

    /// Error when the nonce after a merge is not zero.
    #[error("nonce after merge is not zero")]
    TheMergeNonceIsNotZero,

    /// Error when the ommer root after a merge is not empty.
    #[error("ommer root after merge is not empty")]
    TheMergeOmmerRootIsNotEmpty,

    /// Error when the withdrawals root is missing.
    #[error("missing withdrawals root")]
    WithdrawalsRootMissing,

    /// Error when the requests root is missing.
    #[error("missing requests root")]
    RequestsRootMissing,

    /// Error when an unexpected withdrawals root is encountered.
    #[error("unexpected withdrawals root")]
    WithdrawalsRootUnexpected,

    /// Error when an unexpected requests root is encountered.
    #[error("unexpected requests root")]
    RequestsRootUnexpected,

    /// Error when withdrawals are missing.
    #[error("missing withdrawals")]
    BodyWithdrawalsMissing,

    /// Error when requests are missing.
    #[error("missing requests")]
    BodyRequestsMissing,

    /// Error when blob gas used is missing.
    #[error("missing blob gas used")]
    BlobGasUsedMissing,

    /// Error when unexpected blob gas used is encountered.
    #[error("unexpected blob gas used")]
    BlobGasUsedUnexpected,

    /// Error when excess blob gas is missing.
    #[error("missing excess blob gas")]
    ExcessBlobGasMissing,

    /// Error when unexpected excess blob gas is encountered.
    #[error("unexpected excess blob gas")]
    ExcessBlobGasUnexpected,

    /// Error when the parent beacon block root is missing.
    #[error("missing parent beacon block root")]
    ParentBeaconBlockRootMissing,

    /// Error when an unexpected parent beacon block root is encountered.
    #[error("unexpected parent beacon block root")]
    ParentBeaconBlockRootUnexpected,

    /// Error when blob gas used exceeds the maximum allowed.
    #[error("blob gas used {blob_gas_used} exceeds maximum allowance {max_blob_gas_per_block}")]
    BlobGasUsedExceedsMaxBlobGasPerBlock {
        /// The actual blob gas used.
        blob_gas_used: u64,
        /// The maximum allowed blob gas per block.
        max_blob_gas_per_block: u64,
    },

    /// Error when blob gas used is not a multiple of blob gas per blob.
    #[error(
        "blob gas used {blob_gas_used} is not a multiple of blob gas per blob {blob_gas_per_blob}"
    )]
    BlobGasUsedNotMultipleOfBlobGasPerBlob {
        /// The actual blob gas used.
        blob_gas_used: u64,
        /// The blob gas per blob.
        blob_gas_per_blob: u64,
    },

    /// Error when excess blob gas is not a multiple of blob gas per blob.
    #[error(
    "excess blob gas {excess_blob_gas} is not a multiple of blob gas per blob {blob_gas_per_blob}"
    )]
    ExcessBlobGasNotMultipleOfBlobGasPerBlob {
        /// The actual excess blob gas.
        excess_blob_gas: u64,
        /// The blob gas per blob.
        blob_gas_per_blob: u64,
    },

    /// Error when the blob gas used in the header does not match the expected blob gas used.
    #[error("blob gas used mismatch: {0}")]
    BlobGasUsedDiff(GotExpected<u64>),

    /// Error for invalid block difficulty
    #[error("invalid block difficulty: {difficulty}")]
    InvalidDifficulty {
        /// The block difficulty
        difficulty: U256,
    },

    /// Error for invalid mix hash
    #[error("invalid mix digest")]
    InvalidMixHash,

    /// Error for invalid header extra
    #[error("invalid header extra")]
    InvalidHeaderExtra,

    /// Error for a transaction that violates consensus.
    #[error(transparent)]
    InvalidTransaction(#[from] InvalidTransactionError),

    /// Error when the block's base fee is different from the expected base fee.
    #[error("block base fee mismatch: {0}")]
    BaseFeeDiff(GotExpected<u64>),

    /// Error when there is an invalid excess blob gas.
    #[error(
        "invalid excess blob gas: {diff}; \
            parent excess blob gas: {parent_excess_blob_gas}, \
            parent blob gas used: {parent_blob_gas_used}"
    )]
    ExcessBlobGasDiff {
        /// The excess blob gas diff.
        diff: GotExpected<u64>,
        /// The parent excess blob gas.
        parent_excess_blob_gas: u64,
        /// The parent blob gas used.
        parent_blob_gas_used: u64,
    },

    /// Error when the child gas limit exceeds the maximum allowed increase.
    #[error("child gas_limit {child_gas_limit} max increase is {parent_gas_limit}/1024")]
    GasLimitInvalidIncrease {
        /// The parent gas limit.
        parent_gas_limit: u64,
        /// The child gas limit.
        child_gas_limit: u64,
    },

    /// Error indicating that the child gas limit is below the minimum allowed limit.
    ///
    /// This error occurs when the child gas limit is less than the specified minimum gas limit.
    #[error("child gas limit {child_gas_limit} is below the minimum allowed limit ({MINIMUM_GAS_LIMIT})")]
    GasLimitInvalidMinimum {
        /// The child gas limit.
        child_gas_limit: u64,
    },

    /// Error when the child gas limit exceeds the maximum allowed decrease.
    #[error("child gas_limit {child_gas_limit} max decrease is {parent_gas_limit}/1024")]
    GasLimitInvalidDecrease {
        /// The parent gas limit.
        parent_gas_limit: u64,
        /// The child gas limit.
        child_gas_limit: u64,
    },

    /// Error when the block timestamp is in the past compared to the parent timestamp.
    #[error("block timestamp {timestamp} is in the past compared to the parent timestamp {parent_timestamp}")]
    TimestampIsInPast {
        /// The parent block's timestamp.
        parent_timestamp: u64,
        /// The block's timestamp.
        timestamp: u64,
    },
}

impl ConsensusError {
    /// Returns `true` if the error is a state root error.
    pub const fn is_state_root_error(&self) -> bool {
        matches!(self, Self::BodyStateRootDiff(_))
    }
}

/// `HeaderConsensusError` combines a `ConsensusError` with the `SealedHeader` it relates to.
#[derive(thiserror_no_std::Error, Debug)]
#[error("Consensus error: {0}, Invalid header: {1:?}")]
pub struct HeaderConsensusError(ConsensusError, SealedHeader);
