use crate::{hardfork, ChainHardforks, EthereumHardfork, ForkCondition, Hardfork};
use alloy_chains::Chain;
use alloy_primitives::U256;
use core::{
    any::Any,
    fmt::{self, Display, Formatter},
    str::FromStr,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, format, string::String, vec};

hardfork!(
    /// The name of an optimism hardfork.
    ///
    /// When building a list of hardforks for a chain, it's still expected to mix with [`EthereumHardfork`].
    OptimismHardfork {
        /// Bedrock: <https://blog.oplabs.co/introducing-optimism-bedrock>.
        Bedrock,
        /// Regolith: <https://github.com/ethereum-optimism/specs/blob/main/specs/protocol/superchain-upgrades.md#regolith>.
        Regolith,
        /// `Fermat`
        Fermat,
        /// <https://github.com/ethereum-optimism/specs/blob/main/specs/protocol/superchain-upgrades.md#canyon>.
        Canyon,
        /// Ecotone: <https://github.com/ethereum-optimism/specs/blob/main/specs/protocol/superchain-upgrades.md#ecotone>.
        Ecotone,
        /// `PreContractForkBlock`
        PreContractForkBlock,
        /// `Haber`
        Haber,
        /// `Wright`
        Wright,
        /// Fjord: <https://github.com/ethereum-optimism/specs/blob/main/specs/protocol/superchain-upgrades.md#fjord>
        Fjord,
    }
);

impl OptimismHardfork {
    /// Retrieves the activation block for the specified hardfork on the given chain.
    pub fn activation_block<H: Hardfork>(self, fork: H, chain: Chain) -> Option<u64> {
        if chain == Chain::base_sepolia() {
            return Self::base_sepolia_activation_block(fork)
        }
        if chain == Chain::base_mainnet() {
            return Self::base_mainnet_activation_block(fork)
        }
        if chain == Chain::opbnb_mainnet() {
            return Self::opbnb_mainnet_activation_block(fork)
        }
        if chain == Chain::opbnb_testnet() {
            return Self::opbnb_testnet_activation_block(fork)
        }

        None
    }

    /// Retrieves the activation timestamp for the specified hardfork on the given chain.
    pub fn activation_timestamp<H: Hardfork>(self, fork: H, chain: Chain) -> Option<u64> {
        if chain == Chain::base_sepolia() {
            return Self::base_sepolia_activation_timestamp(fork)
        }
        if chain == Chain::base_mainnet() {
            return Self::base_mainnet_activation_timestamp(fork)
        }
        if chain == Chain::opbnb_mainnet() {
            return Self::opbnb_mainnet_activation_timestamp(fork)
        }
        if chain == Chain::opbnb_testnet() {
            return Self::opbnb_testnet_activation_timestamp(fork)
        }

        None
    }

    /// Retrieves the activation block for the specified hardfork on the Base Sepolia testnet.
    pub fn base_sepolia_activation_block<H: Hardfork>(fork: H) -> Option<u64> {
        match_hardfork(
            fork,
            |fork| match fork {
                EthereumHardfork::Frontier |
                EthereumHardfork::Homestead |
                EthereumHardfork::Dao |
                EthereumHardfork::Tangerine |
                EthereumHardfork::SpuriousDragon |
                EthereumHardfork::Byzantium |
                EthereumHardfork::Constantinople |
                EthereumHardfork::Petersburg |
                EthereumHardfork::Istanbul |
                EthereumHardfork::MuirGlacier |
                EthereumHardfork::Berlin |
                EthereumHardfork::London |
                EthereumHardfork::ArrowGlacier |
                EthereumHardfork::GrayGlacier |
                EthereumHardfork::Paris |
                EthereumHardfork::Shanghai => Some(2106456),
                EthereumHardfork::Cancun => Some(6383256),
                _ => None,
            },
            |fork| match fork {
                Self::Bedrock | Self::Regolith => Some(0),
                Self::Canyon => Some(2106456),
                Self::Ecotone => Some(6383256),
                Self::Fjord => Some(10615056),
                _ => None,
            },
        )
    }

