use std::ops::Deref;
use std::sync::Arc;

use impls::CheckImplCtxt;
use parser::generics::{Choice2, Choice3, Choice4, Choice5, Choice6, Choice12, Choice14};
use parser::{SpanWrapper, pairs};
use rpl_constraints::predicates::{PredicateConjunction, PredicateError};
use rustc_data_structures::fx::FxHashMap;
use rustc_span::Symbol;

use crate::context::MetaContext;
use crate::symbol_table::{
    AdtPatType, EnumInner, FnInner, GetType as _, ImplInner, NonLocalMetaSymTab, SymbolTable, Variant, WithMetaTable,
    WithPath, ident_is_primitive,
};
use crate::utils::{Ident, Path, Record};
use crate::{RPLMetaError, collect_elems_separated_by_comma};

mod impls;

/// Used for checking any errors in RPL patterns.
pub struct CheckCtxt<'i> {
    pub(crate) name: Symbol,
    pub(crate) symbol_table: SymbolTable<'i>,
    pub(crate) errors: Vec<RPLMetaError<'i>>,
}

impl<'i> CheckCtxt<'i> {
    pub fn new(name: Symbol) -> Self {
        Self {
            name,
            symbol_table: SymbolTable::default(),
            errors: Vec::new(),
        }
    }

    pub fn check_import(&mut self, mctx: &MetaContext<'i>, import: &'i pairs::UsePath<'i>) {
        let path = Path::from(import.Path());
        let ident = path.ident();
        if self.symbol_table.imports.try_insert(ident.name, import.Path()).is_err() {
            self.errors.push(RPLMetaError::SymbolAlreadyDeclared {
                span: SpanWrapper::new(import.span, mctx.get_active_path()),
                ident: ident.name,
            });
        }
    }

    pub fn check_pat_item(&mut self, mctx: &MetaContext<'i>, pat_item: &'i pairs::RPLPatternItem<'i>) {
        let (_, _, meta_decl_list, _, rust_item_or_patt_operation) = pat_item.get_matched();
        if let Some(meta_decl_list) = meta_decl_list {
            self.check_meta_decl_list(mctx, meta_decl_list);
        }
        self.check_rust_item_or_patt_operation(mctx, rust_item_or_patt_operation);
    }

    pub fn check_meta_decl_list(
        &mut self,
        mctx: &MetaContext<'i>,
        meta_decl_list: &'i pairs::MetaVariableDeclList<'i>,
    ) {
        if let Some(decls) = meta_decl_list.get_matched().1 {
            let decls = collect_elems_separated_by_comma!(decls).collect::<Vec<_>>();
            for decl in decls {
                let (ident, _, ty, preds) = decl.get_matched();
                let preds = preds.as_ref().map(|preds| preds.get_matched().1);
                self.check_pred_conjunction_opt(mctx, preds);
                let preds = if let Some(preds) = preds {
                    PredicateConjunction::from_pairs(preds, mctx.get_active_path()).unwrap_or_else(|err| {
                        self.errors.push(err.into());
                        PredicateConjunction::default()
                    })
                } else {
                    PredicateConjunction::default()
                };
                // unwrap here is safe because we check the meta decl list before checking the rust items
                // so the Arc is not cloned
                let meta_vars_ref = Arc::get_mut(&mut self.symbol_table.meta_vars).unwrap();
                meta_vars_ref.add_non_local_meta_var(mctx, ident.into(), ty, preds, &mut self.errors);
            }
        }
    }

    fn check_pred_conjunction_opt(
        &mut self,
        mctx: &MetaContext<'i>,
        preds: Option<&'i pairs::PredicateConjunction<'i>>,
    ) {
        if let Some(preds) = preds {
            let (first, following) = preds.get_matched();
            std::iter::once(first)
                .chain(following.iter_matched().map(|and_pred| and_pred.get_matched().1))
                .for_each(|pred| self.check_pred_clause(mctx, pred));
        }
    }

    fn check_pred_clause(&mut self, mctx: &MetaContext<'i>, pred: &'i pairs::PredicateClause<'i>) {
        match pred.deref() {
            Choice2::_0(pred) => self.check_pred_literal(mctx, pred),
            Choice2::_1(preds) => {
                let (_, first, following, _) = preds.get_matched();
                std::iter::once(first)
                    .chain(following.iter_matched().map(|or_pred| or_pred.get_matched().1))
                    .for_each(|pred| self.check_pred_literal(mctx, pred));
            },
        }
    }

