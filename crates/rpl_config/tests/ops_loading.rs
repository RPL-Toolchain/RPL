use rpl_config::{RawOpInstance, RplConfig};

const TOML: &str = r#"
[[patterns.local]]
name = "g"
path = ["patterns/foo.rpl"]

[[ops.sync]]
type = ["$1"]
T = "std::sync::Mutex<$1>"
U = "std::sync::MutexGuard<$1>"
lock = "$T::lock"
unlock = "$U::drop"

[[ops.sync]]
type = ["$1"]
T = "parking_lot::Mutex<$1>"
U = "parking_lot::MutexGuard<$1>"
lock = "$T::lock"
unlock = "$U::drop"
"#;

#[test]
fn loads_two_sync_instances() {
    let cfg: RplConfig = toml::from_str(TOML).expect("parse");
    let sync_instances = cfg.ops.get("sync").expect("sync key present");
    assert_eq!(sync_instances.len(), 2);
    let i0: &RawOpInstance = &sync_instances[0];
    assert_eq!(i0.free, vec!["$1".to_string()]);
    assert_eq!(i0.bindings.get("T").unwrap(), "std::sync::Mutex<$1>");
    assert_eq!(i0.bindings.get("lock").unwrap(), "$T::lock");
}

#[test]
fn type_key_is_not_in_bindings() {
    let cfg: RplConfig = toml::from_str(TOML).expect("parse");
    let i0 = &cfg.ops["sync"][0];
    assert!(!i0.bindings.contains_key("type"));
}
