use rustc_errors::{DiagArgValue, IntoDiagArg, MultiSpan};
use rustc_macros::{Diagnostic, Subdiagnostic};
use rustc_span::Span;

use crate::utils::DumpOrPrintDiagKind;

/*
#[derive(Diagnostic)]
#[diag(rpl_utils_abort_due_to_debugging)]
#[note]
#[note(rpl_utils_remove_note)]
pub(crate) struct AbortDueToDebugging {
    #[primary_span]
    pub span: MultiSpan,
    #[subdiagnostic]
    // A `Vec` of `Subdiagnostic` now means a set of suggestions
    pub suggs: Vec<AbortDueToDebuggingSugg>,
}

#[derive(Subdiagnostic)]
#[multipart_suggestion(rpl_utils_abort_due_to_debugging_sugg, applicability = "machine-applicable")]
pub(crate) struct AbortDueToDebuggingSugg {
    #[suggestion_part(code = "")]
    pub span: Span,
}
*/

#[derive(Diagnostic)]
#[diag(rpl_utils_error_due_to_debugging)]
#[note]
#[note(rpl_utils_remove_note)]
pub(crate) struct ErrorDueToDebugging {
    #[primary_span]
    #[suggestion(code = "", applicability = "machine-applicable")]
    pub span: Span,
}

#[derive(Diagnostic)]
#[diag(rpl_utils_dump_or_print_diag)]
pub(crate) struct DumpOrPrintDiag {
    #[primary_span]
    pub span: Span,
    #[label]
    pub attr_span: Span,
    pub message: String,
    pub kind: DumpOrPrintDiagKind,
}

#[derive(Diagnostic)]
#[diag(rpl_utils_dump_mir)]
pub(crate) struct DumpMir {
    #[primary_span]
    pub span: Span,
    #[label]
    pub attr_span: Span,
    pub def_id: DefId,
    #[subdiagnostic]
    pub files: Vec<DumpMirFile>,
    #[subdiagnostic]
    pub locals_and_source_scopes: DumpMirLocalsAndSourceScopes,
    #[subdiagnostic]
    pub blocks: Vec<DumpMirBlock>,
}

#[derive(Subdiagnostic)]
#[note(rpl_utils_dump_mir_file)]
pub(crate) struct DumpMirFile {
    pub file: String,
    pub content: &'static str,
}

#[derive(Subdiagnostic)]
#[note(rpl_utils_dump_mir_locals_and_source_scopes)]
pub(crate) struct DumpMirLocalsAndSourceScopes {
    #[primary_span]
    pub multi_span: MultiSpan,
}

#[derive(Subdiagnostic)]
#[note(rpl_utils_dump_mir_block)]
pub(crate) struct DumpMirBlock {
    pub block: String,
    #[primary_span]
    pub multi_span: MultiSpan,
}

#[derive(Diagnostic)]
#[diag(rpl_utils_dump_mir_not_available)]
pub(crate) struct DumpMirNotAvailable<'tcx> {
    pub instance: Instance<'tcx>,
    #[primary_span]
    pub span: Span,
}

#[derive(Diagnostic)]
#[diag(rpl_utils_dump_mir_not_fn_path)]
pub(crate) struct DumpMirNotFnPath(#[primary_span] pub Span);

#[derive(Diagnostic)]
#[diag(rpl_utils_dump_mir_invalid)]
pub(crate) struct DumpMirInvalid(#[primary_span] pub Span);

#[derive(Diagnostic)]
#[diag(rpl_utils_dump_mir_expect_init)]
pub(crate) struct DumpMirExpectInit {
    #[primary_span]
    pub span: Span,
    #[suggestion(code = "= /* expr */", applicability = "has-placeholders")]
    pub missing: Span,
}

pub(crate) struct DefId(pub(crate) rustc_span::def_id::DefId);

impl IntoDiagArg for DefId {
    fn into_diag_arg(self) -> DiagArgValue {
        rustc_middle::ty::tls::with_context(|icx| icx.tcx.def_path_str(self.0)).into_diag_arg()
    }
}

impl From<rustc_span::def_id::DefId> for DefId {
    fn from(def_id: rustc_span::def_id::DefId) -> Self {
        DefId(def_id)
    }
}

pub(crate) struct Instance<'tcx>(pub(crate) rustc_middle::ty::Instance<'tcx>);

impl IntoDiagArg for Instance<'_> {
    fn into_diag_arg(self) -> DiagArgValue {
        self.0.to_string().into_diag_arg()
    }
}

impl<'tcx> From<rustc_middle::ty::Instance<'tcx>> for Instance<'tcx> {
    fn from(instance: rustc_middle::ty::Instance<'tcx>) -> Self {
        Instance(instance)
    }
}