    /// Retrieves the activation block for the specified hardfork on the Base mainnet.
    pub fn base_mainnet_activation_block<H: Hardfork>(fork: H) -> Option<u64> {
        match_hardfork(
            fork,
            |fork| match fork {
                EthereumHardfork::Frontier |
                EthereumHardfork::Homestead |
                EthereumHardfork::Dao |
                EthereumHardfork::Tangerine |
                EthereumHardfork::SpuriousDragon |
                EthereumHardfork::Byzantium |
                EthereumHardfork::Constantinople |
                EthereumHardfork::Petersburg |
                EthereumHardfork::Istanbul |
                EthereumHardfork::MuirGlacier |
                EthereumHardfork::Berlin |
                EthereumHardfork::London |
                EthereumHardfork::ArrowGlacier |
                EthereumHardfork::GrayGlacier |
                EthereumHardfork::Paris |
                EthereumHardfork::Shanghai => Some(9101527),
                EthereumHardfork::Cancun => Some(11188936),
                _ => None,
            },
            |fork| match fork {
                Self::Bedrock | Self::Regolith => Some(0),
                Self::Canyon => Some(9101527),
                Self::Ecotone => Some(11188936),
                _ => None,
            },
        )
    }

    /// Retrieves the activation block for the specified hardfork on the opBNB mainnet.
    pub fn opbnb_mainnet_activation_block<H: Hardfork>(fork: H) -> Option<u64> {
        match_hardfork(
            fork,
            |fork| match fork {
                EthereumHardfork::Frontier |
                EthereumHardfork::Homestead |
                EthereumHardfork::Tangerine |
                EthereumHardfork::SpuriousDragon |
                EthereumHardfork::Byzantium |
                EthereumHardfork::Constantinople |
                EthereumHardfork::Petersburg |
                EthereumHardfork::Istanbul |
                EthereumHardfork::MuirGlacier |
                EthereumHardfork::Berlin |
                EthereumHardfork::London |
                EthereumHardfork::ArrowGlacier |
                EthereumHardfork::GrayGlacier |
                EthereumHardfork::Paris => Some(0),
                _ => None,
            },
            |fork| match fork {
                Self::Bedrock => Some(0),
                _ => None,
            },
        )
    }

    /// Retrieves the activation block for the specified hardfork on the opBNB testnet.
    pub fn opbnb_testnet_activation_block<H: Hardfork>(fork: H) -> Option<u64> {
        match_hardfork(
            fork,
            |fork| match fork {
                EthereumHardfork::Frontier |
                EthereumHardfork::Homestead |
                EthereumHardfork::Tangerine |
                EthereumHardfork::SpuriousDragon |
                EthereumHardfork::Byzantium |
                EthereumHardfork::Constantinople |
                EthereumHardfork::Petersburg |
                EthereumHardfork::Istanbul |
                EthereumHardfork::MuirGlacier |
                EthereumHardfork::Berlin |
                EthereumHardfork::London |
                EthereumHardfork::ArrowGlacier |
                EthereumHardfork::GrayGlacier |
                EthereumHardfork::Paris => Some(0),
                _ => None,
            },
            |fork| match fork {
                Self::Bedrock => Some(0),
                Self::PreContractForkBlock => Some(5805494),
                _ => None,
            },
        )
    }

    /// Retrieves the activation timestamp for the specified hardfork on the Base Sepolia testnet.
    pub fn base_sepolia_activation_timestamp<H: Hardfork>(fork: H) -> Option<u64> {
        match_hardfork(
            fork,
            |fork| match fork {
                EthereumHardfork::Frontier |
                EthereumHardfork::Homestead |
                EthereumHardfork::Dao |
                EthereumHardfork::Tangerine |
                EthereumHardfork::SpuriousDragon |
                EthereumHardfork::Byzantium |
                EthereumHardfork::Constantinople |
                EthereumHardfork::Petersburg |
                EthereumHardfork::Istanbul |
                EthereumHardfork::MuirGlacier |
                EthereumHardfork::Berlin |
                EthereumHardfork::London |
                EthereumHardfork::ArrowGlacier |
                EthereumHardfork::GrayGlacier |
                EthereumHardfork::Paris |
                EthereumHardfork::Shanghai => Some(1699981200),
                EthereumHardfork::Cancun => Some(1708534800),
                _ => None,
            },
            |fork| match fork {
                Self::Bedrock | Self::Regolith => Some(1695768288),
                Self::Canyon => Some(1699981200),
                Self::Ecotone => Some(1708534800),
                Self::Fjord => Some(1716998400),
                _ => None,
            },
        )
    }

