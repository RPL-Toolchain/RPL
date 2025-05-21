use rustc_middle::mir;
use rustc_span::symbol::kw;

use super::*;
use std::fmt;

impl fmt::Debug for ItemPath<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            [] => f.write_str(" "),
            [first, rest @ ..] => {
                fmt::Display::fmt(first, f)?;
                for path in rest {
                    write!(f, "::{path}")?;
                }
                Ok(())
            },
        }
    }
}

impl fmt::Display for ItemPath<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl fmt::Debug for Path<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Item(path) => path.fmt(f),
            Self::TypeRelative(ty, path) => write!(f, "< {ty:?} >::{path}"),
            Self::LangItem(lang_item) => write!(f, "#[lang = \"{}\"]", lang_item.name()),
        }
    }
}

impl PathWithArgs<'_> {
    fn fmt_as_ty(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let PathWithArgs { path, args } = self;
        write!(f, "{path:?}{args:?}")
    }
}

impl fmt::Debug for PathWithArgs<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let PathWithArgs { path, args } = self;
        path.fmt(f)?;
        if !args.is_empty() {
            write!(f, ":: {args:?}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for Ty<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind().fmt(f)
    }
}

impl fmt::Debug for TyKind<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::TyVar(ty_var) => ty_var.fmt(f),
            Self::Array(ty, len) => write!(f, "[{ty:?}; {len:?}]"),
            Self::Slice(ty) => write!(f, "[{ty:?}]"),
            Self::Tuple(tys) => {
                f.write_str("(")?;
                for ty in tys {
                    write!(f, "{ty:?}, ")?;
                }
                f.write_str(")")
            },
            Self::Ref(region, ty, mir::Mutability::Not) => write!(f, "&{region} {ty:?}"),
            Self::Ref(region, ty, mir::Mutability::Mut) => write!(f, "&{region}mut {ty:?}"),
            Self::RawPtr(ty, mutability) => write!(f, "*{} {ty:?}", mutability.ptr_str()),
            Self::Path(path_with_args) => path_with_args.fmt_as_ty(f),
            Self::Def(def_id, args) => write!(f, "{def_id:?}{args:?}"),
            Self::Uint(uint) => uint.fmt(f),
            Self::Int(int) => int.fmt(f),
            Self::Float(float) => float.fmt(f),
            Self::Bool => f.write_str("bool"),
            Self::Str => f.write_str("str"),
            Self::Char => f.write_str("char"),
            Self::AdtPat(adt_var) => write!(f, "${adt_var}"),
            Self::Any => f.write_str("_"),
        }
    }
}

impl fmt::Debug for GenericArgsRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            [] => Ok(()),
            [arg] => write!(f, "< {arg:?} >"),
            [first, args @ ..] => {
                if self.0.is_empty() {
                    return Ok(());
                }
                write!(f, "< {first:?}")?;
                for arg in args {
                    f.write_str(",")?;
                    arg.fmt(f)?;
                }
                f.write_str(" >")
            },
        }
    }
}

impl fmt::Debug for GenericArgKind<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lifetime(region) => region.fmt(f),
            Self::Type(ty) => ty.fmt(f),
            Self::Const(konst) => konst.fmt(f),
        }
    }
}

impl fmt::Debug for RegionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReStatic => f.write_str("'static"),
            Self::ReAny => f.write_str("'_"),
        }
    }
}

impl fmt::Display for RegionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReStatic => f.write_str("'static"),
            Self::ReAny => Ok(()),
        }
    }
}

impl fmt::Debug for IntValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.ty {
            IntTy::Bool => write!(f, "{}", self.value != 0),
            IntTy::Int(ty) => write!(f, "{}_{}", self.value, ty.name_str()),
            IntTy::NegInt(ty) => write!(f, "-{}_{}", self.value, ty.name_str()),
            IntTy::Uint(ty) => write!(f, "{}_{}", self.value, ty.name_str()),
        }
    }
}

impl fmt::Debug for TyVar<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.idx.fmt(f)
    }
}

impl fmt::Display for TyVar<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

impl fmt::Debug for ConstVar<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?}: {:?})", self.idx, self.ty)
    }
}

impl fmt::Display for ConstVar<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.idx, f)
    }
}

impl fmt::Display for Fn<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self.name {
            kw::Underscore => "_".to_string(),
            name => format!("${name}"),
        };
        write!(
            f,
            "fn {name}{params:?} -> {ret:?}",
            params = self.params,
            ret = self.ret,
        )
    }
}

impl fmt::Debug for Fn<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.body {
            None => write!(f, "{self};"),
            Some(body) => write!(f, "{self}{body:?}"),
        }
    }
}

impl fmt::Debug for Params<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_tuple = f.debug_tuple("");
        for param in self.iter() {
            debug_tuple.field(param);
        }
        if self.non_exhaustive {
            debug_tuple.finish_non_exhaustive()
        } else {
            debug_tuple.finish()
        }
    }
}

impl fmt::Debug for Param<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.ident {
            kw::Empty => {},
            _ => write!(f, "{}{:?}: ", self.mutability.prefix_str(), self.ident)?,
        }
        write!(f, "{:?}", self.ty)
    }
}

impl fmt::Debug for FnBody<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mir(mir_body) => {
                f.write_str(" = mir!")?;
                f.debug_set().entry(mir_body).finish()
            },
        }
    }
}

impl fmt::Display for Adt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            AdtKind::Struct(variant) => write!(f, "struct {{ {variant}}}"),
            AdtKind::Enum(variants) => {
                f.write_str("enum {")?;
                for (name, variant) in variants {
                    write!(f, "${name} {{ {variant}}}, ")?;
                }
                f.write_str("}")
            },
        }
    }
}

impl fmt::Display for Variant<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (name, field) in &self.fields {
            write!(f, "${name}: {ty:?}, ", ty = field.ty)?;
        }
        Ok(())
    }
}
