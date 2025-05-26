use std::borrow::Cow;

use rustc_errors::{IntoDiagArg, LintDiagnostic};
use rustc_lint::Lint;
use rustc_macros::LintDiagnostic;
use rustc_middle::ty::{self, Ty};
use rustc_span::{Span, Symbol};

pub struct Mutability(ty::Mutability);

impl From<ty::Mutability> for Mutability {
    fn from(mutability: ty::Mutability) -> Self {
        Self(mutability)
    }
}

impl IntoDiagArg for Mutability {
    fn into_diag_arg(self) -> rustc_errors::DiagArgValue {
        self.0.prefix_str().into_diag_arg()
    }
}

#[derive(LintDiagnostic)]
#[diag(rpl_patterns_unsound_slice_cast)]
pub struct UnsoundSliceCast<'tcx> {
    #[note]
    pub cast_from: Span,
    #[label(rpl_patterns_cast_to_label)]
    pub cast_to: Span,
    pub ty: Ty<'tcx>,
    pub mutability: Mutability,
}

#[derive(LintDiagnostic)]
#[diag(rpl_patterns_use_after_drop)]
pub struct UseAfterDrop<'tcx> {
    #[note]
    pub drop_span: Span,
    #[label(rpl_patterns_use_label)]
    pub use_span: Span,
    pub ty: Ty<'tcx>,
}

#[derive(LintDiagnostic)]
#[diag(rpl_patterns_use_after_move)]
pub struct UseAfterMove<'tcx> {
    #[note]
    pub move_span: Span,
    #[label(rpl_patterns_use_label)]
    pub use_span: Span,
    pub ty: Ty<'tcx>,
}

#[derive(LintDiagnostic)]
#[diag(rpl_patterns_unchecked_allocated_pointer)]
#[note]
pub struct UncheckedAllocatedPointer<'tcx> {
    #[label(rpl_patterns_alloc_label)]
    pub alloc: Span,
    #[label(rpl_patterns_write_label)]
    pub write: Span,
    pub ty: Ty<'tcx>,
}

#[derive(LintDiagnostic)]
#[diag(rpl_patterns_misaligned_pointer)]
#[note]
pub struct MisalignedPointer<'tcx> {
    #[label(rpl_patterns_alloc_label)]
    pub alloc: Span,
    #[label(rpl_patterns_write_label)]
    pub write: Span,
    pub ty: Ty<'tcx>,
}

#[derive(LintDiagnostic)]
#[diag(rpl_patterns_alloc_maybe_zero)]
#[note]
pub struct AllocMaybeZero {
    #[label(rpl_patterns_alloc_label)]
    pub alloc: Span,
    #[label(rpl_patterns_size_label)]
    pub size: Span,
    pub fn_name: Symbol,
    pub alloc_fn: &'static str,
}

#[derive(LintDiagnostic)]
#[diag(rpl_patterns_use_after_realloc)]
#[note]
pub struct UseAfterRealloc<'tcx> {
    #[label(rpl_patterns_realloc_label)]
    pub realloc: Span,
    #[label(rpl_patterns_use_label)]
    pub r#use: Span,
    pub ty: Ty<'tcx>,
}

#[derive(LintDiagnostic)]
#[diag(rpl_patterns_offset_by_one)]
pub struct OffsetByOne {
    #[label(rpl_patterns_read_label)]
    pub read: Span,
    #[label(rpl_patterns_ptr_label)]
    pub ptr: Span,
    #[help]
    #[suggestion(code = "({len_local} - 1)")]
    pub len: Span,
    pub len_local: String,
}

// for cve_2018_21000
#[derive(LintDiagnostic)]
#[diag(rpl_patterns_misordered_parameters)]
pub struct MisorderedParameters {
    #[help]
    #[label(rpl_patterns_label)]
    pub span: Span,
}

