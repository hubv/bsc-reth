use crate::{segments::SegmentSet, Pruner};
use reth_chainspec::MAINNET;
use reth_config::PruneConfig;
use reth_db_api::database::Database;
use reth_exex_types::FinishedExExHeight;
use reth_provider::{providers::StaticFileProvider, ProviderFactory, StaticFileProviderFactory};
use reth_prune_types::PruneModes;
use std::time::Duration;
use tokio::sync::watch;

/// Contains the information required to build a pruner
#[derive(Debug, Clone)]
pub struct PrunerBuilder {
    /// Minimum pruning interval measured in blocks.
    block_interval: usize,
    /// Pruning configuration for every part of the data that can be pruned.
    segments: PruneModes,
    /// The delete limit for pruner, per run.
    delete_limit: usize,
    /// Time a pruner job can run before timing out.
    timeout: Option<Duration>,
    /// The finished height of all `ExEx`'s.
    finished_exex_height: watch::Receiver<FinishedExExHeight>,
    /// The number of recent sidecars to keep in the static file provider.
    recent_sidecars_kept_blocks: usize,
}

impl PrunerBuilder {
    /// Default timeout for a prune run.
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_millis(100);

    /// Creates a new [`PrunerBuilder`] from the given [`PruneConfig`].
    pub fn new(pruner_config: PruneConfig) -> Self {
        Self::default()
            .block_interval(pruner_config.block_interval)
            .segments(pruner_config.segments)
            .recent_sidecars_kept_blocks(pruner_config.recent_sidecars_kept_blocks)
    }

    /// Sets the minimum pruning interval measured in blocks.
    pub const fn block_interval(mut self, block_interval: usize) -> Self {
        self.block_interval = block_interval;
        self
    }

    /// Sets the configuration for every part of the data that can be pruned.
    pub fn segments(mut self, segments: PruneModes) -> Self {
        self.segments = segments;
        self
    }

    /// Sets the delete limit for pruner, per run.
    pub const fn delete_limit(mut self, prune_delete_limit: usize) -> Self {
        self.delete_limit = prune_delete_limit;
        self
    }

    /// Sets the timeout for pruner, per run.
    ///
    /// CAUTION: Account and Storage History prune segments treat this timeout as a soft limit,
    /// meaning they can go beyond it.
    pub const fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Sets the receiver for the finished height of all `ExEx`'s.
    pub fn finished_exex_height(
        mut self,
        finished_exex_height: watch::Receiver<FinishedExExHeight>,
    ) -> Self {
        self.finished_exex_height = finished_exex_height;
        self
    }

    /// Sets the number of recent sidecars to keep in the static file provider.
    pub const fn recent_sidecars_kept_blocks(mut self, recent_sidecars_kept_blocks: usize) -> Self {
        self.recent_sidecars_kept_blocks = recent_sidecars_kept_blocks;
        self
    }

    /// Builds a [Pruner] from the current configuration with the given provider factory.
    pub fn build_with_provider_factory<DB: Database>(
        self,
        provider_factory: ProviderFactory<DB>,
    ) -> Pruner<DB, ProviderFactory<DB>> {
        let segments = SegmentSet::<DB>::from_components(
            provider_factory.static_file_provider(),
            self.segments,
        );

        Pruner::<_, ProviderFactory<DB>>::new(
            provider_factory,
            segments.into_vec(),
            self.block_interval,
            self.delete_limit,
            self.timeout,
            self.finished_exex_height,
            self.recent_sidecars_kept_blocks,
        )
    }

    /// Builds a [Pruner] from the current configuration with the given static file provider.
    pub fn build<DB: Database>(self, static_file_provider: StaticFileProvider) -> Pruner<DB, ()> {
        let segments = SegmentSet::<DB>::from_components(static_file_provider, self.segments);

        Pruner::<_, ()>::new(
            segments.into_vec(),
            self.block_interval,
            self.delete_limit,
            self.timeout,
            self.finished_exex_height,
            self.recent_sidecars_kept_blocks,
        )
    }
}

impl Default for PrunerBuilder {
    fn default() -> Self {
        Self {
            block_interval: 5,
            segments: PruneModes::none(),
            delete_limit: MAINNET.prune_delete_limit,
            timeout: None,
            finished_exex_height: watch::channel(FinishedExExHeight::NoExExs).1,
            recent_sidecars_kept_blocks: 0, /* not enabled by default
                                             * recent_sidecars_kept_blocks: 518400, // 18 days */
        }
    }
}
