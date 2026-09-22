use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use cargo_metadata::{Metadata, MetadataCommand, Package};
use serde::Deserialize;

use crate::util::resolve_base_dir;
use crate::{ConfigError, RplConfig};

pub(crate) struct ResolvedPatterns {
    pub paths: Vec<PathBuf>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PatternsConfig {
    pub local: Option<Vec<LocalGroup>>,
    pub remote: Option<Vec<RemoteGroup>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LocalGroup {
    pub name: String,
    pub path: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RemoteGroup {
    pub name: String,
    pub groups: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PackageMetadata {
    rpl: Option<PublishedRpl>,
}

#[derive(Debug, Deserialize)]
struct PublishedRpl {
    path: Option<Vec<String>>,
    groups: Option<HashMap<String, Vec<String>>>,
}

pub(crate) fn resolve_patterns(
    manifest_path: Option<&Path>,
    selected_groups: &[String],
    config: Option<&RplConfig>,
) -> Result<Option<ResolvedPatterns>, ConfigError> {
    if selected_groups.iter().any(|group| group.is_empty()) {
        return Err(ConfigError::EmptyGroupName);
    }
    let base_dir = resolve_base_dir(manifest_path)?;
    let config_path = base_dir.join("rpl.toml");

    let Some(config) = config else {
        if selected_groups.is_empty() {
            return Ok(None);
        }
        if selected_groups.iter().all(|name| is_remote_spec(name)) {
            return Ok(Some(resolve_remote_selection(manifest_path, selected_groups)?));
        }
        return Err(ConfigError::ConfigNotFound { path: config_path });
    };
    let patterns = match config.patterns.as_ref() {
        Some(patterns) => patterns,
        None => {
            if selected_groups.iter().all(|name| is_remote_spec(name)) {
                return Ok(Some(resolve_remote_selection(manifest_path, selected_groups)?));
            }
            return Err(ConfigError::NoGroups);
        },
    };

    let mut local_groups: HashMap<String, Vec<String>> = HashMap::new();
    let mut remote_groups: HashMap<String, Vec<String>> = HashMap::new();
    let mut order = Vec::new();

    if let Some(local) = patterns.local.as_ref() {
        for entry in local {
            if local_groups.contains_key(&entry.name) || remote_groups.contains_key(&entry.name) {
                return Err(ConfigError::DuplicateGroup {
                    name: entry.name.clone(),
                });
            }
            local_groups.insert(entry.name.clone(), entry.path.clone());
            order.push(entry.name.clone());
        }
    }

    if let Some(remote) = patterns.remote.as_ref() {
        for entry in remote {
            if local_groups.contains_key(&entry.name) || remote_groups.contains_key(&entry.name) {
                return Err(ConfigError::DuplicateGroup {
                    name: entry.name.clone(),
                });
            }
            remote_groups.insert(entry.name.clone(), entry.groups.clone());
            order.push(entry.name.clone());
        }
    }

    if local_groups.is_empty() && remote_groups.is_empty() {
        return Err(ConfigError::NoGroups);
    }

    let selected = if selected_groups.is_empty() {
        order
    } else {
        selected_groups.to_vec()
    };

    let mut seen = HashSet::new();
    let mut paths = Vec::new();
    let mut metadata: Option<Metadata> = None;

    for name in &selected {
        let group_paths = if let Some(entries) = local_groups.get(name) {
            resolve_local_group(&base_dir, entries)
        } else if let Some(entries) = remote_groups.get(name) {
            if metadata.is_none() {
                metadata = Some(load_metadata(manifest_path)?);
            }
            resolve_remote_groups(metadata.as_ref().unwrap(), entries)?
        } else if is_remote_spec(name) {
            if metadata.is_none() {
                metadata = Some(load_metadata(manifest_path)?);
            }
            resolve_remote_groups(metadata.as_ref().unwrap(), std::slice::from_ref(name))?
        } else {
            return Err(ConfigError::UnknownGroup { name: name.clone() });
        };

        for path in group_paths {
            if seen.insert(path.clone()) {
                paths.push(path);
            }
        }
    }

    Ok(Some(ResolvedPatterns { paths }))
}

pub(crate) fn resolve_patterns_env(
    manifest_path: Option<&Path>,
    selected_groups: &[String],
    config: Option<&RplConfig>,
) -> Result<Option<String>, ConfigError> {
    let resolved = match resolve_patterns(manifest_path, selected_groups, config)? {
        Some(resolved) => resolved,
        None => return Ok(None),
    };

    // An empty paths vector here can arise when `rpl.toml` exists but contains
    // no `[patterns]` table (e.g. only `[run]` or `[[ops.<group>]]`) and the
    // caller did not pass `--patterns`.  In that case `resolve_patterns` takes
    // the vacuous-true branch of `selected_groups.iter().all(is_remote_spec)`
    // and returns `Some(ResolvedPatterns { paths: vec![] })`.  Joining zero
    // paths yields the empty string, which would propagate to the child
    // process as `RPL_PATS=""` and trip
    // `rpl_meta::cli E100: Cannot locate RPL pattern file ""`.  Map empty to
    // `None` here so the child falls back to the built-in pattern set.
    if resolved.paths.is_empty() {
        return Ok(None);
    }

    let mut entries = Vec::with_capacity(resolved.paths.len());
    for path in resolved.paths {
        if path.to_str().is_none() {
            return Err(ConfigError::NonUnicodePath { path });
        }
        entries.push(path);
    }

    let joined = std::env::join_paths(entries.iter()).map_err(|source| ConfigError::InvalidPatternPathList {
        source: Box::new(source),
    })?;
    Ok(Some(joined.to_string_lossy().into_owned()))
}

pub(crate) fn load_patterns_env(
    manifest_path: Option<&Path>,
    selected_groups: &[String],
    config: Option<&RplConfig>,
) -> Result<Option<String>, ConfigError> {
    let has_selection = !selected_groups.is_empty();
    if std::env::var("RPL_PATS").is_ok() && !has_selection {
        return Ok(None);
    }
    resolve_patterns_env(manifest_path, selected_groups, config)
}

fn resolve_local_group(base_dir: &Path, entries: &[String]) -> Vec<PathBuf> {
    entries.iter().map(|entry| resolve_relative(base_dir, entry)).collect()
}

fn resolve_remote_groups(metadata: &Metadata, specs: &[String]) -> Result<Vec<PathBuf>, ConfigError> {
    let mut resolved = Vec::new();
    for spec in specs {
        let (crate_name, group) = parse_remote_spec(spec)?;
        let package = find_package(metadata, &crate_name)?;
        let published = package_rpl_metadata(package, &crate_name)?;

        let group_entries = published
            .groups
            .as_ref()
            .and_then(|groups| groups.get(&group))
            .ok_or_else(|| ConfigError::MissingRemoteGroup {
                crate_name: crate_name.clone(),
                group: group.clone(),
            })?;

        let base_paths = published.path.as_ref().filter(|paths| !paths.is_empty());
        let default_base = vec![".".to_string()];
        let base_paths = base_paths.unwrap_or(&default_base);

        let manifest_path = PathBuf::from(package.manifest_path.as_str());
        let crate_root = manifest_path.parent().unwrap_or(Path::new("."));

        for base in base_paths {
            let base_dir = resolve_relative(crate_root, base);
            for entry in group_entries {
                resolved.push(resolve_relative(&base_dir, entry));
            }
        }
    }
    Ok(resolved)
}

fn parse_remote_spec(spec: &str) -> Result<(String, String), ConfigError> {
    let (crate_name, group) = spec
        .split_once("::")
        .ok_or_else(|| ConfigError::InvalidRemoteGroup { spec: spec.to_string() })?;
    Ok((crate_name.to_string(), group.to_string()))
}

fn find_package<'a>(metadata: &'a Metadata, name: &str) -> Result<&'a Package, ConfigError> {
    let matches: Vec<&Package> = metadata.packages.iter().filter(|pkg| pkg.name == name).collect();
    match matches.as_slice() {
        [] => Err(ConfigError::CrateNotFound {
            crate_name: name.to_string(),
        }),
        [pkg] => Ok(*pkg),
        _ => Err(ConfigError::AmbiguousCrate {
            crate_name: name.to_string(),
        }),
    }
}

fn package_rpl_metadata(package: &Package, crate_name: &str) -> Result<PublishedRpl, ConfigError> {
    let metadata = serde_json::from_value::<PackageMetadata>(package.metadata.clone()).map_err(|err| {
        ConfigError::InvalidRplMetadata {
            crate_name: crate_name.to_string(),
            error: err.to_string(),
        }
    })?;
    metadata.rpl.ok_or_else(|| ConfigError::MissingRplMetadata {
        crate_name: crate_name.to_string(),
    })
}

fn load_metadata(manifest_path: Option<&Path>) -> Result<Metadata, ConfigError> {
    let mut cmd = MetadataCommand::new();
    if let Some(manifest_path) = manifest_path {
        cmd.manifest_path(manifest_path);
    }
    cmd.exec().map_err(|err| ConfigError::CargoMetadata(Box::new(err)))
}

fn resolve_relative(base: &Path, entry: &str) -> PathBuf {
    let path = Path::new(entry);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

fn is_remote_spec(name: &str) -> bool {
    name.contains("::")
}

fn resolve_remote_selection(
    manifest_path: Option<&Path>,
    selected_groups: &[String],
) -> Result<ResolvedPatterns, ConfigError> {
    let metadata = load_metadata(manifest_path)?;
    let mut seen = HashSet::new();
    let mut paths = Vec::new();
    for spec in selected_groups {
        let group_paths = resolve_remote_groups(&metadata, std::slice::from_ref(spec))?;
        for path in group_paths {
            if seen.insert(path.clone()) {
                paths.push(path);
            }
        }
    }
    Ok(ResolvedPatterns { paths })
}