// for cve_2020_35881
#[derive(LintDiagnostic)]
#[diag(rpl_patterns_wrong_assumption_of_fat_pointer_layout)]
#[help]
pub struct WrongAssumptionOfFatPointerLayout {
    #[label(rpl_patterns_ptr_transmute_label)]
    pub ptr_transmute: Span,
    #[label(rpl_patterns_get_data_ptr_label)]
    pub data_ptr_get: Span,
}

// for cve_2019_15548
#[derive(LintDiagnostic)]
#[diag(rpl_patterns_rust_str_as_c_str)]
#[help]
pub struct RustStrAsCStr {
    #[label(rpl_patterns_label)]
    pub cast_from: Span,
    #[note]
    pub cast_to: Span,
}

// another pattern for cve_2019_15548
#[derive(LintDiagnostic)]
#[diag(rpl_patterns_lengthless_buffer_passed_to_extern_function)]
pub struct LengthlessBufferPassedToExternFunction {
    #[label(rpl_patterns_label)]
    pub ptr: Span,
}

// for cve_2021_27376
#[derive(LintDiagnostic)]
#[diag(rpl_patterns_wrong_assumption_of_layout_compatibility)]
#[help]
pub struct WrongAssumptionOfLayoutCompatibility {
    #[label(rpl_patterns_cast_to_label)]
    pub cast_to: Span,
    #[note]
    pub cast_from: Span,
    pub type_to: &'static str,
    pub type_from: &'static str,
}

// for cve_2021_27376
#[derive(LintDiagnostic)]
#[diag(rpl_patterns_trust_exact_size_iterator)]
#[help]
pub struct TrustExactSizeIterator {
    #[label(rpl_patterns_label)]
    pub set_len: Span,
    #[label(rpl_patterns_len_label)]
    pub len: Span,
    pub fn_name: &'static str,
}

// for CVE-2021-29941 and CVE-2021-29942
#[derive(LintDiagnostic)]
#[diag(rpl_patterns_slice_from_raw_parts_uninitialized)]
#[help]
pub struct SliceFromRawPartsUninitialized {
    #[label(rpl_patterns_slice_label)]
    pub slice: Span,
    #[label(rpl_patterns_len_label)]
    pub len: Span,
    #[label(rpl_patterns_ptr_label)]
    pub ptr: Span,
    #[label(rpl_patterns_vec_label)]
    pub vec: Span,
    pub fn_name: &'static str,
}

// for cve_2018_20992
// use `Vec::set_len` to extend the length of a `Vec` without initializing the new elements
#[derive(LintDiagnostic)]
#[diag(rpl_patterns_vec_set_len_to_extend)]
#[note]
pub struct VecSetLenToExtend {
    #[label(rpl_patterns_set_len_label)]
    pub set_len: Span,
    #[label(rpl_patterns_vec_label)]
    pub vec: Span,
}

// for cve_2019_16138
#[derive(LintDiagnostic)]
#[diag(rpl_patterns_set_len_uninitialized)]
#[help]
pub struct SetLenUninitialized {
    #[label(rpl_patterns_set_len_label)]
    pub set_len: Span,
    #[label(rpl_patterns_vec_label)]
    pub vec: Span,
}

// for cve_2020_35898_9
#[derive(LintDiagnostic)]
#[diag(rpl_patterns_get_mut_in_rc_unsafecell)]
#[help]
pub struct GetMutInRcUnsafeCell {
    #[label(rpl_patterns_get_mut_label)]
    #[note]
    #[help]
    pub get_mut: Span,
}

// for cve_2020_35888
#[derive(LintDiagnostic)]
#[diag(rpl_patterns_drop_uninit_value)]
#[help]
pub struct DropUninitValue {
    #[label(rpl_patterns_drop_label)]
    pub drop: Span,
    #[label(rpl_patterns_alloc_label)]
    pub alloc: Span,
    #[label(rpl_patterns_ptr_label)]
    pub ptr: Span,
    #[label(rpl_patterns_assign_label)]
    pub assign: Span,
}

