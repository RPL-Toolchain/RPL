// macro_rules! one {
//     ($x:ident) => {
//         1
//     };
// }

/// See [`rustc_hir::lang_items::LangItem`].
// The actual lang items defined come at the end of this file in one handy table.
// So you probably just want to nip down to the end.
#[rustfmt::skip]
macro_rules! language_item_table {
    (
        $( $(#[$attr:meta])* $variant:ident, $module:ident :: $name:ident, $method:ident, $target:expr, $generics:expr; )*
    ) => {
        pub fn is_lang_item(name: &str) -> bool {
            // const PREDEFINED_SYMBOLS_COUNT: usize = $(one!($name) + )* 0;
            // debug_assert_eq!(PREDEFINED_SYMBOLS_COUNT, rustc_span::symbol::PREDEFINED_SYMBOLS_COUNT);
            match name {
                $(stringify!($name) => true,)*
                _ => false,
            }
        }
    }
}

#[rustfmt::skip]
language_item_table! {
//  Variant name,            Name,                     Getter method name,         Target                  Generic requirements;
    Sized,                   sym::sized,               sized_trait,                Target::Trait,          GenericRequirement::Exact(0);
    Unsize,                  sym::unsize,              unsize_trait,               Target::Trait,          GenericRequirement::Minimum(1);
    /// Trait injected by `#[derive(PartialEq)]`, (i.e. "Partial EQ").
    StructuralPeq,           sym::structural_peq,      structural_peq_trait,       Target::Trait,          GenericRequirement::None;
    Copy,                    sym::copy,                copy_trait,                 Target::Trait,          GenericRequirement::Exact(0);
    Clone,                   sym::clone,               clone_trait,                Target::Trait,          GenericRequirement::None;
    CloneFn,                 sym::clone_fn,            clone_fn,                   Target::Method(MethodKind::Trait { body: false }), GenericRequirement::None;
    Sync,                    sym::sync,                sync_trait,                 Target::Trait,          GenericRequirement::Exact(0);
    DiscriminantKind,        sym::discriminant_kind,   discriminant_kind_trait,    Target::Trait,          GenericRequirement::None;
    /// The associated item of the `DiscriminantKind` trait.
    Discriminant,            sym::discriminant_type,   discriminant_type,          Target::AssocTy,        GenericRequirement::None;

    PointeeTrait,            sym::pointee_trait,       pointee_trait,              Target::Trait,          GenericRequirement::None;
    Metadata,                sym::metadata_type,       metadata_type,              Target::AssocTy,        GenericRequirement::None;
    DynMetadata,             sym::dyn_metadata,        dyn_metadata,               Target::Struct,         GenericRequirement::None;

    Freeze,                  sym::freeze,              freeze_trait,               Target::Trait,          GenericRequirement::Exact(0);

    FnPtrTrait,              sym::fn_ptr_trait,        fn_ptr_trait,               Target::Trait,          GenericRequirement::Exact(0);
    FnPtrAddr,               sym::fn_ptr_addr,         fn_ptr_addr,                Target::Method(MethodKind::Trait { body: false }), GenericRequirement::None;

    Drop,                    sym::drop,                drop_trait,                 Target::Trait,          GenericRequirement::None;
    Destruct,                sym::destruct,            destruct_trait,             Target::Trait,          GenericRequirement::None;

    AsyncDrop,               sym::async_drop,          async_drop_trait,           Target::Trait,          GenericRequirement::Exact(0);
    AsyncDestruct,           sym::async_destruct,      async_destruct_trait,       Target::Trait,          GenericRequirement::Exact(0);
    AsyncDropInPlace,        sym::async_drop_in_place, async_drop_in_place_fn,     Target::Fn,             GenericRequirement::Exact(1);
    SurfaceAsyncDropInPlace, sym::surface_async_drop_in_place, surface_async_drop_in_place_fn, Target::Fn, GenericRequirement::Exact(1);
    AsyncDropSurfaceDropInPlace, sym::async_drop_surface_drop_in_place, async_drop_surface_drop_in_place_fn, Target::Fn, GenericRequirement::Exact(1);
    AsyncDropSlice,          sym::async_drop_slice,    async_drop_slice_fn,        Target::Fn,             GenericRequirement::Exact(1);
    AsyncDropChain,          sym::async_drop_chain,    async_drop_chain_fn,        Target::Fn,             GenericRequirement::Exact(2);
    AsyncDropNoop,           sym::async_drop_noop,     async_drop_noop_fn,         Target::Fn,             GenericRequirement::Exact(0);
    AsyncDropDeferredDropInPlace, sym::async_drop_deferred_drop_in_place, async_drop_deferred_drop_in_place_fn, Target::Fn, GenericRequirement::Exact(1);
    AsyncDropFuse,           sym::async_drop_fuse,     async_drop_fuse_fn,         Target::Fn,             GenericRequirement::Exact(1);
    AsyncDropDefer,          sym::async_drop_defer,    async_drop_defer_fn,        Target::Fn,             GenericRequirement::Exact(1);
    AsyncDropEither,         sym::async_drop_either,   async_drop_either_fn,       Target::Fn,             GenericRequirement::Exact(3);

    CoerceUnsized,           sym::coerce_unsized,      coerce_unsized_trait,       Target::Trait,          GenericRequirement::Minimum(1);
    DispatchFromDyn,         sym::dispatch_from_dyn,   dispatch_from_dyn_trait,    Target::Trait,          GenericRequirement::Minimum(1);

    // lang items relating to transmutability
    TransmuteOpts,           sym::transmute_opts,      transmute_opts,             Target::Struct,         GenericRequirement::Exact(0);
    TransmuteTrait,          sym::transmute_trait,     transmute_trait,            Target::Trait,          GenericRequirement::Exact(2);

    Add,                     sym::add,                 add_trait,                  Target::Trait,          GenericRequirement::Exact(1);
    Sub,                     sym::sub,                 sub_trait,                  Target::Trait,          GenericRequirement::Exact(1);
    Mul,                     sym::mul,                 mul_trait,                  Target::Trait,          GenericRequirement::Exact(1);
    Div,                     sym::div,                 div_trait,                  Target::Trait,          GenericRequirement::Exact(1);
    Rem,                     sym::rem,                 rem_trait,                  Target::Trait,          GenericRequirement::Exact(1);
    Neg,                     sym::neg,                 neg_trait,                  Target::Trait,          GenericRequirement::Exact(0);
    Not,                     sym::not,                 not_trait,                  Target::Trait,          GenericRequirement::Exact(0);
    BitXor,                  sym::bitxor,              bitxor_trait,               Target::Trait,          GenericRequirement::Exact(1);
    BitAnd,                  sym::bitand,              bitand_trait,               Target::Trait,          GenericRequirement::Exact(1);
    BitOr,                   sym::bitor,               bitor_trait,                Target::Trait,          GenericRequirement::Exact(1);
    Shl,                     sym::shl,                 shl_trait,                  Target::Trait,          GenericRequirement::Exact(1);
    Shr,                     sym::shr,                 shr_trait,                  Target::Trait,          GenericRequirement::Exact(1);
    AddAssign,               sym::add_assign,          add_assign_trait,           Target::Trait,          GenericRequirement::Exact(1);
    SubAssign,               sym::sub_assign,          sub_assign_trait,           Target::Trait,          GenericRequirement::Exact(1);
    MulAssign,               sym::mul_assign,          mul_assign_trait,           Target::Trait,          GenericRequirement::Exact(1);
    DivAssign,               sym::div_assign,          div_assign_trait,           Target::Trait,          GenericRequirement::Exact(1);
    RemAssign,               sym::rem_assign,          rem_assign_trait,           Target::Trait,          GenericRequirement::Exact(1);
    BitXorAssign,            sym::bitxor_assign,       bitxor_assign_trait,        Target::Trait,          GenericRequirement::Exact(1);
    BitAndAssign,            sym::bitand_assign,       bitand_assign_trait,        Target::Trait,          GenericRequirement::Exact(1);
    BitOrAssign,             sym::bitor_assign,        bitor_assign_trait,         Target::Trait,          GenericRequirement::Exact(1);
    ShlAssign,               sym::shl_assign,          shl_assign_trait,           Target::Trait,          GenericRequirement::Exact(1);
    ShrAssign,               sym::shr_assign,          shr_assign_trait,           Target::Trait,          GenericRequirement::Exact(1);
    Index,                   sym::index,               index_trait,                Target::Trait,          GenericRequirement::Exact(1);
    IndexMut,                sym::index_mut,           index_mut_trait,            Target::Trait,          GenericRequirement::Exact(1);

    UnsafeCell,              sym::unsafe_cell,         unsafe_cell_type,           Target::Struct,         GenericRequirement::None;
    VaList,                  sym::va_list,             va_list,                    Target::Struct,         GenericRequirement::None;

    Deref,                   sym::deref,               deref_trait,                Target::Trait,          GenericRequirement::Exact(0);
    DerefMut,                sym::deref_mut,           deref_mut_trait,            Target::Trait,          GenericRequirement::Exact(0);
    DerefPure,               sym::deref_pure,          deref_pure_trait,           Target::Trait,          GenericRequirement::Exact(0);
    DerefTarget,             sym::deref_target,        deref_target,               Target::AssocTy,        GenericRequirement::None;
    Receiver,                sym::receiver,            receiver_trait,             Target::Trait,          GenericRequirement::None;
    ReceiverTarget,          sym::receiver_target,     receiver_target,            Target::AssocTy,        GenericRequirement::None;
    LegacyReceiver,          sym::legacy_receiver,     legacy_receiver_trait,      Target::Trait,          GenericRequirement::None;

    Fn,                      kw::Fn,                   fn_trait,                   Target::Trait,          GenericRequirement::Exact(1);
    FnMut,                   sym::fn_mut,              fn_mut_trait,               Target::Trait,          GenericRequirement::Exact(1);
    FnOnce,                  sym::fn_once,             fn_once_trait,              Target::Trait,          GenericRequirement::Exact(1);

    AsyncFn,                 sym::async_fn,            async_fn_trait,             Target::Trait,          GenericRequirement::Exact(1);
    AsyncFnMut,              sym::async_fn_mut,        async_fn_mut_trait,         Target::Trait,          GenericRequirement::Exact(1);
    AsyncFnOnce,             sym::async_fn_once,       async_fn_once_trait,        Target::Trait,          GenericRequirement::Exact(1);
    AsyncFnOnceOutput,       sym::async_fn_once_output, async_fn_once_output,       Target::AssocTy,        GenericRequirement::Exact(1);
    CallOnceFuture,          sym::call_once_future,    call_once_future,           Target::AssocTy,        GenericRequirement::Exact(1);
    CallRefFuture,           sym::call_ref_future,     call_ref_future,            Target::AssocTy,        GenericRequirement::Exact(2);
    AsyncFnKindHelper,       sym::async_fn_kind_helper, async_fn_kind_helper,      Target::Trait,          GenericRequirement::Exact(1);
    AsyncFnKindUpvars,       sym::async_fn_kind_upvars, async_fn_kind_upvars,      Target::AssocTy,          GenericRequirement::Exact(5);

    FnOnceOutput,            sym::fn_once_output,      fn_once_output,             Target::AssocTy,        GenericRequirement::None;

    Iterator,                sym::iterator,            iterator_trait,             Target::Trait,          GenericRequirement::Exact(0);
    FusedIterator,           sym::fused_iterator,      fused_iterator_trait,       Target::Trait,          GenericRequirement::Exact(0);
    Future,                  sym::future_trait,        future_trait,               Target::Trait,          GenericRequirement::Exact(0);
    FutureOutput,            sym::future_output,       future_output,              Target::AssocTy,        GenericRequirement::Exact(0);
    AsyncIterator,           sym::async_iterator,      async_iterator_trait,       Target::Trait,          GenericRequirement::Exact(0);

    CoroutineState,          sym::coroutine_state,     coroutine_state,            Target::Enum,           GenericRequirement::None;
    Coroutine,               sym::coroutine,           coroutine_trait,            Target::Trait,          GenericRequirement::Exact(1);
    CoroutineReturn,         sym::coroutine_return,    coroutine_return,           Target::AssocTy,        GenericRequirement::Exact(1);
    CoroutineYield,          sym::coroutine_yield,     coroutine_yield,            Target::AssocTy,        GenericRequirement::Exact(1);
    CoroutineResume,         sym::coroutine_resume,    coroutine_resume,           Target::Method(MethodKind::Trait { body: false }), GenericRequirement::None;

    Unpin,                   sym::unpin,               unpin_trait,                Target::Trait,          GenericRequirement::None;
    Pin,                     sym::pin,                 pin_type,                   Target::Struct,         GenericRequirement::None;

    OrderingEnum,            sym::Ordering,            ordering_enum,              Target::Enum,           GenericRequirement::Exact(0);
    PartialEq,               sym::eq,                  eq_trait,                   Target::Trait,          GenericRequirement::Exact(1);
    PartialOrd,              sym::partial_ord,         partial_ord_trait,          Target::Trait,          GenericRequirement::Exact(1);
    CVoid,                   sym::c_void,              c_void,                     Target::Enum,           GenericRequirement::None;

    // A number of panic-related lang items. The `panic` item corresponds to divide-by-zero and
    // various panic cases with `match`. The `panic_bounds_check` item is for indexing arrays.
    //
    // The `begin_unwind` lang item has a predefined symbol name and is sort of a "weak lang item"
    // in the sense that a crate is not required to have it defined to use it, but a final product
    // is required to define it somewhere. Additionally, there are restrictions on crates that use
    // a weak lang item, but do not have it defined.
    Panic,                   sym::panic,               panic_fn,                   Target::Fn,             GenericRequirement::Exact(0);
    PanicNounwind,           sym::panic_nounwind,      panic_nounwind,             Target::Fn,             GenericRequirement::Exact(0);
    PanicFmt,                sym::panic_fmt,           panic_fmt,                  Target::Fn,             GenericRequirement::None;
    ConstPanicFmt,           sym::const_panic_fmt,     const_panic_fmt,            Target::Fn,             GenericRequirement::None;
    PanicBoundsCheck,        sym::panic_bounds_check,  panic_bounds_check_fn,      Target::Fn,             GenericRequirement::Exact(0);
    PanicMisalignedPointerDereference, sym::panic_misaligned_pointer_dereference, panic_misaligned_pointer_dereference_fn, Target::Fn, GenericRequirement::Exact(0);
    PanicInfo,               sym::panic_info,          panic_info,                 Target::Struct,         GenericRequirement::None;
    PanicLocation,           sym::panic_location,      panic_location,             Target::Struct,         GenericRequirement::None;
    PanicImpl,               sym::panic_impl,          panic_impl,                 Target::Fn,             GenericRequirement::None;
    PanicCannotUnwind,       sym::panic_cannot_unwind, panic_cannot_unwind,        Target::Fn,             GenericRequirement::Exact(0);
    PanicInCleanup,          sym::panic_in_cleanup,    panic_in_cleanup,           Target::Fn,             GenericRequirement::Exact(0);
    /// Constant panic messages, used for codegen of MIR asserts.
    PanicAddOverflow,        sym::panic_const_add_overflow, panic_const_add_overflow, Target::Fn, GenericRequirement::None;
    PanicSubOverflow,        sym::panic_const_sub_overflow, panic_const_sub_overflow, Target::Fn, GenericRequirement::None;
    PanicMulOverflow,        sym::panic_const_mul_overflow, panic_const_mul_overflow, Target::Fn, GenericRequirement::None;
    PanicDivOverflow,        sym::panic_const_div_overflow, panic_const_div_overflow, Target::Fn, GenericRequirement::None;
    PanicRemOverflow,        sym::panic_const_rem_overflow, panic_const_rem_overflow, Target::Fn, GenericRequirement::None;
    PanicNegOverflow,        sym::panic_const_neg_overflow, panic_const_neg_overflow, Target::Fn, GenericRequirement::None;
    PanicShrOverflow,        sym::panic_const_shr_overflow, panic_const_shr_overflow, Target::Fn, GenericRequirement::None;
    PanicShlOverflow,        sym::panic_const_shl_overflow, panic_const_shl_overflow, Target::Fn, GenericRequirement::None;
    PanicDivZero,            sym::panic_const_div_by_zero, panic_const_div_by_zero, Target::Fn, GenericRequirement::None;
    PanicRemZero,            sym::panic_const_rem_by_zero, panic_const_rem_by_zero, Target::Fn, GenericRequirement::None;
    PanicCoroutineResumed, sym::panic_const_coroutine_resumed, panic_const_coroutine_resumed, Target::Fn, GenericRequirement::None;
    PanicAsyncFnResumed, sym::panic_const_async_fn_resumed, panic_const_async_fn_resumed, Target::Fn, GenericRequirement::None;
    PanicAsyncGenFnResumed, sym::panic_const_async_gen_fn_resumed, panic_const_async_gen_fn_resumed, Target::Fn, GenericRequirement::None;
    PanicGenFnNone, sym::panic_const_gen_fn_none, panic_const_gen_fn_none, Target::Fn, GenericRequirement::None;
    PanicCoroutineResumedPanic, sym::panic_const_coroutine_resumed_panic, panic_const_coroutine_resumed_panic, Target::Fn, GenericRequirement::None;
    PanicAsyncFnResumedPanic, sym::panic_const_async_fn_resumed_panic, panic_const_async_fn_resumed_panic, Target::Fn, GenericRequirement::None;
    PanicAsyncGenFnResumedPanic, sym::panic_const_async_gen_fn_resumed_panic, panic_const_async_gen_fn_resumed_panic, Target::Fn, GenericRequirement::None;
    PanicGenFnNonePanic, sym::panic_const_gen_fn_none_panic, panic_const_gen_fn_none_panic, Target::Fn, GenericRequirement::None;
    PanicNullPointerDereference, sym::panic_null_pointer_dereference, panic_null_pointer_dereference, Target::Fn, GenericRequirement::None;
    /// libstd panic entry point. Necessary for const eval to be able to catch it
    BeginPanic,              sym::begin_panic,         begin_panic_fn,             Target::Fn,             GenericRequirement::None;

    // Lang items needed for `format_args!()`.
    FormatAlignment,         sym::format_alignment,    format_alignment,           Target::Enum,           GenericRequirement::None;
    FormatArgument,          sym::format_argument,     format_argument,            Target::Struct,         GenericRequirement::None;
    FormatArguments,         sym::format_arguments,    format_arguments,           Target::Struct,         GenericRequirement::None;
    FormatCount,             sym::format_count,        format_count,               Target::Enum,           GenericRequirement::None;
    FormatPlaceholder,       sym::format_placeholder,  format_placeholder,         Target::Struct,         GenericRequirement::None;
    FormatUnsafeArg,         sym::format_unsafe_arg,   format_unsafe_arg,          Target::Struct,         GenericRequirement::None;

    ExchangeMalloc,          sym::exchange_malloc,     exchange_malloc_fn,         Target::Fn,             GenericRequirement::None;
    DropInPlace,             sym::drop_in_place,       drop_in_place_fn,           Target::Fn,             GenericRequirement::Minimum(1);
    FallbackSurfaceDrop,     sym::fallback_surface_drop, fallback_surface_drop_fn, Target::Fn,             GenericRequirement::None;
    AllocLayout,             sym::alloc_layout,        alloc_layout,               Target::Struct,         GenericRequirement::None;

    /// For all binary crates without `#![no_main]`, Rust will generate a "main" function.
    /// The exact name and signature are target-dependent. The "main" function will invoke
    /// this lang item, passing it the `argc` and `argv` (or null, if those don't exist
    /// on the current target) as well as the user-defined `fn main` from the binary crate.
    Start,                   sym::start,               start_fn,                   Target::Fn,             GenericRequirement::Exact(1);

    EhPersonality,           sym::eh_personality,      eh_personality,             Target::Fn,             GenericRequirement::None;
    EhCatchTypeinfo,         sym::eh_catch_typeinfo,   eh_catch_typeinfo,          Target::Static,         GenericRequirement::None;

    OwnedBox,                sym::owned_box,           owned_box,                  Target::Struct,         GenericRequirement::Minimum(1);
    GlobalAlloc,             sym::global_alloc_ty,     global_alloc_ty,            Target::Struct,         GenericRequirement::None;

    // Experimental lang item for Miri
    PtrUnique,               sym::ptr_unique,          ptr_unique,                 Target::Struct,         GenericRequirement::Exact(1);

    PhantomData,             sym::phantom_data,        phantom_data,               Target::Struct,         GenericRequirement::Exact(1);

    ManuallyDrop,            sym::manually_drop,       manually_drop,              Target::Struct,         GenericRequirement::None;
    BikeshedGuaranteedNoDrop, sym::bikeshed_guaranteed_no_drop, bikeshed_guaranteed_no_drop, Target::Trait, GenericRequirement::Exact(0);

    MaybeUninit,             sym::maybe_uninit,        maybe_uninit,               Target::Union,          GenericRequirement::None;

    Termination,             sym::termination,         termination,                Target::Trait,          GenericRequirement::None;

    Try,                     sym::Try,                 try_trait,                  Target::Trait,          GenericRequirement::None;

    Tuple,                   sym::tuple_trait,         tuple_trait,                Target::Trait,          GenericRequirement::Exact(0);

    SliceLen,                sym::slice_len_fn,        slice_len_fn,               Target::Method(MethodKind::Inherent), GenericRequirement::None;

    // Language items from AST lowering
    TryTraitFromResidual,    sym::from_residual,       from_residual_fn,           Target::Method(MethodKind::Trait { body: false }), GenericRequirement::None;
    TryTraitFromOutput,      sym::from_output,         from_output_fn,             Target::Method(MethodKind::Trait { body: false }), GenericRequirement::None;
    TryTraitBranch,          sym::branch,              branch_fn,                  Target::Method(MethodKind::Trait { body: false }), GenericRequirement::None;
    TryTraitFromYeet,        sym::from_yeet,           from_yeet_fn,               Target::Fn,             GenericRequirement::None;

    PointerLike,             sym::pointer_like,        pointer_like,               Target::Trait,          GenericRequirement::Exact(0);

    CoercePointeeValidated, sym::coerce_pointee_validated, coerce_pointee_validated_trait, Target::Trait,     GenericRequirement::Exact(0);

    ConstParamTy,            sym::const_param_ty,      const_param_ty_trait,       Target::Trait,          GenericRequirement::Exact(0);
    UnsizedConstParamTy,     sym::unsized_const_param_ty, unsized_const_param_ty_trait, Target::Trait, GenericRequirement::Exact(0);

    Poll,                    sym::Poll,                poll,                       Target::Enum,           GenericRequirement::None;
    PollReady,               sym::Ready,               poll_ready_variant,         Target::Variant,        GenericRequirement::None;
    PollPending,             sym::Pending,             poll_pending_variant,       Target::Variant,        GenericRequirement::None;

    AsyncGenReady,           sym::AsyncGenReady,       async_gen_ready,            Target::Method(MethodKind::Inherent), GenericRequirement::Exact(1);
    AsyncGenPending,         sym::AsyncGenPending,     async_gen_pending,          Target::AssocConst,     GenericRequirement::Exact(1);
    AsyncGenFinished,        sym::AsyncGenFinished,    async_gen_finished,         Target::AssocConst,     GenericRequirement::Exact(1);

    // FIXME(swatinem): the following lang items are used for async lowering and
    // should become obsolete eventually.
    ResumeTy,                sym::ResumeTy,            resume_ty,                  Target::Struct,         GenericRequirement::None;
    GetContext,              sym::get_context,         get_context_fn,             Target::Fn,             GenericRequirement::None;

    Context,                 sym::Context,             context,                    Target::Struct,         GenericRequirement::None;
    FuturePoll,              sym::poll,                future_poll_fn,             Target::Method(MethodKind::Trait { body: false }), GenericRequirement::None;

    AsyncIteratorPollNext,   sym::async_iterator_poll_next, async_iterator_poll_next, Target::Method(MethodKind::Trait { body: false }), GenericRequirement::Exact(0);
    IntoAsyncIterIntoIter,   sym::into_async_iter_into_iter, into_async_iter_into_iter, Target::Method(MethodKind::Trait { body: false }), GenericRequirement::Exact(0);

    Option,                  sym::Option,              option_type,                Target::Enum,           GenericRequirement::None;
    OptionSome,              sym::Some,                option_some_variant,        Target::Variant,        GenericRequirement::None;
    OptionNone,              sym::None,                option_none_variant,        Target::Variant,        GenericRequirement::None;

    ResultOk,                sym::Ok,                  result_ok_variant,          Target::Variant,        GenericRequirement::None;
    ResultErr,               sym::Err,                 result_err_variant,         Target::Variant,        GenericRequirement::None;

    ControlFlowContinue,     sym::Continue,            cf_continue_variant,        Target::Variant,        GenericRequirement::None;
    ControlFlowBreak,        sym::Break,               cf_break_variant,           Target::Variant,        GenericRequirement::None;

    IntoFutureIntoFuture,    sym::into_future,         into_future_fn,             Target::Method(MethodKind::Trait { body: false }), GenericRequirement::None;
    IntoIterIntoIter,        sym::into_iter,           into_iter_fn,               Target::Method(MethodKind::Trait { body: false }), GenericRequirement::None;
    IteratorNext,            sym::next,                next_fn,                    Target::Method(MethodKind::Trait { body: false}), GenericRequirement::None;

    PinNewUnchecked,         sym::new_unchecked,       new_unchecked_fn,           Target::Method(MethodKind::Inherent), GenericRequirement::None;

    RangeFrom,               sym::RangeFrom,           range_from_struct,          Target::Struct,         GenericRequirement::None;
    RangeFull,               sym::RangeFull,           range_full_struct,          Target::Struct,         GenericRequirement::None;
    RangeInclusiveStruct,    sym::RangeInclusive,      range_inclusive_struct,     Target::Struct,         GenericRequirement::None;
    RangeInclusiveNew,       sym::range_inclusive_new, range_inclusive_new_method, Target::Method(MethodKind::Inherent), GenericRequirement::None;
    Range,                   sym::Range,               range_struct,               Target::Struct,         GenericRequirement::None;
    RangeToInclusive,        sym::RangeToInclusive,    range_to_inclusive_struct,  Target::Struct,         GenericRequirement::None;
    RangeTo,                 sym::RangeTo,             range_to_struct,            Target::Struct,         GenericRequirement::None;

    // `new_range` types that are `Copy + IntoIterator`
    RangeFromCopy,           sym::RangeFromCopy,       range_from_copy_struct,     Target::Struct,         GenericRequirement::None;
    RangeCopy,               sym::RangeCopy,           range_copy_struct,          Target::Struct,         GenericRequirement::None;
    RangeInclusiveCopy,      sym::RangeInclusiveCopy,  range_inclusive_copy_struct, Target::Struct,         GenericRequirement::None;

    String,                  sym::String,              string,                     Target::Struct,         GenericRequirement::None;
    CStr,                    sym::CStr,                c_str,                      Target::Struct,         GenericRequirement::None;

    // Experimental lang items for implementing contract pre- and post-condition checking.
    ContractBuildCheckEnsures, sym::contract_build_check_ensures, contract_build_check_ensures_fn, Target::Fn, GenericRequirement::None;
    ContractCheckRequires,     sym::contract_check_requires,      contract_check_requires_fn,      Target::Fn, GenericRequirement::None;
}
