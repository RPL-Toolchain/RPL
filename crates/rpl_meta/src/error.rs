//! Error type from RPL meta pass.

use error_enum::error_type;
use parser::{ParseError, SpanWrapper};
use pest_typed::Span;
use rustc_span::Symbol;
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;

// TODO: 排版
error_type!(
    #[derive(Clone, Debug)]
    pub RPLMetaError<'i>
        #[color = "red"]
        #[bold]
        Error "Error" {
            001 ParseError {
                error: Arc<ParseError<'i>>,
            }
                "Parse error.\n {error}",
            100 FileError {
                /// Referencing file.
                path: PathBuf,
                /// Cause.
                error: Arc<std::io::Error>,
            }
                "Cannot locate RPL pattern file `{path:?}`. Caused by: {error}",
            200 ImportError {
                /// Referencing position.
                span: SpanWrapper<'i>,
                /// Referencing file.
                path: &'i PathBuf,
                /// Cause.
                error: Arc<std::io::Error>,
            }
                "Cannot locate RPL pattern file `{path:?}` at {span}. Caused by:\n{error}",
            301 SymbolAlreadyDeclared {
                ident: Symbol,
                span: SpanWrapper<'i>,
            }
                "Symbol `{ident}` is already declared. \n{span}",
            302 SymbolNotDeclared {
                ident: Symbol,
                span: SpanWrapper<'i>,
            }
                "Symbol `{ident}` is not declared. \n{span}",
            303 NonLocalMetaVariableAlreadyDeclared {
                meta_var: Symbol,
                span: SpanWrapper<'i>,
            }
                "Non local meta variable `{meta_var}` is already declared. \n{span}",
            304 NonLocalMetaVariableNotDeclared {
                meta_var: Symbol,
                span: SpanWrapper<'i>,
            }
                "Non local meta variable `{meta_var}` is not declared. \n{span}",
            305 ExportAlreadyDeclared {
                _span: Span<'i>,
            }
                "Export is already declared.",
            306 TypeOrPathAlreadyDeclared {
                type_or_path: Symbol,
                span: SpanWrapper<'i>,
            }
                "Type or path `{type_or_path}` is already declared. \n{span}",
            307 TypeOrPathNotDeclared {
                type_or_path: Symbol,
                span: SpanWrapper<'i>,
            }
                "Type or path `{type_or_path}` is not declared. \n{span}",
            308 MethodAlreadyDeclared {
                span: SpanWrapper<'i>,
            }
                "Method is already declared. \n{span}",
            309 MethodNotDeclared {
            }
                "Method is not declared.",
            310 SelfNotDeclared {
                span: SpanWrapper<'i>,
            }
                "`self` is not declared. \n{span}",
            311 SelfAlreadyDeclared {
                span: SpanWrapper<'i>,
            }
                "`self` is already declared. \n{span}",
            312 SelfValueOutsideImpl {
            }
                "Using `self` value outside of an `impl` item.",
            313 SelfTypeOutsideImpl {
                span: SpanWrapper<'i>,
            }
                "Using `Self` type outside of an `impl` item. \n{span}",
            314 ConstantIndexOutOfBound {
                index: SpanWrapper<'i>,
                min_length: SpanWrapper<'i>,
            }
                "Constant index out of bound for minimum length. \n Index: {index} \n Minimum length: {min_length}",
            315 MultipleOtherwiseInSwitchInt {
                span: SpanWrapper<'i>,
            }
                "Multiple otherwise (`_`) branches in switchInt statement. \n{span}",
            316 MissingSuffixInSwitchInt {
                span: SpanWrapper<'i>,
            }
                "Missing integer suffix in switchInt statement. \n{span}",
            317 UnknownLangItem {
                value: Symbol,
                span: SpanWrapper<'i>,
            }
                "Unknown lang item `{value}`. \n{span}",
            318 RetNotDeclared {
                span: SpanWrapper<'i>,
            }
                "The return value `RET` in MIR pattern is not declared. \n{span}",
            319 UnknownPredicate {
                pred_name: String,
                span: SpanWrapper<'i>,
            }
                "Unknown predicate `{pred_name}`. \n{span}",
            320 ImplAlreadyDeclared {
                span: SpanWrapper<'i>,
            }
                "Impl already declared. \n{span}",
        }
);

