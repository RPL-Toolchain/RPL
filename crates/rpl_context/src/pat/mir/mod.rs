use core::iter::IntoIterator;
use std::fmt::{self, Debug};
use std::ops::Index;

use either::Either;
use rpl_meta::symbol_table::{LocalSpecial, WithPath};
use rpl_parser::SpanWrapper;
use rpl_parser::generics::{Choice5, Choice6, Choice12};
use rustc_abi::FieldIdx;
use rustc_data_structures::fx::{FxHashSet, FxIndexMap};
use rustc_hir::Target;
use rustc_index::IndexVec;
use rustc_middle::mir::{self, CoercionSource};
use rustc_middle::ty::adjustment::PointerCoercion;

mod pretty;
pub mod visitor;

use super::utils::{
    binop_from_pair, borrow_kind_from_pair_mutability, collect_operands, mutability_from_pair_ptr_mutability,
    nullop_from_pair, unop_from_pair,
};
pub use super::*;

pub(crate) type FnSymbolTable<'i> = rpl_meta::symbol_table::Fn<'i>;

rustc_index::newtype_index! {
    #[debug_format = "_?{}"]
    pub struct Local {}
}

rustc_index::newtype_index! {
    #[debug_format = "?bb{}"]
    pub struct BasicBlock {}
}

pub struct LocalWithIdent<'pcx> {
    pub local: Local,
    pub ident: Ident<'pcx>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Location {
    pub block: BasicBlock,
    pub statement_index: usize,
}

impl From<(BasicBlock, usize)> for Location {
    fn from((block, statement_index): (BasicBlock, usize)) -> Self {
        Self { block, statement_index }
    }
}

impl Location {
    /// Create a new `Location` that is out of bound.
    ///
    /// Must be assigned to before use.
    ///
    /// # Note
    ///
    /// The return value is actually initialized to an invalid location
    /// `Location { block: 0xFFFF_FF00u32, statement_index: usize::MAX }`.
    pub fn uninitialized() -> Self {
        Self {
            // block: BasicBlock::from(u32::MAX),
            block: BasicBlock::from(0xFFFF_FF00u32),
            statement_index: usize::MAX,
        }
    }
}

pub struct FnPatternBody<'pcx> {
    pub self_idx: Option<Local>,
    pub return_idx: Option<Local>,
    pub params_idx: FxHashSet<Local>,
    pub locals: IndexVec<Local, Ty<'pcx>>,
    pub basic_blocks: IndexVec<BasicBlock, BasicBlockData<'pcx>>,
    pub labels: LabelMap,
}

impl<'pcx> Index<BasicBlock> for FnPatternBody<'pcx> {
    type Output = BasicBlockData<'pcx>;

    fn index(&self, bb: BasicBlock) -> &Self::Output {
        &self.basic_blocks[bb]
    }
}

#[derive(Default)]
pub struct BasicBlockData<'pcx> {
    pub statements: Vec<StatementKind<'pcx>>,
    pub terminator: Option<TerminatorKind<'pcx>>,
}

impl<'pcx> BasicBlockData<'pcx> {
    pub fn has_pat_end(&self) -> bool {
        matches!(self.terminator(), TerminatorKind::PatEnd)
    }
    pub fn terminator(&self) -> &TerminatorKind<'pcx> {
        self.terminator.as_ref().expect("terminator not set")
    }
    pub fn debug_stmt_at(&self, index: usize) -> &dyn core::fmt::Debug {
        if index < self.statements.len() {
            &self.statements[index]
        } else {
            self.terminator()
        }
    }
    fn set_terminator(&mut self, terminator: TerminatorKind<'pcx>) {
        assert!(self.terminator.is_none(), "terminator already set");
        self.terminator = Some(terminator);
    }
    fn set_goto(&mut self, block: BasicBlock) {
        match &mut self.terminator {
            None => self.terminator = Some(TerminatorKind::Goto(block)),
            Some(TerminatorKind::Call { target, .. } | TerminatorKind::Drop { target, .. }) => *target = block,
            // Here the `goto ?bb` termiantor comes from `break` or `continue`,
            // plus the `return` termnator, are all skipped because thay are
            // abnormal control flows.
            Some(TerminatorKind::Goto(_) | TerminatorKind::Return) => {},
            Some(terminator @ (TerminatorKind::SwitchInt { .. } | TerminatorKind::PatEnd)) => {
                panic!("expect `{:?}`, but found `{terminator:?}`", TerminatorKind::Goto(block));
            },
        }
    }
    fn set_switch_targets(&mut self, switch_targets: SwitchTargets) {
        match &mut self.terminator {
            Some(TerminatorKind::SwitchInt { targets, .. }) => *targets = switch_targets,
            None => panic!("`switchInt` terminator not set"),
            Some(terminator) => panic!("expect `switchInt` terminator, but found `{terminator:?}`"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PlaceElem<'pcx> {
    Deref,
    Field(FieldAcc),
    FieldPat(Symbol),
    Index(Local),
    ConstantIndex {
        offset: u64,
        min_length: u64,
        from_end: bool,
    },
    Subslice {
        from: u64,
        to: u64,
        from_end: bool,
    },
    Downcast(Symbol),
    DowncastPat(Symbol),
    OpaqueCast(Ty<'pcx>),
    Subtype(Ty<'pcx>),
}

impl PlaceElem<'_> {
    /// See `rpl_meta::check::CheckFnCtxt::check_mir_place_field`.
    pub fn from_field(field: &pairs::MirPlaceField) -> Self {
        let (_, field) = field.get_matched();
        match field {
            Choice3::_0(ident) => PlaceElem::FieldPat(Symbol::intern(ident.span.as_str())),
            Choice3::_1(ident) => PlaceElem::Field(FieldAcc::from(Symbol::intern(ident.span.as_str()))),
            Choice3::_2(index) => {
                let index = index.span.as_str().trim();
                let index = index.parse::<u32>().expect("invalid field index");
                PlaceElem::Field(FieldAcc::from(index))
            },
        }
    }
}

/// Place base is the base of a place, which can be a local
/// or a [variable](`PlaceVar`) declared in meta table.
#[derive(Clone, Copy)]
pub enum PlaceBase {
    Local(Local),
    Var(PlaceVarIdx),
}

impl PlaceBase {
    pub fn as_local(self) -> Option<Local> {
        match self {
            PlaceBase::Local(local) => Some(local),
            PlaceBase::Var(_) => None,
        }
    }
}

impl Debug for PlaceBase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlaceBase::Local(local) => Debug::fmt(local, f),
            PlaceBase::Var(var) => Debug::fmt(var, f),
        }
    }
}

fn get_place_or_local(sym_tab: &FnSymbolTable<'_>, ident: Symbol) -> PlaceBase {
    if let Some(idx) = sym_tab.meta_vars.place_vars_map().get(&ident) {
        PlaceBase::Var(idx.0.into())
    } else if let Some(idx) = sym_tab.inner.symbol_to_local_idx.get(&ident) {
        PlaceBase::Local((*idx).into())
    } else {
        unreachable!("Identifier `{}` is not a place or local variable", ident);
    }
}

