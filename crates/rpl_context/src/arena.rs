#![allow(rustc::usage_of_ty_tykind)]

// `'tcx` instead of `'pcx` used here because of the `rustc_arena::declare_arena` macro
#[macro_export]
macro_rules! arena_types {
    ($macro:path) => (
        $macro!([
            [] patterns: $crate::pat::Pattern<'tcx>,
            [] fn_patterns: $crate::pat::FnPattern<'tcx>,
            [] fn_pattern_bodys: $crate::pat::FnPatternBody<'tcx>,
            [] ty_kinds: $crate::pat::TyKind<'tcx>,
            [] adt_pats: $crate::pat::Adt<'tcx>,
            [] pat_items: $crate::pat::PatternItem<'tcx>,
        ]);
    )
}

arena_types!(rustc_arena::declare_arena);