impl<'i> From<ParseError<'i>> for RPLMetaError<'i> {
    fn from(value: ParseError<'i>) -> Self {
        Self::ParseError { error: value.into() }
    }
}
impl<'a> RPLMetaError<'a> {
    /// Wrap [`std::io::Error`] as canonicalizating failure.
    pub fn file_error(error: std::io::Error, span: Option<Span<'a>>, path: &'a PathBuf) -> Self {
        let error = Arc::new(error);
        if let Some(span) = span {
            let span = SpanWrapper::new(span, path);
            Self::ImportError { path, error, span }
        } else {
            let path = path.clone();
            Self::FileError { path, error }
        }
    }
}

impl std::error::Error for RPLMetaError<'_> {}

pub(crate) type RPLMetaResult<'a, T> = Result<T, RPLMetaError<'a>>;

// impl Diagnostic<'_, ErrorGuaranteed> for RPLMetaError<'_> {
//     fn into_diag(self, dcx: DiagCtxtHandle<'_>, level: Level) -> Diag<'_, ErrorGuaranteed> {
//         match self {
//             Self::ParseError { error } => error.into_diag(dcx, level),
//             Self::FileError { path, error } => {
//                 dcx.struct_err(format!("Cannot locate RPL pattern file `{path:?}`. Caused by:
// {error}"))             },
//             Self::ImportError { span, path, error } => dcx.struct_span_err(
//                 span_cvt(span),
//                 format!("Cannot locate RPL pattern file `{path:?}` at {span}. Caused
// by:\n{error}"),             ),
//             Self::SymbolAlreadyDeclared { ident, span } => {
//                 dcx.struct_span_err(span_cvt(span), format!("Symbol `{ident}` is already
// declared."))             },
//             Self::SymbolNotDeclared { ident, span } => {
//                 dcx.struct_span_err(span_cvt(span), format!("Symbol `{ident}` is not declared."))
//             },
//             Self::NonLocalMetaVariableAlreadyDeclared { meta_var, span } => dcx.struct_span_err(
//                 span_cvt(span),
//                 format!("Non local meta variable `{meta_var}` is already declared."),
//             ),
//             Self::NonLocalMetaVariableNotDeclared { meta_var, span } => dcx.struct_span_err(
//                 span_cvt(span),
//                 format!("Non local meta variable `{meta_var}` is not declared."),
//             ),
//             Self::ExportAlreadyDeclared { _span } => dcx.struct_err("Export is already
// declared."),             Self::TypeOrPathAlreadyDeclared { type_or_path, span } =>
// dcx.struct_span_err(                 span_cvt(span),
//                 format!("Type or path `{type_or_path}` is already declared."),
//             ),
//             Self::TypeOrPathNotDeclared { type_or_path, span } => dcx.struct_span_err(
//                 span_cvt(span),
//                 format!("Type or path `{type_or_path}` is not declared."),
//             ),
//             Self::MethodAlreadyDeclared { span } => dcx.struct_span_err(span_cvt(span), "Method
// is already declared."),             Self::MethodNotDeclared {} => dcx.struct_err("Method is not
// declared."),             Self::SelfNotDeclared { span } => dcx.struct_span_err(span_cvt(span),
// "`self` is not declared."),             Self::SelfAlreadyDeclared { span } =>
// dcx.struct_span_err(span_cvt(span), "`self` is already declared."),
// Self::SelfValueOutsideImpl {} => dcx.struct_err("Using `self` value outside of an `impl` item."),
//             Self::SelfTypeOutsideImpl { span } => {
//                 dcx.struct_span_err(span_cvt(span), "Using `Self` type outside of an `impl`
// item.")             },
//             Self::ConstantIndexOutOfBound { index, min_length } => dcx.struct_span_err(
//                 span_cvt(index),
//                 format!(
//                     "Constant index out of bound for minimum length. \n Index: {index} \n Minimum
// length: {min_length}"                 ),
//             ),
//             Self::MultipleOtherwiseInSwitchInt { span } => dcx.struct_span_err(
//                 span_cvt(span),
//                 "Multiple otherwise (`_`) branches in switchInt statement.",
//             ),
//             Self::MissingSuffixInSwitchInt { span } => {
//                 dcx.struct_span_err(span_cvt(span), "Missing integer suffix in switchInt
// statement.")             },
//             Self::UnknownLangItem { value, span } => {
//                 dcx.struct_span_err(span_cvt(span), format!("Unknown lang item `{value}`."))
//             },
//             Self::RetNotDeclared { span } => {
//                 dcx.struct_span_err(span_cvt(span), "The return value `RET` in MIR pattern is not
// declared.")             },
//             Self::ImplAlreadyDeclared { span } => dcx.struct_span_err(span_cvt(span), "Impl
// already declared."),             #[expect(unreachable_patterns, reason = "all variants are
// covered")]             _ => dcx.struct_err(self.to_string()),
//         }
//     }
// }
