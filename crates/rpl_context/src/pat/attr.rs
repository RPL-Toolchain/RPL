use std::fmt::Debug;
use std::hash::Hash;

use either::Either;
use rpl_meta::collect_elems_separated_by_comma;
use rpl_parser::generics::Choice2;
use rpl_parser::pairs;
use rustc_data_structures::fx::{FxHashMap, FxHashSet};
use rustc_span::Symbol;

#[derive(Default)]
pub struct PatAttr<'i> {
    pub(super) deduplicate: bool,
    pub(super) diag: Option<Symbol>,
    pub(super) consts: FxHashMap<Symbol, &'i str>,
}

impl<'i> PatAttr<'i> {
    pub(in super::super) fn parse_all<'a>(attrs: impl Iterator<Item = &'a pairs::Attr<'i>>) -> Self
    where
        'i: 'a,
    {
        let mut result = Self::default();
        for attr in attrs {
            result.parse(attr);
        }
        result
    }
    fn parse(&mut self, pairs: &pairs::Attr<'i>) {
        let (_, _, attrs, _) = pairs.get_matched();
        for attr in collect_elems_separated_by_comma!(attrs) {
            let (key, value) = attr.get_matched();
            match key.span.as_str() {
                // #[deduplicate]
                "deduplicate" => match value {
                    Some(_) => unreachable!(),
                    None => {
                        self.deduplicate = true;
                    },
                },
                // #[diag = "diag_name"]
                "diag" => match value {
                    Some(Choice2::_0(_)) | None => unreachable!(),
                    Some(Choice2::_1(msg)) => {
                        let (_, msg) = msg.get_matched();
                        self.diag = Some(Symbol::intern(msg.diagMessageInner().span.as_str()));
                    },
                },
                // #[const(name1 = "...", name2 = "."]
                "const" => match value {
                    Some(Choice2::_1(_)) | None => unreachable!(),
                    Some(Choice2::_0(list)) => {
                        let (_, list, _) = list.get_matched();
                        if let Some(list) = list {
                            for pair in collect_elems_separated_by_comma!(list) {
                                let (name, value) = pair.get_matched();
                                let name = Symbol::intern(name.span.as_str());
                                let value = value
                                    .as_ref()
                                    .unwrap()
                                    ._1()
                                    .unwrap()
                                    .1
                                    .matched
                                    .diagMessageInner()
                                    .span
                                    .as_str();
                                self.consts.insert(name, value);
                            }
                        }
                    },
                },
                _ => unreachable!(),
            }
        }
    }

    pub fn post_process<M: Eq + Hash + Debug>(&self, iter: impl Iterator<Item = M>) -> impl Iterator<Item = M> {
        match self.deduplicate {
            true => {
                let collected: FxHashSet<_> = iter.inspect(|matched| trace!(?matched)).collect();
                trace!(?collected);
                Either::Left(collected.into_iter())
            },
            false => Either::Right(iter),
        }
    }
}