// for cve_2020_35907
#[derive(LintDiagnostic)]
#[diag(rpl_patterns_thread_local_static_ref)]
#[help(rpl_patterns_sync_help)]
#[help]
pub struct ThreadLocalStaticRef<'tcx> {
    #[label(rpl_patterns_fn_label)]
    pub span: Span,
    #[label(rpl_patterns_thread_local_label)]
    pub thread_local: Span,
    #[label(rpl_patterns_ret_label)]
    pub ret: Span,
    pub ty: Ty<'tcx>,
}

// for cve_2021_25904
// FIXME: add a span for `#[help]` containing the function header
#[derive(LintDiagnostic)]
#[diag(rpl_patterns_unvalidated_slice_from_raw_parts)]
#[help]
pub struct UnvalidatedSliceFromRawParts {
    #[label(rpl_patterns_src_label)]
    pub src: Span,
    #[label(rpl_patterns_ptr_label)]
    pub ptr: Span,
    #[label(rpl_patterns_slice_label)]
    pub slice: Span,
}

// for cve_2022_23639
#[derive(LintDiagnostic)]
#[diag(rpl_patterns_unsound_cast_between_u64_and_atomic_u64)]
#[note]
pub struct UnsoundCastBetweenU64AndAtomicU64 {
    #[label(rpl_patterns_cast_label)]
    pub transmute: Span,
    #[label(rpl_patterns_src_label)]
    pub src: Span,
}

// for cve_2020_35860
#[derive(LintDiagnostic)]
#[diag(rpl_patterns_deref_null_pointer)]
#[note]
pub struct DerefNullPointer {
    #[label(rpl_patterns_deref_label)]
    pub deref: Span,
    #[label(rpl_patterns_ptr_label)]
    pub ptr: Span,
}

// for cve_2020_35877
#[derive(LintDiagnostic)]
#[diag(rpl_patterns_deref_unchecked_ptr_offset)]
#[help]
pub struct DerefUncheckedPtrOffset {
    #[label(rpl_patterns_reference_label)]
    pub reference: Span,
    #[label(rpl_patterns_ptr_label)]
    pub ptr: Span,
    #[label(rpl_patterns_offset_label)]
    pub offset: Span,
}

// for cve_2020_35901
#[derive(LintDiagnostic)]
#[diag(rpl_patterns_unsound_pin_project)]
#[note]
pub struct UnsoundPinNewUnchecked<'tcx> {
    #[label(rpl_patterns_pin_label)]
    pub span: Span,
    #[label(rpl_patterns_ref_label)]
    pub mut_self: Span,
    pub ty: Ty<'tcx>,
}

// for cve_2020_35877
#[derive(LintDiagnostic)]
#[diag(rpl_patterns_unchecked_ptr_offset)]
#[help]
#[note]
pub struct UncheckedPtrOffset {
    #[label(rpl_patterns_offset_label)]
    pub offset: Span,
    #[label(rpl_patterns_ptr_label)]
    pub ptr: Span,
}

// for cve_2024_27284
#[derive(LintDiagnostic)]
#[diag(rpl_patterns_cassandra_iter_next_ptr_passed_to_cass_iter_get)]
#[help]
pub struct CassandraIterNextPtrPassedToCassIterGet {
    #[label(rpl_patterns_cass_iter_next_label)]
    pub cass_iter_next: Span,
}

// for cve_2021_25905
#[derive(LintDiagnostic)]
#[diag(rpl_patterns_slice_from_raw_parts_uninitialized_)]
#[help]
pub struct SliceFromRawPartsUninitialized_ {
    #[label(rpl_patterns_slice_label)]
    pub slice: Span,
    #[label(rpl_patterns_len_label)]
    pub len: Span,
    #[label(rpl_patterns_ptr_label)]
    pub ptr: Span,
    pub fn_name: &'static str,
}

#[derive(LintDiagnostic)]
#[diag(rpl_patterns_private_function_marked_inline)]
#[help]
#[note]
pub struct PrivateFunctionMarkedInline {
    #[label(rpl_patterns_label)]
    pub span: Span,
    #[label(rpl_patterns_attr_label)]
    pub attr: Span,
}