    /// Retrieves the activation timestamp for the specified hardfork on the Base mainnet.
    pub fn base_mainnet_activation_timestamp<H: Hardfork>(fork: H) -> Option<u64> {
        match_hardfork(
            fork,
            |fork| match fork {
                EthereumHardfork::Frontier |
                EthereumHardfork::Homestead |
                EthereumHardfork::Dao |
                EthereumHardfork::Tangerine |
                EthereumHardfork::SpuriousDragon |
                EthereumHardfork::Byzantium |
                EthereumHardfork::Constantinople |
                EthereumHardfork::Petersburg |
                EthereumHardfork::Istanbul |
                EthereumHardfork::MuirGlacier |
                EthereumHardfork::Berlin |
                EthereumHardfork::London |
                EthereumHardfork::ArrowGlacier |
                EthereumHardfork::GrayGlacier |
                EthereumHardfork::Paris |
                EthereumHardfork::Shanghai => Some(1704992401),
                EthereumHardfork::Cancun => Some(1710374401),
                _ => None,
            },
            |fork| match fork {
                Self::Bedrock | Self::Regolith => Some(1686789347),
                Self::Canyon => Some(1704992401),
                Self::Ecotone => Some(1710374401),
                Self::Fjord => Some(1720627201),
                _ => None,
            },
        )
    }

    /// Retrieves the activation timestamp for the specified hardfork on the opBNB mainnet.
    pub fn opbnb_mainnet_activation_timestamp<H: Hardfork>(fork: H) -> Option<u64> {
        match_hardfork(
            fork,
            |fork| match fork {
                EthereumHardfork::Shanghai => Some(1718870400),
                EthereumHardfork::Cancun => Some(1718871600),
                _ => None,
            },
            |fork| match fork {
                Self::Regolith => Some(0),
                Self::Fermat => Some(1701151200),
                Self::Canyon => Some(1718870400),
                Self::Ecotone => Some(1718871600),
                Self::Haber => Some(1718872200),
                _ => None,
            },
        )
    }

    /// Retrieves the activation timestamp for the specified hardfork on the opBNB testnet.
    pub fn opbnb_testnet_activation_timestamp<H: Hardfork>(fork: H) -> Option<u64> {
        match_hardfork(
            fork,
            |fork| match fork {
                EthereumHardfork::Shanghai => Some(1715753400),
                EthereumHardfork::Cancun => Some(1715754600),
                _ => None,
            },
            |fork| match fork {
                Self::Regolith => Some(0),
                Self::Fermat => Some(1698991506),
                Self::Canyon => Some(1715753400),
                Self::Ecotone => Some(1715754600),
                Self::Haber => Some(1717048800),
                _ => None,
            },
        )
    }

    /// Optimism mainnet list of hardforks.
    pub fn op_mainnet() -> ChainHardforks {
        ChainHardforks::new(vec![
            (EthereumHardfork::Frontier.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Homestead.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Tangerine.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::SpuriousDragon.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Byzantium.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Constantinople.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Petersburg.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Istanbul.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::MuirGlacier.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Berlin.boxed(), ForkCondition::Block(3950000)),
            (EthereumHardfork::London.boxed(), ForkCondition::Block(105235063)),
            (EthereumHardfork::ArrowGlacier.boxed(), ForkCondition::Block(105235063)),
            (EthereumHardfork::GrayGlacier.boxed(), ForkCondition::Block(105235063)),
            (
                EthereumHardfork::Paris.boxed(),
                ForkCondition::TTD { fork_block: Some(105235063), total_difficulty: U256::ZERO },
            ),
            (Self::Bedrock.boxed(), ForkCondition::Block(105235063)),
            (Self::Regolith.boxed(), ForkCondition::Timestamp(0)),
            (EthereumHardfork::Shanghai.boxed(), ForkCondition::Timestamp(1704992401)),
            (Self::Canyon.boxed(), ForkCondition::Timestamp(1704992401)),
            (EthereumHardfork::Cancun.boxed(), ForkCondition::Timestamp(1710374401)),
            (Self::Ecotone.boxed(), ForkCondition::Timestamp(1710374401)),
            (Self::Fjord.boxed(), ForkCondition::Timestamp(1720627201)),
        ])
    }

