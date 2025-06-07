use derive_more::Debug;
use parser::pairs;
use pest_typed::Span;
use rustc_span::Symbol;

use crate::collect_elems_separated_by_comma;

/// Identifier in RPL meta language.
#[derive(Copy, Clone, Debug)]
#[debug("{name}")]
pub struct Ident<'i> {
    pub name: Symbol,
    pub span: Span<'i>,
}

impl PartialEq for Ident<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Ident<'_> {}

use std::fmt;
use std::hash::{Hash, Hasher};

impl Hash for Ident<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl<'i> From<&pairs::Identifier<'i>> for Ident<'i> {
    fn from(ident: &pairs::Identifier<'i>) -> Self {
        let span = ident.span;
        let name = Symbol::intern(span.as_str());
        Self { name, span }
    }
}

impl<'i> From<&pairs::MetaVariable<'i>> for Ident<'i> {
    fn from(meta: &pairs::MetaVariable<'i>) -> Self {
        let span = meta.span;
        let name = Symbol::intern(span.as_str());
        Self { name, span }
    }
}

impl<'i> From<&pairs::Dollarself<'i>> for Ident<'i> {
    fn from(meta: &pairs::Dollarself<'i>) -> Self {
        let span = meta.span;
        let name = Symbol::intern(span.as_str());
        Self { name, span }
    }
}

impl<'i> From<&pairs::DollarRET<'i>> for Ident<'i> {
    fn from(meta: &pairs::DollarRET<'i>) -> Self {
        let span = meta.span;
        let name = Symbol::intern(span.as_str());
        Self { name, span }
    }
}

impl<'i> From<Span<'i>> for Ident<'i> {
    fn from(span: Span<'i>) -> Self {
        let name = Symbol::intern(span.as_str());
        Self { name, span }
    }
}

impl std::fmt::Display for Ident<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.name, f)
    }
}

pub struct Path<'i> {
    pub leading: Option<&'i pairs::PathLeading<'i>>,
    pub segments: Vec<(Ident<'i>, Vec<&'i pairs::GenericArgument<'i>>)>,
    pub _span: Span<'i>,
}

impl<'i> From<&'i pairs::Path<'i>> for Path<'i> {
    fn from(path: &'i pairs::Path<'i>) -> Self {
        fn collect_args<'i>(args: Option<&'i pairs::PathArguments<'i>>) -> Vec<&'i pairs::GenericArgument<'i>> {
            args.map(|args| {
                let args = args.get_matched().2;
                collect_elems_separated_by_comma!(args).collect()
            })
            .unwrap_or_default()
        }
        let (leading, seg, segs) = path.get_matched();
        let mut segments = vec![(Ident::from(seg.Identifier()), collect_args(seg.PathArguments()))];
        segs.iter_matched().for_each(|seg| {
            let (_, seg) = seg.get_matched();
            let args = collect_args(seg.PathArguments());
            segments.push((Ident::from(seg.Identifier()), args));
        });
        let span = path.span;
        Self {
            leading: leading.as_ref(),
            segments,
            _span: span,
        }
    }
}

impl<'i> Path<'i> {
    /// Returns `Some(ident)` if `self` is a single identifier.
    pub fn as_ident(&self) -> Option<Ident<'i>> {
        if self.leading.is_none() && self.segments.len() == 1 && self.segments[0].1.is_empty() {
            // If the path has no leading identifier and only one segment with no generic arguments,
            // it can be treated as a single identifier.
            Some(self.segments[0].0)
        } else {
            None
        }
    }
    /// Returns the last segment.
    pub fn ident(&self) -> Ident<'i> {
        //FIXME: use a non-empty `Vec` type
        let last = self.segments.last().unwrap();
        last.0
    }

    /// Returns the leading identifier if it's the path is not starting with `::`.
    pub fn leading_ident(&self) -> Option<Ident<'i>> {
        if self.leading.is_none() {
            Some(self.segments[0].0)
        } else {
            None
        }
    }
    /// Replaces the leading identifier with a new one.
    pub fn replace_leading_ident(mut self, mut prefix: Self) -> Self {
        assert!(self.leading.is_none());
        prefix.segments.reserve(self.segments.len().saturating_sub(1));
        assert!(!prefix.segments.is_empty());
        prefix
            .segments
            .last_mut()
            .unwrap()
            .1
            .extend(std::mem::take(&mut self.segments[0].1));
        prefix.segments.extend(self.segments.into_iter().skip(1));
        prefix._span = self._span;
        prefix
    }
}

impl fmt::Debug for Path<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(leading) = self.leading {
            write!(f, "{}", leading.span.as_str().trim())?;
        }
        for (i, segment) in self.segments.iter().enumerate() {
            if i > 0 {
                write!(f, "::")?;
            }
            write!(f, "{}", segment.0)?;
            if !segment.1.is_empty() {
                write!(f, "::<")?;
                for (j, arg) in segment.1.iter().enumerate() {
                    if j > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg.span.as_str().trim())?;
                }
                write!(f, ">")?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for Path<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

#[macro_export]
macro_rules! collect_elems_separated_by_comma {
    ($decls:expr) => {{
        let (first, following, _) = $decls.get_matched();
        let following = following
            .iter_matched()
            .map(|comma_with_elem| comma_with_elem.get_matched().1);
        std::iter::once(first).chain(following)
    }};
}

pub trait Record: Sized {
    type Ok;
    type Err;

    fn or_record(self, errors: &mut Vec<Self::Err>) -> Option<Self::Ok>;
    fn or_record_and_default(self, errors: &mut Vec<Self::Err>) -> Self::Ok
    where
        Self::Ok: Default,
    {
        self.or_record(errors).unwrap_or_default()
    }
}

impl<T, E> Record for Result<T, E> {
    type Ok = T;
    type Err = E;

    fn or_record(self, errors: &mut Vec<<Self as Record>::Err>) -> Option<<Self as Record>::Ok> {
        match self {
            Ok(value) => Some(value),
            Err(err) => {
                errors.push(err);
                None
            },
        }
    }
}
