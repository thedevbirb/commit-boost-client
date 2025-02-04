use std::path::PathBuf;

use alloy::primitives::{hex, Bytes};
use derive_more::{Deref, Display, From, Into};
use eyre::{bail, Context};
use serde::{Deserialize, Serialize};

use crate::{constants::APPLICATION_BUILDER_DOMAIN, signature::compute_domain};

#[derive(Clone, Debug, Display, PartialEq, Eq, Hash, Deref, From, Into, Serialize, Deserialize)]
#[into(owned, ref, ref_mut)]
#[serde(transparent)]
pub struct ModuleId(pub String);

#[derive(Clone, Debug, Display, PartialEq, Eq, Hash, Deref, From, Into, Serialize, Deserialize)]
#[into(owned, ref, ref_mut)]
#[serde(transparent)]
pub struct Jwt(pub String);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Chain {
    Mainnet,
    Holesky,
    Helder,
    Custom { genesis_time_secs: u64, slot_time_secs: u64, genesis_fork_version: [u8; 4] },
}

impl std::fmt::Debug for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mainnet => write!(f, "Mainnet"),
            Self::Holesky => write!(f, "Holesky"),
            Self::Helder => write!(f, "Helder"),
            Self::Custom { genesis_time_secs, slot_time_secs, genesis_fork_version } => f
                .debug_struct("Custom")
                .field("genesis_time_secs", genesis_time_secs)
                .field("slot_time_secs", slot_time_secs)
                .field("genesis_fork_version", &hex::encode_prefixed(genesis_fork_version))
                .finish(),
        }
    }
}

impl Chain {
    pub fn builder_domain(&self) -> [u8; 32] {
        match self {
            Chain::Mainnet => KnownChain::Mainnet.builder_domain(),
            Chain::Holesky => KnownChain::Holesky.builder_domain(),
            Chain::Helder => KnownChain::Helder.builder_domain(),
            Chain::Custom { .. } => compute_domain(*self, APPLICATION_BUILDER_DOMAIN),
        }
    }

    pub fn genesis_fork_version(&self) -> [u8; 4] {
        match self {
            Chain::Mainnet => KnownChain::Mainnet.genesis_fork_version(),
            Chain::Holesky => KnownChain::Holesky.genesis_fork_version(),
            Chain::Helder => KnownChain::Helder.genesis_fork_version(),
            Chain::Custom { genesis_fork_version, .. } => *genesis_fork_version,
        }
    }

    pub fn genesis_time_sec(&self) -> u64 {
        match self {
            Chain::Mainnet => KnownChain::Mainnet.genesis_time_sec(),
            Chain::Holesky => KnownChain::Holesky.genesis_time_sec(),
            Chain::Helder => KnownChain::Helder.genesis_time_sec(),
            Chain::Custom { genesis_time_secs, .. } => *genesis_time_secs,
        }
    }