    /// Optimism sepolia list of hardforks.
    pub fn op_sepolia() -> ChainHardforks {
        ChainHardforks::new(vec![
            (EthereumHardfork::Frontier.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Homestead.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Tangerine.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::SpuriousDragon.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Byzantium.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Constantinople.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Petersburg.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Istanbul.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::MuirGlacier.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Berlin.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::London.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::ArrowGlacier.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::GrayGlacier.boxed(), ForkCondition::Block(0)),
            (
                EthereumHardfork::Paris.boxed(),
                ForkCondition::TTD { fork_block: Some(0), total_difficulty: U256::ZERO },
            ),
            (Self::Bedrock.boxed(), ForkCondition::Block(0)),
            (Self::Regolith.boxed(), ForkCondition::Timestamp(0)),
            (EthereumHardfork::Shanghai.boxed(), ForkCondition::Timestamp(1699981200)),
            (Self::Canyon.boxed(), ForkCondition::Timestamp(1699981200)),
            (EthereumHardfork::Cancun.boxed(), ForkCondition::Timestamp(1708534800)),
            (Self::Ecotone.boxed(), ForkCondition::Timestamp(1708534800)),
            (Self::Fjord.boxed(), ForkCondition::Timestamp(1716998400)),
        ])
    }

    /// Base sepolia list of hardforks.
    pub fn base_sepolia() -> ChainHardforks {
        ChainHardforks::new(vec![
            (EthereumHardfork::Frontier.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Homestead.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Tangerine.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::SpuriousDragon.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Byzantium.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Constantinople.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Petersburg.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Istanbul.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::MuirGlacier.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Berlin.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::London.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::ArrowGlacier.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::GrayGlacier.boxed(), ForkCondition::Block(0)),
            (
                EthereumHardfork::Paris.boxed(),
                ForkCondition::TTD { fork_block: Some(0), total_difficulty: U256::ZERO },
            ),
            (Self::Bedrock.boxed(), ForkCondition::Block(0)),
            (Self::Regolith.boxed(), ForkCondition::Timestamp(0)),
            (EthereumHardfork::Shanghai.boxed(), ForkCondition::Timestamp(1699981200)),
            (Self::Canyon.boxed(), ForkCondition::Timestamp(1699981200)),
            (EthereumHardfork::Cancun.boxed(), ForkCondition::Timestamp(1708534800)),
            (Self::Ecotone.boxed(), ForkCondition::Timestamp(1708534800)),
            (Self::Fjord.boxed(), ForkCondition::Timestamp(1716998400)),
        ])
    }

    /// Base mainnet list of hardforks.
    pub fn base_mainnet() -> ChainHardforks {
        ChainHardforks::new(vec![
            (EthereumHardfork::Frontier.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Homestead.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Tangerine.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::SpuriousDragon.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Byzantium.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Constantinople.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Petersburg.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Istanbul.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::MuirGlacier.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Berlin.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::London.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::ArrowGlacier.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::GrayGlacier.boxed(), ForkCondition::Block(0)),
            (
                EthereumHardfork::Paris.boxed(),
                ForkCondition::TTD { fork_block: Some(0), total_difficulty: U256::ZERO },
            ),
            (Self::Bedrock.boxed(), ForkCondition::Block(0)),
            (Self::Regolith.boxed(), ForkCondition::Timestamp(0)),
            (EthereumHardfork::Shanghai.boxed(), ForkCondition::Timestamp(1704992401)),
            (Self::Canyon.boxed(), ForkCondition::Timestamp(1704992401)),
            (EthereumHardfork::Cancun.boxed(), ForkCondition::Timestamp(1710374401)),
            (Self::Ecotone.boxed(), ForkCondition::Timestamp(1710374401)),
            (Self::Fjord.boxed(), ForkCondition::Timestamp(1720627201)),
        ])
    }

    /// opBNB mainnet list of hardforks.
    pub fn opbnb_mainnet() -> ChainHardforks {
        ChainHardforks::new(vec![
            (EthereumHardfork::Frontier.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Homestead.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Tangerine.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::SpuriousDragon.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Byzantium.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Constantinople.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Petersburg.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Istanbul.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::MuirGlacier.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Berlin.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::London.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::ArrowGlacier.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::GrayGlacier.boxed(), ForkCondition::Block(0)),
            (
                EthereumHardfork::Paris.boxed(),
                ForkCondition::TTD { fork_block: Some(0), total_difficulty: U256::ZERO },
            ),
            (Self::Bedrock.boxed(), ForkCondition::Block(0)),
            (Self::Regolith.boxed(), ForkCondition::Timestamp(0)),
            (Self::Fermat.boxed(), ForkCondition::Timestamp(1701151200)),
            (EthereumHardfork::Shanghai.boxed(), ForkCondition::Timestamp(1718870400)),
            (Self::Canyon.boxed(), ForkCondition::Timestamp(1718870400)),
            (EthereumHardfork::Cancun.boxed(), ForkCondition::Timestamp(1718871600)),
            (Self::Ecotone.boxed(), ForkCondition::Timestamp(1718871600)),
            (Self::Haber.boxed(), ForkCondition::Timestamp(1718872200)),
            (Self::Wright.boxed(), ForkCondition::Timestamp(1724738400)),
        ])
    }