#[derive(LintDiagnostic)]
#[diag(rpl_patterns_generic_function_marked_inline)]
#[help]
#[note]
pub struct GenericFunctionMarkedInline {
    #[label(rpl_patterns_label)]
    pub span: Span,
    #[label(rpl_patterns_attr_label)]
    pub attr: Span,
}

// for std::mem::transmute : transmuting a type to a boolean
#[derive(LintDiagnostic)]
#[diag(rpl_patterns_transmuting_type_to_bool)]
#[help]
#[note]
pub struct TransmutingTypeToBool<'tcx> {
    #[label(rpl_patterns_from_label)]
    pub from: Span,
    #[label(rpl_patterns_to_label)]
    pub to: Span,
    pub ty: Ty<'tcx>,
}

// for std::mem::transmute: transmuting an integer_type to a pointer_type
#[derive(LintDiagnostic)]
#[diag(rpl_patterns_transmuting_int_to_ptr)]
#[help]
#[note]
pub struct TransmutingIntToPtr<'tcx> {
    #[label(rpl_patterns_from_label)]
    pub from: Span,
    #[label(rpl_patterns_to_label)]
    pub to: Span,
    pub int_ty: Ty<'tcx>,
    pub ptr_ty: Ty<'tcx>,
}

/// Bad operation sequence to [`std::mem::ManuallyDrop`].
#[derive(LintDiagnostic)]
#[diag(rpl_patterns_bad_manually_drop_operation_sequence)]
#[help]
pub struct BadManuallyDropOperationSequence {
    #[label(rpl_patterns_create_label)]
    pub create: Span,
    pub fn_1: &'static str,
    pub fn_2: &'static str,
    #[label(rpl_patterns_call_1_label)]
    pub call_1: Span,
    #[label(rpl_patterns_call_2_label)]
    pub call_2: Span,
}