/// A place is a path to a value in memory.
#[derive(Clone, Copy)]
pub struct Place<'pcx, B = PlaceBase> {
    pub base: B,
    pub projection: &'pcx [PlaceElem<'pcx>],
}

impl<'pcx> Place<'pcx, PlaceBase> {
    pub fn new(local: Local, projection: &'pcx [PlaceElem<'pcx>]) -> Self {
        Self {
            base: PlaceBase::Local(local),
            projection,
        }
    }
    pub fn as_local(&self) -> Option<Local> {
        self.projection.is_empty().then(|| self.base.as_local()).flatten()
    }

    pub fn from(
        place: WithPath<'pcx, &pairs::MirPlace<'pcx>>,
        pcx: PatCtxt<'pcx>,
        sym_tab: &FnSymbolTable<'pcx>,
    ) -> Self {
        let path = place.path;
        let (base, suffix) = place.get_matched();
        let (base, mut base_projections) = match base.deref() {
            Choice3::_0(local) => match local.deref() {
                Choice4::_0(place_holder) => {
                    panic!(
                        "expect a non-placeholder local:\n{}",
                        SpanWrapper::new(place_holder.span, place.path)
                    );
                },
                _ => {
                    let base = get_place_or_local(sym_tab, Symbol::intern(local.span.as_str()));
                    (base, vec![])
                },
            },
            Choice3::_1(paren) => {
                let (_, place, _) = paren.get_matched();
                let Place { base, projection } = Place::from(WithPath::new(path, place), pcx, sym_tab);
                (base, projection.to_vec())
            },
            Choice3::_2(deref) => {
                let (_, place) = deref.get_matched();
                let Place { base, projection } = Place::from(WithPath::new(path, place), pcx, sym_tab);
                let mut new_projection = vec![PlaceElem::Deref];
                new_projection.extend(projection);
                (base, new_projection)
            },
        };
        let suffix_projections = suffix
            .iter_matched()
            .map(|suffix| match suffix.deref() {
                Choice5::_0(field) => PlaceElem::from_field(field),
                Choice5::_1(index) => {
                    let (_, local, _) = index.get_matched();
                    let local = sym_tab.inner.get_local_idx(Symbol::intern(local.span.as_str()));
                    PlaceElem::Index(Local::from(local))
                },
                Choice5::_2(const_index) => {
                    let (_, _, index, _, min_length, _) = const_index.get_matched();
                    let a = index.span.as_str().parse::<u64>().expect("invalid constant index");
                    let b = min_length.span.as_str().parse::<u64>().expect("invalid constant index");
                    PlaceElem::ConstantIndex {
                        offset: a,
                        min_length: b,
                        from_end: false, // FIXME
                    }
                },
                Choice5::_3(subslice) => {
                    let (_, from, _, minus, to, _) = subslice.get_matched();
                    let from = from
                        .as_ref()
                        .map(|from| from.span.as_str().parse::<u64>().expect("invalid subslice"));
                    let to = to
                        .as_ref()
                        .map(|to| to.span.as_str().parse::<u64>().expect("invalid subslice"));
                    PlaceElem::Subslice {
                        from: from.unwrap_or(0),
                        to: to.unwrap_or(0),
                        from_end: minus.is_some(),
                    }
                },
                Choice5::_4(downcast) => {
                    let (_, ident, _) = downcast.get_matched();
                    match ident {
                        Choice2::_0(ident) => PlaceElem::DowncastPat(Symbol::intern(ident.span.as_str())),
                        Choice2::_1(ident) => PlaceElem::Downcast(Symbol::intern(ident.span.as_str())),
                    }
                },
            })
            .collect::<Vec<_>>();
        base_projections.extend(suffix_projections);
        Self {
            base,
            projection: pcx.mk_slice(&base_projections),
        }
    }
}

impl<'pcx, B: Copy> Place<'pcx, B> {
    /// Iterate over the projections in evaluation order, i.e., the first element is the base with
    /// its projection and then subsequently more projections are added.
    /// As a concrete example, given the place a.b.c, this would yield:
    /// - (a, .b)
    /// - (a.b, .c)
    ///
    /// Given a place without projections, the iterator is empty.
    #[inline]
    pub fn iter_projections(self) -> impl DoubleEndedIterator<Item = (Self, PlaceElem<'pcx>)> {
        self.projection.iter().enumerate().map(move |(i, proj)| {
            let base = Place {
                base: self.base,
                projection: &self.projection[..i],
            };
            (base, *proj)
        })
    }

    /// Identity.
    pub fn into_place(self) -> Self {
        self
    }
}

impl<B> From<B> for Place<'_, B> {
    fn from(base: B) -> Self {
        Place { base, projection: &[] }
    }
}

impl From<Local> for Place<'_, PlaceBase> {
    fn from(local: Local) -> Self {
        Place {
            base: PlaceBase::Local(local),
            projection: &[],
        }
    }
}

impl From<PlaceVarIdx> for Place<'_, PlaceBase> {
    fn from(var: PlaceVarIdx) -> Self {
        Place {
            base: PlaceBase::Var(var),
            projection: &[],
        }
    }
}

impl Local {
    pub fn into_place<'pcx>(self) -> Place<'pcx> {
        self.into()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PlaceTy<'pcx> {
    pub ty: Ty<'pcx>,
    pub variant: Option<Symbol>,
}

impl<'pcx> PlaceTy<'pcx> {
    pub fn from_ty(ty: Ty<'pcx>) -> Self {
        Self { ty, variant: None }
    }
    pub fn projection_ty(&self, pat: &'pcx RustItems<'pcx>, proj: PlaceElem<'pcx>) -> Option<Self> {
        match proj {
            PlaceElem::Deref => match self.ty.kind() {
                &TyKind::Ref(_, ty, _) | &TyKind::RawPtr(ty, _) => Some(PlaceTy::from_ty(ty)),
                _ => None,
            },
            PlaceElem::Field(_) => None,
            PlaceElem::FieldPat(field) => {
                let &TyKind::AdtPat(adt) = self.ty.kind() else {
                    return None;
                };
                let adt = pat.get_adt(adt)?;
                let variant = if adt.is_enum() {
                    adt.variant(
                        self.variant
                            .expect("Cannot assess field without downcasting to a variant"),
                    )
                } else {
                    adt.non_enum_variant()
                };
                Some(PlaceTy::from_ty(variant.fields.get(&field)?.ty))
            },
            PlaceElem::Index(_) | PlaceElem::ConstantIndex { .. } => match self.ty.kind() {
                &TyKind::Array(ty, _) | &TyKind::Slice(ty) => Some(PlaceTy::from_ty(ty)),
                _ => None,
            },
            PlaceElem::Subslice { .. } => match self.ty.kind() {
                &TyKind::Array(ty, _) | &TyKind::Slice(ty) => Some(PlaceTy::from_ty(pat.pcx.mk_slice_ty(ty))),
                _ => None,
            },
            PlaceElem::Downcast(_) => None,
            PlaceElem::DowncastPat(variant) => Some(PlaceTy {
                ty: self.ty,
                variant: Some(variant),
            }),
            PlaceElem::OpaqueCast(ty) | PlaceElem::Subtype(ty) => Some(PlaceTy::from_ty(ty)),
        }
    }
}

pub enum StatementKind<'pcx> {
    Assign(Place<'pcx>, Rvalue<'pcx>),
}