    pub fn slot_time_sec(&self) -> u64 {
        match self {
            Chain::Mainnet => KnownChain::Mainnet.slot_time_sec(),
            Chain::Holesky => KnownChain::Holesky.slot_time_sec(),
            Chain::Helder => KnownChain::Helder.slot_time_sec(),
            Chain::Custom { slot_time_secs, .. } => *slot_time_secs,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum KnownChain {
    #[serde(alias = "mainnet")]
    Mainnet,
    #[serde(alias = "holesky")]
    Holesky,
    #[serde(alias = "helder")]
    Helder,
}

// Constants
impl KnownChain {
    pub fn builder_domain(&self) -> [u8; 32] {
        match self {
            KnownChain::Mainnet => [
                0, 0, 0, 1, 245, 165, 253, 66, 209, 106, 32, 48, 39, 152, 239, 110, 211, 9, 151,
                155, 67, 0, 61, 35, 32, 217, 240, 232, 234, 152, 49, 169,
            ],
            KnownChain::Holesky => [
                0, 0, 0, 1, 91, 131, 162, 55, 89, 197, 96, 178, 208, 198, 69, 118, 225, 220, 252,
                52, 234, 148, 196, 152, 143, 62, 13, 159, 119, 240, 83, 135,
            ],
            KnownChain::Helder => [
                0, 0, 0, 1, 148, 196, 26, 244, 132, 255, 247, 150, 73, 105, 224, 189, 217, 34, 248,
                45, 255, 15, 75, 232, 122, 96, 208, 102, 76, 201, 209, 255,
            ],
        }
    }

    pub fn genesis_fork_version(&self) -> [u8; 4] {
        match self {
            KnownChain::Mainnet => [0u8; 4],
            KnownChain::Holesky => [1, 1, 112, 0],
            KnownChain::Helder => [16, 0, 0, 0],
        }
    }

    fn genesis_time_sec(&self) -> u64 {
        match self {
            KnownChain::Mainnet => 1606824023,
            KnownChain::Holesky => 1695902400,
            KnownChain::Helder => 1718967660,
        }
    }

    pub fn slot_time_sec(&self) -> u64 {
        match self {
            KnownChain::Mainnet | KnownChain::Holesky | KnownChain::Helder => 12,
        }
    }
}

impl From<KnownChain> for Chain {
    fn from(value: KnownChain) -> Self {
        match value {
            KnownChain::Mainnet => Chain::Mainnet,
            KnownChain::Holesky => Chain::Holesky,
            KnownChain::Helder => Chain::Helder,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum ChainLoader {
    Known(KnownChain),
    Path(PathBuf),
    Custom { genesis_time_secs: u64, slot_time_secs: u64, genesis_fork_version: Bytes },
}

impl Serialize for Chain {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let loader = match self {
            Chain::Mainnet => ChainLoader::Known(KnownChain::Mainnet),
            Chain::Holesky => ChainLoader::Known(KnownChain::Holesky),
            Chain::Helder => ChainLoader::Known(KnownChain::Helder),
            Chain::Custom { genesis_time_secs, slot_time_secs, genesis_fork_version } => {
                ChainLoader::Custom {
                    genesis_time_secs: *genesis_time_secs,
                    slot_time_secs: *slot_time_secs,
                    genesis_fork_version: Bytes::from(*genesis_fork_version),
                }
            }
        };

        loader.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Chain {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let loader = ChainLoader::deserialize(deserializer)?;

        match loader {
            ChainLoader::Known(known) => Ok(Chain::from(known)),
            ChainLoader::Path(path) => load_chain_from_file(path).map_err(serde::de::Error::custom),
            ChainLoader::Custom { genesis_time_secs, slot_time_secs, genesis_fork_version } => {
                let genesis_fork_version: [u8; 4] =
                    genesis_fork_version.as_ref().try_into().map_err(serde::de::Error::custom)?;
                Ok(Chain::Custom { genesis_time_secs, slot_time_secs, genesis_fork_version })
            }
        }
    }
}

/// Load a chain config from a spec file, such as returned by
/// /eth/v1/config/spec ref: https://ethereum.github.io/beacon-APIs/#/Config/getSpec
/// Try to load two formats:
/// - JSON as return the getSpec endpoint, either with or without the `data`
///   field
/// - YAML as used e.g. in Kurtosis/Ethereum Package
pub fn load_chain_from_file(path: PathBuf) -> eyre::Result<Chain> {
    #[derive(Deserialize)]
    #[serde(rename_all = "UPPERCASE")]
    struct QuotedSpecFile {
        #[serde(with = "serde_utils::quoted_u64")]
        min_genesis_time: u64,
        #[serde(with = "serde_utils::quoted_u64")]
        genesis_delay: u64,
        #[serde(with = "serde_utils::quoted_u64")]
        seconds_per_slot: u64,
        genesis_fork_version: Bytes,
    }

    impl QuotedSpecFile {
        fn to_chain(&self) -> eyre::Result<Chain> {
            let genesis_fork_version: [u8; 4] = self.genesis_fork_version.as_ref().try_into()?;

            Ok(Chain::Custom {
                genesis_time_secs: self.min_genesis_time + self.genesis_delay,
                slot_time_secs: self.seconds_per_slot,
                genesis_fork_version,
            })
        }
    }

    #[derive(Deserialize)]
    struct SpecFileJson {
        data: QuotedSpecFile,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "UPPERCASE")]
    struct SpecFile {
        min_genesis_time: u64,
        genesis_delay: u64,
        seconds_per_slot: u64,
        genesis_fork_version: u32,
    }

    impl SpecFile {
        fn to_chain(&self) -> Chain {
            let genesis_fork_version: [u8; 4] = self.genesis_fork_version.to_be_bytes();

            Chain::Custom {
                genesis_time_secs: self.min_genesis_time + self.genesis_delay,
                slot_time_secs: self.seconds_per_slot,
                genesis_fork_version,
            }
        }
    }

    let file =
        std::fs::read(&path).wrap_err(format!("Unable to find chain spec file: {path:?}"))?;

    if let Ok(decoded) = serde_json::from_slice::<SpecFileJson>(&file) {
        decoded.data.to_chain()
    } else if let Ok(decoded) = serde_json::from_slice::<QuotedSpecFile>(&file) {
        decoded.to_chain()
    } else if let Ok(decoded) = serde_yaml::from_slice::<SpecFile>(&file) {
        Ok(decoded.to_chain())
    } else {
        bail!("unable to decode file: {path:?}, accepted formats are: json or yml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    struct MockConfig {
        chain: Chain,
    }

    #[test]
    fn test_load_known() {
        let s = r#"chain = "Mainnet""#;
        let decoded: MockConfig = toml::from_str(s).unwrap();
        assert_eq!(decoded.chain, Chain::Mainnet);
    }

    #[test]
    fn test_load_custom() {
        let s = r#"chain = { genesis_time_secs = 1, slot_time_secs = 2, genesis_fork_version = "0x01000000" }"#;
        let decoded: MockConfig = toml::from_str(s).unwrap();
        assert_eq!(decoded.chain, Chain::Custom {
            genesis_time_secs: 1,
            slot_time_secs: 2,
            genesis_fork_version: [1, 0, 0, 0]
        })
    }

    #[test]
    fn test_load_file_data_json() {
        let a = env!("CARGO_MANIFEST_DIR");
        let mut path = PathBuf::from(a);

        path.pop();
        path.pop();
        path.push("tests/data/holesky_spec_data.json");

        let s = format!("chain = {path:?}");

        let decoded: MockConfig = toml::from_str(&s).unwrap();
        assert_eq!(decoded.chain, Chain::Custom {
            genesis_time_secs: KnownChain::Holesky.genesis_time_sec(),
            slot_time_secs: KnownChain::Holesky.slot_time_sec(),
            genesis_fork_version: KnownChain::Holesky.genesis_fork_version()
        })
    }

    #[test]
    fn test_load_file_json() {
        let a = env!("CARGO_MANIFEST_DIR");
        let mut path = PathBuf::from(a);

        path.pop();
        path.pop();
        path.push("tests/data/holesky_spec.json");

        let s = format!("chain = {path:?}");

        let decoded: MockConfig = toml::from_str(&s).unwrap();
        assert_eq!(decoded.chain, Chain::Custom {
            genesis_time_secs: KnownChain::Holesky.genesis_time_sec(),
            slot_time_secs: KnownChain::Holesky.slot_time_sec(),
            genesis_fork_version: KnownChain::Holesky.genesis_fork_version()
        })
    }

    #[test]
    fn test_load_file_yml() {
        let a = env!("CARGO_MANIFEST_DIR");
        let mut path = PathBuf::from(a);

        path.pop();
        path.pop();
        path.push("tests/data/helder_spec.yml");

        let s = format!("chain = {path:?}");

        let decoded: MockConfig = toml::from_str(&s).unwrap();
        assert_eq!(decoded.chain, Chain::Custom {
            genesis_time_secs: KnownChain::Helder.genesis_time_sec(),
            slot_time_secs: KnownChain::Helder.slot_time_sec(),
            genesis_fork_version: KnownChain::Helder.genesis_fork_version()
        })
    }
}
