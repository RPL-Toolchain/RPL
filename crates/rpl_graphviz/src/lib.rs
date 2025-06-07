#![allow(internal_features)]
#![feature(rustc_private)]
#![feature(iter_chain)]

extern crate rustc_graphviz;
extern crate rustc_index;
extern crate rustc_middle;

mod graph;

use std::io::{self, Write};

pub use graph::DdgConfig;
use gsgdt::{GraphvizSettings, NodeStyle};
use rpl_context::pat::FnPatternBody;
use rustc_middle::mir;
use rustc_middle::mir::interpret::PointerArithmetic;
use rustc_middle::ty::TyCtxt;

#[derive(Default)]
pub struct Config {
    pub graphviz: GraphvizSettings,
    pub node_style: NodeStyle,
    pub ddg_config: DdgConfig,
    pub pointer_bytes: PointerBytes,
}

pub fn pat_cfg_to_graphviz(patterns: &FnPatternBody<'_>, f: &mut impl Write, config: &Config) -> io::Result<()> {
    let builder = graph::CfgBuilder::from_patterns(patterns, config.pointer_bytes.get(), config.node_style.clone());
    builder.build().to_dot(f, &config.graphviz, false)
}

pub fn pat_ddg_to_graphviz(patterns: &FnPatternBody<'_>, f: &mut impl Write, config: &Config) -> io::Result<()> {
    let builder = graph::DdgBuilder::from_patterns(
        patterns,
        config.pointer_bytes.get(),
        config.node_style.clone(),
        config.ddg_config.clone(),
    );
    builder.build().to_dot(f, &config.graphviz)
}

pub fn mir_cfg_to_graphviz(body: &mir::Body<'_>, f: &mut impl Write, config: &Config) -> io::Result<()> {
    let builder = graph::CfgBuilder::from_mir(body, config.node_style.clone());
    builder.build().to_dot(f, &config.graphviz, false)
}

pub fn mir_ddg_to_graphviz(body: &mir::Body<'_>, f: &mut impl Write, config: &Config) -> io::Result<()> {
    let builder = graph::DdgBuilder::from_mir(body, config.node_style.clone(), config.ddg_config.clone());
    builder.build().to_dot(f, &config.graphviz)
}

pub struct PointerBytes(u64);

const POINTER_BITS: u64 = const {
    if cfg!(target_pointer_width = "16") {
        16
    } else if cfg!(target_pointer_width = "32") {
        32
    } else {
        64
    }
};

impl Default for PointerBytes {
    fn default() -> Self {
        rustc_middle::ty::tls::with_opt(|tcx| tcx.map(Self::from_tcx))
            .unwrap_or(PointerBytes(POINTER_BITS / u64::from(u8::BITS)))
    }
}

impl PointerBytes {
    pub fn from_tcx(tcx: TyCtxt<'_>) -> Self {
        PointerBytes(tcx.pointer_size().bytes())
    }
    pub fn get(&self) -> u64 {
        self.0
    }
}