    fn check_pred_literal(&mut self, mctx: &MetaContext<'i>, pred: &'i pairs::PredicateTerm<'i>) {
        let pred = match pred.deref() {
            Choice2::_0(pred) => pred,
            Choice2::_1(pred) => pred.get_matched().1,
        };
        let pred_name = pred.get_matched().0.span.as_str();
        if !rpl_constraints::predicates::ALL_PREDICATES.contains(&pred_name) {
            self.errors.push(
                PredicateError::InvalidPredicate {
                    pred: pred_name,
                    span: SpanWrapper::new(pred.span, mctx.get_active_path()),
                }
                .into(),
            );
        }
    }

    fn check_rust_item_or_patt_operation(
        &mut self,
        mctx: &MetaContext<'i>,
        rust_item_or_patt_operation: &'i pairs::RustItemsOrPatternOperation<'i>,
    ) {
        match rust_item_or_patt_operation.deref() {
            Choice3::_2(_patt_operations) => {
                // FIXME: process the patt operation
            },
            _ => {
                let item = rust_item_or_patt_operation.RustItemWithConstraint();
                let items = rust_item_or_patt_operation.RustItemsWithConstraint();
                let rust_items = if let Some(items) = items {
                    items.get_matched().1.iter_matched().collect::<Vec<_>>()
                } else {
                    // unwrap here is safe because the `RustItem` or `RustItems` is not `None`
                    vec![item.unwrap()]
                };
                self.check_rust_items(mctx, rust_items)
            },
        }
    }

    fn check_rust_items(&mut self, mctx: &MetaContext<'i>, rust_items: Vec<&'i pairs::RustItemWithConstraint<'i>>) {
        // FIXME: check the constraints in meta_pass
        let rust_items = rust_items
            .into_iter()
            .map(|item| item.get_matched().1)
            .collect::<Vec<_>>();
        for rust_item in rust_items {
            match rust_item.deref() {
                Choice4::_0(rust_fn) => self.check_fn(mctx, rust_fn),
                Choice4::_1(rust_struct) => self.check_struct(mctx, rust_struct),
                Choice4::_2(rust_enum) => self.check_enum(mctx, rust_enum),
                Choice4::_3(rust_impl) => self.check_impl(mctx, rust_impl),
            }
        }
    }

    fn check_fn(&mut self, mctx: &MetaContext<'i>, rust_fn: &'i pairs::Fn<'i>) {
        let fn_name = rust_fn.FnSig().FnName();
        let fn_def = self.symbol_table.add_fn(mctx, fn_name, None, &mut self.errors);
        if let Some((fn_def, imports)) = fn_def {
            CheckFnCtxt {
                meta_vars: fn_def.meta_vars.clone(),
                impl_def: None,
                fn_def: &mut fn_def.inner,
                imports,
                errors: &mut self.errors,
            }
            .check_fn(mctx, rust_fn);
        }
    }

    fn check_struct(&mut self, mctx: &MetaContext<'i>, rust_struct: &'i pairs::Struct<'i>) {
        let struct_name = rust_struct.get_matched().2;
        let struct_name = struct_name.into();
        self.symbol_table
            .meta_vars
            .add_adt_pat(mctx, struct_name, AdtPatType::Struct, &mut self.errors);
        let struct_def = self.symbol_table.add_struct(mctx, struct_name, &mut self.errors);
        if let Some(struct_def) = struct_def {
            CheckVariantCtxt {
                _meta_vars: struct_def.meta_vars.clone(),
                variant_def: &mut struct_def.inner,
                errors: &mut self.errors,
            }
            .check_struct(mctx, rust_struct);
        }
    }

    fn check_enum(&mut self, mctx: &MetaContext<'i>, rust_enum: &'i pairs::Enum<'i>) {
        let enum_name = rust_enum.get_matched().1;
        let enum_name = enum_name.into();
        self.symbol_table
            .meta_vars
            .add_adt_pat(mctx, enum_name, AdtPatType::Enum, &mut self.errors);
        let enum_def = self.symbol_table.add_enum(mctx, enum_name, &mut self.errors);
        if let Some(enum_def) = enum_def {
            CheckEnumCtxt {
                meta_vars: enum_def.meta_vars.clone(),
                enum_def: &mut enum_def.inner,
                errors: &mut self.errors,
            }
            .check_enum(mctx, rust_enum);
        }
    }

    fn check_impl(&mut self, mctx: &MetaContext<'i>, rust_impl: &'i pairs::Impl<'i>) {
        let meta_vars = self.symbol_table.meta_vars.clone();
        let impl_def = self.symbol_table.add_impl(mctx, rust_impl, &mut self.errors);
        if let Some((impl_def, imports)) = impl_def {
            CheckImplCtxt {
                meta_vars,
                impl_def: &mut impl_def.inner,
                imports,
                errors: &mut self.errors,
            }
            .check_impl(mctx, rust_impl);
        }
    }
}

