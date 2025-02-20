//! Helper provider traits to encapsulate all provider traits for simplicity.

use crate::{
    AccountReader, BlockReaderIdExt, CanonStateSubscriptions, ChainSpecProvider, ChangeSetReader,
    DatabaseProviderFactory, EvmEnvProvider, HeaderProvider, ParliaSnapshotReader,
    StageCheckpointReader, StateProviderFactory, StaticFileProviderFactory, TransactionsProvider,
};
use reth_db_api::database::Database;

/// Helper trait to unify all provider traits for simplicity.
pub trait FullProvider<DB: Database>:
    DatabaseProviderFactory<DB>
    + StaticFileProviderFactory
    + BlockReaderIdExt
    + AccountReader
    + StateProviderFactory
    + EvmEnvProvider
    + ChainSpecProvider
    + ChangeSetReader
    + CanonStateSubscriptions
    + StageCheckpointReader
    + HeaderProvider
    + ParliaSnapshotReader
    + Clone
    + Unpin
    + 'static
{
}

impl<T, DB: Database> FullProvider<DB> for T where
    T: DatabaseProviderFactory<DB>
        + StaticFileProviderFactory
        + BlockReaderIdExt
        + AccountReader
        + StateProviderFactory
        + EvmEnvProvider
        + ChainSpecProvider
        + ChangeSetReader
        + CanonStateSubscriptions
        + StageCheckpointReader
        + HeaderProvider
        + ParliaSnapshotReader
        + Clone
        + Unpin
        + 'static
{
}

/// Helper trait to unify all provider traits required to support `eth` RPC server behaviour, for
/// simplicity.
pub trait FullRpcProvider:
    StateProviderFactory
    + EvmEnvProvider
    + ChainSpecProvider
    + BlockReaderIdExt
    + HeaderProvider
    + TransactionsProvider
    + Clone
    + Unpin
    + 'static
{
}

impl<T> FullRpcProvider for T where
    T: StateProviderFactory
        + EvmEnvProvider
        + ChainSpecProvider
        + BlockReaderIdExt
        + HeaderProvider
        + TransactionsProvider
        + Clone
        + Unpin
        + 'static
{
}