pub enum RawDecleration<'pcx> {
    TypeAlias(Symbol, Ty<'pcx>),
    LocalInit(Option<Label>, Local, Option<RvalueOrCall<'pcx>>), /* In meta pass, we have already collect the local
                                                                  * and its ty */
}

impl<'pcx> RawDecleration<'pcx> {
    pub fn from(
        decl: WithPath<'pcx, &'pcx pairs::MirDecl<'pcx>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &'pcx FnSymbolTable<'pcx>,
    ) -> Self {
        let p = decl.path;
        match decl.inner.deref() {
            Choice2::_0(type_alias) => {
                let (_, name, _, ty, _) = type_alias.get_matched();
                Self::TypeAlias(
                    Symbol::intern(name.span.as_str()),
                    Ty::from(WithPath::new(p, ty), pcx, fn_sym_tab),
                )
            },
            Choice2::_1(local_init) => {
                let (label, _, _, local, _, _, init, _) = local_init.get_matched();
                let local = Local::from(fn_sym_tab.inner.get_local_idx(Symbol::intern(local.span.as_str())));
                let rvalue_or_call = if let Some(init) = init {
                    let (_, init) = init.get_matched();
                    let rvalue_or_call = RvalueOrCall::from(WithPath::new(p, init), pcx, fn_sym_tab);
                    Some(rvalue_or_call)
                } else {
                    None
                };
                let label = label
                    .as_ref()
                    .map(|label| Symbol::intern(label.Label().LabelName().span.as_str()));
                Self::LocalInit(label, local, rvalue_or_call)
            },
        }
    }
}

pub enum RawStatement<'pcx> {
    Assign(Option<Label>, Place<'pcx>, Rvalue<'pcx>),
    Call(Option<Label>, Place<'pcx>, Call<'pcx>),
    CallIgnoreRet(Option<Label>, Call<'pcx>),
    Drop(Option<Label>, Place<'pcx>),
    Break,
    Continue,
    Loop(Vec<RawStatement<'pcx>>),
    SwitchInt {
        label: Option<Label>,
        operand: Operand<'pcx>,
        targets: Vec<(IntValue, Vec<RawStatement<'pcx>>)>,
        otherwise: Option<Vec<RawStatement<'pcx>>>,
    },
}

impl<'pcx> RawStatement<'pcx> {
    pub fn from(
        stmt: WithPath<'pcx, &'pcx pairs::MirStmt<'pcx>>,
        pcx: PatCtxt<'pcx>,
        sym_tab: &'pcx FnSymbolTable<'pcx>,
    ) -> Self {
        let p = stmt.path;
        match stmt.inner.deref() {
            Choice6::_0(call_ignore_ret) => {
                Self::from_call_ignore_ret(with_path(p, call_ignore_ret.get_matched().0), pcx, sym_tab)
            },
            Choice6::_1(drop_) => Self::from_drop(WithPath::new(p, drop_.get_matched().0), pcx, sym_tab),
            Choice6::_2(control) => Self::from_control(control.get_matched().0),
            Choice6::_3(assign) => Self::from_assign(WithPath::new(p, assign.get_matched().0), pcx, sym_tab),
            Choice6::_4(loop_) => Self::from_loop(WithPath::new(p, loop_), pcx, sym_tab),
            Choice6::_5(switch_int) => Self::from_switch_int(WithPath::new(p, switch_int), pcx, sym_tab),
        }
    }

    pub fn from_assign(
        stmt: WithPath<'pcx, &'pcx pairs::MirAssign<'pcx>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &'pcx FnSymbolTable<'pcx>,
    ) -> Self {
        let p = stmt.path;
        let (label, place, _, rvalue_or_call) = stmt.get_matched();
        let label = label
            .as_ref()
            .map(|label| Symbol::intern(label.Label().LabelName().span.as_str()));
        let place = Place::from(WithPath::new(p, place), pcx, fn_sym_tab);
        match rvalue_or_call.deref() {
            Choice2::_0(call) => Self::from_call_assign(label, p, place, call, pcx, fn_sym_tab),
            Choice2::_1(rvalue) => {
                let rvalue = Rvalue::from_rvalue(WithPath::new(p, rvalue), pcx, fn_sym_tab);
                Self::Assign(label, place, rvalue)
            },
        }
    }

    /// See [`mir::TerminatorKind::Call`]
    fn from_call_assign(
        label: Option<Symbol>,
        p: &'pcx std::path::Path,
        place: Place<'pcx>,
        call: &'pcx pairs::MirCall<'pcx>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &'pcx FnSymbolTable<'pcx>,
    ) -> Self {
        let call = Call::from(with_path(p, call), pcx, fn_sym_tab);
        Self::Call(label, place, call)
    }

    pub fn from_call_ignore_ret(
        call_ignore_ret: WithPath<'pcx, &'pcx pairs::MirCallIgnoreRet<'pcx>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &'pcx FnSymbolTable<'pcx>,
    ) -> Self {
        let p = call_ignore_ret.path;
        let (label, _, _, call) = call_ignore_ret.get_matched();
        let call = Call::from(with_path(p, call), pcx, fn_sym_tab);
        Self::CallIgnoreRet(
            label
                .as_ref()
                .map(|label| Symbol::intern(label.Label().LabelName().span.as_str())),
            call,
        )
    }

    pub fn from_drop(
        drop_: WithPath<'pcx, &pairs::MirDrop<'pcx>>,
        pcx: PatCtxt<'pcx>,
        sym_tab: &FnSymbolTable<'pcx>,
    ) -> Self {
        let (label, _, _, place, _) = drop_.get_matched();
        let place = Place::from(WithPath::new(drop_.path, place), pcx, sym_tab);
        Self::Drop(
            label
                .as_ref()
                .map(|label| Symbol::intern(label.Label().LabelName().span.as_str())),
            place,
        )
    }

    pub fn from_loop(
        loop_: WithPath<'pcx, &'pcx pairs::MirLoop<'pcx>>,
        pcx: PatCtxt<'pcx>,
        sym_tab: &'pcx FnSymbolTable<'pcx>,
    ) -> Self {
        let p = loop_.path;
        let (_, _, block) = loop_.get_matched();
        let statements = block
            .get_matched()
            .1
            .iter_matched()
            .map(|stmt| Self::from(WithPath::new(p, stmt), pcx, sym_tab))
            .collect();
        Self::Loop(statements)
    }

    pub fn from_control(control: &pairs::MirControl<'pcx>) -> Self {
        let (_label, break_or_continue, _label2) = control.get_matched();
        match break_or_continue {
            Choice2::_0(_break) => Self::Break,
            Choice2::_1(_continue) => Self::Continue,
        }
    }

    pub fn from_switch_int(
        switch_int: WithPath<'pcx, &'pcx pairs::MirSwitchInt<'pcx>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &'pcx FnSymbolTable<'pcx>,
    ) -> Self {
        let p = switch_int.path;
        let (label, _, _, op, _, _, targets, _) = switch_int.get_matched();
        let operand = Operand::from(with_path(p, op), pcx, fn_sym_tab);
        let mut target_value_and_stmts: Vec<(IntValue, Vec<Self>)> = Vec::new();
        let mut otherwise_stmts: Option<Vec<Self>> = None;
        targets.iter_matched().for_each(|target| {
            let (target_value, _, block) = target.get_matched();
            let target_value = IntValue::from_switch_int_value(target_value);
            let statements = Self::from_switch_int_block(WithPath::new(p, block), pcx, fn_sym_tab);
            if let Some(target_value) = target_value {
                target_value_and_stmts.push((target_value, statements));
            } else {
                otherwise_stmts = Some(statements);
            }
        });
        let label = label
            .as_ref()
            .map(|label| Symbol::intern(label.Label().LabelName().span.as_str()));
        Self::SwitchInt {
            label,
            operand,
            targets: target_value_and_stmts,
            otherwise: otherwise_stmts,
        }
    }

    pub fn from_switch_int_block(
        block: WithPath<'pcx, &'pcx pairs::MirSwitchBody<'pcx>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &'pcx FnSymbolTable<'pcx>,
    ) -> Vec<Self> {
        let p = block.path;
        let mut stmts = Vec::new();
        match block.inner.deref() {
            Choice4::_0(mir_stmt_block) => {
                return Self::from_mir_stmt_block(WithPath::new(p, mir_stmt_block), pcx, fn_sym_tab);
            },
            Choice4::_1(single_stmt_with_comma) => {
                let (stmt, _) = single_stmt_with_comma.get_matched();
                match stmt {
                    Choice4::_0(call_ignore_ret) => {
                        stmts.push(Self::from_call_ignore_ret(
                            with_path(p, call_ignore_ret),
                            pcx,
                            fn_sym_tab,
                        ));
                    },
                    Choice4::_1(drop) => stmts.push(Self::from_drop(WithPath::new(p, drop), pcx, fn_sym_tab)),
                    Choice4::_2(control) => stmts.push(Self::from_control(control)),
                    Choice4::_3(assign) => stmts.push(Self::from_assign(WithPath::new(p, assign), pcx, fn_sym_tab)),
                };
            },
            Choice4::_2(loop_) => stmts.push(Self::from_loop(WithPath::new(p, loop_), pcx, fn_sym_tab)),
            Choice4::_3(switch_int) => stmts.push(Self::from_switch_int(WithPath::new(p, switch_int), pcx, fn_sym_tab)),
        }
        stmts
    }

    pub fn from_mir_stmt_block(
        block: WithPath<'pcx, &'pcx pairs::MirStmtBlock<'pcx>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &'pcx FnSymbolTable<'pcx>,
    ) -> Vec<Self> {
        let p = block.path;
        block
            .get_matched()
            .1
            .iter_matched()
            .map(|stmt| Self::from(WithPath::new(p, stmt), pcx, fn_sym_tab))
            .collect()
    }
}

#[derive(Default)]
pub struct SwitchTargets {
    pub targets: FxIndexMap<IntValue, BasicBlock>,
    pub otherwise: Option<BasicBlock>,
}

pub enum TerminatorKind<'pcx> {
    SwitchInt {
        operand: Operand<'pcx>,
        targets: SwitchTargets,
    },
    Goto(BasicBlock),
    Call {
        func: Operand<'pcx>,
        args: List<Operand<'pcx>>,
        destination: Option<Place<'pcx>>,
        target: BasicBlock,
    },
    Drop {
        place: Place<'pcx>,
        target: BasicBlock,
    },
    Return,
    /// Pattern ends here
    PatEnd,
}

trait Cast<T> {
    fn cast(self) -> T;
}

impl<'i> Cast<PointerCoercion> for &'i pairs::PointerCoercion<'i> {
    fn cast(self) -> PointerCoercion {
        PointerCoercion::Unsize
    }
}

impl<'i> Cast<CoercionSource> for &'i pairs::CoercionSource<'i> {
    fn cast(self) -> CoercionSource {
        CoercionSource::Implicit
    }
}

/// A value that can be used in an rvalue.
///
/// See [`mir::Rvalue`].
pub enum Rvalue<'pcx> {
    Any,
    Use(Operand<'pcx>),
    Repeat(Operand<'pcx>, Const<'pcx>),
    Ref(RegionKind, mir::BorrowKind, Place<'pcx>),
    RawPtr(mir::Mutability, Place<'pcx>),
    Len(Place<'pcx>),
    Cast(mir::CastKind, Operand<'pcx>, Ty<'pcx>),
    BinaryOp(mir::BinOp, Box<[Operand<'pcx>; 2]>),
    NullaryOp(mir::NullOp<'pcx>, Ty<'pcx>),
    UnaryOp(mir::UnOp, Operand<'pcx>),
    Discriminant(Place<'pcx>),
    Aggregate(AggKind<'pcx>, List<Operand<'pcx>>),
    ShallowInitBox(Operand<'pcx>, Ty<'pcx>),
    CopyForDeref(Place<'pcx>),
}

impl<'pcx> Rvalue<'pcx> {
    fn from_rvalue(
        rvalue: WithPath<'pcx, &'pcx pairs::MirRvalue<'pcx>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &'pcx FnSymbolTable<'pcx>,
    ) -> Self {
        let p = rvalue.path;
        match rvalue.inner.deref() {
            Choice12::_0(_any) => Rvalue::Any,
            Choice12::_1(cast) => {
                let (operand, _, ty, _, cast_kind, _) = cast.get_matched();
                let operand = Operand::from(with_path(p, operand), pcx, fn_sym_tab);
                let ty = Ty::from(WithPath::new(p, ty), pcx, fn_sym_tab);
                let cast_kind = match cast_kind.deref() {
                    Choice6::_0(_ptr_to_ptr) => mir::CastKind::PtrToPtr,
                    Choice6::_1(_int_to_int) => mir::CastKind::IntToInt,
                    Choice6::_2(_transmute) => mir::CastKind::Transmute,
                    Choice6::_3(pointer_coercion) => {
                        let (_, _, pointer_coercion, _, coercion_source, _) = pointer_coercion.get_matched();
                        mir::CastKind::PointerCoercion(pointer_coercion.cast(), coercion_source.cast())
                    },
                    Choice6::_4(_expose_provenance) => mir::CastKind::PointerExposeProvenance,
                    Choice6::_5(_with_exposed_provenance) => mir::CastKind::PointerWithExposedProvenance,
                };
                Rvalue::Cast(cast_kind, operand, ty)
            },
            Choice12::_2(rvalue_use) => {
                let operand = match rvalue_use.deref() {
                    Choice2::_0(op) => Operand::from(with_path(p, op.get_matched().1), pcx, fn_sym_tab),
                    Choice2::_1(op) => Operand::from(with_path(p, op), pcx, fn_sym_tab),
                };
                Self::Use(operand)
            },
            Choice12::_3(repeat) => {
                let (_, operand, _, count, _) = repeat.get_matched();
                let operand = Operand::from(with_path(p, operand), pcx, fn_sym_tab);
                let count = Const::from_integer(count);
                Self::Repeat(operand, count)
            },
            Choice12::_4(rvalue_ref) => {
                let (_, region, mutability, place) = rvalue_ref.get_matched();
                let region_kind = if let Some(region) = region {
                    RegionKind::from(region)
                } else {
                    RegionKind::ReAny
                };
                let mutability = borrow_kind_from_pair_mutability(mutability);
                let place = Place::from(WithPath::new(p, place), pcx, fn_sym_tab);
                Self::Ref(region_kind, mutability, place)
            },
            Choice12::_5(raw_ptr) => {
                let (_, _, ptr_mutability, place) = raw_ptr.get_matched();
                let mutability = mutability_from_pair_ptr_mutability(ptr_mutability);
                let place = Place::from(WithPath::new(p, place), pcx, fn_sym_tab);
                Self::RawPtr(mutability, place)
            },
            Choice12::_6(len) => {
                let (_, _, place, _) = len.get_matched();
                let place = Place::from(WithPath::new(p, place), pcx, fn_sym_tab);
                Self::Len(place)
            },
            Choice12::_7(bin_op) => {
                let (bin_op, _, lop, _, rop, _) = bin_op.get_matched();
                let bin_op = binop_from_pair(bin_op);
                let lop = Operand::from(with_path(p, lop), pcx, fn_sym_tab);
                let rop = Operand::from(with_path(p, rop), pcx, fn_sym_tab);
                Self::BinaryOp(bin_op, Box::new([lop, rop]))
            },
            Choice12::_8(nullary_op) => {
                let (nullary_op, _, ty, _) = nullary_op.get_matched();
                let nullary_op = nullop_from_pair(nullary_op);
                let ty = Ty::from(WithPath::new(p, ty), pcx, fn_sym_tab);
                Self::NullaryOp(nullary_op, ty)
            },
            Choice12::_9(un_op) => {
                let (un_op, _, operand, _) = un_op.get_matched();
                let un_op = unop_from_pair(un_op);
                let operand = Operand::from(with_path(p, operand), pcx, fn_sym_tab);
                Self::UnaryOp(un_op, operand)
            },
            Choice12::_10(discriminant) => {
                let (_, _, place, _) = discriminant.get_matched();
                let place = Place::from(WithPath::new(p, place), pcx, fn_sym_tab);
                Self::Discriminant(place)
            },
            Choice12::_11(agg) => {
                let (agg_kind, operands) = AggKind::from(WithPath::new(p, agg), pcx, fn_sym_tab);
                Self::Aggregate(agg_kind, operands)
            },
        }
    }
}

#[derive(Clone)]
pub enum Operand<'pcx> {
    Any,
    Copy(Place<'pcx>),
    Move(Place<'pcx>),
    Constant(ConstOperand<'pcx>),
    FnPat(Symbol),
}

impl<'pcx> Operand<'pcx> {
    pub fn from(
        op: WithPath<'pcx, &'pcx pairs::MirOperand<'pcx>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &'pcx FnSymbolTable<'pcx>,
    ) -> Self {
        let p = op.path;
        match op.inner.deref() {
            Choice6::_0(_any) => Self::Any,
            Choice6::_1(_any_multiple) => Self::Any, // FIXME
            Choice6::_2(meta_var) => Self::from_meta_var(meta_var),
            Choice6::_3(move_) => Self::from_move(WithPath::new(p, move_), pcx, fn_sym_tab),
            Choice6::_4(copy_) => Self::from_copy(WithPath::new(p, copy_), pcx, fn_sym_tab),
            Choice6::_5(konst) => Self::from_constant(WithPath::new(p, konst), pcx, fn_sym_tab),
        }
    }

    pub fn from_meta_var(meta_var: &pairs::MetaVariable<'_>) -> Self {
        Self::FnPat(Symbol::intern(meta_var.span.as_str()))
    }

    pub fn from_move(
        move_: WithPath<'pcx, &pairs::MirOperandMove<'pcx>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &FnSymbolTable<'pcx>,
    ) -> Self {
        Self::Move(Place::from(move_.map(|move_| move_.MirPlace()), pcx, fn_sym_tab))
    }

    pub fn from_copy(
        copy_: WithPath<'pcx, &pairs::MirOperandCopy<'pcx>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &FnSymbolTable<'pcx>,
    ) -> Self {
        Self::Copy(Place::from(copy_.map(|copy_| copy_.MirPlace()), pcx, fn_sym_tab))
    }

    pub fn from_constant(
        konst: WithPath<'pcx, &'pcx pairs::MirOperandConst<'_>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &'pcx FnSymbolTable<'pcx>,
    ) -> Self {
        Self::Constant(ConstOperand::from(konst, pcx, fn_sym_tab))
    }

    pub fn from_fn_op(
        op: WithPath<'pcx, &'pcx pairs::MirFnOperand<'pcx>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &'pcx FnSymbolTable<'pcx>,
    ) -> Self {
        let p = op.path;
        match op.inner.deref() {
            Choice5::_0(copy_) => Self::from_copy(WithPath::new(p, copy_.get_matched().1), pcx, fn_sym_tab),
            Choice5::_1(move_) => Self::from_move(WithPath::new(p, move_.get_matched().1), pcx, fn_sym_tab),
            Choice5::_2(type_path) => Self::Constant(ConstOperand::from_type_path(
                WithPath::new(p, type_path),
                pcx,
                fn_sym_tab,
            )),
            Choice5::_3(lang_item) => Self::Constant(ConstOperand::from_lang_item(
                WithPath::new(p, lang_item),
                pcx,
                fn_sym_tab,
            )),
            Choice5::_4(meta_var) => Self::from_meta_var(meta_var),
        }
    }
}

pub struct Call<'pcx>(Operand<'pcx>, Vec<Operand<'pcx>>);

impl<'pcx> Call<'pcx> {
    pub fn from(
        call: WithPath<'pcx, &'pcx pairs::MirCall<'pcx>>,
        pcx: PatCtxt<'pcx>,
        sym_tab: &'pcx FnSymbolTable<'pcx>,
    ) -> Self {
        let p = call.path;
        let (fn_op, _, args, _) = call.get_matched();
        let func = Operand::from_fn_op(WithPath::new(p, fn_op), pcx, sym_tab);
        let args = collect_operands(args.as_ref().map(|args| with_path(p, args)), pcx, sym_tab);
        Self(func, args)
    }
}

pub enum RvalueOrCall<'pcx> {
    Rvalue(Rvalue<'pcx>),
    Call(Call<'pcx>),
}

impl<'pcx> RvalueOrCall<'pcx> {
    pub fn from(
        rvalue_or_call: WithPath<'pcx, &'pcx pairs::MirRvalueOrCall<'pcx>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &'pcx FnSymbolTable<'pcx>,
    ) -> Self {
        let p = rvalue_or_call.path;
        match rvalue_or_call.inner.deref() {
            Choice2::_0(call) => Self::Call(Call::from(WithPath::new(p, call), pcx, fn_sym_tab)),
            Choice2::_1(rvalue) => Self::Rvalue(Rvalue::from_rvalue(WithPath::new(p, rvalue), pcx, fn_sym_tab)),
        }
    }
}

pub type List<T> = Box<[T]>;

#[derive(Clone)]
pub enum ConstOperand<'pcx> {
    ConstVar(ConstVar<'pcx>),
    ScalarInt(IntValue),
    ZeroSized(PathWithArgs<'pcx>),
}

impl<'pcx> ConstOperand<'pcx> {
    fn from(
        op: WithPath<'pcx, &'pcx pairs::MirOperandConst<'_>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &'pcx FnSymbolTable<'pcx>,
    ) -> Self {
        let p = op.path;
        let (_, op) = op.get_matched();
        match op {
            Choice4::_0(lit) => Self::from_literal(lit),
            Choice4::_1(lang_item_with_args) => {
                Self::from_lang_item(WithPath::new(p, lang_item_with_args), pcx, fn_sym_tab)
            },
            Choice4::_2(type_path) => Self::from_type_path(WithPath::new(p, type_path), pcx, fn_sym_tab),
            Choice4::_3(meta_var) => Self::from_meta_var(WithPath::new(p, meta_var), pcx, fn_sym_tab),
        }
    }

    fn from_meta_var(
        meta_var: WithPath<'pcx, &'pcx pairs::MetaVariable<'_>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &'pcx FnSymbolTable<'pcx>,
    ) -> Self {
        let (idx, ty, pred) = fn_sym_tab
            .meta_vars
            .get_from_symbol(Symbol::intern(meta_var.span.as_str()))
            .unwrap()
            .expect_const();
        Self::ConstVar(ConstVar::from(
            pcx,
            fn_sym_tab,
            idx,
            WithPath::new(meta_var.path, ty),
            pred,
        ))
    }

    fn from_type_path(
        type_path: WithPath<'pcx, &'pcx pairs::TypePath<'_>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &'pcx FnSymbolTable<'pcx>,
    ) -> Self {
        Self::ZeroSized(PathWithArgs::from_type_path(type_path, pcx, fn_sym_tab))
    }

    fn from_lang_item(
        lang_item: WithPath<'pcx, &'pcx pairs::LangItemWithArgs<'_>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &'pcx FnSymbolTable<'pcx>,
    ) -> Self {
        Self::ZeroSized(PathWithArgs::from_lang_item(lang_item, pcx, fn_sym_tab))
    }

    fn from_literal(lit: &pairs::Literal<'_>) -> Self {
        match lit.deref() {
            Choice3::_0(integer) => Self::ScalarInt(IntValue::from_integer(integer)),
            _ => todo!("literal other than integer as const operand"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AggAdtKind {
    Unit,
    Tuple,
    Struct(List<Symbol>),
}

impl From<List<Symbol>> for AggAdtKind {
    fn from(fields: List<Symbol>) -> Self {
        AggAdtKind::Struct(fields)
    }
}

#[derive(Debug, Clone)]
pub enum AggKind<'pcx> {
    Array,
    Tuple,
    Adt(PathWithArgs<'pcx>, AggAdtKind),
    RawPtr(Ty<'pcx>, mir::Mutability),
}

impl<'pcx> AggKind<'pcx> {
    pub fn from(
        agg: WithPath<'pcx, &'pcx pairs::MirRvalueAggregate<'pcx>>,
        pcx: PatCtxt<'pcx>,
        fn_sym_tab: &'pcx FnSymbolTable<'pcx>,
    ) -> (Self, List<Operand<'pcx>>) {
        let p = agg.path;
        match agg.inner.deref() {
            Choice6::_0(array) => {
                let (_, operands, _) = array.get_matched();
                let operands = collect_operands(
                    operands.as_ref().map(|operands| with_path(p, operands)),
                    pcx,
                    fn_sym_tab,
                );
                (Self::Array, operands.into_boxed_slice())
            },
            Choice6::_1(tuple) => {
                let (_, operands, _) = tuple.get_matched();
                let operands = collect_operands(
                    operands.as_ref().map(|operands| with_path(p, operands)),
                    pcx,
                    fn_sym_tab,
                );
                (Self::Tuple, operands.into_boxed_slice())
            },
            Choice6::_2(adt_struct) => {
                let (path_or_lang_item, _, fields, _) = adt_struct.get_matched();
                let path_or_lang_item =
                    PathWithArgs::from_path_or_lang_item(WithPath::new(p, path_or_lang_item), pcx, fn_sym_tab);
                let (symbol_list, op_list): (List<Symbol>, List<Operand>) = if let Some(fields) = fields {
                    let fields = collect_elems_separated_by_comma!(fields);
                    let (symbols, ops): (Vec<Symbol>, Vec<Operand>) = fields
                        .map(|field| {
                            (
                                Symbol::intern(field.Identifier().span.as_str()),
                                Operand::from(with_path(p, field.MirOperand()), pcx, fn_sym_tab),
                            )
                        })
                        .unzip();
                    (symbols.into_boxed_slice(), ops.into_boxed_slice())
                } else {
                    (Box::new([]), Box::new([]))
                };
                let kind = AggAdtKind::Struct(symbol_list);
                (Self::Adt(path_or_lang_item, kind), op_list)
            },
            Choice6::_3(tuple) => {
                let (_, _, _, _, path, _, operands, _) = tuple.get_matched();
                let path = PathWithArgs::from_path(WithPath::new(p, path), pcx, fn_sym_tab);
                let operands = collect_operands(
                    operands.as_ref().map(|operands| with_path(p, operands)),
                    pcx,
                    fn_sym_tab,
                );
                (Self::Adt(path, AggAdtKind::Tuple), operands.into_boxed_slice())
            },
            Choice6::_4(unit) => {
                let unit = unit.deref();
                let path_or_lang_item = PathWithArgs::from_path_or_lang_item(WithPath::new(p, unit), pcx, fn_sym_tab);
                (Self::Adt(path_or_lang_item, AggAdtKind::Unit), Box::new([]))
            },
            Choice6::_5(raw_ptr) => {
                let (ty_ptr, _, _, op1, _, op2, _) = raw_ptr.get_matched();
                let (_, ptr_mutability, ty) = ty_ptr.get_matched();
                let ty = Ty::from(WithPath::new(p, ty), pcx, fn_sym_tab);
                let mutability = mutability_from_pair_ptr_mutability(ptr_mutability);
                let operands = Box::new([
                    Operand::from(with_path(p, op1), pcx, fn_sym_tab),
                    Operand::from(with_path(p, op2), pcx, fn_sym_tab),
                ]);
                (Self::RawPtr(ty, mutability), operands)
            },
        }
    }
}

#[derive(Clone, Copy)]
pub enum FieldAcc {
    Named(Symbol),
    Unnamed(FieldIdx),
}

impl From<&str> for FieldAcc {
    fn from(name: &str) -> Self {
        Symbol::intern(name).into()
    }
}

impl From<Symbol> for FieldAcc {
    fn from(name: Symbol) -> Self {
        FieldAcc::Named(name)
    }
}

impl From<u32> for FieldAcc {
    fn from(field: u32) -> Self {
        FieldIdx::from_u32(field).into()
    }
}

impl From<FieldIdx> for FieldAcc {
    fn from(field: FieldIdx) -> Self {
        FieldAcc::Unnamed(field)
    }
}

pub struct FnPatternBodyBuilder<'pcx> {
    pattern: FnPatternBody<'pcx>,
    loop_stack: Vec<Loop>,
    current: BasicBlock,
}

struct Loop {
    enter: BasicBlock,
    exit: BasicBlock,
}

impl<'pcx> FnPatternBody<'pcx> {
    pub fn builder() -> FnPatternBodyBuilder<'pcx> {
        FnPatternBodyBuilder::new()
    }
    pub fn stmt_at(&self, loc: Location) -> Either<&StatementKind<'pcx>, &TerminatorKind<'pcx>> {
        if loc.statement_index < self[loc.block].statements.len() {
            Either::Left(&self[loc.block].statements[loc.statement_index])
        } else {
            Either::Right(self[loc.block].terminator())
        }
    }
}

impl<'pcx> FnPatternBodyBuilder<'pcx> {
    fn new() -> Self {
        let mut pattern = FnPatternBody {
            locals: IndexVec::new(),
            return_idx: None,
            self_idx: None,
            params_idx: FxHashSet::default(),
            basic_blocks: IndexVec::new(),
            labels: FxHashMap::default(),
        };
        let current = pattern.basic_blocks.push(BasicBlockData::default());
        Self {
            pattern,
            loop_stack: Vec::new(),
            current,
        }
    }

    pub fn build(mut self, name: Symbol, output: Option<Symbol>) -> FnPatternBody<'pcx> {
        self.new_block_if_terminated();
        self.pattern.basic_blocks[self.current].set_terminator(TerminatorKind::PatEnd);

        // If the function name starts with `$`, it now refers to the function's body.
        let name_str = name.as_str();
        if let Some(ident) = name_str.strip_prefix("$") {
            _ = self.pattern.labels.try_insert(Symbol::intern(ident), Spanned::Body);
        }

        // If `output` is `Some`, it refers to the function's output type.
        if let Some(output) = output {
            _ = self.pattern.labels.try_insert(output, Spanned::Output);
        }

        self.pattern
    }

    pub fn mk_locals(&mut self, fn_sym_tab: &'pcx FnSymbolTable<'pcx>, pcx: PatCtxt<'pcx>) {
        let WithPath { path, inner: locals } = fn_sym_tab.inner.get_sorted_locals();
        for (label, _, ty, special) in locals {
            let ty = Ty::from(with_path(path, ty), pcx, fn_sym_tab);
            let local = self.mk_local(ty);
            match special {
                LocalSpecial::Return => {
                    self.pattern.return_idx = Some(local);
                },
                LocalSpecial::Self_ => {
                    self.pattern.self_idx = Some(local);
                },
                LocalSpecial::Arg => {
                    self.pattern.params_idx.insert(local);
                },
                LocalSpecial::None => {},
            }
            if let Some(label) = label {
                self.pattern.labels.insert(label, Spanned::Local(local));
            }
        }
    }

    fn mk_local(&mut self, ty: Ty<'pcx>) -> Local {
        self.pattern.locals.push(ty)
    }

    #[allow(unused)]
    fn mk_return(&mut self, ty: Ty<'pcx>) -> Local {
        *self.pattern.return_idx.insert(self.pattern.locals.push(ty))
    }

    #[allow(unused)]
    fn mk_self(&mut self, ty: Ty<'pcx>) -> Local {
        *self.pattern.self_idx.insert(self.pattern.locals.push(ty))
    }

    fn new_block_if_terminated(&mut self) {
        if self.pattern.basic_blocks[self.current].terminator.is_some() {
            self.current = self.pattern.basic_blocks.push(BasicBlockData::default());
        }
    }
    fn next_block(&mut self) -> BasicBlock {
        self.new_block_if_terminated();
        self.pattern.basic_blocks.next_index()
    }

    pub fn mk_raw_stmts(&mut self, stmts: impl IntoIterator<Item = RawStatement<'pcx>>) {
        for stmt in stmts {
            let _loc = self.mk_raw_stmt(stmt);
        }
    }

    fn mk_raw_stmt(&mut self, kind: RawStatement<'pcx>) -> Location {
        match kind {
            RawStatement::Assign(label, place, rvalue) => self.mk_assign(label, StatementKind::Assign(place, rvalue)),
            RawStatement::Call(label, place, Call(func, args)) => {
                self.mk_fn_call(label, func, args.into_boxed_slice(), Some(place))
            },
            RawStatement::CallIgnoreRet(label, Call(func, args)) => {
                self.mk_fn_call(label, func, args.into_boxed_slice(), None)
            },
            RawStatement::Drop(label, place) => self.mk_drop(label, place),
            RawStatement::Break => self.mk_break(),
            RawStatement::Continue => self.mk_continue(),
            RawStatement::Loop(stmts) => self.mk_loop(stmts),
            RawStatement::SwitchInt {
                label,
                operand,
                targets,
                otherwise,
            } => self.mk_switch_int(label, operand, targets, otherwise),
        }
    }

    pub fn mk_raw_decls(&mut self, decls: impl IntoIterator<Item = RawDecleration<'pcx>>) {
        for decl in decls {
            self.mk_raw_decl(decl);
        }
    }

    fn mk_raw_decl(&mut self, kind: RawDecleration<'pcx>) {
        if let RawDecleration::LocalInit(label, local, Some(rvalue_or_call)) = kind {
            match rvalue_or_call {
                RvalueOrCall::Rvalue(rvalue) => _ = self.mk_assign(label, StatementKind::Assign(local.into(), rvalue)),
                RvalueOrCall::Call(call) => {
                    _ = self.mk_fn_call(label, call.0, call.1.into_boxed_slice(), Some(local.into()))
                },
            }
        }
    }

    fn mk_assign(&mut self, label: Option<Label>, assign: StatementKind<'pcx>) -> Location {
        self.new_block_if_terminated();

        let block = self.current;
        let statement_index = self.pattern.basic_blocks[block].statements.len();

        self.pattern.basic_blocks[block].statements.push(assign);
        let loc = Location { block, statement_index };
        if let Some(label) = label {
            self.pattern.labels.insert(label, Spanned::Location(loc));
        }
        loc
    }

    fn set_terminator(&mut self, kind: TerminatorKind<'pcx>) -> Location {
        self.pattern.basic_blocks[self.current].set_terminator(kind);
        self.pattern.terminator_loc(self.current)
    }

    pub fn mk_fn_call(
        &mut self,
        label: Option<Label>,
        func: Operand<'pcx>,
        args: List<Operand<'pcx>>,
        destination: Option<Place<'pcx>>,
    ) -> Location {
        if let Some(place) = destination
            && let Operand::Constant(ConstOperand::ZeroSized(
                path_with_args @ PathWithArgs {
                    path: Path::LangItem(lang_item),
                    ..
                },
            )) = func
            && let Target::Variant | Target::Struct | Target::Union = lang_item.target()
        {
            return self.mk_assign(
                label,
                StatementKind::Assign(
                    place,
                    Rvalue::Aggregate(AggKind::Adt(path_with_args, AggAdtKind::Tuple), args),
                ),
            );
        }

        let target = self.next_block();
        let loc = self.set_terminator(TerminatorKind::Call {
            func,
            args,
            destination,
            target,
        });

        if let Some(label) = label {
            self.pattern.labels.insert(label, Spanned::Location(loc));
        }

        loc
    }
    pub fn mk_drop(&mut self, label: Option<Label>, place: impl Into<Place<'pcx>>) -> Location {
        let target = self.next_block();
        let place = place.into();
        let loc = self.set_terminator(TerminatorKind::Drop { place, target });

        if let Some(label) = label {
            self.pattern.labels.insert(label, Spanned::Location(loc));
        }

        loc
    }
    pub fn mk_switch_int(
        &mut self,
        label: Option<Label>,
        operand: Operand<'pcx>,
        target_value_and_stmts: Vec<(IntValue, Vec<RawStatement<'pcx>>)>,
        otherwise_stmts: Option<Vec<RawStatement<'pcx>>>,
    ) -> Location {
        self.new_block_if_terminated();
        let current = self.current;
        self.pattern.basic_blocks[current].set_terminator(TerminatorKind::SwitchInt {
            operand,
            targets: SwitchTargets::default(),
        });
        let next = self.pattern.basic_blocks.push(BasicBlockData::default());
        let mut targets = SwitchTargets::default();
        for (value, stmts) in target_value_and_stmts {
            self.mk_switch_target(value, stmts, &mut targets, next);
        }
        if let Some(stmts) = otherwise_stmts {
            self.mk_otherwise(stmts, &mut targets, next);
        }
        self.pattern.basic_blocks[current].set_switch_targets(targets);
        self.current = next;
        let loc = self.pattern.terminator_loc(current);

        if let Some(label) = label {
            self.pattern.labels.insert(label, Spanned::Location(loc));
        }

        loc
    }
    pub fn mk_switch_target(
        &mut self,
        value: IntValue,
        stmts: impl IntoIterator<Item = RawStatement<'pcx>>,
        targets: &mut SwitchTargets,
        next: BasicBlock,
    ) {
        let target = self.pattern.basic_blocks.push(BasicBlockData::default());
        targets.targets.insert(value, target);
        self.current = target;
        for stmt in stmts {
            self.mk_raw_stmt(stmt);
        }
        self.mk_goto(next);
    }

    pub fn mk_otherwise(
        &mut self,
        stmts: impl IntoIterator<Item = RawStatement<'pcx>>,
        targets: &mut SwitchTargets,
        next: BasicBlock,
    ) {
        let target = self.pattern.basic_blocks.push(BasicBlockData::default());
        targets.otherwise = Some(target);
        self.current = target;
        for stmt in stmts {
            self.mk_raw_stmt(stmt);
        }
        self.mk_goto(next);
    }

    fn mk_goto(&mut self, block: BasicBlock) -> Location {
        self.pattern.basic_blocks[self.current].set_goto(block);
        self.pattern.terminator_loc(self.current)
    }
    pub fn mk_loop(&mut self, stmts: impl IntoIterator<Item = RawStatement<'pcx>>) -> Location {
        let enter = self.pattern.basic_blocks.push(BasicBlockData::default());
        self.mk_goto(enter);
        let exit = self.pattern.basic_blocks.push(BasicBlockData::default());
        self.loop_stack.push(Loop { enter, exit });
        self.current = enter;
        for stmt in stmts {
            self.mk_raw_stmt(stmt);
        }
        self.loop_stack.pop();
        let location = self.mk_goto(enter);
        self.current = exit;
        location
    }
    pub fn mk_break(&mut self) -> Location {
        let exit = self.loop_stack.last().expect("no loop to break from").exit;
        self.mk_goto(exit)
    }
    pub fn mk_continue(&mut self) -> Location {
        let enter = self.loop_stack.last().expect("no loop to continue").enter;
        self.mk_goto(enter)
    }
}

impl<'pcx> std::ops::Deref for FnPatternBodyBuilder<'pcx> {
    type Target = FnPatternBody<'pcx>;

    fn deref(&self) -> &Self::Target {
        &self.pattern
    }
}

impl FnPatternBody<'_> {
    pub fn terminator_loc(&self, block: BasicBlock) -> Location {
        // assert the terminator is set
        let _ = self.basic_blocks[block].terminator();
        let statement_index = self.basic_blocks[block].statements.len();
        Location { block, statement_index }
    }
}

impl<'pcx> FnPatternBody<'pcx> {
    pub fn mk_zeroed(&self, path_with_args: PathWithArgs<'pcx>) -> ConstOperand<'pcx> {
        ConstOperand::ZeroSized(path_with_args)
    }
    pub fn mk_list<T>(&self, items: impl IntoIterator<Item = T>) -> List<T> {
        items.into_iter().collect()
    }
}

impl BasicBlockData<'_> {
    pub fn num_statements_and_terminator(&self) -> usize {
        self.statements.len() + self.terminator.is_some() as usize
    }
}

pub(crate) fn with_path<T>(path: &'_ std::path::Path, inner: T) -> WithPath<'_, T> {
    WithPath { path, inner }
}
