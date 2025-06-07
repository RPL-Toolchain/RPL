use std::fmt::{Debug, Display};
use std::path::Path;

use colored::Colorize;
use serde::Serialize;
use serde::ser::SerializeTuple;

/// Human-readable and serializable wrapper of [Span](pest_typed::Span).
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct SpanWrapper<'a> {
    inner: pest_typed::Span<'a>,
    path: &'a Path,
}

impl<'a> SpanWrapper<'a> {
    /// Create a new span
    pub fn new(value: pest_typed::Span<'a>, path: &'a Path) -> Self {
        Self { inner: value, path }
    }

    /// Get the inner [Span](pest_typed::Span).
    pub fn inner(&self) -> pest_typed::Span<'a> {
        self.inner
    }
    /// Get the path.
    pub fn path(&self) -> &Path {
        self.path
    }
}

impl Debug for SpanWrapper<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.inner, f)
    }
}

impl Serialize for SpanWrapper<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let path = self.path.to_string_lossy();
        let start = self.inner.start_pos().line_col();
        let end = self.inner.end_pos().line_col();
        let mut s = serializer.serialize_tuple(5)?;
        s.serialize_element(path.as_ref())?;
        s.serialize_element(&start.0)?;
        s.serialize_element(&start.1)?;
        s.serialize_element(&end.0)?;
        s.serialize_element(&end.1)?;
        s.end()
    }
}

impl Display for SpanWrapper<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path = self.path.to_string_lossy();
        let start = self.inner.start_pos().line_col();
        let end = self.inner.end_pos().line_col();
        writeln!(
            f,
            "   {} \"{}\":{}.{}-{}.{}",
            "-->".blue(),
            path,
            start.0,
            start.1,
            end.0,
            end.1
        )?;
        self.inner.display(f, {
            let mut opt = Default::default();
            if false {
                // for opt's type inference
                opt
            } else {
                opt.marker_formatter = |s, f| write!(f, "{}", s.yellow());
                opt.number_formatter = |s, f| write!(f, "{}", s.blue());
                opt.span_formatter = |s, f| write!(f, "{}", s.red());
                opt
            }
        })
    }
}
