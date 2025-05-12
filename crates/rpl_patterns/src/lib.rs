#![feature(rustc_private)]
#![feature(let_chains)]
#![feature(if_let_guard)]

extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_fluent_macro;
extern crate rustc_hir;
extern crate rustc_infer;
extern crate rustc_lint_defs;
extern crate rustc_macros;
extern crate rustc_middle;
extern crate rustc_passes;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_trait_selection;
#[macro_use]
extern crate tracing;
extern crate either;
extern crate rpl_macros;

use rpl_context::PatCtxt;
use rustc_hir::ItemId;
use rustc_middle::ty::TyCtxt;
use rustc_session::config::OptLevel;

mod inline;
mod normal;
mod others;

pub(crate) mod errors;
pub(crate) mod lints;

rustc_fluent_macro::fluent_messages! { "../messages.en.ftl" }

static ALL_PATTERNS: &[fn(TyCtxt<'_>, PatCtxt<'_>, ItemId)] = &[
    normal::cve_2018_20992::check_item,
    inline::cve_2018_21000::t_to_u8::check_item,
    inline::cve_2018_21000::u8_to_t::check_item,
    normal::cve_2018_21000::t_to_u8::check_item,
    normal::cve_2018_21000::u8_to_t::check_item,
    inline::cve_2019_15548::check_item,
    normal::cve_2019_15548::check_item,
    normal::cve_2019_16138::check_item,
    inline::cve_2020_25016::check_item,
    normal::cve_2020_35860::check_item,
    inline::cve_2020_35862::check_item,
    inline::cve_2020_35873::check_item,
    inline::cve_2020_35877::check_item,
    inline::cve_2020_35881::const_const_Transmute_ver::check_item,
    inline::cve_2020_35881::mut_mut_Transmute_ver::check_item,
    inline::cve_2020_35881::mut_const_PtrToPtr_ver::check_item,
    inline::cve_2020_35888::check_item,
    inline::cve_2020_35892_3::check_item,
    inline::cve_2020_35898_9::check_item,
    inline::cve_2020_35901_2::check_item,
    inline::cve_2020_35907::check_item,
    normal::cve_2020_35907::check_item,
    inline::cve_2021_25904::check_item,
    normal::cve_2021_25904::check_item,
    // inline::cve_2021_25905::check_item,
    normal::cve_2021_25905::check_item,
    normal::cve_2021_27376::check_item,
    inline::cve_2021_29941_2::check_item,
    normal::cve_2021_29941_2::check_item,
    normal::cve_2022_23639::check_item,
    inline::cve_2024_27284::check_item,
    others::private_or_generic_function_marked_inline::check_item,
    inline::transmute_type_to_bool::check_item,
    // FIXME: Too loose
    inline::transmute_int_to_ptr::check_item,
    normal::manually_drop::check_item,
    inline::alloc_unchecked::check_item,
    normal::alloc_unchecked::check_item,
];

#[allow(unused)]
static DEBUG_PATTERN: &[fn(TyCtxt<'_>, PatCtxt<'_>, ItemId)] = &[inline::transmute_type_to_bool::check_item];

#[instrument(level = "info", skip_all, fields(item = ?item.owner_id.def_id))]
pub fn check_item(tcx: TyCtxt<'_>, pcx: PatCtxt<'_>, item: ItemId) {
    // rustc_data_structures::sync::par_for_each_in(ALL_PATTERNS, |check| check(tcx, pcx, item))
    rustc_data_structures::sync::par_for_each_in(ALL_PATTERNS, |check| check(tcx, pcx, item));
}

#[allow(unused)]
pub(crate) fn is_inline_mir(sess: &rustc_session::Session) -> bool {
    // FIXME(#127234): Coverage instrumentation currently doesn't handle inlined
    // MIR correctly when Modified Condition/Decision Coverage is enabled.
    if sess.instrument_coverage_mcdc() {
        return false;
    }

    if let Some(enabled) = sess.opts.unstable_opts.inline_mir {
        return enabled;
    }

    match sess.mir_opt_level() {
        0 | 1 => false,
        2 => {
            (sess.opts.optimize == OptLevel::Default || sess.opts.optimize == OptLevel::Aggressive)
                && sess.opts.incremental.is_none()
        },
        _ => true,
    }
}
