use std::fmt;

use super::*;

impl fmt::Debug for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}[{:?}]", self.block, self.statement_index)
    }
}

impl fmt::Debug for Place<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.projection {
            [] => self.base.fmt(f),
            [projection @ .., last] => fmt_projection(
                f,
                Place {
                    base: self.base,
                    projection,
                },
                last,
            ),
        }
    }
}

fn fmt_projection<'pcx>(f: &mut fmt::Formatter<'_>, place: Place<'pcx>, proj: &PlaceElem<'pcx>) -> fmt::Result {
    struct FromEnd(bool);
    impl fmt::Display for FromEnd {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if self.0 { f.write_str("-") } else { Ok(()) }
        }
    }
    match proj {
        PlaceElem::Deref => write!(f, "(*{place:?})"),
        PlaceElem::Field(field) => write!(f, "({place:?}.{field:?})"),
        PlaceElem::FieldPat(field) => write!(f, "({place:?}.${field})"),
        PlaceElem::Index(local) => write!(f, "({place:?}[{local:?}])"),
        &PlaceElem::ConstantIndex {
            offset,
            min_length,
            from_end,
        } => {
            let from_end = FromEnd(from_end);
            write!(f, "({place:?}[{from_end}{offset} of {min_length}])")
        },
        &PlaceElem::Subslice { from, to, from_end } => {
            let from_end = FromEnd(from_end);
            write!(f, "({place:?}[{from}:{from_end}{to}])")
        },
        PlaceElem::Downcast(variant) => write!(f, "({place:?} as {variant})"),
        PlaceElem::DowncastPat(variant) => write!(f, "({place:?} as ${variant})"),
        PlaceElem::OpaqueCast(ty) | PlaceElem::Subtype(ty) => write!(f, "({place:?} as {ty:?})"),
    }
}

impl fmt::Debug for CopyNonOverlapping<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { src, dst, count } = self;
        write!(f, "copy_non_overlapping({src:?}, {dst:?}, {count:?})")
    }
}

impl fmt::Debug for NonDivergingIntrinsic<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CopyNonOverlapping(copy) => {
                write!(f, "{copy:?}")
            },
        }
    }
}

impl fmt::Debug for StatementKind<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Assign(place, rvalue) => write!(f, "{place:?} = {rvalue:?}"),
            Self::Intrinsic(intrinsic) => write!(f, "{intrinsic:?}"),
            Self::Move(place) => write!(f, "take {place:?}"),
        }
    }
}

fn fmt_list<T>(
    f: &mut fmt::Formatter<'_>,
    list: impl IntoIterator<Item = T>,
    end: &str,
    mut fmt: impl FnMut(T, &mut fmt::Formatter<'_>) -> fmt::Result,
) -> fmt::Result {
    let mut iter = list.into_iter();
    if let Some(first) = iter.next() {
        fmt(first, f)?;
    }
    for elem in iter {
        write!(f, ", ")?;
        fmt(elem, f)?;
    }
    f.write_str(end)
}

impl fmt::Debug for TerminatorKind<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TerminatorKind::Call {
                func,
                args,
                destination,
                target,
            } => {
                if let Some(destination) = destination {
                    write!(f, "{destination:?} = ")?
                }
                func.fmt_fn_operand(f)?;
                write!(f, "(")?;
                fmt_list(f, args, ")", fmt::Debug::fmt)?;
                write!(f, " -> {target:?}")
            },
            TerminatorKind::Drop { place, target } => {
                write!(f, "drop({place:?}) -> {target:?}")
            },
            TerminatorKind::Unreachable => {
                write!(f, "unreachable")
            },
            TerminatorKind::SwitchInt { operand, targets } => write!(f, "switchInt({operand:?}) -> {targets:?}"),
            TerminatorKind::Goto(basic_block) => write!(f, "goto {basic_block:?}"),
            TerminatorKind::Return => f.write_str("return"),
            TerminatorKind::PatEnd => f.write_str("end"),
        }
    }
}

impl fmt::Debug for SwitchTargets {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut targets = f.debug_list();
        for (val, bb) in &self.targets {
            targets.entry_with(|f| write!(f, "{val:?}: {bb:?}"));
        }
        if let Some(bb) = self.otherwise {
            targets.entry_with(|f| write!(f, "otherwise: {bb:?}"));
        }
        targets.finish()
    }
}