struct CheckFnCtxt<'i, 'r> {
    meta_vars: Arc<NonLocalMetaSymTab<'i>>,
    #[allow(dead_code)]
    impl_def: Option<&'r ImplInner<'i>>,
    fn_def: &'r mut FnInner<'i>,
    imports: &'r FxHashMap<Symbol, &'i pairs::Path<'i>>,
    errors: &'r mut Vec<RPLMetaError<'i>>,
}

impl<'i> CheckFnCtxt<'i, '_> {
    fn check_fn(mut self, mctx: &MetaContext<'i>, rust_fn: &'i pairs::Fn<'i>) {
        let (fn_sig, fn_body) = rust_fn.get_matched();
        self.check_fn_sig(mctx, fn_sig);
        self.check_fn_body(mctx, fn_body);
    }

    fn check_fn_sig(&mut self, mctx: &MetaContext<'i>, fn_sig: &'i pairs::FnSig<'i>) {
        let (_, _, _, _, _, params, _, ret) = fn_sig.get_matched();
        if let Some(params) = params {
            let params = collect_elems_separated_by_comma!(params).collect::<Vec<_>>();
            for param in params {
                self.check_fn_param(mctx, param);
            }
        }
        if let Some(ret) = ret {
            self.check_fn_ret(mctx, ret);
        }
    }

    fn check_fn_ret(&mut self, mctx: &MetaContext<'i>, ret: &'i pairs::FnRet<'i>) {
        let (_, ret_ty) = ret.get_matched();

        match ret_ty {
            Choice2::_0(_ty_placeholder) => {},
            Choice2::_1(ty) => self.check_type(mctx, ty),
        }
    }

    fn check_fn_param(&mut self, mctx: &MetaContext<'i>, param: &'i pairs::FnParam<'i>) {
        match param.deref() {
            Choice4::_0(self_param) => self.check_self_param(mctx, self_param),
            Choice4::_1(normal_param) => self.check_normal_param(mctx, normal_param),
            _ => {
                // FIXME: the `_` and the `..` in the fn param
            },
        }
    }

    fn check_self_param(&mut self, mctx: &MetaContext<'i>, self_param: &'i pairs::SelfParam<'i>) {
        self.fn_def.add_self_param(mctx, self_param, self.errors);
    }

    fn check_normal_param(&mut self, mctx: &MetaContext<'i>, normal_param: &'i pairs::NormalParam<'i>) {
        let (_, ident, _, ty) = normal_param.get_matched();
        let ident = ident.into();
        self.fn_def.add_param(mctx, ident, ty, self.errors);
        self.check_type(mctx, ty);
    }

    fn check_fn_body(&mut self, mctx: &MetaContext<'i>, fn_body: &'i pairs::FnBody<'i>) {
        if let Some(mir) = fn_body.MirBody() {
            self.check_mir(mctx, mir);
        }
    }
}