    /// opBNB testnet list of hardforks.
    pub fn opbnb_testnet() -> ChainHardforks {
        ChainHardforks::new(vec![
            (EthereumHardfork::Frontier.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Homestead.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Tangerine.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::SpuriousDragon.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Byzantium.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Constantinople.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Petersburg.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Istanbul.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::MuirGlacier.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Berlin.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::London.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::ArrowGlacier.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::GrayGlacier.boxed(), ForkCondition::Block(0)),
            (
                EthereumHardfork::Paris.boxed(),
                ForkCondition::TTD { fork_block: Some(0), total_difficulty: U256::ZERO },
            ),
            (Self::Bedrock.boxed(), ForkCondition::Block(0)),
            (Self::Regolith.boxed(), ForkCondition::Timestamp(0)),
            (Self::PreContractForkBlock.boxed(), ForkCondition::Block(5805494)),
            (Self::Fermat.boxed(), ForkCondition::Timestamp(1698991506)),
            (EthereumHardfork::Shanghai.boxed(), ForkCondition::Timestamp(1715753400)),
            (Self::Canyon.boxed(), ForkCondition::Timestamp(1715753400)),
            (EthereumHardfork::Cancun.boxed(), ForkCondition::Timestamp(1715754600)),
            (Self::Ecotone.boxed(), ForkCondition::Timestamp(1715754600)),
            (Self::Haber.boxed(), ForkCondition::Timestamp(1717048800)),
            (Self::Wright.boxed(), ForkCondition::Timestamp(1723701600)),
        ])
    }

    /// opBNB qa network list of hardforks.
    pub fn opbnb_qa() -> ChainHardforks {
        ChainHardforks::new(vec![
            (EthereumHardfork::Frontier.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Homestead.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Tangerine.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::SpuriousDragon.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Byzantium.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Constantinople.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Petersburg.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Istanbul.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::MuirGlacier.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Berlin.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::London.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::ArrowGlacier.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::GrayGlacier.boxed(), ForkCondition::Block(0)),
            (
                EthereumHardfork::Paris.boxed(),
                ForkCondition::TTD { fork_block: Some(0), total_difficulty: U256::ZERO },
            ),
            (Self::Bedrock.boxed(), ForkCondition::Block(0)),
            (Self::Regolith.boxed(), ForkCondition::Timestamp(0)),
            (Self::Fermat.boxed(), ForkCondition::Timestamp(0)),
            (EthereumHardfork::Shanghai.boxed(), ForkCondition::Timestamp(0)),
            (Self::Canyon.boxed(), ForkCondition::Timestamp(0)),
            (EthereumHardfork::Cancun.boxed(), ForkCondition::Timestamp(0)),
            (Self::Ecotone.boxed(), ForkCondition::Timestamp(0)),
            (Self::Wright.boxed(), ForkCondition::Timestamp(0)),
        ])
    }
}

/// Match helper method since it's not possible to match on `dyn Hardfork`
fn match_hardfork<H, HF, OHF>(fork: H, hardfork_fn: HF, optimism_hardfork_fn: OHF) -> Option<u64>
where
    H: Hardfork,
    HF: Fn(&EthereumHardfork) -> Option<u64>,
    OHF: Fn(&OptimismHardfork) -> Option<u64>,
{
    let fork: &dyn Any = &fork;
    if let Some(fork) = fork.downcast_ref::<EthereumHardfork>() {
        return hardfork_fn(fork)
    }
    fork.downcast_ref::<OptimismHardfork>().and_then(optimism_hardfork_fn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_hardfork() {
        assert_eq!(
            OptimismHardfork::base_mainnet_activation_block(EthereumHardfork::Cancun),
            Some(11188936)
        );
        assert_eq!(
            OptimismHardfork::base_mainnet_activation_block(OptimismHardfork::Canyon),
            Some(9101527)
        );
    }
}
