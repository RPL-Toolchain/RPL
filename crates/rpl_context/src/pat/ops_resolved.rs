use std::collections::{BTreeMap, BTreeSet};

use rustc_data_structures::fx::FxHashMap;
use rustc_span::Symbol;

use crate::pat::Pattern;

/// A fully resolved op-group instance.
///
/// Post-expansion strings are stored rather than typed AST nodes (the deferred-parse
/// path from Task 9 Step 5). This keeps the implementation self-contained and avoids
/// threading a `PatCtxt` through the resolution logic. The matcher (Task 12) will
/// parse each string into a `Ty<'pcx>` or `Path<'pcx>` at match time.
#[derive(Debug, Clone)]
pub struct ResolvedOpInstance {
    /// Name of the op-group this instance belongs to.
    pub group: Symbol,
    /// Existential placeholders (declared via `type = [...]` in TOML), without the `$` prefix.
    pub free: BTreeSet<Symbol>,
    /// Post-expansion strings for meta-var bindings (e.g. `T` → `"std::sync::Mutex<$1>"`).
    pub types: BTreeMap<Symbol, String>,
    /// Post-expansion strings for op-name bindings (e.g. `lock` → `"std::sync::Mutex<$1>::lock"`).
    pub paths: BTreeMap<Symbol, String>,
}

/// A collection of resolved op-group instances, keyed by group name.
///
/// Uses `FxHashMap` (rustc's deterministic hasher) so diagnostic ordering and
/// iteration order are stable across runs.
#[derive(Debug, Default)]
pub struct OpsConfig {
    pub instances: FxHashMap<Symbol, Vec<ResolvedOpInstance>>,
}

/// A single (group-name → resolved instance) assignment built from one element
/// of the cartesian product over op-group instance vectors.
///
/// Passed from the driver's cartesian loop into `CheckMirCtxt` so that the
/// matcher (Task 12) can substitute concrete types/paths for `OpRef` operands.
#[derive(Debug, Clone, Default)]
pub struct ResolvedOpBindings {
    pub by_group: FxHashMap<Symbol, ResolvedOpInstance>,
}

impl ResolvedOpBindings {
    /// Build bindings from parallel slices of group names and cloned instances.
    pub fn from_combo(groups: &[Symbol], combo: Vec<&ResolvedOpInstance>) -> Self {
        let by_group = groups.iter().copied().zip(combo.into_iter().cloned()).collect();
        ResolvedOpBindings { by_group }
    }

    /// The empty binding set — used when a pattern references no op groups.
    pub fn empty() -> Self {
        ResolvedOpBindings {
            by_group: FxHashMap::default(),
        }
    }

    /// Look up the instance bound to `group`, if any.
    pub fn get(&self, group: &Symbol) -> Option<&ResolvedOpInstance> {
        self.by_group.get(group)
    }
}

