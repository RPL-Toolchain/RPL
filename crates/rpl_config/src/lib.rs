use std::path::{Path, PathBuf};

use serde::Deserialize;

mod ops;
mod patterns;
mod run;
mod util;

pub use ops::RawOpInstance;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read {path}: {source}")]
    Io { path: PathBuf, source: Box<std::io::Error> },
    #[error("failed to parse {path}: {source}")]
    Toml {
        path: PathBuf,
        source: Box<toml::de::Error>,
    },
    #[error("cargo metadata failed: {0}")]
    CargoMetadata(#[from] Box<cargo_metadata::Error>),
    #[error("rpl.toml not found at {path}")]
    ConfigNotFound { path: PathBuf },
    #[error("rpl.toml defines no pattern groups")]
    NoGroups,
    #[error("duplicate pattern group name `{name}`")]
    DuplicateGroup { name: String },
    #[error("unknown pattern group `{name}`")]
    UnknownGroup { name: String },
    #[error("invalid remote group reference `{spec}`; expected `crate::group`")]
    InvalidRemoteGroup { spec: String },
    #[error("crate `{crate_name}` not found in cargo metadata")]
    CrateNotFound { crate_name: String },
    #[error("multiple crates named `{crate_name}` found; disambiguation not supported")]
    AmbiguousCrate { crate_name: String },
    #[error("crate `{crate_name}` does not publish RPL metadata")]
    MissingRplMetadata { crate_name: String },
    #[error("crate `{crate_name}` has invalid RPL metadata: {error}")]
    InvalidRplMetadata { crate_name: String, error: String },
    #[error("crate `{crate_name}` does not define RPL group `{group}`")]
    MissingRemoteGroup { crate_name: String, group: String },
    #[error("pattern group name cannot be empty")]
    EmptyGroupName,
    #[error("pattern path is not valid unicode: {path}")]
    NonUnicodePath { path: PathBuf },
    #[error("pattern paths cannot be joined for environment variable: {source}")]
    InvalidPatternPathList { source: Box<std::env::JoinPathsError> },
}

#[derive(Debug, Deserialize)]
pub struct RplConfig {
    pub(crate) run: Option<run::RunConfig>,
    pub(crate) patterns: Option<patterns::PatternsConfig>,
    #[serde(default)]
    pub ops: std::collections::HashMap<String, Vec<RawOpInstance>>,
}

#[derive(Debug)]
pub struct Config {
    pub patterns_env: Option<String>,
    pub inline_mir: Option<bool>,
}

pub fn load_config(manifest_path: Option<&Path>, selected_groups: &[String]) -> Result<Config, ConfigError> {
    let base_dir = util::resolve_base_dir(manifest_path)?;
    let config_path = base_dir.join("rpl.toml");
    let config = if config_path.exists() {
        Some(util::read_config(&config_path)?)
    } else {
        None
    };
    let inline_mir = run::load_inline_mir(config.as_ref());
    let patterns_env = patterns::load_patterns_env(manifest_path, selected_groups, config.as_ref())?;
    Ok(Config {
        patterns_env,
        inline_mir,
    })
}

/// Load the raw op-group instances from `rpl.toml` in the current directory
/// (or adjacent to `manifest_path` if given).
///
/// Returns an empty map when no `rpl.toml` exists or when the file contains
/// no `[ops]` table.  Errors during config file reading are surfaced as
/// `Err(ConfigError)`.
pub fn load_raw_ops(
    manifest_path: Option<&Path>,
) -> Result<std::collections::HashMap<String, Vec<RawOpInstance>>, ConfigError> {
    let base_dir = util::resolve_base_dir(manifest_path)?;
    let config_path = base_dir.join("rpl.toml");
    if config_path.exists() {
        let config = util::read_config(&config_path)?;
        Ok(config.ops)
    } else {
        Ok(std::collections::HashMap::new())
    }
}