impl<'i> CheckFnCtxt<'i, '_> {
    /// - Add collected imports
    /// - Check function body
    fn check_mir(&mut self, mctx: &MetaContext<'i>, mir: &'i pairs::MirBody<'i>) {
        let (mir_decls, mir_stmts) = mir.get_matched();
        //FIXME: the key in the line below may be reused for cloning, as `Path::from` may be expensive
        for (_, path) in self.imports.iter() {
            self.fn_def.add_import(mctx, path, self.errors);
        }
        mir_decls
            .iter_matched()
            .for_each(|decl| self.check_mir_decl(mctx, decl));
        mir_stmts
            .iter_matched()
            .for_each(|stmt| self.check_mir_stmt(mctx, stmt));
    }

    fn check_mir_stmt(&mut self, mctx: &MetaContext<'i>, stmt: &'i pairs::MirStmt<'i>) {
        match stmt.deref() {
            Choice6::_0(mir_call) => {
                let call = mir_call.get_matched().0.MirCall();
                self.check_mir_call(mctx, call);
            },
            Choice6::_1(mir_drop) => {
                let place = mir_drop.get_matched().0.MirPlace();
                self.check_mir_place(mctx, place);
            },
            Choice6::_2(control) => {
                let control = control.get_matched().0;
                self.check_mir_control(mctx, control);
            },
            Choice6::_3(mir_assign) => {
                let mir_assign = mir_assign.get_matched().0;
                self.check_mir_place(mctx, mir_assign.MirPlace());
                self.check_mir_rvalue_or_call(mctx, mir_assign.MirRvalueOrCall());
            },
            Choice6::_4(mir_loop) => self.check_mir_loop(mctx, mir_loop),

            Choice6::_5(mir_switchint) => self.check_mir_switch_int(mctx, mir_switchint),
        }
    }

    fn check_mir_switch_int(&mut self, mctx: &MetaContext<'i>, switch_int: &'i pairs::MirSwitchInt<'i>) {
        let (_, _, _, operand, _, _, targets, _) = switch_int.get_matched();
        self.check_mir_operand(mctx, operand);
        let mut has_otherwise = false;
        targets.iter_matched().for_each(|target| {
            let (value, _, body) = target.get_matched();
            if matches!(value.deref(), Choice3::_2(_)) {
                if has_otherwise {
                    self.errors.push(RPLMetaError::MultipleOtherwiseInSwitchInt {
                        span: SpanWrapper::new(value.span, mctx.get_active_path()),
                    });
                } else {
                    has_otherwise = true;
                }
            }
            self.check_switch_int_value(mctx, value);
            self.check_switch_int_body(mctx, body);
        });
    }

    fn check_switch_int_value(&mut self, mctx: &MetaContext<'i>, value: &'i pairs::MirSwitchValue<'i>) {
        match value.deref() {
            Choice3::_0(_bool) => {},
            Choice3::_1(int) => {
                let mut missing_suffix = false;
                if let Some(suffix) = int.IntegerSuffix() {
                    if !crate::symbol_table::str_is_primitive(suffix.span.as_str()) {
                        missing_suffix = true;
                    }
                } else {
                    missing_suffix = true;
                }
                if missing_suffix {
                    self.errors.push(RPLMetaError::MissingSuffixInSwitchInt {
                        span: SpanWrapper::new(int.span, mctx.get_active_path()),
                    });
                }
            },
            Choice3::_2(_placeholder) => {},
        }
    }

    fn check_switch_int_body(&mut self, mctx: &MetaContext<'i>, body: &'i pairs::MirSwitchBody<'i>) {
        match body.deref() {
            Choice4::_0(block) => {
                self.check_mir_block(mctx, block);
            },
            Choice4::_1(stmt) => {
                let stmt = stmt.get_matched().0;
                match stmt {
                    Choice4::_0(call) => self.check_mir_call(mctx, call.MirCall()),
                    Choice4::_1(drop) => self.check_mir_place(mctx, drop.MirPlace()),
                    Choice4::_2(control) => self.check_mir_control(mctx, control),
                    Choice4::_3(assign) => {
                        self.check_mir_place(mctx, assign.MirPlace());
                        self.check_mir_rvalue_or_call(mctx, assign.MirRvalueOrCall());
                    },
                }
            },
            Choice4::_2(mir_loop) => self.check_mir_loop(mctx, mir_loop),
            Choice4::_3(mir_switch_int) => self.check_mir_switch_int(mctx, mir_switch_int),
        }
    }

    fn check_mir_control(&mut self, _mctx: &MetaContext<'i>, _control: &'i pairs::MirControl<'i>) {}

    fn check_mir_loop(&mut self, mctx: &MetaContext<'i>, mir_loop: &'i pairs::MirLoop<'i>) {
        let (label, _, block) = mir_loop.get_matched();
        let _label = label.as_ref().map(|label| label.get_matched().0);
        self.check_mir_block(mctx, block);
    }

    fn check_mir_block(&mut self, mctx: &MetaContext<'i>, block: &'i pairs::MirStmtBlock<'i>) {
        let (_, stmts, _) = block.get_matched();
        stmts.iter_matched().for_each(|stmt| self.check_mir_stmt(mctx, stmt));
    }

    fn check_mir_decl(&mut self, mctx: &MetaContext<'i>, mir_decl: &'i pairs::MirDecl<'i>) {
        match mir_decl.deref() {
            Choice2::_0(type_decl) => self.check_mir_type_decl(mctx, type_decl),
            Choice2::_1(local_decl) => self.check_mir_local_decl(mctx, local_decl),
        }
    }

    fn check_mir_type_decl(&mut self, mctx: &MetaContext<'i>, type_decl: &'i pairs::MirTypeDecl<'i>) {
        let (_, ident, _, ty, _) = type_decl.get_matched();
        self.fn_def.add_type_impl(mctx, ident.into(), ty.into(), self.errors);
    }

    fn check_mir_local_decl(&mut self, mctx: &MetaContext<'i>, local_decl: &'i pairs::MirLocalDecl<'i>) {
        //FIXME: check whether label names conflict
        let (label, _, _, local, _, ty, rvalue_or_call, _) = local_decl.get_matched();
        let label = label
            .as_ref()
            .map(|label| Symbol::intern(label.get_matched().0.get_matched().1.span.as_str()));
        self.fn_def.add_place_local(mctx, label, local, ty, self.errors);
        self.check_type(mctx, ty);
        if let Some(rvalue_or_call) = rvalue_or_call {
            self.check_mir_rvalue_or_call(mctx, rvalue_or_call.get_matched().1);
        }
    }

    fn check_mir_rvalue_or_call(&mut self, mctx: &MetaContext<'i>, rvalue_or_call: &'i pairs::MirRvalueOrCall<'i>) {
        match rvalue_or_call.deref() {
            Choice2::_0(call) => self.check_mir_call(mctx, call),
            Choice2::_1(rvalue) => self.check_mir_rvalue(mctx, rvalue),
        }
    }

    fn check_mir_call(&mut self, mctx: &MetaContext<'i>, call: &'i pairs::MirCall<'i>) {
        let (fn_operand, _, args, _) = call.get_matched();
        self.check_mir_fn_operand(mctx, fn_operand);
        if let Some(args) = args {
            let args = collect_elems_separated_by_comma!(args).collect::<Vec<_>>();
            for arg in args {
                self.check_mir_operand(mctx, arg);
            }
        }
    }

    fn check_mir_fn_operand(&mut self, mctx: &MetaContext<'i>, fn_operand: &'i pairs::MirFnOperand<'i>) {
        match fn_operand.deref() {
            Choice5::_0(copy_) => self.check_mir_place(mctx, copy_.get_matched().1.MirPlace()),
            Choice5::_1(move_) => self.check_mir_place(mctx, move_.get_matched().1.MirPlace()),
            Choice5::_2(ty_path) => self.check_type_path(mctx, ty_path),
            Choice5::_3(lang_item) => self.check_lang_item_with_args(mctx, lang_item),
            Choice5::_4(_ident) => self.check_mir_fn_pat(mctx),
        }
    }

    fn check_mir_fn_pat(&mut self, _mctx: &MetaContext<'i>) {
        // TODO: check if the function pattern is defined
    }

    fn check_mir_rvalue(&mut self, mctx: &MetaContext<'i>, rvalue: &'i pairs::MirRvalue<'i>) {
        match rvalue.deref() {
            Choice12::_0(_place_holder) => {},
            Choice12::_1(mir_rvalue_cast) => {
                let (operand, _, ty, _, _, _) = mir_rvalue_cast.get_matched();
                self.check_mir_operand(mctx, operand);
                self.check_type(mctx, ty);
            },
            Choice12::_2(mir_rvalue_use) => {
                let operand = match mir_rvalue_use.deref() {
                    Choice2::_0(op) => op.get_matched().1,
                    Choice2::_1(op) => op,
                };
                self.check_mir_operand(mctx, operand);
            },
            Choice12::_3(mir_rvalue_repeat) => self.check_mir_operand(mctx, mir_rvalue_repeat.MirOperand()),
            Choice12::_4(mir_rvalue_ref) => self.check_mir_place(mctx, mir_rvalue_ref.MirPlace()),
            Choice12::_5(mir_rvalue_raw_ptr) => self.check_mir_place(mctx, mir_rvalue_raw_ptr.MirPlace()),
            Choice12::_6(mir_rvalue_len) => self.check_mir_place(mctx, mir_rvalue_len.MirPlace()),
            Choice12::_7(mir_rvalue_bin_op) => {
                let (_, _, lhs, _, rhs, _) = mir_rvalue_bin_op.get_matched();
                self.check_mir_operand(mctx, lhs);
                self.check_mir_operand(mctx, rhs);
            },
            Choice12::_8(mir_rvalue_null_op) => self.check_type(mctx, mir_rvalue_null_op.Type()),
            Choice12::_9(mir_rvalue_un_op) => self.check_mir_operand(mctx, mir_rvalue_un_op.MirOperand()),
            Choice12::_10(mir_rvalue_discriminant) => self.check_mir_place(mctx, mir_rvalue_discriminant.MirPlace()),
            Choice12::_11(mir_rvalue_aggregate) => self.check_mir_rvalue_aggregate(mctx, mir_rvalue_aggregate),
        }
    }

    fn check_mir_operand(&mut self, mctx: &MetaContext<'i>, operand: &'i pairs::MirOperand<'i>) {
        match operand.deref() {
            Choice6::_0(_) | Choice6::_1(_) => {},
            Choice6::_2(meta_var) => {
                _ = self
                    .meta_vars
                    .get_non_local_meta_var(mctx, meta_var.into(), self.errors)
            },
            Choice6::_3(op_move) => self.check_mir_place(mctx, op_move.MirPlace()),
            Choice6::_4(op_copy) => self.check_mir_place(mctx, op_copy.MirPlace()),
            Choice6::_5(op_const) => self.check_mir_const_operand(mctx, op_const),
        }
    }

    fn check_mir_const_operand(&mut self, mctx: &MetaContext<'i>, konst: &'i pairs::MirOperandConst<'i>) {
        let (_, konst) = konst.get_matched();
        match konst {
            Choice4::_0(_lit) => {},
            Choice4::_1(lang_item) => self.check_lang_item_with_args(mctx, lang_item),
            Choice4::_2(path) => self.check_type_path(mctx, path),
            Choice4::_3(ident) => {
                let _: Option<_> = self.meta_vars.get_non_local_meta_var(mctx, ident.into(), self.errors);
            },
        }
    }

    pub fn check_mir_place_field(&mut self, mctx: &MetaContext<'i>, field: &pairs::MirPlaceField<'i>) {
        let (_, field) = field.get_matched();
        match field {
            Choice3::_0(_) | Choice3::_1(_) => (),
            Choice3::_2(index) => {
                let index_str = index.span.as_str().trim();
                if let Err(err) = index_str.parse::<u32>() {
                    self.errors.push(RPLMetaError::InvalidFieldIndex {
                        index: index_str,
                        source: err.to_string(),
                        span: SpanWrapper::new(index.span, mctx.get_active_path()),
                    });
                }
            },
        }
    }

    fn check_mir_place(&mut self, mctx: &MetaContext<'i>, place: &'i pairs::MirPlace<'i>) {
        let (base, suffix) = place.get_matched();
        match base.deref() {
            Choice3::_0(local) => self.check_mir_place_local(mctx, local),
            Choice3::_1(paren) => self.check_mir_place(mctx, paren.MirPlace()),
            Choice3::_2(deref) => self.check_mir_place(mctx, deref.MirPlace()),
        }
        suffix.iter_matched().for_each(|suffix| match suffix.deref() {
            Choice5::_0(field) => self.check_mir_place_field(mctx, field),
            Choice5::_1(index) => self.check_mir_place_local(mctx, index.MirPlaceLocal()),
            Choice5::_2(const_index) => {
                let (_, _, index, _, min_length, _) = const_index.get_matched();
                let a = index.span.as_str().parse::<i32>();
                let b = min_length.span.as_str().parse::<i32>();
                if let (Ok(a), Ok(b)) = (a, b) {
                    if a >= b {
                        self.errors.push(RPLMetaError::ConstantIndexOutOfBound {
                            index: SpanWrapper::new(index.span, mctx.get_active_path()),
                            min_length: SpanWrapper::new(min_length.span, mctx.get_active_path()),
                        });
                    }
                }
            },
            Choice5::_3(_subslice) => {},
            Choice5::_4(_downcast) => {},
        });
    }

    fn check_mir_place_local(&mut self, mctx: &MetaContext<'i>, local: &'i pairs::MirPlaceLocal<'i>) {
        self.fn_def.get_place_local(mctx, local, &self.meta_vars, self.errors);
    }

    fn check_mir_rvalue_aggregate(
        &mut self,
        mctx: &MetaContext<'i>,
        mir_rvalue_aggregate: &'i pairs::MirRvalueAggregate<'i>,
    ) {
        match mir_rvalue_aggregate.deref() {
            Choice6::_0(array) => {
                let (_, ops, _) = array.get_matched();
                if let Some(ops) = ops {
                    let ops = collect_elems_separated_by_comma!(ops).collect::<Vec<_>>();
                    for op in ops {
                        self.check_mir_operand(mctx, op);
                    }
                }
            },
            Choice6::_1(tuple) => {
                let (_, ops, _) = tuple.get_matched();
                if let Some(ops) = ops {
                    let ops = collect_elems_separated_by_comma!(ops).collect::<Vec<_>>();
                    for op in ops {
                        self.check_mir_operand(mctx, op);
                    }
                }
            },
            Choice6::_2(adt_struct) => {
                let (path_or_lang_item, _, ops, _) = adt_struct.get_matched();
                self.check_path_or_lang_item(mctx, path_or_lang_item);
                if let Some(ops) = ops {
                    let fields = collect_elems_separated_by_comma!(ops).collect::<Vec<_>>();
                    for field in fields {
                        self.check_mir_operand(mctx, field.MirOperand());
                    }
                }
            },
            Choice6::_3(adt_tuple) => {
                let (_, _, _, _, path, _, ops, _) = adt_tuple.get_matched();
                self.check_path(mctx, path);
                if let Some(ops) = ops {
                    let ops = collect_elems_separated_by_comma!(ops).collect::<Vec<_>>();
                    for op in ops {
                        self.check_mir_operand(mctx, op);
                    }
                }
            },
            Choice6::_4(adt_unit) => {
                let path_or_lang_item = adt_unit.PathOrLangItem();
                self.check_path_or_lang_item(mctx, path_or_lang_item);
            },
            Choice6::_5(raw_ptr) => {
                let (ty, _, _, ptr, _, meta_data, _) = raw_ptr.get_matched();
                self.check_type(mctx, ty.Type());
                self.check_mir_operand(mctx, ptr);
                self.check_mir_operand(mctx, meta_data);
            },
        }
    }

    fn check_path_or_lang_item(&mut self, mctx: &MetaContext<'i>, path_or_lang_item: &'i pairs::PathOrLangItem<'i>) {
        match path_or_lang_item.deref() {
            Choice2::_0(path) => self.check_path(mctx, path),
            Choice2::_1(lang_item) => self.check_lang_item_with_args(mctx, lang_item),
        }
    }

    fn check_type(&mut self, mctx: &MetaContext<'i>, ty: &'i pairs::Type<'i>) {
        match ty.deref() {
            Choice14::_0(ty_array) => self.check_type(mctx, ty_array.Type()),
            Choice14::_1(ty_group) => self.check_type(mctx, ty_group.Type()),
            Choice14::_2(_ty_never) => {},
            Choice14::_3(ty_paren) => self.check_type(mctx, ty_paren.Type()),
            Choice14::_4(ty_ptr) => self.check_type(mctx, ty_ptr.Type()),
            Choice14::_5(ty_ref) => {
                if let Some(region) = ty_ref.Region() {
                    self.check_region(mctx, region);
                }
                self.check_type(mctx, ty_ref.Type());
            },
            Choice14::_6(ty_slice) => self.check_type(mctx, ty_slice.Type()),
            Choice14::_7(ty_tuple) => {
                let (_, tys, _) = ty_tuple.get_matched();
                if let Some(tys) = tys {
                    let tys = collect_elems_separated_by_comma!(tys).collect::<Vec<_>>();
                    for ty in tys {
                        self.check_type(mctx, ty);
                    }
                }
            },
            Choice14::_8(ty_meta_var) => {
                let ident = Ident::from(ty_meta_var.MetaVariable());
                // debug!(?ident, ?self.meta_vars, "checking meta variable");
                let _: Option<_> = self.meta_vars.get_non_local_meta_var(mctx, ident, self.errors);
            },
            Choice14::_9(_ty_self) => {},
            Choice14::_10(_primitive_types) => {},
            Choice14::_11(_place_holder) => {},
            Choice14::_12(ty_path) => self.check_type_path(mctx, ty_path),
            Choice14::_13(lang_item) => {
                self.check_lang_item_with_args(mctx, lang_item);
            },
        }
    }

    fn check_type_path(&mut self, mctx: &MetaContext<'i>, ty_path: &'i pairs::TypePath<'i>) {
        let (qself, path) = ty_path.get_matched();
        if let Some(qself) = qself {
            self.check_type(mctx, qself.Type());
        }
        self.check_path(mctx, path);
    }

    fn check_path(&mut self, mctx: &MetaContext<'i>, path: &'i pairs::Path<'i>) {
        let path: Path<'i> = path.into();
        if let Some(ident) = path.as_ident() {
            if !ident_is_primitive(&ident) {
                WithMetaTable::from((&*self.fn_def, self.meta_vars.clone()))
                    .get_type_or_path(&WithPath::with_ctx(mctx, ident))
                    .or_record(self.errors);
            }
        } else {
            for segment in path.segments {
                self.check_generic_args(mctx, &segment.1);
            }
        }
    }

    fn check_generic_args(&mut self, mctx: &MetaContext<'i>, args: &[&'i pairs::GenericArgument<'i>]) {
        for arg in args {
            self.check_generic_arg(mctx, arg);
        }
    }

    fn check_generic_arg(&mut self, mctx: &MetaContext<'i>, arg: &'i pairs::GenericArgument<'i>) {
        match arg.deref() {
            Choice3::_0(region) => self.check_region(mctx, region),
            Choice3::_1(ty) => self.check_type(mctx, ty),
            Choice3::_2(konst) => {
                let konst = match konst.deref() {
                    Choice2::_0(konst) => konst.get_matched().1,
                    Choice2::_1(konst) => konst,
                };
                self.check_const(mctx, konst);
            },
        }
    }

    fn check_region(&mut self, _mctx: &MetaContext<'i>, _region: &'i pairs::Region<'i>) {}

    fn check_const(&mut self, mctx: &MetaContext<'i>, konst: &'i pairs::Konst<'i>) {
        match konst.deref() {
            Choice2::_0(_lit) => {},
            Choice2::_1(ty_path) => {
                self.check_type_path(mctx, ty_path);
            },
        }
    }

    fn check_lang_item_with_args(&mut self, mctx: &MetaContext<'i>, lang_item: &'i pairs::LangItemWithArgs<'i>) {
        let item_span = lang_item.String().span;
        let args = lang_item.AngleBracketedGenericArguments();

        // FIXME: check if the lang item is defined
        // remove the `""` around the item name
        let item_name = item_span.as_str();
        let item_name = item_name.trim_matches('"');
        _ = rustc_hir::LangItem::from_name(rustc_span::Symbol::intern(item_name)).ok_or_else(|| {
            self.errors.push(RPLMetaError::UnknownLangItem {
                value: Symbol::intern(item_name),
                span: SpanWrapper::new(item_span, mctx.get_active_path()),
            });
        });

        if let Some(args) = args {
            let (_, _, args, _) = args.get_matched();
            let args = collect_elems_separated_by_comma!(args).collect::<Vec<_>>();
            for arg in args {
                self.check_generic_arg(mctx, arg);
            }
        }
    }
}

struct CheckVariantCtxt<'i, 'r> {
    _meta_vars: Arc<NonLocalMetaSymTab<'i>>,
    variant_def: &'r mut Variant<'i>,
    errors: &'r mut Vec<RPLMetaError<'i>>,
}

impl<'i> CheckVariantCtxt<'i, '_> {
    fn check_struct(mut self, mctx: &MetaContext<'i>, struct_: &'i pairs::Struct<'i>) {
        let (_, _, _, _, fields, _) = struct_.get_matched();
        if let Some(fields) = fields {
            let fields = collect_elems_separated_by_comma!(fields).collect::<Vec<_>>();
            self.check_fields(mctx, fields.into_iter());
        }
    }

    fn check_fields(&mut self, mctx: &MetaContext<'i>, fields: impl Iterator<Item = &'i pairs::Field<'i>>) {
        for field in fields {
            self.check_field(mctx, field);
        }
    }

    fn check_field(&mut self, mctx: &MetaContext<'i>, field: &'i pairs::Field<'i>) {
        let (ident, _, ty) = field.get_matched();
        let ident = ident.into();
        self.variant_def.add_field(mctx, ident, ty, self.errors);
        self.check_ident(mctx, ident);
        self.check_type(mctx, ty);
    }

    fn check_ident(&mut self, _mctx: &MetaContext<'i>, _ident: Ident<'i>) {}

    fn check_type(&mut self, _mctx: &MetaContext<'i>, _ty: &'i pairs::Type<'i>) {}
}

struct CheckEnumCtxt<'i, 'r> {
    meta_vars: Arc<NonLocalMetaSymTab<'i>>,
    enum_def: &'r mut EnumInner<'i>,
    errors: &'r mut Vec<RPLMetaError<'i>>,
}

impl<'i> CheckEnumCtxt<'i, '_> {
    fn check_enum(&mut self, mctx: &MetaContext<'i>, enum_: &'i pairs::Enum<'i>) {
        let (_, _, _, enum_variants, _) = enum_.get_matched();
        if let Some(enum_variants) = enum_variants {
            let enum_variants = collect_elems_separated_by_comma!(enum_variants).collect::<Vec<_>>();
            for variant in enum_variants {
                let (ident, fields) = match variant.deref() {
                    Choice3::_0(variant) => {
                        let (ident, _, fields, _) = variant.get_matched();
                        (
                            Ident::from(ident),
                            fields
                                .as_ref()
                                .map(|fields| collect_elems_separated_by_comma!(fields).collect::<Vec<_>>()),
                        )
                    },
                    Choice3::_1(variant) => {
                        let (ident, _, _ty, _) = variant.get_matched();
                        (Ident::from(ident), None)
                    },
                    Choice3::_2(ident) => (Ident::from(ident), None),
                };
                let variant_def = self.enum_def.add_variant(mctx, ident, self.errors);
                if let Some(variant_def) = variant_def
                    && fields.is_some()
                {
                    CheckVariantCtxt {
                        _meta_vars: self.meta_vars.clone(),
                        variant_def,
                        errors: self.errors,
                    }
                    .check_fields(mctx, fields.into_iter().flatten());
                }
            }
        }
    }
}
