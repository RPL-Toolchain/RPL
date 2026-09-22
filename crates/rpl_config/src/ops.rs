use std::collections::BTreeMap;

use serde::Deserialize;

/// One instance of an op-group as declared in `rpl.toml`.
///
/// All values are stored as raw strings; substitution happens later in `rpl_context`.
#[derive(Debug, Clone)]
pub struct RawOpInstance {
    /// Existential type-placeholder names declared via `type = [...]`.
    pub free: Vec<String>,
    /// Every other key/value pair (op-level meta-var bindings + op-name bindings).
    pub bindings: BTreeMap<String, String>,
}

impl<'de> Deserialize<'de> for RawOpInstance {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut raw: BTreeMap<String, toml::Value> = BTreeMap::deserialize(deserializer)?;

        let free = match raw.remove("type") {
            None => Vec::new(),
            Some(toml::Value::Array(arr)) => arr
                .into_iter()
                .map(|v| match v {
                    toml::Value::String(s) => Ok(s),
                    other => Err(serde::de::Error::custom(format!(
                        "ops 'type' entries must be strings, got {other:?}"
                    ))),
                })
                .collect::<Result<Vec<_>, D::Error>>()?,
            Some(other) => {
                return Err(serde::de::Error::custom(format!(
                    "ops 'type' must be an array of strings, got {other:?}"
                )));
            },
        };

        let mut bindings = BTreeMap::new();
        for (k, v) in raw {
            match v {
                toml::Value::String(s) => {
                    bindings.insert(k, s);
                },
                other => {
                    return Err(serde::de::Error::custom(format!(
                        "ops binding '{k}' must be a string, got {other:?}"
                    )));
                },
            }
        }

        Ok(RawOpInstance { free, bindings })
    }
}
