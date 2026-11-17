pub use inline::Inline;
use rpl_parser::generics::Choice2;
use rpl_parser::{collect_elems_separated_by_comma, pairs};
use rustc_data_structures::fx::FxHashMap;
use rustc_hir::def_id::LocalDefId;
use rustc_hir::{Attribute, FnHeader};
use rustc_middle::ty::TyCtxt;
use rustc_span::Symbol;
pub use safety::Safety;
pub use visibility::Visibility;

use crate::attributes::body::contains_unsafe_block;

mod body;
mod inline;
mod safety;
mod visibility;

/// Extra spans that are required for diagnostics or other purposes.
pub type ExtraSpan<'tcx> = FxHashMap<Symbol, &'tcx Attribute>;

/// Attributes for single function, including visibility, safety, and other metadata.
#[derive(Debug, Clone, Default)]
pub struct FnAttr {
    pub visibility: Visibility,
    pub safety: Safety,
    pub requires_monomorphization: Option<bool>,
    pub inner_unsafe: Option<bool>,
    pub inline: Option<Inline>,
    pub output_name: Option<Symbol>,
}

impl FnAttr {
    #[instrument(level = "trace", skip(pre, post), ret)]
    pub(super) fn parse<'i>(pre: impl Iterator<Item = &'i pairs::Attr<'i>>, post: &[&pairs::Attribute<'_>]) -> Self {
        let mut result = Self::default();
        for pairs in pre {
            let (_, _, attrs, _) = pairs.get_matched();
            for attr in collect_elems_separated_by_comma!(attrs) {
                let (key, value) = attr.get_matched();
                match key.span.as_str() {
                    "output" => match value {
                        Some(Choice2::_0(_)) | None => unreachable!(),
                        Some(Choice2::_1(msg)) => {
                            let (_, msg) = msg.get_matched();
                            result.output_name = Some(Symbol::intern(msg.diagMessageInner().span.as_str()));
                        },
                    },
                    "inline" => match value {
                        Some(Choice2::_1(_)) | None => unreachable!(),
                        Some(Choice2::_0(msg)) => {
                            let (_, msg, _) = msg.get_matched();
                            if let Some(msg) = msg {
                                let inner = collect_elems_separated_by_comma!(msg).collect::<Vec<_>>();
                                if inner.len() != 1 {
                                    panic!("Expected exactly one output, found {}", inner.len());
                                }
                                let (level, attr) = inner[0].get_matched();
                                assert!(attr.is_none(), "Unexpected attribute in output: {attr:?}");
                                result.inline = match level.span.as_str() {
                                    "always" => Some(Inline::Always),
                                    "any" => Some(Inline::Any),
                                    "never" => Some(Inline::Never),
                                    _ => panic!("Unexpected inline level: {}", level.span.as_str()),
                                };
                            } else {
                                result.inline = Some(Inline::Normal);
                            }
                        },
                    },
                    "rpl" => match value {
                        Some(Choice2::_1(_)) | None => unreachable!(),
                        Some(Choice2::_0(inner)) => {
                            let (_, inner, _) = inner.get_matched();
                            if let Some(inner) = inner {
                                for inner in collect_elems_separated_by_comma!(inner) {
                                    let (key, value) = inner.get_matched();
                                    assert!(value.is_none(), "Unexpected value in RPL attribute: {value:?}");
                                    match key.span.as_str() {
                                        "requires_monomorphization" => {
                                            result.requires_monomorphization = Some(true);
                                        },
                                        "inner_unsafe" => {
                                            result.inner_unsafe = Some(true);
                                        },
                                        _ => panic!("Unexpected RPL attribute: {}", key.span.as_str()),
                                    }
                                }
                            }
                        },
                    },
                    _ => unreachable!(),
                }
            }
        }

        for attr in post {
            let (name, _, value) = attr.get_matched();
            let name = name.span.as_str();
            let value = value.span.as_str();
            // FIXME: find a better way to do this.
            // FIXME: check predicates and attributes in meta pass.
            match name {
                "visibility" => match value {
                    "public" => result.visibility = Visibility::Public,
                    "restricted" => result.visibility = Visibility::Restricted,
                    _ => panic!("Unexpected visibility: {}", value),
                },
                "safety" => {
                    if value == "safe" {
                        result.safety = Safety::Safe;
                    } else if value == "unsafe" {
                        result.safety = Safety::Unsafe;
                    }
                    match value {
                        "safe" => result.safety = Safety::Safe,
                        "unsafe" => result.safety = Safety::Unsafe,
                        _ => panic!("Unexpected safety level: {}", value),
                    }
                },
                "requires_monomorphization" => {
                    result.requires_monomorphization = Some(value.parse().unwrap());
                },
                "inner_unsafe" => {
                    result.inner_unsafe = Some(value.parse().unwrap());
                },
                "marked_inline" => {
                    result.inline = match value {
                        "always" => Some(Inline::Always),
                        "any" => Some(Inline::Any),
                        "never" => Some(Inline::Never),
                        _ => panic!("Unexpected inline level: {}", value),
                    };
                },
                "output" => {
                    result.output_name = Some(Symbol::intern(value));
                },
                _ => panic!("Unexpected attribute: {}", name),
            }
        }
        result
    }
    pub fn add_visibility(&mut self, visibility: Visibility) {
        self.visibility = visibility;
    }
    pub fn add_safety(&mut self, safety: Safety) {
        self.safety = safety;
    }

    #[instrument(level = "trace", skip(tcx, header), ret)]
    pub fn filter(&self, tcx: TyCtxt<'_>, def_id: LocalDefId, header: Option<FnHeader>) -> bool {
        self.visibility.check(tcx.visibility(def_id))
            && self.safety.check_option_header(header.map(|h| h.safety))
            && self
                .requires_monomorphization
                .is_none_or(|req| tcx.generics_of(def_id).requires_monomorphization(tcx) == req)
            && self.inner_unsafe.is_none_or(|inner_unsafe| {
                // Foreign/`extern` fns have no HIR body; do not call `body_owned_by`.
                let has_inner_unsafe =
                    tcx.is_mir_available(def_id) && contains_unsafe_block(tcx, tcx.hir().body_owned_by(def_id).value);
                inner_unsafe == has_inner_unsafe || header.is_some_and(|header| header.is_unsafe())
            })
    }

    /// Returns the extra spans for this function pattern.
    #[instrument(level = "trace", skip(tcx), ret)]
    pub fn extra_span<'tcx>(&self, tcx: TyCtxt<'tcx>, def_id: LocalDefId) -> Option<ExtraSpan<'tcx>> {
        let mut attr_map = ExtraSpan::default();
        if let Some(inline) = self.inline {
            let inline_ = Symbol::intern("inline");
            let attr = inline.check(tcx.get_attrs(def_id, inline_))?;
            _ = attr_map.try_insert(inline_, attr);
        }
        Some(attr_map)
    }
}