impl OpsConfig {
    /// Returns the slice of resolved instances for the named group, or `&[]` if none exist.
    pub fn instances_of(&self, group: &str) -> &[ResolvedOpInstance] {
        self.instances
            .get(&Symbol::intern(group))
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}

/// A diagnostic emitted during op-instance resolution.
///
/// Each variant corresponds to one of the C2–C7 checks described in the spec
/// (Section 4, "Config schema and substitution").
#[derive(Debug)]
pub enum ResolveDiagnostic {
    /// C1 / unknown group: the TOML references a group not declared in the `.rpl` file.
    UnknownGroup { group: String },
    /// C2 / C3: a meta-var or op binding required by the group is absent from the instance.
    MissingBinding { group: String, name: String },
    /// C4: the instance contains a key that is not a declared meta-var or op name.
    UnknownKey { group: String, name: String },
    /// C5: after expansion, a `$<id>` remains that is neither in `free` nor a meta-var.
    UndeclaredPlaceholder {
        group: String,
        name: String,
        in_value: String,
    },
    /// C7: the meta-var bindings form a cycle (did not converge in 16 passes).
    Cycle { group: String, names: Vec<String> },
    /// C6: the post-expansion string could not be parsed (placeholder for matcher-time check).
    ParseFailure {
        group: String,
        key: String,
        message: String,
    },
}

/// Resolve raw op instances from `rpl.toml` into a typed [`OpsConfig`].
///
/// For each instance, validation checks C2–C7 are run in order.  Instances that
/// fail any check are skipped (their diagnostic is pushed into the returned
/// `Vec<ResolveDiagnostic>`).  Good instances are collected into the returned
/// [`OpsConfig`].
///
/// **Deferred-parse note (Step 5):** binding values are stored as post-expansion
/// strings rather than typed `Ty<'pcx>` / `Path<'pcx>`.  C6 (parse-clean check)
/// is therefore deferred to match time; if a string fails to parse there, the
/// matcher will surface a runtime error.
pub fn resolve_ops_config(
    pattern: &Pattern<'_>,
    raw: &[(String, Vec<rpl_config::RawOpInstance>)],
) -> (OpsConfig, Vec<ResolveDiagnostic>) {
    let mut diagnostics = Vec::new();
    let mut instances: FxHashMap<Symbol, Vec<ResolvedOpInstance>> = FxHashMap::default();

    for (group_name, raw_instances) in raw {
        let group_sym = Symbol::intern(group_name);
        let group = match pattern.ops_block.groups.get(&group_sym) {
            Some(g) => g,
            None => {
                diagnostics.push(ResolveDiagnostic::UnknownGroup {
                    group: group_name.clone(),
                });
                continue;
            },
        };

        for raw_inst in raw_instances {
            match resolve_one(group_name, group, raw_inst) {
                Ok(resolved) => {
                    instances.entry(group_sym).or_default().push(resolved);
                },
                Err(diag) => diagnostics.push(diag),
            }
        }
    }

    (OpsConfig { instances }, diagnostics)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn resolve_one(
    group_name: &str,
    group: &crate::pat::OpGroup<'_>,
    raw: &rpl_config::RawOpInstance,
) -> Result<ResolvedOpInstance, ResolveDiagnostic> {
    // C2: every type meta-var declared in the group must have a binding.
    for ty_var in &group.meta_vars.ty_vars {
        let bare_name = ty_var.name.as_str().trim_start_matches('$');
        if !raw.bindings.contains_key(bare_name) {
            return Err(ResolveDiagnostic::MissingBinding {
                group: group_name.to_string(),
                name: bare_name.to_string(),
            });
        }
    }

    // C3: every op declared in the group must have a binding.
    for op_name in group.ops.keys() {
        if !raw.bindings.contains_key(op_name.as_str()) {
            return Err(ResolveDiagnostic::MissingBinding {
                group: group_name.to_string(),
                name: op_name.as_str().to_string(),
            });
        }
    }

    // C4: no extra keys beyond declared meta-vars and op names.
    let known_keys: BTreeSet<&str> = group
        .meta_vars
        .ty_vars
        .iter()
        .map(|tv| tv.name.as_str().trim_start_matches('$'))
        .chain(group.ops.keys().map(|s| s.as_str()))
        .collect();

    for k in raw.bindings.keys() {
        if !known_keys.contains(k.as_str()) {
            return Err(ResolveDiagnostic::UnknownKey {
                group: group_name.to_string(),
                name: k.clone(),
            });
        }
    }

    // Build the set of meta-var bare names for expansion.
    let meta_var_names: BTreeSet<String> = group
        .meta_vars
        .ty_vars
        .iter()
        .map(|tv| tv.name.as_str().trim_start_matches('$').to_string())
        .collect();

    // Free placeholder names from `type = [...]`, stripped of leading `$`.
    let free_names: BTreeSet<String> = raw.free.iter().map(|s| s.trim_start_matches('$').to_string()).collect();

    // C5/C7: fixed-point textual expansion of meta-var `$<name>` references.
    //
    // Up to 16 passes.  If no convergence after 16 → cycle (C7).
    // After convergence, any residual `$<id>` whose <id> is a meta-var name
    // (i.e. *should* have been substituted away) means there was a cycle even
    // before the bound was hit — we treat this as C7 as well.
    // Residual `$<id>` whose <id> is in `free_names` are existential placeholders
    // (valid). Anything else is C5 (undeclared placeholder).
    let mut expanded: BTreeMap<String, String> = raw.bindings.clone();
    let mut converged = false;
    for _iter in 0..16 {
        let mut changed = false;
        let snapshot: BTreeMap<String, String> = expanded.clone();
        for (_key, val) in expanded.iter_mut() {
            let new_val = expand_one(val, &snapshot, &meta_var_names);
            if new_val != *val {
                *val = new_val;
                changed = true;
            }
        }
        if !changed {
            converged = true;
            break;
        }
    }

    if !converged {
        return Err(ResolveDiagnostic::Cycle {
            group: group_name.to_string(),
            names: expanded.keys().cloned().collect(),
        });
    }

    // Post-convergence: scan for residual `$<id>` placeholders.
    // - In free_names  → OK (existential)
    // - In meta_var_names → cycle (should have been substituted; see note above)
    // - Otherwise     → undeclared placeholder (C5)
    for (key, val) in &expanded {
        for placeholder in collect_placeholders(val) {
            if meta_var_names.contains(&placeholder) {
                // Still present after convergence → cycle in disguise.
                return Err(ResolveDiagnostic::Cycle {
                    group: group_name.to_string(),
                    names: vec![key.clone(), placeholder],
                });
            }
            if !free_names.contains(&placeholder) {
                return Err(ResolveDiagnostic::UndeclaredPlaceholder {
                    group: group_name.to_string(),
                    name: placeholder,
                    in_value: val.clone(),
                });
            }
        }
    }

    // Build the output maps.
    let mut types: BTreeMap<Symbol, String> = BTreeMap::new();
    for ty_var in &group.meta_vars.ty_vars {
        let bare = ty_var.name.as_str().trim_start_matches('$');
        let s = expanded.get(bare).expect("validated above").clone();
        types.insert(Symbol::intern(bare), s);
    }

    let mut paths: BTreeMap<Symbol, String> = BTreeMap::new();
    for op_name in group.ops.keys() {
        let s = expanded.get(op_name.as_str()).expect("validated above").clone();
        paths.insert(*op_name, s);
    }

    Ok(ResolvedOpInstance {
        group: Symbol::intern(group_name),
        free: free_names.iter().map(|s| Symbol::intern(s)).collect(),
        types,
        paths,
    })
}

/// Perform one pass of meta-var expansion over `value`.
///
/// Replaces every `$<id>` token where `<id>` is in `meta_vars` with the
/// corresponding entry from `bindings`.  Other `$<id>` tokens (free
/// placeholders, undeclared names) are left intact.
fn expand_one(value: &str, bindings: &BTreeMap<String, String>, meta_vars: &BTreeSet<String>) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' {
            let mut id = String::new();
            while let Some(&nc) = chars.peek() {
                if nc.is_alphanumeric() || nc == '_' {
                    id.push(nc);
                    chars.next();
                } else {
                    break;
                }
            }
            if meta_vars.contains(&id) {
                // Substitute meta-var → its binding value.
                out.push_str(bindings.get(&id).map(String::as_str).unwrap_or(""));
            } else {
                // Free placeholder or undeclared name — leave as-is.
                out.push('$');
                out.push_str(&id);
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Collect all `$<id>` placeholder names (without the `$`) in `value`.
fn collect_placeholders(value: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = value.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' {
            let mut id = String::new();
            while let Some(&nc) = chars.peek() {
                if nc.is_alphanumeric() || nc == '_' {
                    id.push(nc);
                    chars.next();
                } else {
                    break;
                }
            }
            if !id.is_empty() {
                out.push(id);
            }
        }
    }
    out
}
