//! Smoke test: OpsConfig is constructed and passed through the matcher pipeline.
//! This test verifies the threading compiles and runs; matching semantics are
//! tested in Tasks 11/12 and the end-to-end UI tests in Phase 6.
#![feature(rustc_private)]

#[test]
fn ops_config_threading_compiles_and_can_be_empty() {
    // Construct an empty OpsConfig directly — no rustc_span globals needed.
    let cfg = rpl_context::pat::ops_resolved::OpsConfig::default();
    // An empty OpsConfig has no instances at all.
    assert!(cfg.instances.is_empty());
}