impl fmt::Debug for Rvalue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Any => f.write_str("_"),
            Self::Use(operand) => operand.fmt(f),
            Self::Repeat(elem, len) => write!(f, "[{elem:?}; {len:?}]"),
            Self::Ref(region, bor, place) => write!(f, "&{region}{}{place:?}", bor.mutability().prefix_str()),
            Self::RawPtr(mutability, place) => write!(f, "&raw {} {place:?}", mutability.ptr_str()),
            Self::Len(place) => f.debug_tuple("Len").field(place).finish(),
            Self::Cast(cast_kind, operand, ty) => write!(f, "{operand:?} as {ty:?} ({cast_kind:?})"),
            Self::BinaryOp(op, box [lhs, rhs]) => write!(f, "{op:?}({lhs:?}, {rhs:?})"),
            Self::NullaryOp(op, ty) => write!(f, "{op:?}({ty:?})"),
            Self::UnaryOp(op, operand) => write!(f, "{op:?}({operand:?})"),
            Self::Discriminant(place) => f.debug_tuple("discriminant").field(place).finish(),
            Self::Aggregate(agg_kind, operands) => format_aggregate(agg_kind, operands, f),
            Self::ShallowInitBox(operand, ty) => write!(f, "Box< {ty:?} >({operand:?})"),
            Self::CopyForDeref(place) => write!(f, "&(*{place:?})"),
        }
    }
}

fn format_aggregate<'pcx>(
    agg_kind: &AggKind<'pcx>,
    operands: &[Operand<'pcx>],
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match agg_kind {
        AggKind::Array => {
            f.write_str("[")?;
            fmt_list(f, operands, "]", fmt::Debug::fmt)
        },
        AggKind::Tuple => {
            f.write_str("(")?;
            fmt_list(f, operands, ")", fmt::Debug::fmt)
        },
        AggKind::Adt(path_with_args, AggAdtKind::Unit) => {
            write!(f, "{path_with_args:?}")
        },
        AggKind::Adt(path_with_args, AggAdtKind::Tuple) => {
            write!(f, "{path_with_args:?}")?;
            f.write_str("(")?;
            fmt_list(f, operands, ")", fmt::Debug::fmt)
        },
        AggKind::Adt(path_with_args, AggAdtKind::Struct(fields)) => {
            write!(f, "{path_with_args:?} ")?;
            if fields.is_empty() {
                return f.write_str("{}");
            }
            f.write_str("{ ")?;
            let mut fields = fields.iter();
            fmt_list(f, operands, " }", |operand, f| {
                let field = fields.next().ok_or(std::fmt::Error)?;
                write!(f, "{field}: {operand:?}")
            })
        },
        AggKind::RawPtr(ty, mutability) => {
            write!(f, "*{} {ty:?} from (", mutability.ptr_str())?;
            fmt_list(f, operands, ")", fmt::Debug::fmt)
        },
    }
}

impl Operand<'_> {
    fn fmt_fn_operand(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Any => f.write_str("_"),
            Self::AnyArgs => f.write_str(".."),
            Self::Copy(place) => write!(f, "(copy {place:?})"),
            Self::Move(place) => write!(f, "(move {place:?})"),
            Self::Constant(konst) => write!(f, "{konst:?}"),
            Self::FnPat(fn_pat) => write!(f, "${fn_pat}"),
        }
    }
}

impl fmt::Debug for Operand<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Any => f.write_str("_"),
            Self::AnyArgs => f.write_str(".."),
            Self::Copy(place) => write!(f, "copy {place:?}"),
            Self::Move(place) => write!(f, "move {place:?}"),
            Self::Constant(konst) => write!(f, "const {konst:?}"),
            Self::FnPat(fn_pat) => write!(f, "const ${fn_pat}"),
        }
    }
}

impl fmt::Debug for ConstOperand<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConstVar(const_var) => const_var.fmt(f),
            Self::ScalarInt(scalar) => write!(f, "{scalar:?}"),
            Self::ZeroSized(path_with_args) => path_with_args.fmt(f),
        }
    }
}

impl fmt::Debug for FieldAcc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Named(sym) => write!(f, "{sym}"),
            Self::Unnamed(field) => field.fmt(f),
        }
    }
}

impl fmt::Debug for FnPatternBody<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let new_line = if f.alternate() { "\n" } else { " " };
        let indent = if f.alternate() { "    " } else { "" };
        for (local, ty) in self.locals.iter_enumerated() {
            write!(f, "let {local:?}: {ty:?} ;{new_line}")?;
        }
        for (bb, block) in self.basic_blocks.iter_enumerated() {
            write!(f, "{bb:?}: {{{new_line}")?;
            for statement in &block.statements {
                write!(f, "{indent}{statement:?};{new_line}")?;
            }
            if let Some(terminator) = &block.terminator {
                write!(f, "{indent}{terminator:?};{new_line}")?;
            }
            write!(f, "}}{new_line}")?;
        }
        Ok(())
    }
}