/// A dynamic error that can be used to report user-defined errors
///
/// See:
/// - [`rustc_errors::LintDiagnostic`].
/// - [`rustc_macros::diagnostics::lint_diagnostic_derive`].
/// - [`rustc_macros::LintDiagnostic`].
pub struct DynamicError {
    /// Primary message and its span.
    ///
    /// See [`rustc_errors::Diag::primary_message`].
    /// The primary message is the main error message that will be displayed to the user.
    primary: (Cow<'static, str>, Span),
    /// Label description, and the span of the label.
    ///
    /// See [`rustc_errors::Diag::span_label`].
    /// Labels are used to highlight specific parts of the code that are relevant to the error.
    labels: Vec<(Cow<'static, str>, Span)>,
    notes: Vec<(Cow<'static, str>, Option<Span>)>,
    helps: Vec<(Cow<'static, str>, Option<Span>)>,
}

impl LintDiagnostic<'_, ()> for DynamicError {
    fn decorate_lint(self, diag: &mut rustc_errors::Diag<'_, ()>) {
        let primary_message = self.primary.0;
        diag.primary_message(primary_message);
        for (label, span) in self.labels {
            diag.span_label(span, label);
        }
        for (help, span_help) in self.helps {
            if let Some(span_help) = span_help {
                diag.span_help(span_help, help);
            } else {
                diag.help(help);
            }
        }
        for (note, span_note) in self.notes {
            if let Some(span_note) = span_note {
                diag.span_note(span_note, note);
            } else {
                diag.note(note);
            }
        }
    }
}

impl DynamicError {
    pub(crate) const fn primary_span(&self) -> Span {
        self.primary.1
    }
    pub(crate) const fn lint(&self) -> &'static Lint {
        const LINT: Lint = Lint {
            name: "RPL::DYNAMIC",
            desc: "dynamic RPL pattern",
            ..Lint::default_fields_for_macro()
        };
        &LINT
    }
    // const fn attr_error(span: Span) -> DynamicError {
    //     DynamicError {
    //         primary: (Cow::Borrowed("Ill-formed RPL dynamic attribute"), span),
    //         labels: Vec::new(),
    //         notes: Vec::new(),
    //         helps: Vec::new(),
    //     }
    // }
    fn unknown_attribute_error(span: Span) -> Self {
        Self {
            primary: (Cow::Borrowed("Unknown attribute key"), span),
            labels: Vec::new(),
            notes: vec![(
                Cow::Borrowed("Allowed attribute keys are: `primary_message`, `labels`, `note`, `help`"),
                None,
            )],
            helps: Vec::new(),
        }
    }
    const fn missing_primary_message_error(attr: &rustc_hir::Attribute) -> Self {
        Self {
            primary: (Cow::Borrowed("Missing primary message"), attr.span),
            labels: Vec::new(),
            notes: Vec::new(),
            helps: Vec::new(),
        }
    }
    fn item_to_value_str(item: &rustc_ast::MetaItemInner) -> Result<Symbol, Self> {
        item.value_str().ok_or_else(|| {
            // If the value is not a string, we return an error.
            // This is a fallback to ensure that we always return a valid error.
            Self {
                primary: (Cow::Borrowed("Expected a string value"), item.span()),
                labels: Vec::new(),
                notes: Vec::new(),
                helps: Vec::new(),
            }
        })
    }
    const fn expected_meta_item_list_error(span: Span) -> Self {
        Self {
            primary: (Cow::Borrowed("Expected a meta item list"), span),
            labels: Vec::new(),
            notes: Vec::new(),
            helps: Vec::new(),
        }
    }
    fn attr_to_meta_item_list(
        attr: &rustc_hir::Attribute,
    ) -> Result<impl Iterator<Item = rustc_ast::MetaItemInner>, Self> {
        attr.meta_item_list().map_or_else(
            || Err(Self::expected_meta_item_list_error(attr.span())),
            |vec| Ok(vec.into_iter()),
        )
    }
    fn item_to_meta_item_list(
        item: &rustc_ast::MetaItemInner,
    ) -> Result<impl Iterator<Item = &rustc_ast::MetaItemInner>, Self> {
        item.meta_item_list().map_or_else(
            || Err(Self::expected_meta_item_list_error(item.span())),
            |vec| Ok(vec.iter()),
        )
    }
    fn from_attr_impl(attr: &rustc_hir::Attribute, span: Span) -> Result<DynamicError, DynamicError> {
        let items = Self::attr_to_meta_item_list(attr)?;
        let mut primary_message = None;
        let mut labels = Vec::new();
        let mut notes = Vec::new();
        let mut helps = Vec::new();
        for item in items {
            match item.name_or_empty().as_str() {
                "primary_message" => {
                    primary_message = Some(Cow::Owned(Self::item_to_value_str(&item)?.to_string()));
                },
                "labels" => {
                    let label_list = Self::item_to_meta_item_list(&item)?;
                    for label_item in label_list {
                        // FIXME: `label_item.span()` is not the actual span it refers to,
                        labels.push((
                            Cow::Owned(Self::item_to_value_str(label_item)?.to_string()),
                            label_item.span(),
                        ));
                    }
                },
                "note" => {
                    notes.push((Cow::Owned(Self::item_to_value_str(&item)?.to_string()), None));
                },
                "help" => {
                    helps.push((Cow::Owned(Self::item_to_value_str(&item)?.to_string()), None));
                },
                _ => {
                    // error!("Unknown attribute key {:?}", item.name_or_empty())
                    return Err(Self::unknown_attribute_error(item.span()));
                },
            }
        }
        let primary_message = primary_message.ok_or_else(|| Self::missing_primary_message_error(attr))?;
        let primary = (primary_message, span);
        Ok(DynamicError {
            primary,
            labels,
            notes,
            helps,
        })
    }
    pub(crate) fn from_attr(attr: &rustc_hir::Attribute, span: Span) -> DynamicError {
        Self::from_attr_impl(attr, span).unwrap_or_else(|err| {
            // If we fail to parse the attribute, we return an error.
            // This is a fallback to ensure that we always return a valid error.
            err
        })
    }
}
