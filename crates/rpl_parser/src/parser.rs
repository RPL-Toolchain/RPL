#![allow(warnings)]
/// Underlying definition of the RPL parser written with Pest.
pub struct Grammar;
#[doc = ""]
#[allow(dead_code, non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Rule {
    EOI,
    r#kw_pattern,
    r#kw_patt,
    r#kw_util,
    r#kw_cstr,
    r#kw_diag,
    r#kw_meta,
    r#kw_import,
    r#kw_self,
    r#kw_Self,
    r#kw_fn,
    r#kw_mut,
    r#kw_const,
    r#kw_static,
    r#kw_lang,
    r#kw_as,
    r#kw_crate,
    r#kw_use,
    r#kw_type,
    r#kw_let,
    r#kw_move,
    r#kw_copy,
    r#kw_Len,
    r#kw_PtrToPtr,
    r#kw_IntToInt,
    r#kw_Transmute,
    r#kw_PointerCoercion,
    r#kw_Unsize,
    r#kw_Implicit,
    r#kw_Add,
    r#kw_Sub,
    r#kw_Mul,
    r#kw_Div,
    r#kw_Rem,
    r#kw_Lt,
    r#kw_Le,
    r#kw_Gt,
    r#kw_Ge,
    r#kw_Eq,
    r#kw_Ne,
    r#kw_Offset,
    r#kw_SizeOf,
    r#kw_AlignOf,
    r#kw_Neg,
    r#kw_Not,
    r#kw_PtrMetadata,
    r#kw_discriminant,
    r#kw_Ctor,
    r#kw_from,
    r#kw_of,
    r#kw_raw,
    r#kw_drop,
    r#kw_break,
    r#kw_continue,
    r#kw_loop,
    r#kw_switchInt,
    r#kw_true,
    r#kw_false,
    r#kw_unsafe,
    r#kw_pub,
    r#kw_struct,
    r#kw_enum,
    r#kw_impl,
    r#kw_for,
    r#kw_u8,
    r#kw_u16,
    r#kw_u32,
    r#kw_u64,
    r#kw_usize,
    r#kw_i8,
    r#kw_i16,
    r#kw_i32,
    r#kw_i64,
    r#kw_isize,
    r#kw_RET,
    r#kw_bool,
    r#kw_str,
    r#kw_place,
    r#kw_where,
    r#Keywords,
    r#COMMENT,
    r#WHITESPACE,
    r#LeftBrace,
    r#RightBrace,
    r#LeftBracket,
    r#RightBracket,
    r#LeftParen,
    r#RightParen,
    r#LessThan,
    r#GreaterThan,
    r#Dollar,
    r#Assign,
    r#Comma,
    r#Dot,
    r#Dot2,
    r#Colon,
    r#Colon2,
    r#SemiColon,
    r#Hash,
    r#And,
    r#AndAnd,
    r#OrOr,
    r#Bang,
    r#Star,
    r#Arrow,
    r#RightArrow,
    r#Quote,
    r#Minus,
    r#PlaceHolder,
    r#Literal,
    r#BIN_DIGIT,
    r#OCT_DIGIT,
    r#DEC_DIGIT,
    r#HEX_DIGIT,
    r#DEC_LITERAL,
    r#BIN_LITERAL,
    r#OCT_LITERAL,
    r#HEX_LITERAL,
    r#IntegerSuffix,
    r#PrimitiveType,
    r#Integer,
    r#String,
    r#Bool,
    r#WordLeading,
    r#WordFollowing,
    r#Word,
    r#Identifier,
    r#LabelName,
    r#MetaVariable,
    r#MetaVariableType,
    r#Predicate,
    r#PredicateTerm,
    r#PredicateArgs,
    r#PredicateArg,
    r#PredicateClause,
    r#PredicateConjunction,
    r#MetaVariableDecl,
    r#MetaVariableDeclsSeparatedByComma,
    r#MetaVariableDeclList,
    r#MetaVariableAssign,
    r#MetaVariableAssignsSeparatedByComma,
    r#MetaVariableAssignList,
    r#Attribute,
    r#AttributesSeparatedByComma,
    r#WhereBlock,
    r#Mutability,
    r#PtrMutability,
    r#Region,
    r#QSelf,
    r#Path,
    r#PathArguments,
    r#Pathcrate,
    r#PathLeading,
    r#PathSegment,
    r#TypePath,
    r#Konst,
    r#GenericConst,
    r#GenericArgument,
    r#GenericArgumentsSepretatedByComma,
    r#AngleBracketedGenericArguments,
    r#LangItemWithArgs,
    r#TypesSeparatedByComma,
    r#TypeArray,
    r#TypeGroup,
    r#TypeNever,
    r#TypeParen,
    r#TypePtr,
    r#TypeReference,
    r#TypeSlice,
    r#TypeTuple,
    r#TypeMetaVariable,
    r#Type,
    r#SelfParam,
    r#PlaceHolderWithType,
    r#FnParam,
    r#NormalParam,
    r#Dollarself,
    r#DollarRET,
    r#MirPlaceLocal,
    r#MirPlaceParen,
    r#MirPlaceDeref,
    r#MirPlaceField,
    r#MirPlaceIndex,
    r#MirPlaceConstIndex,
    r#MirPlaceSubslice,
    r#MirPlaceDowncast,
    r#MirBasicPlace,
    r#MirPlaceSuffix,
    r#MirPlace,
    r#MirOperandMove,
    r#MirOperandCopy,
    r#MirOperandConst,
    r#MirOperand,
    r#MirRvalueUse,
    r#MirRvalueRepeat,
    r#MirRvalueRef,
    r#MirRvalueRawPtr,
    r#MirRvalueLen,
    r#PointerCoercion,
    r#CoercionSource,
    r#MirCastKind,
    r#MirRvalueCast,
    r#MirBinOp,
    r#MirRvalueBinOp,
    r#MirNullOp,
    r#MirRvalueNullOp,
    r#MirUnOp,
    r#MirRvalueUnOp,
    r#MirRvalueDiscriminant,
    r#MirOperandsSeparatedByComma,
    r#MirAggregateArray,
    r#MirAggregateTuple,
    r#PathOrLangItem,
    r#MirAggregateAdtStructField,
    r#MirAggregateAdtStructFieldsSeparatedByComma,
    r#MirAggregateAdtStruct,
    r#MirAggregateAdtTuple,
    r#MirAggregateAdtUnit,
    r#MirAggregateRawPtr,
    r#MirRvalueAggregate,
    r#MirRvalue,
    r#MirFnOperand,
    r#MirCall,
    r#MirRvalueOrCall,
    r#MirTypeDecl,
    r#MirLocalDecl,
    r#UsePath,
    r#MirDecl,
    r#MirCallIgnoreRet,
    r#MirDrop,
    r#Label,
    r#LabelWithColon,
    r#MirControl,
    r#MirStmtBlock,
    r#MirLoop,
    r#MirSwitchValue,
    r#MirSwitchBody,
    r#MirSwitchTarget,
    r#MirSwitchInt,
    r#MirAssign,
    r#MirStmt,
    r#MirBody,
    r#FnName,
    r#FnParamsSeparatedByComma,
    r#FnSig,
    r#FnBody,
    r#FnRet,
    r#Fn,
    r#Field,
    r#FieldsSeparatedByComma,
    r#Struct,
    r#EnumVariant,
    r#EnumVariantsSeparatedByComma,
    r#Enum,
    r#ImplKind,
    r#Impl,
    r#RustItem,
    r#RustItems,
    r#PatternConfiguration,
    r#PatternExpression,
    r#PatternOperation,
    r#RustItemsOrPatternOperation,
    r#RPLPatternItem,
    r#MetaVariableWithDiagMessage,
    r#MetaVariableWithDiagMessageSeparatedByComma,
    r#diagMessageArg,
    r#diagMessageText,
    r#diagMessageInner,
    r#diagMessage,
    r#diagAttr,
    r#diagAttrs,
    r#diagItem,
    r#diagItems,
    r#diagBlockItem,
    r#pattBlock,
    r#utilBlock,
    r#diagBlock,
    r#Block,
    r#RPLHeader,
    r#RPLPattern,
    r#main,
}
#[doc = "Unicode rules."]
pub mod unicode {}
mod constant_wrappers {
    #[doc = "A wrapper for `\"pattern\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_0;
    impl ::pest_typed::StringWrapper for r#w_0 {
        const CONTENT: &'static ::core::primitive::str = "pattern";
    }
    #[doc = "A wrapper for `\"patt\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_1;
    impl ::pest_typed::StringWrapper for r#w_1 {
        const CONTENT: &'static ::core::primitive::str = "patt";
    }
    #[doc = "A wrapper for `\"util\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_2;
    impl ::pest_typed::StringWrapper for r#w_2 {
        const CONTENT: &'static ::core::primitive::str = "util";
    }
    #[doc = "A wrapper for `\"cstr\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_3;
    impl ::pest_typed::StringWrapper for r#w_3 {
        const CONTENT: &'static ::core::primitive::str = "cstr";
    }
    #[doc = "A wrapper for `\"diag\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_4;
    impl ::pest_typed::StringWrapper for r#w_4 {
        const CONTENT: &'static ::core::primitive::str = "diag";
    }
    #[doc = "A wrapper for `\"meta\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_5;
    impl ::pest_typed::StringWrapper for r#w_5 {
        const CONTENT: &'static ::core::primitive::str = "meta";
    }
    #[doc = "A wrapper for `\"import\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_6;
    impl ::pest_typed::StringWrapper for r#w_6 {
        const CONTENT: &'static ::core::primitive::str = "import";
    }
    #[doc = "A wrapper for `\"self\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_7;
    impl ::pest_typed::StringWrapper for r#w_7 {
        const CONTENT: &'static ::core::primitive::str = "self";
    }
    #[doc = "A wrapper for `\"Self\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_8;
    impl ::pest_typed::StringWrapper for r#w_8 {
        const CONTENT: &'static ::core::primitive::str = "Self";
    }
    #[doc = "A wrapper for `\"fn\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_9;
    impl ::pest_typed::StringWrapper for r#w_9 {
        const CONTENT: &'static ::core::primitive::str = "fn";
    }
    #[doc = "A wrapper for `\"mut\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_10;
    impl ::pest_typed::StringWrapper for r#w_10 {
        const CONTENT: &'static ::core::primitive::str = "mut";
    }
    #[doc = "A wrapper for `\"const\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_11;
    impl ::pest_typed::StringWrapper for r#w_11 {
        const CONTENT: &'static ::core::primitive::str = "const";
    }
    #[doc = "A wrapper for `\"static\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_12;
    impl ::pest_typed::StringWrapper for r#w_12 {
        const CONTENT: &'static ::core::primitive::str = "static";
    }
    #[doc = "A wrapper for `\"lang\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_13;
    impl ::pest_typed::StringWrapper for r#w_13 {
        const CONTENT: &'static ::core::primitive::str = "lang";
    }
    #[doc = "A wrapper for `\"as\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_14;
    impl ::pest_typed::StringWrapper for r#w_14 {
        const CONTENT: &'static ::core::primitive::str = "as";
    }
    #[doc = "A wrapper for `\"crate\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_15;
    impl ::pest_typed::StringWrapper for r#w_15 {
        const CONTENT: &'static ::core::primitive::str = "crate";
    }
    #[doc = "A wrapper for `\"use\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_16;
    impl ::pest_typed::StringWrapper for r#w_16 {
        const CONTENT: &'static ::core::primitive::str = "use";
    }
    #[doc = "A wrapper for `\"type\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_17;
    impl ::pest_typed::StringWrapper for r#w_17 {
        const CONTENT: &'static ::core::primitive::str = "type";
    }
    #[doc = "A wrapper for `\"let\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_18;
    impl ::pest_typed::StringWrapper for r#w_18 {
        const CONTENT: &'static ::core::primitive::str = "let";
    }
    #[doc = "A wrapper for `\"move\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_19;
    impl ::pest_typed::StringWrapper for r#w_19 {
        const CONTENT: &'static ::core::primitive::str = "move";
    }
    #[doc = "A wrapper for `\"copy\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_20;
    impl ::pest_typed::StringWrapper for r#w_20 {
        const CONTENT: &'static ::core::primitive::str = "copy";
    }
    #[doc = "A wrapper for `\"Len\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_21;
    impl ::pest_typed::StringWrapper for r#w_21 {
        const CONTENT: &'static ::core::primitive::str = "Len";
    }
    #[doc = "A wrapper for `\"PtrToPtr\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_22;
    impl ::pest_typed::StringWrapper for r#w_22 {
        const CONTENT: &'static ::core::primitive::str = "PtrToPtr";
    }
    #[doc = "A wrapper for `\"IntToInt\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_23;
    impl ::pest_typed::StringWrapper for r#w_23 {
        const CONTENT: &'static ::core::primitive::str = "IntToInt";
    }
    #[doc = "A wrapper for `\"Transmute\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_24;
    impl ::pest_typed::StringWrapper for r#w_24 {
        const CONTENT: &'static ::core::primitive::str = "Transmute";
    }
    #[doc = "A wrapper for `\"PointerCoercion\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_25;
    impl ::pest_typed::StringWrapper for r#w_25 {
        const CONTENT: &'static ::core::primitive::str = "PointerCoercion";
    }
    #[doc = "A wrapper for `\"Unsize\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_26;
    impl ::pest_typed::StringWrapper for r#w_26 {
        const CONTENT: &'static ::core::primitive::str = "Unsize";
    }
    #[doc = "A wrapper for `\"Implicit\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_27;
    impl ::pest_typed::StringWrapper for r#w_27 {
        const CONTENT: &'static ::core::primitive::str = "Implicit";
    }
    #[doc = "A wrapper for `\"Add\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_28;
    impl ::pest_typed::StringWrapper for r#w_28 {
        const CONTENT: &'static ::core::primitive::str = "Add";
    }
    #[doc = "A wrapper for `\"Sub\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_29;
    impl ::pest_typed::StringWrapper for r#w_29 {
        const CONTENT: &'static ::core::primitive::str = "Sub";
    }
    #[doc = "A wrapper for `\"Mul\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_30;
    impl ::pest_typed::StringWrapper for r#w_30 {
        const CONTENT: &'static ::core::primitive::str = "Mul";
    }
    #[doc = "A wrapper for `\"Div\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_31;
    impl ::pest_typed::StringWrapper for r#w_31 {
        const CONTENT: &'static ::core::primitive::str = "Div";
    }
    #[doc = "A wrapper for `\"Rem\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_32;
    impl ::pest_typed::StringWrapper for r#w_32 {
        const CONTENT: &'static ::core::primitive::str = "Rem";
    }
    #[doc = "A wrapper for `\"Lt\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_33;
    impl ::pest_typed::StringWrapper for r#w_33 {
        const CONTENT: &'static ::core::primitive::str = "Lt";
    }
    #[doc = "A wrapper for `\"Le\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_34;
    impl ::pest_typed::StringWrapper for r#w_34 {
        const CONTENT: &'static ::core::primitive::str = "Le";
    }
    #[doc = "A wrapper for `\"Gt\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_35;
    impl ::pest_typed::StringWrapper for r#w_35 {
        const CONTENT: &'static ::core::primitive::str = "Gt";
    }
    #[doc = "A wrapper for `\"Ge\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_36;
    impl ::pest_typed::StringWrapper for r#w_36 {
        const CONTENT: &'static ::core::primitive::str = "Ge";
    }
    #[doc = "A wrapper for `\"Eq\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_37;
    impl ::pest_typed::StringWrapper for r#w_37 {
        const CONTENT: &'static ::core::primitive::str = "Eq";
    }
    #[doc = "A wrapper for `\"Ne\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_38;
    impl ::pest_typed::StringWrapper for r#w_38 {
        const CONTENT: &'static ::core::primitive::str = "Ne";
    }
    #[doc = "A wrapper for `\"Offset\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_39;
    impl ::pest_typed::StringWrapper for r#w_39 {
        const CONTENT: &'static ::core::primitive::str = "Offset";
    }
    #[doc = "A wrapper for `\"SizeOf\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_40;
    impl ::pest_typed::StringWrapper for r#w_40 {
        const CONTENT: &'static ::core::primitive::str = "SizeOf";
    }
    #[doc = "A wrapper for `\"AlignOf\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_41;
    impl ::pest_typed::StringWrapper for r#w_41 {
        const CONTENT: &'static ::core::primitive::str = "AlignOf";
    }
    #[doc = "A wrapper for `\"Neg\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_42;
    impl ::pest_typed::StringWrapper for r#w_42 {
        const CONTENT: &'static ::core::primitive::str = "Neg";
    }
    #[doc = "A wrapper for `\"Not\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_43;
    impl ::pest_typed::StringWrapper for r#w_43 {
        const CONTENT: &'static ::core::primitive::str = "Not";
    }
    #[doc = "A wrapper for `\"PtrMetadata\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_44;
    impl ::pest_typed::StringWrapper for r#w_44 {
        const CONTENT: &'static ::core::primitive::str = "PtrMetadata";
    }
    #[doc = "A wrapper for `\"discriminant\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_45;
    impl ::pest_typed::StringWrapper for r#w_45 {
        const CONTENT: &'static ::core::primitive::str = "discriminant";
    }
    #[doc = "A wrapper for `\"Ctor\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_46;
    impl ::pest_typed::StringWrapper for r#w_46 {
        const CONTENT: &'static ::core::primitive::str = "Ctor";
    }
    #[doc = "A wrapper for `\"from\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_47;
    impl ::pest_typed::StringWrapper for r#w_47 {
        const CONTENT: &'static ::core::primitive::str = "from";
    }
    #[doc = "A wrapper for `\"of\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_48;
    impl ::pest_typed::StringWrapper for r#w_48 {
        const CONTENT: &'static ::core::primitive::str = "of";
    }
    #[doc = "A wrapper for `\"raw\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_49;
    impl ::pest_typed::StringWrapper for r#w_49 {
        const CONTENT: &'static ::core::primitive::str = "raw";
    }
    #[doc = "A wrapper for `\"drop\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_50;
    impl ::pest_typed::StringWrapper for r#w_50 {
        const CONTENT: &'static ::core::primitive::str = "drop";
    }
    #[doc = "A wrapper for `\"break\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_51;
    impl ::pest_typed::StringWrapper for r#w_51 {
        const CONTENT: &'static ::core::primitive::str = "break";
    }
    #[doc = "A wrapper for `\"continue\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_52;
    impl ::pest_typed::StringWrapper for r#w_52 {
        const CONTENT: &'static ::core::primitive::str = "continue";
    }
    #[doc = "A wrapper for `\"loop\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_53;
    impl ::pest_typed::StringWrapper for r#w_53 {
        const CONTENT: &'static ::core::primitive::str = "loop";
    }
    #[doc = "A wrapper for `\"switchInt\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_54;
    impl ::pest_typed::StringWrapper for r#w_54 {
        const CONTENT: &'static ::core::primitive::str = "switchInt";
    }
    #[doc = "A wrapper for `\"true\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_55;
    impl ::pest_typed::StringWrapper for r#w_55 {
        const CONTENT: &'static ::core::primitive::str = "true";
    }
    #[doc = "A wrapper for `\"false\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_56;
    impl ::pest_typed::StringWrapper for r#w_56 {
        const CONTENT: &'static ::core::primitive::str = "false";
    }
    #[doc = "A wrapper for `\"unsafe\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_57;
    impl ::pest_typed::StringWrapper for r#w_57 {
        const CONTENT: &'static ::core::primitive::str = "unsafe";
    }
    #[doc = "A wrapper for `\"pub\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_58;
    impl ::pest_typed::StringWrapper for r#w_58 {
        const CONTENT: &'static ::core::primitive::str = "pub";
    }
    #[doc = "A wrapper for `\"struct\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_59;
    impl ::pest_typed::StringWrapper for r#w_59 {
        const CONTENT: &'static ::core::primitive::str = "struct";
    }
    #[doc = "A wrapper for `\"enum\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_60;
    impl ::pest_typed::StringWrapper for r#w_60 {
        const CONTENT: &'static ::core::primitive::str = "enum";
    }
    #[doc = "A wrapper for `\"impl\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_61;
    impl ::pest_typed::StringWrapper for r#w_61 {
        const CONTENT: &'static ::core::primitive::str = "impl";
    }
    #[doc = "A wrapper for `\"for\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_62;
    impl ::pest_typed::StringWrapper for r#w_62 {
        const CONTENT: &'static ::core::primitive::str = "for";
    }
    #[doc = "A wrapper for `\"u8\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_63;
    impl ::pest_typed::StringWrapper for r#w_63 {
        const CONTENT: &'static ::core::primitive::str = "u8";
    }
    #[doc = "A wrapper for `\"u16\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_64;
    impl ::pest_typed::StringWrapper for r#w_64 {
        const CONTENT: &'static ::core::primitive::str = "u16";
    }
    #[doc = "A wrapper for `\"u32\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_65;
    impl ::pest_typed::StringWrapper for r#w_65 {
        const CONTENT: &'static ::core::primitive::str = "u32";
    }
    #[doc = "A wrapper for `\"u64\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_66;
    impl ::pest_typed::StringWrapper for r#w_66 {
        const CONTENT: &'static ::core::primitive::str = "u64";
    }
    #[doc = "A wrapper for `\"usize\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_67;
    impl ::pest_typed::StringWrapper for r#w_67 {
        const CONTENT: &'static ::core::primitive::str = "usize";
    }
    #[doc = "A wrapper for `\"i8\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_68;
    impl ::pest_typed::StringWrapper for r#w_68 {
        const CONTENT: &'static ::core::primitive::str = "i8";
    }
    #[doc = "A wrapper for `\"i16\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_69;
    impl ::pest_typed::StringWrapper for r#w_69 {
        const CONTENT: &'static ::core::primitive::str = "i16";
    }
    #[doc = "A wrapper for `\"i32\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_70;
    impl ::pest_typed::StringWrapper for r#w_70 {
        const CONTENT: &'static ::core::primitive::str = "i32";
    }
    #[doc = "A wrapper for `\"i64\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_71;
    impl ::pest_typed::StringWrapper for r#w_71 {
        const CONTENT: &'static ::core::primitive::str = "i64";
    }
    #[doc = "A wrapper for `\"isize\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_72;
    impl ::pest_typed::StringWrapper for r#w_72 {
        const CONTENT: &'static ::core::primitive::str = "isize";
    }
    #[doc = "A wrapper for `\"RET\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_73;
    impl ::pest_typed::StringWrapper for r#w_73 {
        const CONTENT: &'static ::core::primitive::str = "RET";
    }
    #[doc = "A wrapper for `\"bool\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_74;
    impl ::pest_typed::StringWrapper for r#w_74 {
        const CONTENT: &'static ::core::primitive::str = "bool";
    }
    #[doc = "A wrapper for `\"str\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_75;
    impl ::pest_typed::StringWrapper for r#w_75 {
        const CONTENT: &'static ::core::primitive::str = "str";
    }
    #[doc = "A wrapper for `\"place\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_76;
    impl ::pest_typed::StringWrapper for r#w_76 {
        const CONTENT: &'static ::core::primitive::str = "place";
    }
    #[doc = "A wrapper for `\"where\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_77;
    impl ::pest_typed::StringWrapper for r#w_77 {
        const CONTENT: &'static ::core::primitive::str = "where";
    }
    #[doc = "A wrapper for `\"//\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_78;
    impl ::pest_typed::StringWrapper for r#w_78 {
        const CONTENT: &'static ::core::primitive::str = "//";
    }
    #[doc = "A wrapper for `\"/*\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_79;
    impl ::pest_typed::StringWrapper for r#w_79 {
        const CONTENT: &'static ::core::primitive::str = "/*";
    }
    #[doc = "A wrapper for `\"*/\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_80;
    impl ::pest_typed::StringWrapper for r#w_80 {
        const CONTENT: &'static ::core::primitive::str = "*/";
    }
    #[doc = "A wrapper for `\"*/\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_81;
    impl ::pest_typed::StringWrapper for r#w_81 {
        const CONTENT: &'static ::core::primitive::str = "*/";
    }
    #[doc = "A wrapper for `\" \"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_82;
    impl ::pest_typed::StringWrapper for r#w_82 {
        const CONTENT: &'static ::core::primitive::str = " ";
    }
    #[doc = "A wrapper for `\"\\t\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_83;
    impl ::pest_typed::StringWrapper for r#w_83 {
        const CONTENT: &'static ::core::primitive::str = "\t";
    }
    #[doc = "A wrapper for `\"\\r\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_84;
    impl ::pest_typed::StringWrapper for r#w_84 {
        const CONTENT: &'static ::core::primitive::str = "\r";
    }
    #[doc = "A wrapper for `\"\\n\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_85;
    impl ::pest_typed::StringWrapper for r#w_85 {
        const CONTENT: &'static ::core::primitive::str = "\n";
    }
    #[doc = "A wrapper for `\"{\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_86;
    impl ::pest_typed::StringWrapper for r#w_86 {
        const CONTENT: &'static ::core::primitive::str = "{";
    }
    #[doc = "A wrapper for `\"}\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_87;
    impl ::pest_typed::StringWrapper for r#w_87 {
        const CONTENT: &'static ::core::primitive::str = "}";
    }
    #[doc = "A wrapper for `\"[\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_88;
    impl ::pest_typed::StringWrapper for r#w_88 {
        const CONTENT: &'static ::core::primitive::str = "[";
    }
    #[doc = "A wrapper for `\"]\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_89;
    impl ::pest_typed::StringWrapper for r#w_89 {
        const CONTENT: &'static ::core::primitive::str = "]";
    }
    #[doc = "A wrapper for `\"(\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_90;
    impl ::pest_typed::StringWrapper for r#w_90 {
        const CONTENT: &'static ::core::primitive::str = "(";
    }
    #[doc = "A wrapper for `\")\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_91;
    impl ::pest_typed::StringWrapper for r#w_91 {
        const CONTENT: &'static ::core::primitive::str = ")";
    }
    #[doc = "A wrapper for `\"<\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_92;
    impl ::pest_typed::StringWrapper for r#w_92 {
        const CONTENT: &'static ::core::primitive::str = "<";
    }
    #[doc = "A wrapper for `\">\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_93;
    impl ::pest_typed::StringWrapper for r#w_93 {
        const CONTENT: &'static ::core::primitive::str = ">";
    }
    #[doc = "A wrapper for `\"$\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_94;
    impl ::pest_typed::StringWrapper for r#w_94 {
        const CONTENT: &'static ::core::primitive::str = "$";
    }
    #[doc = "A wrapper for `\"=\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_95;
    impl ::pest_typed::StringWrapper for r#w_95 {
        const CONTENT: &'static ::core::primitive::str = "=";
    }
    #[doc = "A wrapper for `\",\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_96;
    impl ::pest_typed::StringWrapper for r#w_96 {
        const CONTENT: &'static ::core::primitive::str = ",";
    }
    #[doc = "A wrapper for `\".\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_97;
    impl ::pest_typed::StringWrapper for r#w_97 {
        const CONTENT: &'static ::core::primitive::str = ".";
    }
    #[doc = "A wrapper for `\"..\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_98;
    impl ::pest_typed::StringWrapper for r#w_98 {
        const CONTENT: &'static ::core::primitive::str = "..";
    }
    #[doc = "A wrapper for `\":\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_99;
    impl ::pest_typed::StringWrapper for r#w_99 {
        const CONTENT: &'static ::core::primitive::str = ":";
    }
    #[doc = "A wrapper for `\"::\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_100;
    impl ::pest_typed::StringWrapper for r#w_100 {
        const CONTENT: &'static ::core::primitive::str = "::";
    }
    #[doc = "A wrapper for `\";\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_101;
    impl ::pest_typed::StringWrapper for r#w_101 {
        const CONTENT: &'static ::core::primitive::str = ";";
    }
    #[doc = "A wrapper for `\"#\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_102;
    impl ::pest_typed::StringWrapper for r#w_102 {
        const CONTENT: &'static ::core::primitive::str = "#";
    }
    #[doc = "A wrapper for `\"&\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_103;
    impl ::pest_typed::StringWrapper for r#w_103 {
        const CONTENT: &'static ::core::primitive::str = "&";
    }
    #[doc = "A wrapper for `\"&&\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_104;
    impl ::pest_typed::StringWrapper for r#w_104 {
        const CONTENT: &'static ::core::primitive::str = "&&";
    }
    #[doc = "A wrapper for `\"||\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_105;
    impl ::pest_typed::StringWrapper for r#w_105 {
        const CONTENT: &'static ::core::primitive::str = "||";
    }
    #[doc = "A wrapper for `\"!\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_106;
    impl ::pest_typed::StringWrapper for r#w_106 {
        const CONTENT: &'static ::core::primitive::str = "!";
    }
    #[doc = "A wrapper for `\"*\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_107;
    impl ::pest_typed::StringWrapper for r#w_107 {
        const CONTENT: &'static ::core::primitive::str = "*";
    }
    #[doc = "A wrapper for `\"->\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_108;
    impl ::pest_typed::StringWrapper for r#w_108 {
        const CONTENT: &'static ::core::primitive::str = "->";
    }
    #[doc = "A wrapper for `\"=>\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_109;
    impl ::pest_typed::StringWrapper for r#w_109 {
        const CONTENT: &'static ::core::primitive::str = "=>";
    }
    #[doc = "A wrapper for `\"'\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_110;
    impl ::pest_typed::StringWrapper for r#w_110 {
        const CONTENT: &'static ::core::primitive::str = "'";
    }
    #[doc = "A wrapper for `\"-\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_111;
    impl ::pest_typed::StringWrapper for r#w_111 {
        const CONTENT: &'static ::core::primitive::str = "-";
    }
    #[doc = "A wrapper for `\"_\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_112;
    impl ::pest_typed::StringWrapper for r#w_112 {
        const CONTENT: &'static ::core::primitive::str = "_";
    }
    #[doc = "A wrapper for `\"_\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_113;
    impl ::pest_typed::StringWrapper for r#w_113 {
        const CONTENT: &'static ::core::primitive::str = "_";
    }
    #[doc = "A wrapper for `\"0b\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_114;
    impl ::pest_typed::StringWrapper for r#w_114 {
        const CONTENT: &'static ::core::primitive::str = "0b";
    }
    #[doc = "A wrapper for `\"_\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_115;
    impl ::pest_typed::StringWrapper for r#w_115 {
        const CONTENT: &'static ::core::primitive::str = "_";
    }
    #[doc = "A wrapper for `\"_\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_116;
    impl ::pest_typed::StringWrapper for r#w_116 {
        const CONTENT: &'static ::core::primitive::str = "_";
    }
    #[doc = "A wrapper for `\"0o\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_117;
    impl ::pest_typed::StringWrapper for r#w_117 {
        const CONTENT: &'static ::core::primitive::str = "0o";
    }
    #[doc = "A wrapper for `\"_\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_118;
    impl ::pest_typed::StringWrapper for r#w_118 {
        const CONTENT: &'static ::core::primitive::str = "_";
    }
    #[doc = "A wrapper for `\"_\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_119;
    impl ::pest_typed::StringWrapper for r#w_119 {
        const CONTENT: &'static ::core::primitive::str = "_";
    }
    #[doc = "A wrapper for `\"0x\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_120;
    impl ::pest_typed::StringWrapper for r#w_120 {
        const CONTENT: &'static ::core::primitive::str = "0x";
    }
    #[doc = "A wrapper for `\"_\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_121;
    impl ::pest_typed::StringWrapper for r#w_121 {
        const CONTENT: &'static ::core::primitive::str = "_";
    }
    #[doc = "A wrapper for `\"_\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_122;
    impl ::pest_typed::StringWrapper for r#w_122 {
        const CONTENT: &'static ::core::primitive::str = "_";
    }
    #[doc = "A wrapper for `\"\\\"\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_123;
    impl ::pest_typed::StringWrapper for r#w_123 {
        const CONTENT: &'static ::core::primitive::str = "\"";
    }
    #[doc = "A wrapper for `[\"\\\"\"]`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, PartialEq)]
    pub struct r#w_124;
    impl ::pest_typed::StringArrayWrapper for r#w_124 {
        const CONTENT: &'static [&'static ::core::primitive::str] = &["\""];
    }
    #[doc = "A wrapper for `\"\\\"\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_125;
    impl ::pest_typed::StringWrapper for r#w_125 {
        const CONTENT: &'static ::core::primitive::str = "\"";
    }
    #[doc = "A wrapper for `\"_\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_126;
    impl ::pest_typed::StringWrapper for r#w_126 {
        const CONTENT: &'static ::core::primitive::str = "_";
    }
    #[doc = "A wrapper for `\"-\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_127;
    impl ::pest_typed::StringWrapper for r#w_127 {
        const CONTENT: &'static ::core::primitive::str = "-";
    }
    #[doc = "A wrapper for `\"Group\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_128;
    impl ::pest_typed::StringWrapper for r#w_128 {
        const CONTENT: &'static ::core::primitive::str = "Group";
    }
    #[doc = "A wrapper for `\"\\\"\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_129;
    impl ::pest_typed::StringWrapper for r#w_129 {
        const CONTENT: &'static ::core::primitive::str = "\"";
    }
    #[doc = "A wrapper for `\"\\\"\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_130;
    impl ::pest_typed::StringWrapper for r#w_130 {
        const CONTENT: &'static ::core::primitive::str = "\"";
    }
    #[doc = "A wrapper for `\"\\\"\"`."]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct r#w_131;
    impl ::pest_typed::StringWrapper for r#w_131 {
        const CONTENT: &'static ::core::primitive::str = "\"";
    }
}
#[doc = "Generated structs for tags."]
pub mod tags {}
#[doc = "Definitions of statically typed nodes generated by pest-generator."]
pub mod rules_impl {
    #[doc = "Definitions of statically typed nodes generated by pest-generator."]
    pub mod rules {
        :: pest_typed :: rule ! (r#kw_pattern , "Corresponds to expression: `(\"pattern\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_pattern , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_0 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_pattern<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_patt , "Corresponds to expression: `(\"patt\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_patt , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_1 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_patt<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_util , "Corresponds to expression: `(\"util\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_util , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_2 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_util<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_cstr , "Corresponds to expression: `(\"cstr\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_cstr , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_3 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_cstr<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_diag , "Corresponds to expression: `(\"diag\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_diag , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_4 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_diag<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_meta , "Corresponds to expression: `(\"meta\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_meta , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_5 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_meta<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_import , "Corresponds to expression: `(\"import\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_import , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_6 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_import<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_self , "Corresponds to expression: `(\"self\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_self , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_7 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_self<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_Self , "Corresponds to expression: `(\"Self\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_Self , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_8 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_Self<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_fn , "Corresponds to expression: `(\"fn\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_fn , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_9 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_fn<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_mut , "Corresponds to expression: `(\"mut\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_mut , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_10 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_mut<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_const , "Corresponds to expression: `(\"const\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_const , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_11 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_const<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_static , "Corresponds to expression: `(\"static\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_static , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_12 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_static<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_lang , "Corresponds to expression: `(\"lang\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_lang , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_13 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_lang<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_as , "Corresponds to expression: `(\"as\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_as , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_14 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_as<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_crate , "Corresponds to expression: `(\"crate\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_crate , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_15 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_crate<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_use , "Corresponds to expression: `(\"use\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_use , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_16 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_use<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_type , "Corresponds to expression: `(\"type\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_type , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_17 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_type<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_let , "Corresponds to expression: `(\"let\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_let , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_18 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_let<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_move , "Corresponds to expression: `(\"move\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_move , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_19 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_move<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_copy , "Corresponds to expression: `(\"copy\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_copy , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_20 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_copy<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_Len , "Corresponds to expression: `(\"Len\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_Len , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_21 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_Len<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_PtrToPtr , "Corresponds to expression: `(\"PtrToPtr\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_PtrToPtr , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_22 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_PtrToPtr<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_IntToInt , "Corresponds to expression: `(\"IntToInt\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_IntToInt , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_23 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_IntToInt<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_Transmute , "Corresponds to expression: `(\"Transmute\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_Transmute , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_24 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_Transmute<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_PointerCoercion , "Corresponds to expression: `(\"PointerCoercion\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_PointerCoercion , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_25 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_PointerCoercion<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_Unsize , "Corresponds to expression: `(\"Unsize\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_Unsize , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_26 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_Unsize<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_Implicit , "Corresponds to expression: `(\"Implicit\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_Implicit , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_27 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_Implicit<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_Add , "Corresponds to expression: `(\"Add\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_Add , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_28 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_Add<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_Sub , "Corresponds to expression: `(\"Sub\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_Sub , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_29 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_Sub<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_Mul , "Corresponds to expression: `(\"Mul\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_Mul , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_30 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_Mul<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_Div , "Corresponds to expression: `(\"Div\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_Div , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_31 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_Div<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_Rem , "Corresponds to expression: `(\"Rem\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_Rem , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_32 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_Rem<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_Lt , "Corresponds to expression: `(\"Lt\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_Lt , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_33 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_Lt<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_Le , "Corresponds to expression: `(\"Le\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_Le , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_34 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_Le<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_Gt , "Corresponds to expression: `(\"Gt\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_Gt , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_35 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_Gt<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_Ge , "Corresponds to expression: `(\"Ge\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_Ge , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_36 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_Ge<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_Eq , "Corresponds to expression: `(\"Eq\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_Eq , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_37 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_Eq<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_Ne , "Corresponds to expression: `(\"Ne\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_Ne , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_38 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_Ne<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_Offset , "Corresponds to expression: `(\"Offset\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_Offset , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_39 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_Offset<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_SizeOf , "Corresponds to expression: `(\"SizeOf\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_SizeOf , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_40 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_SizeOf<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_AlignOf , "Corresponds to expression: `(\"AlignOf\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_AlignOf , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_41 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_AlignOf<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_Neg , "Corresponds to expression: `(\"Neg\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_Neg , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_42 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_Neg<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_Not , "Corresponds to expression: `(\"Not\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_Not , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_43 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_Not<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_PtrMetadata , "Corresponds to expression: `(\"PtrMetadata\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_PtrMetadata , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_44 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_PtrMetadata<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_discriminant , "Corresponds to expression: `(\"discriminant\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_discriminant , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_45 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_discriminant<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_Ctor , "Corresponds to expression: `(\"Ctor\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_Ctor , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_46 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_Ctor<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_from , "Corresponds to expression: `(\"from\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_from , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_47 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_from<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_of , "Corresponds to expression: `(\"of\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_of , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_48 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_of<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_raw , "Corresponds to expression: `(\"raw\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_raw , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_49 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_raw<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_drop , "Corresponds to expression: `(\"drop\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_drop , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_50 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_drop<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_break , "Corresponds to expression: `(\"break\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_break , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_51 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_break<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_continue , "Corresponds to expression: `(\"continue\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_continue , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_52 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_continue<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_loop , "Corresponds to expression: `(\"loop\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_loop , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_53 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_loop<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_switchInt , "Corresponds to expression: `(\"switchInt\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_switchInt , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_54 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_switchInt<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_true , "Corresponds to expression: `(\"true\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_true , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_55 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_true<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_false , "Corresponds to expression: `(\"false\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_false , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_56 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_false<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_unsafe , "Corresponds to expression: `(\"unsafe\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_unsafe , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_57 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_unsafe<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_pub , "Corresponds to expression: `(\"pub\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_pub , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_58 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_pub<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_struct , "Corresponds to expression: `(\"struct\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_struct , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_59 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_struct<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_enum , "Corresponds to expression: `(\"enum\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_enum , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_60 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_enum<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_impl , "Corresponds to expression: `(\"impl\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_impl , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_61 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_impl<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_for , "Corresponds to expression: `(\"for\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_for , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_62 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_for<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_u8 , "Corresponds to expression: `(\"u8\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_u8 , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_63 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_u8<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_u16 , "Corresponds to expression: `(\"u16\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_u16 , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_64 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_u16<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_u32 , "Corresponds to expression: `(\"u32\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_u32 , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_65 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_u32<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_u64 , "Corresponds to expression: `(\"u64\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_u64 , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_66 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_u64<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_usize , "Corresponds to expression: `(\"usize\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_usize , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_67 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_usize<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_i8 , "Corresponds to expression: `(\"i8\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_i8 , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_68 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_i8<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_i16 , "Corresponds to expression: `(\"i16\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_i16 , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_69 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_i16<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_i32 , "Corresponds to expression: `(\"i32\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_i32 , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_70 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_i32<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_i64 , "Corresponds to expression: `(\"i64\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_i64 , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_71 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_i64<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_isize , "Corresponds to expression: `(\"isize\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_isize , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_72 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_isize<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_RET , "Corresponds to expression: `(\"RET\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_RET , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_73 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_RET<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_bool , "Corresponds to expression: `(\"bool\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_bool , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_74 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_bool<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_str , "Corresponds to expression: `(\"str\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_str , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_75 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_str<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_place , "Corresponds to expression: `(\"place\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_place , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_76 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_place<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#kw_where , "Corresponds to expression: `(\"where\" ~ !WordFollowing)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#kw_where , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_77 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#kw_where<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#Keywords , "Corresponds to expression: `(kw_pattern | kw_patt | kw_util | kw_cstr | kw_diag | kw_meta | kw_import | kw_self | kw_Self | kw_fn | kw_mut | kw_const | kw_static | kw_lang | kw_as | kw_crate | kw_use | kw_type | kw_let | kw_move | kw_copy | kw_Len | kw_PtrToPtr | kw_IntToInt | kw_Transmute | kw_PointerCoercion | kw_Add | kw_Sub | kw_Mul | kw_Div | kw_Rem | kw_Lt | kw_Le | kw_Gt | kw_Ge | kw_Eq | kw_Ne | kw_Offset | kw_SizeOf | kw_AlignOf | kw_Neg | kw_Not | kw_PtrMetadata | kw_discriminant | kw_Ctor | kw_from | kw_of | kw_raw | kw_drop | kw_break | kw_continue | kw_loop | kw_switchInt | kw_true | kw_false | kw_unsafe | kw_pub | kw_struct | kw_enum | kw_impl | kw_for | kw_u8 | kw_u16 | kw_u32 | kw_u64 | kw_usize | kw_i8 | kw_i16 | kw_i32 | kw_i64 | kw_isize | kw_bool | kw_str | kw_where)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Keywords , super :: super :: generics :: Choice74 :: < super :: super :: rules :: r#kw_pattern :: < 'i , 0 > , super :: super :: rules :: r#kw_patt :: < 'i , 0 > , super :: super :: rules :: r#kw_util :: < 'i , 0 > , super :: super :: rules :: r#kw_cstr :: < 'i , 0 > , super :: super :: rules :: r#kw_diag :: < 'i , 0 > , super :: super :: rules :: r#kw_meta :: < 'i , 0 > , super :: super :: rules :: r#kw_import :: < 'i , 0 > , super :: super :: rules :: r#kw_self :: < 'i , 0 > , super :: super :: rules :: r#kw_Self :: < 'i , 0 > , super :: super :: rules :: r#kw_fn :: < 'i , 0 > , super :: super :: rules :: r#kw_mut :: < 'i , 0 > , super :: super :: rules :: r#kw_const :: < 'i , 0 > , super :: super :: rules :: r#kw_static :: < 'i , 0 > , super :: super :: rules :: r#kw_lang :: < 'i , 0 > , super :: super :: rules :: r#kw_as :: < 'i , 0 > , super :: super :: rules :: r#kw_crate :: < 'i , 0 > , super :: super :: rules :: r#kw_use :: < 'i , 0 > , super :: super :: rules :: r#kw_type :: < 'i , 0 > , super :: super :: rules :: r#kw_let :: < 'i , 0 > , super :: super :: rules :: r#kw_move :: < 'i , 0 > , super :: super :: rules :: r#kw_copy :: < 'i , 0 > , super :: super :: rules :: r#kw_Len :: < 'i , 0 > , super :: super :: rules :: r#kw_PtrToPtr :: < 'i , 0 > , super :: super :: rules :: r#kw_IntToInt :: < 'i , 0 > , super :: super :: rules :: r#kw_Transmute :: < 'i , 0 > , super :: super :: rules :: r#kw_PointerCoercion :: < 'i , 0 > , super :: super :: rules :: r#kw_Add :: < 'i , 0 > , super :: super :: rules :: r#kw_Sub :: < 'i , 0 > , super :: super :: rules :: r#kw_Mul :: < 'i , 0 > , super :: super :: rules :: r#kw_Div :: < 'i , 0 > , super :: super :: rules :: r#kw_Rem :: < 'i , 0 > , super :: super :: rules :: r#kw_Lt :: < 'i , 0 > , super :: super :: rules :: r#kw_Le :: < 'i , 0 > , super :: super :: rules :: r#kw_Gt :: < 'i , 0 > , super :: super :: rules :: r#kw_Ge :: < 'i , 0 > , super :: super :: rules :: r#kw_Eq :: < 'i , 0 > , super :: super :: rules :: r#kw_Ne :: < 'i , 0 > , super :: super :: rules :: r#kw_Offset :: < 'i , 0 > , super :: super :: rules :: r#kw_SizeOf :: < 'i , 0 > , super :: super :: rules :: r#kw_AlignOf :: < 'i , 0 > , super :: super :: rules :: r#kw_Neg :: < 'i , 0 > , super :: super :: rules :: r#kw_Not :: < 'i , 0 > , super :: super :: rules :: r#kw_PtrMetadata :: < 'i , 0 > , super :: super :: rules :: r#kw_discriminant :: < 'i , 0 > , super :: super :: rules :: r#kw_Ctor :: < 'i , 0 > , super :: super :: rules :: r#kw_from :: < 'i , 0 > , super :: super :: rules :: r#kw_of :: < 'i , 0 > , super :: super :: rules :: r#kw_raw :: < 'i , 0 > , super :: super :: rules :: r#kw_drop :: < 'i , 0 > , super :: super :: rules :: r#kw_break :: < 'i , 0 > , super :: super :: rules :: r#kw_continue :: < 'i , 0 > , super :: super :: rules :: r#kw_loop :: < 'i , 0 > , super :: super :: rules :: r#kw_switchInt :: < 'i , 0 > , super :: super :: rules :: r#kw_true :: < 'i , 0 > , super :: super :: rules :: r#kw_false :: < 'i , 0 > , super :: super :: rules :: r#kw_unsafe :: < 'i , 0 > , super :: super :: rules :: r#kw_pub :: < 'i , 0 > , super :: super :: rules :: r#kw_struct :: < 'i , 0 > , super :: super :: rules :: r#kw_enum :: < 'i , 0 > , super :: super :: rules :: r#kw_impl :: < 'i , 0 > , super :: super :: rules :: r#kw_for :: < 'i , 0 > , super :: super :: rules :: r#kw_u8 :: < 'i , 0 > , super :: super :: rules :: r#kw_u16 :: < 'i , 0 > , super :: super :: rules :: r#kw_u32 :: < 'i , 0 > , super :: super :: rules :: r#kw_u64 :: < 'i , 0 > , super :: super :: rules :: r#kw_usize :: < 'i , 0 > , super :: super :: rules :: r#kw_i8 :: < 'i , 0 > , super :: super :: rules :: r#kw_i16 :: < 'i , 0 > , super :: super :: rules :: r#kw_i32 :: < 'i , 0 > , super :: super :: rules :: r#kw_i64 :: < 'i , 0 > , super :: super :: rules :: r#kw_isize :: < 'i , 0 > , super :: super :: rules :: r#kw_bool :: < 'i , 0 > , super :: super :: rules :: r#kw_str :: < 'i , 0 > , super :: super :: rules :: r#kw_where :: < 'i , 0 > , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Keywords<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#COMMENT , "Corresponds to expression: `((\"//\" ~ (!NEWLINE ~ ANY)*) | (\"/*\" ~ (!\"*/\" ~ ANY)* ~ \"*/\"))`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#COMMENT , super :: super :: generics :: Choice2 :: < super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_78 > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#NEWLINE > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#ANY , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_79 > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_80 > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#ANY , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_81 > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Expression , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#COMMENT<'i, INHERITED> {
            #[doc = "A helper function to access [`ANY`]."]
            #[allow(non_snake_case)]
            pub fn r#ANY<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Option<::pest_typed::re_exported::Vec<&'s super::super::rules::r#ANY>>,
                ::pest_typed::re_exported::Option<::pest_typed::re_exported::Vec<&'s super::super::rules::r#ANY>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = res._0().map(|res| {
                                let res = &res.content.1.matched;
                                {
                                    let res = res
                                        .content
                                        .iter()
                                        .map(|res| {
                                            let res = &res.matched;
                                            {
                                                let res = &res.content.1.matched;
                                                res
                                            }
                                        })
                                        .collect::<::pest_typed::re_exported::Vec<_>>();
                                    res
                                }
                            });
                            res
                        },
                        {
                            let res = res._1().map(|res| {
                                let res = &res.content.1.matched;
                                {
                                    let res = res
                                        .content
                                        .iter()
                                        .map(|res| {
                                            let res = &res.matched;
                                            {
                                                let res = &res.content.1.matched;
                                                res
                                            }
                                        })
                                        .collect::<::pest_typed::re_exported::Vec<_>>();
                                    res
                                }
                            });
                            res
                        },
                    );
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#WHITESPACE , "Corresponds to expression: `(\" \" | \"\\t\" | \"\\r\" | \"\\n\")`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#WHITESPACE , super :: super :: generics :: Choice4 :: < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_82 > , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_83 > , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_84 > , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_85 > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Expression , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#WHITESPACE<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#LeftBrace , "Corresponds to expression: `\"{\"`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#LeftBrace , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_86 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#LeftBrace<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#RightBrace , "Corresponds to expression: `\"}\"`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#RightBrace , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_87 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#RightBrace<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#LeftBracket , "Corresponds to expression: `\"[\"`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#LeftBracket , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_88 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#LeftBracket<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#RightBracket , "Corresponds to expression: `\"]\"`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#RightBracket , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_89 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#RightBracket<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#LeftParen , "Corresponds to expression: `\"(\"`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#LeftParen , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_90 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#LeftParen<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#RightParen , "Corresponds to expression: `\")\"`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#RightParen , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_91 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#RightParen<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#LessThan , "Corresponds to expression: `\"<\"`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#LessThan , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_92 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#LessThan<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#GreaterThan , "Corresponds to expression: `\">\"`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#GreaterThan , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_93 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#GreaterThan<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#Dollar , "Corresponds to expression: `\"$\"`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Dollar , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_94 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Dollar<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#Assign , "Corresponds to expression: `\"=\"`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Assign , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_95 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Assign<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#Comma , "Corresponds to expression: `\",\"`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Comma , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_96 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Comma<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#Dot , "Corresponds to expression: `\".\"`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Dot , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_97 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Dot<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#Dot2 , "Corresponds to expression: `\"..\"`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Dot2 , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_98 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Dot2<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#Colon , "Corresponds to expression: `\":\"`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Colon , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_99 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Colon<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#Colon2 , "Corresponds to expression: `\"::\"`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Colon2 , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_100 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Colon2<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#SemiColon , "Corresponds to expression: `\";\"`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#SemiColon , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_101 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#SemiColon<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#Hash , "Corresponds to expression: `\"#\"`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Hash , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_102 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Hash<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#And , "Corresponds to expression: `\"&\"`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#And , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_103 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#And<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#AndAnd , "Corresponds to expression: `\"&&\"`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#AndAnd , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_104 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#AndAnd<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#OrOr , "Corresponds to expression: `\"||\"`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#OrOr , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_105 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#OrOr<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#Bang , "Corresponds to expression: `\"!\"`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Bang , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_106 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Bang<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#Star , "Corresponds to expression: `\"*\"`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Star , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_107 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Star<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#Arrow , "Corresponds to expression: `\"->\"`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Arrow , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_108 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Arrow<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#RightArrow , "Corresponds to expression: `\"=>\"`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#RightArrow , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_109 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#RightArrow<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#Quote , "Corresponds to expression: `\"'\"`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Quote , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_110 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Quote<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#Minus , "Corresponds to expression: `\"-\"`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Minus , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_111 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Minus<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#PlaceHolder , "Corresponds to expression: `\"_\"`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#PlaceHolder , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_112 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#PlaceHolder<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#Literal , "Corresponds to expression: `(Integer | String | Bool)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Literal , super :: super :: generics :: Choice3 :: < super :: super :: rules :: r#Integer :: < 'i , INHERITED > , super :: super :: rules :: r#String :: < 'i , INHERITED > , super :: super :: rules :: r#Bool :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Literal<'i, INHERITED> {
            #[doc = "A helper function to access [`Bool`]."]
            #[allow(non_snake_case)]
            pub fn r#Bool<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Bool<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._2().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`Integer`]."]
            #[allow(non_snake_case)]
            pub fn r#Integer<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Integer<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`String`]."]
            #[allow(non_snake_case)]
            pub fn r#String<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#String<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#BIN_DIGIT , "Corresponds to expression: `('0'..'1')`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#BIN_DIGIT , super :: super :: generics :: CharRange :: < '0' , '1' > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#BIN_DIGIT<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#OCT_DIGIT , "Corresponds to expression: `('0'..'7')`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#OCT_DIGIT , super :: super :: generics :: CharRange :: < '0' , '7' > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#OCT_DIGIT<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#DEC_DIGIT , "Corresponds to expression: `('0'..'9')`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#DEC_DIGIT , super :: super :: generics :: CharRange :: < '0' , '9' > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#DEC_DIGIT<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#HEX_DIGIT , "Corresponds to expression: `(('0'..'9') | ('a'..'f') | ('A'..'F'))`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#HEX_DIGIT , super :: super :: generics :: Choice3 :: < super :: super :: generics :: CharRange :: < '0' , '9' > , super :: super :: generics :: CharRange :: < 'a' , 'f' > , super :: super :: generics :: CharRange :: < 'A' , 'F' > , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#HEX_DIGIT<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#DEC_LITERAL , "Corresponds to expression: `(DEC_DIGIT ~ (DEC_DIGIT | \"_\")*)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#DEC_LITERAL , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#DEC_DIGIT :: < 'i , 0 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , 0 , super :: super :: generics :: Choice2 :: < super :: super :: rules :: r#DEC_DIGIT :: < 'i , 0 > , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_113 > , > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#DEC_LITERAL<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#BIN_LITERAL , "Corresponds to expression: `(\"0b\" ~ (BIN_DIGIT | \"_\")* ~ BIN_DIGIT ~ (BIN_DIGIT | \"_\")*)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#BIN_LITERAL , super :: super :: generics :: Seq4 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_114 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , 0 , super :: super :: generics :: Choice2 :: < super :: super :: rules :: r#BIN_DIGIT :: < 'i , 0 > , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_115 > , > > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#BIN_DIGIT :: < 'i , 0 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , 0 , super :: super :: generics :: Choice2 :: < super :: super :: rules :: r#BIN_DIGIT :: < 'i , 0 > , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_116 > , > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#BIN_LITERAL<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#OCT_LITERAL , "Corresponds to expression: `(\"0o\" ~ (OCT_DIGIT | \"_\")* ~ OCT_DIGIT ~ (OCT_DIGIT | \"_\")*)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#OCT_LITERAL , super :: super :: generics :: Seq4 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_117 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , 0 , super :: super :: generics :: Choice2 :: < super :: super :: rules :: r#OCT_DIGIT :: < 'i , 0 > , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_118 > , > > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#OCT_DIGIT :: < 'i , 0 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , 0 , super :: super :: generics :: Choice2 :: < super :: super :: rules :: r#OCT_DIGIT :: < 'i , 0 > , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_119 > , > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#OCT_LITERAL<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#HEX_LITERAL , "Corresponds to expression: `(\"0x\" ~ (HEX_DIGIT | \"_\")* ~ HEX_DIGIT ~ (HEX_DIGIT | \"_\")*)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#HEX_LITERAL , super :: super :: generics :: Seq4 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_120 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , 0 , super :: super :: generics :: Choice2 :: < super :: super :: rules :: r#HEX_DIGIT :: < 'i , 0 > , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_121 > , > > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#HEX_DIGIT :: < 'i , 0 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , 0 , super :: super :: generics :: Choice2 :: < super :: super :: rules :: r#HEX_DIGIT :: < 'i , 0 > , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_122 > , > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#HEX_LITERAL<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#IntegerSuffix , "Corresponds to expression: `(kw_u8 | kw_u16 | kw_u32 | kw_u64 | kw_usize | kw_i8 | kw_i16 | kw_i32 | kw_i64 | kw_isize)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#IntegerSuffix , super :: super :: generics :: Choice10 :: < super :: super :: rules :: r#kw_u8 :: < 'i , INHERITED > , super :: super :: rules :: r#kw_u16 :: < 'i , INHERITED > , super :: super :: rules :: r#kw_u32 :: < 'i , INHERITED > , super :: super :: rules :: r#kw_u64 :: < 'i , INHERITED > , super :: super :: rules :: r#kw_usize :: < 'i , INHERITED > , super :: super :: rules :: r#kw_i8 :: < 'i , INHERITED > , super :: super :: rules :: r#kw_i16 :: < 'i , INHERITED > , super :: super :: rules :: r#kw_i32 :: < 'i , INHERITED > , super :: super :: rules :: r#kw_i64 :: < 'i , INHERITED > , super :: super :: rules :: r#kw_isize :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#IntegerSuffix<'i, INHERITED> {
            #[doc = "A helper function to access [`kw_i16`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_i16<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_i16<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._6().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_i32`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_i32<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_i32<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._7().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_i64`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_i64<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_i64<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._8().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_i8`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_i8<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_i8<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._5().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_isize`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_isize<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_isize<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._9().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_u16`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_u16<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_u16<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_u32`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_u32<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_u32<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._2().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_u64`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_u64<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_u64<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._3().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_u8`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_u8<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_u8<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_usize`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_usize<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_usize<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._4().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#PrimitiveType , "Corresponds to expression: `(kw_u8 | kw_u16 | kw_u32 | kw_u64 | kw_usize | kw_i8 | kw_i16 | kw_i32 | kw_i64 | kw_isize | kw_bool | kw_str)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#PrimitiveType , super :: super :: generics :: Choice12 :: < super :: super :: rules :: r#kw_u8 :: < 'i , INHERITED > , super :: super :: rules :: r#kw_u16 :: < 'i , INHERITED > , super :: super :: rules :: r#kw_u32 :: < 'i , INHERITED > , super :: super :: rules :: r#kw_u64 :: < 'i , INHERITED > , super :: super :: rules :: r#kw_usize :: < 'i , INHERITED > , super :: super :: rules :: r#kw_i8 :: < 'i , INHERITED > , super :: super :: rules :: r#kw_i16 :: < 'i , INHERITED > , super :: super :: rules :: r#kw_i32 :: < 'i , INHERITED > , super :: super :: rules :: r#kw_i64 :: < 'i , INHERITED > , super :: super :: rules :: r#kw_isize :: < 'i , INHERITED > , super :: super :: rules :: r#kw_bool :: < 'i , INHERITED > , super :: super :: rules :: r#kw_str :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#PrimitiveType<'i, INHERITED> {
            #[doc = "A helper function to access [`kw_bool`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_bool<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_bool<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._10().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_i16`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_i16<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_i16<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._6().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_i32`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_i32<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_i32<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._7().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_i64`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_i64<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_i64<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._8().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_i8`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_i8<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_i8<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._5().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_isize`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_isize<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_isize<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._9().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_str`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_str<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_str<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._11().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_u16`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_u16<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_u16<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_u32`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_u32<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_u32<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._2().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_u64`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_u64<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_u64<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._3().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_u8`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_u8<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_u8<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_usize`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_usize<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_usize<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._4().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#Integer , "Corresponds to expression: `((DEC_LITERAL | BIN_LITERAL | OCT_LITERAL | HEX_LITERAL) ~ IntegerSuffix?)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Integer , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Choice4 :: < super :: super :: rules :: r#DEC_LITERAL :: < 'i , INHERITED > , super :: super :: rules :: r#BIN_LITERAL :: < 'i , INHERITED > , super :: super :: rules :: r#OCT_LITERAL :: < 'i , INHERITED > , super :: super :: rules :: r#HEX_LITERAL :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#IntegerSuffix :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Integer<'i, INHERITED> {
            #[doc = "A helper function to access [`BIN_LITERAL`]."]
            #[allow(non_snake_case)]
            pub fn r#BIN_LITERAL<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#BIN_LITERAL<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    {
                        let res = res._1().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`DEC_LITERAL`]."]
            #[allow(non_snake_case)]
            pub fn r#DEC_LITERAL<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#DEC_LITERAL<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    {
                        let res = res._0().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`HEX_LITERAL`]."]
            #[allow(non_snake_case)]
            pub fn r#HEX_LITERAL<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#HEX_LITERAL<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    {
                        let res = res._3().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`IntegerSuffix`]."]
            #[allow(non_snake_case)]
            pub fn r#IntegerSuffix<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#IntegerSuffix<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`OCT_LITERAL`]."]
            #[allow(non_snake_case)]
            pub fn r#OCT_LITERAL<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#OCT_LITERAL<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    {
                        let res = res._2().map(|res| res);
                        res
                    }
                }
            }
        }
        :: pest_typed :: rule ! (r#String , "Corresponds to expression: `(\"\\\"\" ~ (!(\"\\\"\") ~ ANY)* ~ \"\\\"\")`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#String , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_123 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Skip :: < super :: super :: constant_wrappers :: r#w_124 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_125 > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#String<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#Bool , "Corresponds to expression: `(kw_true | kw_false)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Bool , super :: super :: generics :: Choice2 :: < super :: super :: rules :: r#kw_true :: < 'i , INHERITED > , super :: super :: rules :: r#kw_false :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Bool<'i, INHERITED> {
            #[doc = "A helper function to access [`kw_false`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_false<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_false<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_true`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_true<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_true<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#WordLeading , "Corresponds to expression: `(('a'..'z') | ('A'..'Z') | ('一'..'龥') | ('_'..'_'))`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#WordLeading , super :: super :: generics :: Choice4 :: < super :: super :: generics :: CharRange :: < 'a' , 'z' > , super :: super :: generics :: CharRange :: < 'A' , 'Z' > , super :: super :: generics :: CharRange :: < '一' , '龥' > , super :: super :: generics :: CharRange :: < '_' , '_' > , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#WordLeading<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#WordFollowing , "Corresponds to expression: `(WordLeading | \"_\" | \"-\" | ('0'..'9'))`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#WordFollowing , super :: super :: generics :: Choice4 :: < super :: super :: rules :: r#WordLeading :: < 'i , 0 > , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_126 > , super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_127 > , super :: super :: generics :: CharRange :: < '0' , '9' > , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#WordFollowing<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#Word , "Corresponds to expression: `(WordLeading ~ WordFollowing*)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Word , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#WordLeading :: < 'i , 0 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , 0 , super :: super :: rules :: r#WordFollowing :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Word<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#Identifier , "Corresponds to expression: `(!Keywords ~ Word)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Identifier , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#Keywords :: < 'i , 0 > > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Word :: < 'i , 0 > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Identifier<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#LabelName , "Corresponds to expression: `Word`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#LabelName , super :: super :: rules :: r#Word :: < 'i , 0 > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#LabelName<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#MetaVariable , "Corresponds to expression: `(Dollar ~ Word)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MetaVariable , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Dollar :: < 'i , 0 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Word :: < 'i , 0 > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MetaVariable<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#MetaVariableType , "Corresponds to expression: `(kw_type | (kw_const ~ LeftParen ~ Type ~ RightParen) | (kw_place ~ LeftParen ~ Type ~ RightParen))`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MetaVariableType , super :: super :: generics :: Choice3 :: < super :: super :: rules :: r#kw_type :: < 'i , INHERITED > , super :: super :: generics :: Seq4 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_const :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Type :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Seq4 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_place :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Type :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MetaVariableType<'i, INHERITED> {
            #[doc = "A helper function to access [`LeftParen`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftParen<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#LeftParen<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#LeftParen<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = res._1().map(|res| {
                                let res = &res.content.1.matched;
                                res
                            });
                            res
                        },
                        {
                            let res = res._2().map(|res| {
                                let res = &res.content.1.matched;
                                res
                            });
                            res
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`RightParen`]."]
            #[allow(non_snake_case)]
            pub fn r#RightParen<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#RightParen<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#RightParen<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = res._1().map(|res| {
                                let res = &res.content.3.matched;
                                res
                            });
                            res
                        },
                        {
                            let res = res._2().map(|res| {
                                let res = &res.content.3.matched;
                                res
                            });
                            res
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`Type`]."]
            #[allow(non_snake_case)]
            pub fn r#Type<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#Type<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#Type<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = res._1().map(|res| {
                                let res = &res.content.2.matched;
                                res
                            });
                            res
                        },
                        {
                            let res = res._2().map(|res| {
                                let res = &res.content.2.matched;
                                res
                            });
                            res
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`kw_const`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_const<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_const<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| {
                        let res = &res.content.0.matched;
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`kw_place`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_place<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_place<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._2().map(|res| {
                        let res = &res.content.0.matched;
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`kw_type`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_type<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_type<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#Predicate , "Corresponds to expression: `(Identifier ~ LeftParen ~ PredicateArgs? ~ RightParen)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Predicate , super :: super :: generics :: Seq4 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Identifier :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#PredicateArgs :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Predicate<'i, INHERITED> {
            #[doc = "A helper function to access [`Identifier`]."]
            #[allow(non_snake_case)]
            pub fn r#Identifier<'s>(&'s self) -> &'s super::super::rules::r#Identifier<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`LeftParen`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftParen<'s>(&'s self) -> &'s super::super::rules::r#LeftParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`PredicateArgs`]."]
            #[allow(non_snake_case)]
            pub fn r#PredicateArgs<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#PredicateArgs<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`RightParen`]."]
            #[allow(non_snake_case)]
            pub fn r#RightParen<'s>(&'s self) -> &'s super::super::rules::r#RightParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#PredicateTerm , "Corresponds to expression: `(Predicate | (Bang ~ Predicate))`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#PredicateTerm , super :: super :: generics :: Choice2 :: < super :: super :: rules :: r#Predicate :: < 'i , INHERITED > , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Bang :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Predicate :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#PredicateTerm<'i, INHERITED> {
            #[doc = "A helper function to access [`Bang`]."]
            #[allow(non_snake_case)]
            pub fn r#Bang<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Bang<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| {
                        let res = &res.content.0.matched;
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`Predicate`]."]
            #[allow(non_snake_case)]
            pub fn r#Predicate<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#Predicate<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#Predicate<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = res._0().map(|res| res);
                            res
                        },
                        {
                            let res = res._1().map(|res| {
                                let res = &res.content.1.matched;
                                res
                            });
                            res
                        },
                    );
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#PredicateArgs , "Corresponds to expression: `(PredicateArg ~ (Comma ~ PredicateArg)* ~ Comma?)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#PredicateArgs , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#PredicateArg :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Comma :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#PredicateArg :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#Comma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#PredicateArgs<'i, INHERITED> {
            #[doc = "A helper function to access [`Comma`]."]
            #[allow(non_snake_case)]
            pub fn r#Comma<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#Comma<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#Comma<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.0.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                        {
                            let res = &res.content.2.matched;
                            {
                                let res = res.as_ref().map(|res| res);
                                res
                            }
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`PredicateArg`]."]
            #[allow(non_snake_case)]
            pub fn r#PredicateArg<'s>(
                &'s self,
            ) -> (
                &'s super::super::rules::r#PredicateArg<'i, INHERITED>,
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#PredicateArg<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.0.matched;
                            res
                        },
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.1.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                    );
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#PredicateArg , "Corresponds to expression: `(Label | MetaVariable | Type | kw_self)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#PredicateArg , super :: super :: generics :: Choice4 :: < super :: super :: rules :: r#Label :: < 'i , INHERITED > , super :: super :: rules :: r#MetaVariable :: < 'i , INHERITED > , super :: super :: rules :: r#Type :: < 'i , INHERITED > , super :: super :: rules :: r#kw_self :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#PredicateArg<'i, INHERITED> {
            #[doc = "A helper function to access [`Label`]."]
            #[allow(non_snake_case)]
            pub fn r#Label<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Label<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MetaVariable`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariable<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MetaVariable<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`Type`]."]
            #[allow(non_snake_case)]
            pub fn r#Type<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Type<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._2().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_self`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_self<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_self<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._3().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#PredicateClause , "Corresponds to expression: `(PredicateTerm | (LeftParen ~ PredicateTerm ~ (OrOr ~ PredicateTerm)* ~ RightParen))`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#PredicateClause , super :: super :: generics :: Choice2 :: < super :: super :: rules :: r#PredicateTerm :: < 'i , INHERITED > , super :: super :: generics :: Seq4 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#PredicateTerm :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#OrOr :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#PredicateTerm :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#PredicateClause<'i, INHERITED> {
            #[doc = "A helper function to access [`LeftParen`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftParen<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#LeftParen<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| {
                        let res = &res.content.0.matched;
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`OrOr`]."]
            #[allow(non_snake_case)]
            pub fn r#OrOr<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#OrOr<'i, INHERITED>>,
            > {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| {
                        let res = &res.content.2.matched;
                        {
                            let res = res
                                .content
                                .iter()
                                .map(|res| {
                                    let res = &res.matched;
                                    {
                                        let res = &res.content.0.matched;
                                        res
                                    }
                                })
                                .collect::<::pest_typed::re_exported::Vec<_>>();
                            res
                        }
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`PredicateTerm`]."]
            #[allow(non_snake_case)]
            pub fn r#PredicateTerm<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#PredicateTerm<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<(
                    &'s super::super::rules::r#PredicateTerm<'i, INHERITED>,
                    ::pest_typed::re_exported::Vec<&'s super::super::rules::r#PredicateTerm<'i, INHERITED>>,
                )>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = res._0().map(|res| res);
                            res
                        },
                        {
                            let res = res._1().map(|res| {
                                let res = (
                                    {
                                        let res = &res.content.1.matched;
                                        res
                                    },
                                    {
                                        let res = &res.content.2.matched;
                                        {
                                            let res = res
                                                .content
                                                .iter()
                                                .map(|res| {
                                                    let res = &res.matched;
                                                    {
                                                        let res = &res.content.1.matched;
                                                        res
                                                    }
                                                })
                                                .collect::<::pest_typed::re_exported::Vec<_>>();
                                            res
                                        }
                                    },
                                );
                                res
                            });
                            res
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`RightParen`]."]
            #[allow(non_snake_case)]
            pub fn r#RightParen<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#RightParen<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| {
                        let res = &res.content.3.matched;
                        res
                    });
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#PredicateConjunction , "Corresponds to expression: `(PredicateClause ~ (AndAnd ~ PredicateClause)*)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#PredicateConjunction , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#PredicateClause :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#AndAnd :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#PredicateClause :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#PredicateConjunction<'i, INHERITED> {
            #[doc = "A helper function to access [`AndAnd`]."]
            #[allow(non_snake_case)]
            pub fn r#AndAnd<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Vec<&'s super::super::rules::r#AndAnd<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res
                            .content
                            .iter()
                            .map(|res| {
                                let res = &res.matched;
                                {
                                    let res = &res.content.0.matched;
                                    res
                                }
                            })
                            .collect::<::pest_typed::re_exported::Vec<_>>();
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`PredicateClause`]."]
            #[allow(non_snake_case)]
            pub fn r#PredicateClause<'s>(
                &'s self,
            ) -> (
                &'s super::super::rules::r#PredicateClause<'i, INHERITED>,
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#PredicateClause<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.0.matched;
                            res
                        },
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.1.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                    );
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MetaVariableDecl , "Corresponds to expression: `(MetaVariable ~ Colon ~ MetaVariableType ~ (kw_where ~ PredicateConjunction)?)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MetaVariableDecl , super :: super :: generics :: Seq4 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MetaVariable :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Colon :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MetaVariableType :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_where :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#PredicateConjunction :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MetaVariableDecl<'i, INHERITED> {
            #[doc = "A helper function to access [`Colon`]."]
            #[allow(non_snake_case)]
            pub fn r#Colon<'s>(&'s self) -> &'s super::super::rules::r#Colon<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MetaVariable`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariable<'s>(&'s self) -> &'s super::super::rules::r#MetaVariable<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MetaVariableType`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariableType<'s>(&'s self) -> &'s super::super::rules::r#MetaVariableType<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`PredicateConjunction`]."]
            #[allow(non_snake_case)]
            pub fn r#PredicateConjunction<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#PredicateConjunction<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    {
                        let res = res.as_ref().map(|res| {
                            let res = &res.content.1.matched;
                            res
                        });
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`kw_where`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_where<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_where<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    {
                        let res = res.as_ref().map(|res| {
                            let res = &res.content.0.matched;
                            res
                        });
                        res
                    }
                }
            }
        }
        :: pest_typed :: rule ! (r#MetaVariableDeclsSeparatedByComma , "Corresponds to expression: `(MetaVariableDecl ~ (Comma ~ MetaVariableDecl)* ~ Comma?)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MetaVariableDeclsSeparatedByComma , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MetaVariableDecl :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Comma :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MetaVariableDecl :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#Comma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MetaVariableDeclsSeparatedByComma<'i, INHERITED> {
            #[doc = "A helper function to access [`Comma`]."]
            #[allow(non_snake_case)]
            pub fn r#Comma<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#Comma<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#Comma<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.0.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                        {
                            let res = &res.content.2.matched;
                            {
                                let res = res.as_ref().map(|res| res);
                                res
                            }
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`MetaVariableDecl`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariableDecl<'s>(
                &'s self,
            ) -> (
                &'s super::super::rules::r#MetaVariableDecl<'i, INHERITED>,
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#MetaVariableDecl<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.0.matched;
                            res
                        },
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.1.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                    );
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MetaVariableDeclList , "Corresponds to expression: `(LeftBracket ~ MetaVariableDeclsSeparatedByComma? ~ RightBracket)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MetaVariableDeclList , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftBracket :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#MetaVariableDeclsSeparatedByComma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightBracket :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MetaVariableDeclList<'i, INHERITED> {
            #[doc = "A helper function to access [`LeftBracket`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftBracket<'s>(&'s self) -> &'s super::super::rules::r#LeftBracket<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MetaVariableDeclsSeparatedByComma`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariableDeclsSeparatedByComma<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<
                &'s super::super::rules::r#MetaVariableDeclsSeparatedByComma<'i, INHERITED>,
            > {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`RightBracket`]."]
            #[allow(non_snake_case)]
            pub fn r#RightBracket<'s>(&'s self) -> &'s super::super::rules::r#RightBracket<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MetaVariableAssign , "Corresponds to expression: `((MetaVariable ~ Assign ~ (Identifier | MetaVariable | Type)) | (Label ~ Assign ~ Label))`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MetaVariableAssign , super :: super :: generics :: Choice2 :: < super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MetaVariable :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Assign :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Choice3 :: < super :: super :: rules :: r#Identifier :: < 'i , INHERITED > , super :: super :: rules :: r#MetaVariable :: < 'i , INHERITED > , super :: super :: rules :: r#Type :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Label :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Assign :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Label :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MetaVariableAssign<'i, INHERITED> {
            #[doc = "A helper function to access [`Assign`]."]
            #[allow(non_snake_case)]
            pub fn r#Assign<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#Assign<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#Assign<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = res._0().map(|res| {
                                let res = &res.content.1.matched;
                                res
                            });
                            res
                        },
                        {
                            let res = res._1().map(|res| {
                                let res = &res.content.1.matched;
                                res
                            });
                            res
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`Identifier`]."]
            #[allow(non_snake_case)]
            pub fn r#Identifier<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Identifier<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res
                        ._0()
                        .map(|res| {
                            let res = &res.content.2.matched;
                            {
                                let res = res._0().map(|res| res);
                                res
                            }
                        })
                        .flatten();
                    res
                }
            }
            #[doc = "A helper function to access [`Label`]."]
            #[allow(non_snake_case)]
            pub fn r#Label<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<(
                &'s super::super::rules::r#Label<'i, INHERITED>,
                &'s super::super::rules::r#Label<'i, INHERITED>,
            )> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| {
                        let res = (
                            {
                                let res = &res.content.0.matched;
                                res
                            },
                            {
                                let res = &res.content.2.matched;
                                res
                            },
                        );
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`MetaVariable`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariable<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<(
                &'s super::super::rules::r#MetaVariable<'i, INHERITED>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#MetaVariable<'i, INHERITED>>,
            )> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| {
                        let res = (
                            {
                                let res = &res.content.0.matched;
                                res
                            },
                            {
                                let res = &res.content.2.matched;
                                {
                                    let res = res._1().map(|res| res);
                                    res
                                }
                            },
                        );
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`Type`]."]
            #[allow(non_snake_case)]
            pub fn r#Type<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Type<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res
                        ._0()
                        .map(|res| {
                            let res = &res.content.2.matched;
                            {
                                let res = res._2().map(|res| res);
                                res
                            }
                        })
                        .flatten();
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MetaVariableAssignsSeparatedByComma , "Corresponds to expression: `(MetaVariableAssign ~ (Comma ~ MetaVariableAssign)* ~ Comma?)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MetaVariableAssignsSeparatedByComma , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MetaVariableAssign :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Comma :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MetaVariableAssign :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#Comma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MetaVariableAssignsSeparatedByComma<'i, INHERITED> {
            #[doc = "A helper function to access [`Comma`]."]
            #[allow(non_snake_case)]
            pub fn r#Comma<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#Comma<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#Comma<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.0.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                        {
                            let res = &res.content.2.matched;
                            {
                                let res = res.as_ref().map(|res| res);
                                res
                            }
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`MetaVariableAssign`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariableAssign<'s>(
                &'s self,
            ) -> (
                &'s super::super::rules::r#MetaVariableAssign<'i, INHERITED>,
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#MetaVariableAssign<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.0.matched;
                            res
                        },
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.1.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                    );
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MetaVariableAssignList , "Corresponds to expression: `(LeftBracket ~ MetaVariableAssignsSeparatedByComma? ~ RightBracket)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MetaVariableAssignList , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftBracket :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#MetaVariableAssignsSeparatedByComma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightBracket :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MetaVariableAssignList<'i, INHERITED> {
            #[doc = "A helper function to access [`LeftBracket`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftBracket<'s>(&'s self) -> &'s super::super::rules::r#LeftBracket<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MetaVariableAssignsSeparatedByComma`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariableAssignsSeparatedByComma<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<
                &'s super::super::rules::r#MetaVariableAssignsSeparatedByComma<'i, INHERITED>,
            > {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`RightBracket`]."]
            #[allow(non_snake_case)]
            pub fn r#RightBracket<'s>(&'s self) -> &'s super::super::rules::r#RightBracket<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#Attribute , "Corresponds to expression: `((Word ~ Assign ~ Word) | PredicateConjunction)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Attribute , super :: super :: generics :: Choice2 :: < super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Word :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Assign :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Word :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: rules :: r#PredicateConjunction :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Attribute<'i, INHERITED> {
            #[doc = "A helper function to access [`Assign`]."]
            #[allow(non_snake_case)]
            pub fn r#Assign<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Assign<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| {
                        let res = &res.content.1.matched;
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`PredicateConjunction`]."]
            #[allow(non_snake_case)]
            pub fn r#PredicateConjunction<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#PredicateConjunction<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`Word`]."]
            #[allow(non_snake_case)]
            pub fn r#Word<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<(
                &'s super::super::rules::r#Word<'i, INHERITED>,
                &'s super::super::rules::r#Word<'i, INHERITED>,
            )> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| {
                        let res = (
                            {
                                let res = &res.content.0.matched;
                                res
                            },
                            {
                                let res = &res.content.2.matched;
                                res
                            },
                        );
                        res
                    });
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#AttributesSeparatedByComma , "Corresponds to expression: `(Attribute ~ (Comma ~ Attribute)* ~ Comma?)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#AttributesSeparatedByComma , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Attribute :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Comma :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Attribute :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#Comma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#AttributesSeparatedByComma<'i, INHERITED> {
            #[doc = "A helper function to access [`Attribute`]."]
            #[allow(non_snake_case)]
            pub fn r#Attribute<'s>(
                &'s self,
            ) -> (
                &'s super::super::rules::r#Attribute<'i, INHERITED>,
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#Attribute<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.0.matched;
                            res
                        },
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.1.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`Comma`]."]
            #[allow(non_snake_case)]
            pub fn r#Comma<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#Comma<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#Comma<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.0.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                        {
                            let res = &res.content.2.matched;
                            {
                                let res = res.as_ref().map(|res| res);
                                res
                            }
                        },
                    );
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#WhereBlock , "Corresponds to expression: `(kw_where ~ LeftBrace ~ AttributesSeparatedByComma? ~ RightBrace)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#WhereBlock , super :: super :: generics :: Seq4 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_where :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#AttributesSeparatedByComma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#WhereBlock<'i, INHERITED> {
            #[doc = "A helper function to access [`AttributesSeparatedByComma`]."]
            #[allow(non_snake_case)]
            pub fn r#AttributesSeparatedByComma<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#AttributesSeparatedByComma<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`LeftBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftBrace<'s>(&'s self) -> &'s super::super::rules::r#LeftBrace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RightBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#RightBrace<'s>(&'s self) -> &'s super::super::rules::r#RightBrace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_where`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_where<'s>(&'s self) -> &'s super::super::rules::r#kw_where<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#Mutability , "Corresponds to expression: `kw_mut?`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Mutability , :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#kw_mut :: < 'i , INHERITED > > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Mutability<'i, INHERITED> {
            #[doc = "A helper function to access [`kw_mut`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_mut<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_mut<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res.as_ref().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#PtrMutability , "Corresponds to expression: `(kw_mut | kw_const)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#PtrMutability , super :: super :: generics :: Choice2 :: < super :: super :: rules :: r#kw_mut :: < 'i , INHERITED > , super :: super :: rules :: r#kw_const :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#PtrMutability<'i, INHERITED> {
            #[doc = "A helper function to access [`kw_const`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_const<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_const<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_mut`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_mut<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_mut<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#Region , "Corresponds to expression: `(Quote ~ (PlaceHolder | kw_static))`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Region , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Quote :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Choice2 :: < super :: super :: rules :: r#PlaceHolder :: < 'i , INHERITED > , super :: super :: rules :: r#kw_static :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Region<'i, INHERITED> {
            #[doc = "A helper function to access [`PlaceHolder`]."]
            #[allow(non_snake_case)]
            pub fn r#PlaceHolder<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#PlaceHolder<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res._0().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`Quote`]."]
            #[allow(non_snake_case)]
            pub fn r#Quote<'s>(&'s self) -> &'s super::super::rules::r#Quote<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_static`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_static<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_static<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res._1().map(|res| res);
                        res
                    }
                }
            }
        }
        :: pest_typed :: rule ! (r#QSelf , "Corresponds to expression: `(LessThan ~ Type ~ kw_as ~ (Identifier | MetaVariable) ~ GreaterThan)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#QSelf , super :: super :: generics :: Seq5 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LessThan :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Type :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_as :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Choice2 :: < super :: super :: rules :: r#Identifier :: < 'i , INHERITED > , super :: super :: rules :: r#MetaVariable :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#GreaterThan :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#QSelf<'i, INHERITED> {
            #[doc = "A helper function to access [`GreaterThan`]."]
            #[allow(non_snake_case)]
            pub fn r#GreaterThan<'s>(&'s self) -> &'s super::super::rules::r#GreaterThan<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.4.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Identifier`]."]
            #[allow(non_snake_case)]
            pub fn r#Identifier<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Identifier<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    {
                        let res = res._0().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`LessThan`]."]
            #[allow(non_snake_case)]
            pub fn r#LessThan<'s>(&'s self) -> &'s super::super::rules::r#LessThan<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MetaVariable`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariable<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MetaVariable<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    {
                        let res = res._1().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`Type`]."]
            #[allow(non_snake_case)]
            pub fn r#Type<'s>(&'s self) -> &'s super::super::rules::r#Type<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_as`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_as<'s>(&'s self) -> &'s super::super::rules::r#kw_as<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#Path , "Corresponds to expression: `(PathLeading? ~ PathSegment ~ (Colon2 ~ PathSegment)*)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Path , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#PathLeading :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#PathSegment :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Colon2 :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#PathSegment :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Path<'i, INHERITED> {
            #[doc = "A helper function to access [`Colon2`]."]
            #[allow(non_snake_case)]
            pub fn r#Colon2<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Vec<&'s super::super::rules::r#Colon2<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    {
                        let res = res
                            .content
                            .iter()
                            .map(|res| {
                                let res = &res.matched;
                                {
                                    let res = &res.content.0.matched;
                                    res
                                }
                            })
                            .collect::<::pest_typed::re_exported::Vec<_>>();
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`PathLeading`]."]
            #[allow(non_snake_case)]
            pub fn r#PathLeading<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#PathLeading<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`PathSegment`]."]
            #[allow(non_snake_case)]
            pub fn r#PathSegment<'s>(
                &'s self,
            ) -> (
                &'s super::super::rules::r#PathSegment<'i, INHERITED>,
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#PathSegment<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.1.matched;
                            res
                        },
                        {
                            let res = &res.content.2.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.1.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                    );
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#PathArguments , "Corresponds to expression: `AngleBracketedGenericArguments`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#PathArguments , super :: super :: rules :: r#AngleBracketedGenericArguments :: < 'i , INHERITED > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#PathArguments<'i, INHERITED> {
            #[doc = "A helper function to access [`AngleBracketedGenericArguments`]."]
            #[allow(non_snake_case)]
            pub fn r#AngleBracketedGenericArguments<'s>(
                &'s self,
            ) -> &'s super::super::rules::r#AngleBracketedGenericArguments<'i, INHERITED> {
                let res = &*self.content;
                res
            }
        }
        :: pest_typed :: rule ! (r#Pathcrate , "Corresponds to expression: `(Dollar ~ kw_crate)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Pathcrate , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Dollar :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_crate :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Pathcrate<'i, INHERITED> {
            #[doc = "A helper function to access [`Dollar`]."]
            #[allow(non_snake_case)]
            pub fn r#Dollar<'s>(&'s self) -> &'s super::super::rules::r#Dollar<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_crate`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_crate<'s>(&'s self) -> &'s super::super::rules::r#kw_crate<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#PathLeading , "Corresponds to expression: `(Pathcrate? ~ Colon2)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#PathLeading , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#Pathcrate :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Colon2 :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#PathLeading<'i, INHERITED> {
            #[doc = "A helper function to access [`Colon2`]."]
            #[allow(non_snake_case)]
            pub fn r#Colon2<'s>(&'s self) -> &'s super::super::rules::r#Colon2<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Pathcrate`]."]
            #[allow(non_snake_case)]
            pub fn r#Pathcrate<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Pathcrate<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
        }
        :: pest_typed :: rule ! (r#PathSegment , "Corresponds to expression: `(Identifier ~ PathArguments?)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#PathSegment , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Identifier :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#PathArguments :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#PathSegment<'i, INHERITED> {
            #[doc = "A helper function to access [`Identifier`]."]
            #[allow(non_snake_case)]
            pub fn r#Identifier<'s>(&'s self) -> &'s super::super::rules::r#Identifier<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`PathArguments`]."]
            #[allow(non_snake_case)]
            pub fn r#PathArguments<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#PathArguments<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
        }
        :: pest_typed :: rule ! (r#TypePath , "Corresponds to expression: `(QSelf? ~ Path)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#TypePath , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#QSelf :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Path :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#TypePath<'i, INHERITED> {
            #[doc = "A helper function to access [`Path`]."]
            #[allow(non_snake_case)]
            pub fn r#Path<'s>(&'s self) -> &'s super::super::rules::r#Path<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`QSelf`]."]
            #[allow(non_snake_case)]
            pub fn r#QSelf<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#QSelf<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
        }
        :: pest_typed :: rule ! (r#Konst , "Corresponds to expression: `(Literal | TypePath)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Konst , super :: super :: generics :: Choice2 :: < super :: super :: rules :: r#Literal :: < 'i , INHERITED > , super :: super :: rules :: r#TypePath :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Konst<'i, INHERITED> {
            #[doc = "A helper function to access [`Literal`]."]
            #[allow(non_snake_case)]
            pub fn r#Literal<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Literal<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`TypePath`]."]
            #[allow(non_snake_case)]
            pub fn r#TypePath<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#TypePath<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#GenericConst , "Corresponds to expression: `((LeftBrace ~ Konst ~ RightBrace) | Konst)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#GenericConst , super :: super :: generics :: Choice2 :: < super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Konst :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: rules :: r#Konst :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#GenericConst<'i, INHERITED> {
            #[doc = "A helper function to access [`Konst`]."]
            #[allow(non_snake_case)]
            pub fn r#Konst<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#Konst<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#Konst<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = res._0().map(|res| {
                                let res = &res.content.1.matched;
                                res
                            });
                            res
                        },
                        {
                            let res = res._1().map(|res| res);
                            res
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`LeftBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftBrace<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#LeftBrace<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| {
                        let res = &res.content.0.matched;
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`RightBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#RightBrace<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#RightBrace<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| {
                        let res = &res.content.2.matched;
                        res
                    });
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#GenericArgument , "Corresponds to expression: `(Region | Type | GenericConst)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#GenericArgument , super :: super :: generics :: Choice3 :: < super :: super :: rules :: r#Region :: < 'i , INHERITED > , super :: super :: rules :: r#Type :: < 'i , INHERITED > , super :: super :: rules :: r#GenericConst :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#GenericArgument<'i, INHERITED> {
            #[doc = "A helper function to access [`GenericConst`]."]
            #[allow(non_snake_case)]
            pub fn r#GenericConst<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#GenericConst<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._2().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`Region`]."]
            #[allow(non_snake_case)]
            pub fn r#Region<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Region<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`Type`]."]
            #[allow(non_snake_case)]
            pub fn r#Type<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Type<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#GenericArgumentsSepretatedByComma , "Corresponds to expression: `(GenericArgument ~ (Comma ~ GenericArgument)* ~ Comma?)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#GenericArgumentsSepretatedByComma , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#GenericArgument :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Comma :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#GenericArgument :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#Comma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#GenericArgumentsSepretatedByComma<'i, INHERITED> {
            #[doc = "A helper function to access [`Comma`]."]
            #[allow(non_snake_case)]
            pub fn r#Comma<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#Comma<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#Comma<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.0.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                        {
                            let res = &res.content.2.matched;
                            {
                                let res = res.as_ref().map(|res| res);
                                res
                            }
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`GenericArgument`]."]
            #[allow(non_snake_case)]
            pub fn r#GenericArgument<'s>(
                &'s self,
            ) -> (
                &'s super::super::rules::r#GenericArgument<'i, INHERITED>,
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#GenericArgument<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.0.matched;
                            res
                        },
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.1.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                    );
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#AngleBracketedGenericArguments , "Corresponds to expression: `(Colon2? ~ LessThan ~ GenericArgumentsSepretatedByComma ~ GreaterThan)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#AngleBracketedGenericArguments , super :: super :: generics :: Seq4 :: < (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#Colon2 :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LessThan :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#GenericArgumentsSepretatedByComma :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#GreaterThan :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#AngleBracketedGenericArguments<'i, INHERITED> {
            #[doc = "A helper function to access [`Colon2`]."]
            #[allow(non_snake_case)]
            pub fn r#Colon2<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Colon2<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`GenericArgumentsSepretatedByComma`]."]
            #[allow(non_snake_case)]
            pub fn r#GenericArgumentsSepretatedByComma<'s>(
                &'s self,
            ) -> &'s super::super::rules::r#GenericArgumentsSepretatedByComma<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`GreaterThan`]."]
            #[allow(non_snake_case)]
            pub fn r#GreaterThan<'s>(&'s self) -> &'s super::super::rules::r#GreaterThan<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`LessThan`]."]
            #[allow(non_snake_case)]
            pub fn r#LessThan<'s>(&'s self) -> &'s super::super::rules::r#LessThan<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#LangItemWithArgs , "Corresponds to expression: `(Hash ~ LeftBracket ~ kw_lang ~ Assign ~ String ~ RightBracket ~ AngleBracketedGenericArguments?)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#LangItemWithArgs , super :: super :: generics :: Seq7 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Hash :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftBracket :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_lang :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Assign :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#String :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightBracket :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#AngleBracketedGenericArguments :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#LangItemWithArgs<'i, INHERITED> {
            #[doc = "A helper function to access [`AngleBracketedGenericArguments`]."]
            #[allow(non_snake_case)]
            pub fn r#AngleBracketedGenericArguments<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<
                &'s super::super::rules::r#AngleBracketedGenericArguments<'i, INHERITED>,
            > {
                let res = &*self.content;
                {
                    let res = &res.content.6.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`Assign`]."]
            #[allow(non_snake_case)]
            pub fn r#Assign<'s>(&'s self) -> &'s super::super::rules::r#Assign<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Hash`]."]
            #[allow(non_snake_case)]
            pub fn r#Hash<'s>(&'s self) -> &'s super::super::rules::r#Hash<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`LeftBracket`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftBracket<'s>(&'s self) -> &'s super::super::rules::r#LeftBracket<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RightBracket`]."]
            #[allow(non_snake_case)]
            pub fn r#RightBracket<'s>(&'s self) -> &'s super::super::rules::r#RightBracket<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.5.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`String`]."]
            #[allow(non_snake_case)]
            pub fn r#String<'s>(&'s self) -> &'s super::super::rules::r#String<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.4.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_lang`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_lang<'s>(&'s self) -> &'s super::super::rules::r#kw_lang<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#TypesSeparatedByComma , "Corresponds to expression: `(Type ~ (Comma ~ Type)* ~ Comma?)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#TypesSeparatedByComma , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Type :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Comma :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Type :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#Comma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#TypesSeparatedByComma<'i, INHERITED> {
            #[doc = "A helper function to access [`Comma`]."]
            #[allow(non_snake_case)]
            pub fn r#Comma<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#Comma<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#Comma<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.0.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                        {
                            let res = &res.content.2.matched;
                            {
                                let res = res.as_ref().map(|res| res);
                                res
                            }
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`Type`]."]
            #[allow(non_snake_case)]
            pub fn r#Type<'s>(
                &'s self,
            ) -> (
                &'s super::super::rules::r#Type<'i, INHERITED>,
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#Type<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.0.matched;
                            res
                        },
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.1.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                    );
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#TypeArray , "Corresponds to expression: `(LeftBracket ~ Type ~ SemiColon ~ (Integer | MetaVariable) ~ RightBracket)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#TypeArray , super :: super :: generics :: Seq5 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftBracket :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Type :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#SemiColon :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Choice2 :: < super :: super :: rules :: r#Integer :: < 'i , INHERITED > , super :: super :: rules :: r#MetaVariable :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightBracket :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#TypeArray<'i, INHERITED> {
            #[doc = "A helper function to access [`Integer`]."]
            #[allow(non_snake_case)]
            pub fn r#Integer<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Integer<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    {
                        let res = res._0().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`LeftBracket`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftBracket<'s>(&'s self) -> &'s super::super::rules::r#LeftBracket<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MetaVariable`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariable<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MetaVariable<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    {
                        let res = res._1().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`RightBracket`]."]
            #[allow(non_snake_case)]
            pub fn r#RightBracket<'s>(&'s self) -> &'s super::super::rules::r#RightBracket<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.4.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`SemiColon`]."]
            #[allow(non_snake_case)]
            pub fn r#SemiColon<'s>(&'s self) -> &'s super::super::rules::r#SemiColon<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Type`]."]
            #[allow(non_snake_case)]
            pub fn r#Type<'s>(&'s self) -> &'s super::super::rules::r#Type<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#TypeGroup , "Corresponds to expression: `(\"Group\" ~ Type)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#TypeGroup , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_128 > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Type :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#TypeGroup<'i, INHERITED> {
            #[doc = "A helper function to access [`Type`]."]
            #[allow(non_snake_case)]
            pub fn r#Type<'s>(&'s self) -> &'s super::super::rules::r#Type<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#TypeNever , "Corresponds to expression: `Bang`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#TypeNever , super :: super :: rules :: r#Bang :: < 'i , INHERITED > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#TypeNever<'i, INHERITED> {
            #[doc = "A helper function to access [`Bang`]."]
            #[allow(non_snake_case)]
            pub fn r#Bang<'s>(&'s self) -> &'s super::super::rules::r#Bang<'i, INHERITED> {
                let res = &*self.content;
                res
            }
        }
        :: pest_typed :: rule ! (r#TypeParen , "Corresponds to expression: `(LeftParen ~ Type ~ RightParen)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#TypeParen , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Type :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#TypeParen<'i, INHERITED> {
            #[doc = "A helper function to access [`LeftParen`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftParen<'s>(&'s self) -> &'s super::super::rules::r#LeftParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RightParen`]."]
            #[allow(non_snake_case)]
            pub fn r#RightParen<'s>(&'s self) -> &'s super::super::rules::r#RightParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Type`]."]
            #[allow(non_snake_case)]
            pub fn r#Type<'s>(&'s self) -> &'s super::super::rules::r#Type<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#TypePtr , "Corresponds to expression: `(Star ~ PtrMutability ~ Type)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#TypePtr , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Star :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#PtrMutability :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Type :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#TypePtr<'i, INHERITED> {
            #[doc = "A helper function to access [`PtrMutability`]."]
            #[allow(non_snake_case)]
            pub fn r#PtrMutability<'s>(&'s self) -> &'s super::super::rules::r#PtrMutability<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Star`]."]
            #[allow(non_snake_case)]
            pub fn r#Star<'s>(&'s self) -> &'s super::super::rules::r#Star<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Type`]."]
            #[allow(non_snake_case)]
            pub fn r#Type<'s>(&'s self) -> &'s super::super::rules::r#Type<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#TypeReference , "Corresponds to expression: `(And ~ Region? ~ Mutability ~ Type)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#TypeReference , super :: super :: generics :: Seq4 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#And :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#Region :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Mutability :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Type :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#TypeReference<'i, INHERITED> {
            #[doc = "A helper function to access [`And`]."]
            #[allow(non_snake_case)]
            pub fn r#And<'s>(&'s self) -> &'s super::super::rules::r#And<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Mutability`]."]
            #[allow(non_snake_case)]
            pub fn r#Mutability<'s>(&'s self) -> &'s super::super::rules::r#Mutability<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Region`]."]
            #[allow(non_snake_case)]
            pub fn r#Region<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Region<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`Type`]."]
            #[allow(non_snake_case)]
            pub fn r#Type<'s>(&'s self) -> &'s super::super::rules::r#Type<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#TypeSlice , "Corresponds to expression: `(LeftBracket ~ Type ~ RightBracket)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#TypeSlice , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftBracket :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Type :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightBracket :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#TypeSlice<'i, INHERITED> {
            #[doc = "A helper function to access [`LeftBracket`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftBracket<'s>(&'s self) -> &'s super::super::rules::r#LeftBracket<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RightBracket`]."]
            #[allow(non_snake_case)]
            pub fn r#RightBracket<'s>(&'s self) -> &'s super::super::rules::r#RightBracket<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Type`]."]
            #[allow(non_snake_case)]
            pub fn r#Type<'s>(&'s self) -> &'s super::super::rules::r#Type<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#TypeTuple , "Corresponds to expression: `(LeftParen ~ TypesSeparatedByComma? ~ RightParen)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#TypeTuple , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#TypesSeparatedByComma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#TypeTuple<'i, INHERITED> {
            #[doc = "A helper function to access [`LeftParen`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftParen<'s>(&'s self) -> &'s super::super::rules::r#LeftParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RightParen`]."]
            #[allow(non_snake_case)]
            pub fn r#RightParen<'s>(&'s self) -> &'s super::super::rules::r#RightParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`TypesSeparatedByComma`]."]
            #[allow(non_snake_case)]
            pub fn r#TypesSeparatedByComma<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#TypesSeparatedByComma<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
        }
        :: pest_typed :: rule ! (r#TypeMetaVariable , "Corresponds to expression: `MetaVariable`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#TypeMetaVariable , super :: super :: rules :: r#MetaVariable :: < 'i , INHERITED > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#TypeMetaVariable<'i, INHERITED> {
            #[doc = "A helper function to access [`MetaVariable`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariable<'s>(&'s self) -> &'s super::super::rules::r#MetaVariable<'i, INHERITED> {
                let res = &*self.content;
                res
            }
        }
        :: pest_typed :: rule ! (r#Type , "Corresponds to expression: `(TypeArray | TypeGroup | TypeNever | TypeParen | TypePtr | TypeReference | TypeSlice | TypeTuple | TypeMetaVariable | kw_Self | PrimitiveType | PlaceHolder | TypePath | LangItemWithArgs)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Type , super :: super :: generics :: Choice14 :: < super :: super :: rules :: r#TypeArray :: < 'i , INHERITED > , super :: super :: rules :: r#TypeGroup :: < 'i , INHERITED > , super :: super :: rules :: r#TypeNever :: < 'i , INHERITED > , super :: super :: rules :: r#TypeParen :: < 'i , INHERITED > , super :: super :: rules :: r#TypePtr :: < 'i , INHERITED > , super :: super :: rules :: r#TypeReference :: < 'i , INHERITED > , super :: super :: rules :: r#TypeSlice :: < 'i , INHERITED > , super :: super :: rules :: r#TypeTuple :: < 'i , INHERITED > , super :: super :: rules :: r#TypeMetaVariable :: < 'i , INHERITED > , super :: super :: rules :: r#kw_Self :: < 'i , INHERITED > , super :: super :: rules :: r#PrimitiveType :: < 'i , INHERITED > , super :: super :: rules :: r#PlaceHolder :: < 'i , INHERITED > , super :: super :: rules :: r#TypePath :: < 'i , INHERITED > , super :: super :: rules :: r#LangItemWithArgs :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Type<'i, INHERITED> {
            #[doc = "A helper function to access [`LangItemWithArgs`]."]
            #[allow(non_snake_case)]
            pub fn r#LangItemWithArgs<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#LangItemWithArgs<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._13().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`PlaceHolder`]."]
            #[allow(non_snake_case)]
            pub fn r#PlaceHolder<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#PlaceHolder<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._11().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`PrimitiveType`]."]
            #[allow(non_snake_case)]
            pub fn r#PrimitiveType<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#PrimitiveType<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._10().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`TypeArray`]."]
            #[allow(non_snake_case)]
            pub fn r#TypeArray<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#TypeArray<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`TypeGroup`]."]
            #[allow(non_snake_case)]
            pub fn r#TypeGroup<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#TypeGroup<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`TypeMetaVariable`]."]
            #[allow(non_snake_case)]
            pub fn r#TypeMetaVariable<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#TypeMetaVariable<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._8().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`TypeNever`]."]
            #[allow(non_snake_case)]
            pub fn r#TypeNever<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#TypeNever<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._2().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`TypeParen`]."]
            #[allow(non_snake_case)]
            pub fn r#TypeParen<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#TypeParen<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._3().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`TypePath`]."]
            #[allow(non_snake_case)]
            pub fn r#TypePath<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#TypePath<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._12().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`TypePtr`]."]
            #[allow(non_snake_case)]
            pub fn r#TypePtr<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#TypePtr<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._4().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`TypeReference`]."]
            #[allow(non_snake_case)]
            pub fn r#TypeReference<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#TypeReference<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._5().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`TypeSlice`]."]
            #[allow(non_snake_case)]
            pub fn r#TypeSlice<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#TypeSlice<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._6().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`TypeTuple`]."]
            #[allow(non_snake_case)]
            pub fn r#TypeTuple<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#TypeTuple<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._7().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_Self`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_Self<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_Self<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._9().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#SelfParam , "Corresponds to expression: `(And? ~ Mutability ~ kw_self)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#SelfParam , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#And :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Mutability :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_self :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#SelfParam<'i, INHERITED> {
            #[doc = "A helper function to access [`And`]."]
            #[allow(non_snake_case)]
            pub fn r#And<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#And<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`Mutability`]."]
            #[allow(non_snake_case)]
            pub fn r#Mutability<'s>(&'s self) -> &'s super::super::rules::r#Mutability<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_self`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_self<'s>(&'s self) -> &'s super::super::rules::r#kw_self<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#PlaceHolderWithType , "Corresponds to expression: `(Mutability ~ PlaceHolder ~ Colon ~ Type)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#PlaceHolderWithType , super :: super :: generics :: Seq4 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Mutability :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#PlaceHolder :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Colon :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Type :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#PlaceHolderWithType<'i, INHERITED> {
            #[doc = "A helper function to access [`Colon`]."]
            #[allow(non_snake_case)]
            pub fn r#Colon<'s>(&'s self) -> &'s super::super::rules::r#Colon<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Mutability`]."]
            #[allow(non_snake_case)]
            pub fn r#Mutability<'s>(&'s self) -> &'s super::super::rules::r#Mutability<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`PlaceHolder`]."]
            #[allow(non_snake_case)]
            pub fn r#PlaceHolder<'s>(&'s self) -> &'s super::super::rules::r#PlaceHolder<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Type`]."]
            #[allow(non_snake_case)]
            pub fn r#Type<'s>(&'s self) -> &'s super::super::rules::r#Type<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#FnParam , "Corresponds to expression: `(SelfParam | NormalParam | PlaceHolderWithType | Dot2)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#FnParam , super :: super :: generics :: Choice4 :: < super :: super :: rules :: r#SelfParam :: < 'i , INHERITED > , super :: super :: rules :: r#NormalParam :: < 'i , INHERITED > , super :: super :: rules :: r#PlaceHolderWithType :: < 'i , INHERITED > , super :: super :: rules :: r#Dot2 :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#FnParam<'i, INHERITED> {
            #[doc = "A helper function to access [`Dot2`]."]
            #[allow(non_snake_case)]
            pub fn r#Dot2<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Dot2<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._3().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`NormalParam`]."]
            #[allow(non_snake_case)]
            pub fn r#NormalParam<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#NormalParam<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`PlaceHolderWithType`]."]
            #[allow(non_snake_case)]
            pub fn r#PlaceHolderWithType<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#PlaceHolderWithType<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._2().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`SelfParam`]."]
            #[allow(non_snake_case)]
            pub fn r#SelfParam<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#SelfParam<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#NormalParam , "Corresponds to expression: `(Mutability ~ MetaVariable ~ Colon ~ Type)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#NormalParam , super :: super :: generics :: Seq4 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Mutability :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MetaVariable :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Colon :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Type :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#NormalParam<'i, INHERITED> {
            #[doc = "A helper function to access [`Colon`]."]
            #[allow(non_snake_case)]
            pub fn r#Colon<'s>(&'s self) -> &'s super::super::rules::r#Colon<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MetaVariable`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariable<'s>(&'s self) -> &'s super::super::rules::r#MetaVariable<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Mutability`]."]
            #[allow(non_snake_case)]
            pub fn r#Mutability<'s>(&'s self) -> &'s super::super::rules::r#Mutability<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Type`]."]
            #[allow(non_snake_case)]
            pub fn r#Type<'s>(&'s self) -> &'s super::super::rules::r#Type<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#Dollarself , "Corresponds to expression: `(Dollar ~ kw_self)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Dollarself , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Dollar :: < 'i , 0 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_self :: < 'i , 0 > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Dollarself<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#DollarRET , "Corresponds to expression: `(Dollar ~ kw_RET)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#DollarRET , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Dollar :: < 'i , 0 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_RET :: < 'i , 0 > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#DollarRET<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#MirPlaceLocal , "Corresponds to expression: `(PlaceHolder | Dollarself | DollarRET | MetaVariable)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirPlaceLocal , super :: super :: generics :: Choice4 :: < super :: super :: rules :: r#PlaceHolder :: < 'i , INHERITED > , super :: super :: rules :: r#Dollarself :: < 'i , INHERITED > , super :: super :: rules :: r#DollarRET :: < 'i , INHERITED > , super :: super :: rules :: r#MetaVariable :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirPlaceLocal<'i, INHERITED> {
            #[doc = "A helper function to access [`DollarRET`]."]
            #[allow(non_snake_case)]
            pub fn r#DollarRET<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#DollarRET<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._2().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`Dollarself`]."]
            #[allow(non_snake_case)]
            pub fn r#Dollarself<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Dollarself<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MetaVariable`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariable<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MetaVariable<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._3().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`PlaceHolder`]."]
            #[allow(non_snake_case)]
            pub fn r#PlaceHolder<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#PlaceHolder<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirPlaceParen , "Corresponds to expression: `(LeftParen ~ MirPlace ~ RightParen)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirPlaceParen , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirPlace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirPlaceParen<'i, INHERITED> {
            #[doc = "A helper function to access [`LeftParen`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftParen<'s>(&'s self) -> &'s super::super::rules::r#LeftParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirPlace`]."]
            #[allow(non_snake_case)]
            pub fn r#MirPlace<'s>(&'s self) -> &'s super::super::rules::r#MirPlace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RightParen`]."]
            #[allow(non_snake_case)]
            pub fn r#RightParen<'s>(&'s self) -> &'s super::super::rules::r#RightParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirPlaceDeref , "Corresponds to expression: `(Star ~ MirPlace)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirPlaceDeref , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Star :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirPlace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirPlaceDeref<'i, INHERITED> {
            #[doc = "A helper function to access [`MirPlace`]."]
            #[allow(non_snake_case)]
            pub fn r#MirPlace<'s>(&'s self) -> &'s super::super::rules::r#MirPlace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Star`]."]
            #[allow(non_snake_case)]
            pub fn r#Star<'s>(&'s self) -> &'s super::super::rules::r#Star<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirPlaceField , "Corresponds to expression: `(Dot ~ (MetaVariable | Identifier | Integer))`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirPlaceField , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Dot :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Choice3 :: < super :: super :: rules :: r#MetaVariable :: < 'i , INHERITED > , super :: super :: rules :: r#Identifier :: < 'i , INHERITED > , super :: super :: rules :: r#Integer :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirPlaceField<'i, INHERITED> {
            #[doc = "A helper function to access [`Dot`]."]
            #[allow(non_snake_case)]
            pub fn r#Dot<'s>(&'s self) -> &'s super::super::rules::r#Dot<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Identifier`]."]
            #[allow(non_snake_case)]
            pub fn r#Identifier<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Identifier<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res._1().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`Integer`]."]
            #[allow(non_snake_case)]
            pub fn r#Integer<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Integer<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res._2().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`MetaVariable`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariable<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MetaVariable<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res._0().map(|res| res);
                        res
                    }
                }
            }
        }
        :: pest_typed :: rule ! (r#MirPlaceIndex , "Corresponds to expression: `(LeftBracket ~ MirPlaceLocal ~ RightBracket)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirPlaceIndex , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftBracket :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirPlaceLocal :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightBracket :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirPlaceIndex<'i, INHERITED> {
            #[doc = "A helper function to access [`LeftBracket`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftBracket<'s>(&'s self) -> &'s super::super::rules::r#LeftBracket<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirPlaceLocal`]."]
            #[allow(non_snake_case)]
            pub fn r#MirPlaceLocal<'s>(&'s self) -> &'s super::super::rules::r#MirPlaceLocal<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RightBracket`]."]
            #[allow(non_snake_case)]
            pub fn r#RightBracket<'s>(&'s self) -> &'s super::super::rules::r#RightBracket<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirPlaceConstIndex , "Corresponds to expression: `(LeftBracket ~ Minus? ~ Integer ~ kw_of ~ Integer ~ RightBracket)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirPlaceConstIndex , super :: super :: generics :: Seq6 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftBracket :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#Minus :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Integer :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_of :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Integer :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightBracket :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirPlaceConstIndex<'i, INHERITED> {
            #[doc = "A helper function to access [`Integer`]."]
            #[allow(non_snake_case)]
            pub fn r#Integer<'s>(
                &'s self,
            ) -> (
                &'s super::super::rules::r#Integer<'i, INHERITED>,
                &'s super::super::rules::r#Integer<'i, INHERITED>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.2.matched;
                            res
                        },
                        {
                            let res = &res.content.4.matched;
                            res
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`LeftBracket`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftBracket<'s>(&'s self) -> &'s super::super::rules::r#LeftBracket<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Minus`]."]
            #[allow(non_snake_case)]
            pub fn r#Minus<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Minus<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`RightBracket`]."]
            #[allow(non_snake_case)]
            pub fn r#RightBracket<'s>(&'s self) -> &'s super::super::rules::r#RightBracket<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.5.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_of`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_of<'s>(&'s self) -> &'s super::super::rules::r#kw_of<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirPlaceSubslice , "Corresponds to expression: `(LeftBracket ~ Integer? ~ Colon ~ Minus? ~ Integer? ~ RightBracket)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirPlaceSubslice , super :: super :: generics :: Seq6 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftBracket :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#Integer :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Colon :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#Minus :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#Integer :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightBracket :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirPlaceSubslice<'i, INHERITED> {
            #[doc = "A helper function to access [`Colon`]."]
            #[allow(non_snake_case)]
            pub fn r#Colon<'s>(&'s self) -> &'s super::super::rules::r#Colon<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Integer`]."]
            #[allow(non_snake_case)]
            pub fn r#Integer<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#Integer<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#Integer<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res.as_ref().map(|res| res);
                                res
                            }
                        },
                        {
                            let res = &res.content.4.matched;
                            {
                                let res = res.as_ref().map(|res| res);
                                res
                            }
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`LeftBracket`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftBracket<'s>(&'s self) -> &'s super::super::rules::r#LeftBracket<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Minus`]."]
            #[allow(non_snake_case)]
            pub fn r#Minus<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Minus<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`RightBracket`]."]
            #[allow(non_snake_case)]
            pub fn r#RightBracket<'s>(&'s self) -> &'s super::super::rules::r#RightBracket<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.5.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirPlaceDowncast , "Corresponds to expression: `(kw_as ~ (MetaVariable | Identifier) ~ !LeftParen)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirPlaceDowncast , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_as :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Choice2 :: < super :: super :: rules :: r#MetaVariable :: < 'i , INHERITED > , super :: super :: rules :: r#Identifier :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirPlaceDowncast<'i, INHERITED> {
            #[doc = "A helper function to access [`Identifier`]."]
            #[allow(non_snake_case)]
            pub fn r#Identifier<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Identifier<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res._1().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`MetaVariable`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariable<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MetaVariable<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res._0().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`kw_as`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_as<'s>(&'s self) -> &'s super::super::rules::r#kw_as<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirBasicPlace , "Corresponds to expression: `(MirPlaceLocal | MirPlaceParen | MirPlaceDeref)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirBasicPlace , super :: super :: generics :: Choice3 :: < super :: super :: rules :: r#MirPlaceLocal :: < 'i , INHERITED > , super :: super :: rules :: r#MirPlaceParen :: < 'i , INHERITED > , super :: super :: rules :: r#MirPlaceDeref :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirBasicPlace<'i, INHERITED> {
            #[doc = "A helper function to access [`MirPlaceDeref`]."]
            #[allow(non_snake_case)]
            pub fn r#MirPlaceDeref<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirPlaceDeref<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._2().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirPlaceLocal`]."]
            #[allow(non_snake_case)]
            pub fn r#MirPlaceLocal<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirPlaceLocal<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirPlaceParen`]."]
            #[allow(non_snake_case)]
            pub fn r#MirPlaceParen<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirPlaceParen<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirPlaceSuffix , "Corresponds to expression: `(MirPlaceField | MirPlaceIndex | MirPlaceConstIndex | MirPlaceSubslice | MirPlaceDowncast)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirPlaceSuffix , super :: super :: generics :: Choice5 :: < super :: super :: rules :: r#MirPlaceField :: < 'i , INHERITED > , super :: super :: rules :: r#MirPlaceIndex :: < 'i , INHERITED > , super :: super :: rules :: r#MirPlaceConstIndex :: < 'i , INHERITED > , super :: super :: rules :: r#MirPlaceSubslice :: < 'i , INHERITED > , super :: super :: rules :: r#MirPlaceDowncast :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirPlaceSuffix<'i, INHERITED> {
            #[doc = "A helper function to access [`MirPlaceConstIndex`]."]
            #[allow(non_snake_case)]
            pub fn r#MirPlaceConstIndex<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirPlaceConstIndex<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._2().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirPlaceDowncast`]."]
            #[allow(non_snake_case)]
            pub fn r#MirPlaceDowncast<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirPlaceDowncast<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._4().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirPlaceField`]."]
            #[allow(non_snake_case)]
            pub fn r#MirPlaceField<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirPlaceField<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirPlaceIndex`]."]
            #[allow(non_snake_case)]
            pub fn r#MirPlaceIndex<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirPlaceIndex<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirPlaceSubslice`]."]
            #[allow(non_snake_case)]
            pub fn r#MirPlaceSubslice<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirPlaceSubslice<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._3().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirPlace , "Corresponds to expression: `(MirBasicPlace ~ MirPlaceSuffix*)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirPlace , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirBasicPlace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: rules :: r#MirPlaceSuffix :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirPlace<'i, INHERITED> {
            #[doc = "A helper function to access [`MirBasicPlace`]."]
            #[allow(non_snake_case)]
            pub fn r#MirBasicPlace<'s>(&'s self) -> &'s super::super::rules::r#MirBasicPlace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirPlaceSuffix`]."]
            #[allow(non_snake_case)]
            pub fn r#MirPlaceSuffix<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Vec<&'s super::super::rules::r#MirPlaceSuffix<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res
                            .content
                            .iter()
                            .map(|res| {
                                let res = &res.matched;
                                res
                            })
                            .collect::<::pest_typed::re_exported::Vec<_>>();
                        res
                    }
                }
            }
        }
        :: pest_typed :: rule ! (r#MirOperandMove , "Corresponds to expression: `(kw_move ~ MirPlace)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirOperandMove , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_move :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirPlace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirOperandMove<'i, INHERITED> {
            #[doc = "A helper function to access [`MirPlace`]."]
            #[allow(non_snake_case)]
            pub fn r#MirPlace<'s>(&'s self) -> &'s super::super::rules::r#MirPlace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_move`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_move<'s>(&'s self) -> &'s super::super::rules::r#kw_move<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirOperandCopy , "Corresponds to expression: `(kw_copy ~ MirPlace)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirOperandCopy , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_copy :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirPlace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirOperandCopy<'i, INHERITED> {
            #[doc = "A helper function to access [`MirPlace`]."]
            #[allow(non_snake_case)]
            pub fn r#MirPlace<'s>(&'s self) -> &'s super::super::rules::r#MirPlace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_copy`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_copy<'s>(&'s self) -> &'s super::super::rules::r#kw_copy<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirOperandConst , "Corresponds to expression: `(kw_const ~ (Literal | LangItemWithArgs | TypePath | MetaVariable))`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirOperandConst , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_const :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Choice4 :: < super :: super :: rules :: r#Literal :: < 'i , INHERITED > , super :: super :: rules :: r#LangItemWithArgs :: < 'i , INHERITED > , super :: super :: rules :: r#TypePath :: < 'i , INHERITED > , super :: super :: rules :: r#MetaVariable :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirOperandConst<'i, INHERITED> {
            #[doc = "A helper function to access [`LangItemWithArgs`]."]
            #[allow(non_snake_case)]
            pub fn r#LangItemWithArgs<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#LangItemWithArgs<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res._1().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`Literal`]."]
            #[allow(non_snake_case)]
            pub fn r#Literal<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Literal<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res._0().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`MetaVariable`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariable<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MetaVariable<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res._3().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`TypePath`]."]
            #[allow(non_snake_case)]
            pub fn r#TypePath<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#TypePath<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res._2().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`kw_const`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_const<'s>(&'s self) -> &'s super::super::rules::r#kw_const<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirOperand , "Corresponds to expression: `(PlaceHolder | Dot2 | MetaVariable | MirOperandMove | MirOperandCopy | MirOperandConst)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirOperand , super :: super :: generics :: Choice6 :: < super :: super :: rules :: r#PlaceHolder :: < 'i , INHERITED > , super :: super :: rules :: r#Dot2 :: < 'i , INHERITED > , super :: super :: rules :: r#MetaVariable :: < 'i , INHERITED > , super :: super :: rules :: r#MirOperandMove :: < 'i , INHERITED > , super :: super :: rules :: r#MirOperandCopy :: < 'i , INHERITED > , super :: super :: rules :: r#MirOperandConst :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirOperand<'i, INHERITED> {
            #[doc = "A helper function to access [`Dot2`]."]
            #[allow(non_snake_case)]
            pub fn r#Dot2<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Dot2<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MetaVariable`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariable<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MetaVariable<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._2().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirOperandConst`]."]
            #[allow(non_snake_case)]
            pub fn r#MirOperandConst<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirOperandConst<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._5().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirOperandCopy`]."]
            #[allow(non_snake_case)]
            pub fn r#MirOperandCopy<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirOperandCopy<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._4().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirOperandMove`]."]
            #[allow(non_snake_case)]
            pub fn r#MirOperandMove<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirOperandMove<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._3().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`PlaceHolder`]."]
            #[allow(non_snake_case)]
            pub fn r#PlaceHolder<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#PlaceHolder<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirRvalueUse , "Corresponds to expression: `((LeftParen ~ MirOperand ~ RightParen) | MirOperand)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirRvalueUse , super :: super :: generics :: Choice2 :: < super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirOperand :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: rules :: r#MirOperand :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirRvalueUse<'i, INHERITED> {
            #[doc = "A helper function to access [`LeftParen`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftParen<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#LeftParen<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| {
                        let res = &res.content.0.matched;
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`MirOperand`]."]
            #[allow(non_snake_case)]
            pub fn r#MirOperand<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirOperand<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirOperand<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = res._0().map(|res| {
                                let res = &res.content.1.matched;
                                res
                            });
                            res
                        },
                        {
                            let res = res._1().map(|res| res);
                            res
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`RightParen`]."]
            #[allow(non_snake_case)]
            pub fn r#RightParen<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#RightParen<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| {
                        let res = &res.content.2.matched;
                        res
                    });
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirRvalueRepeat , "Corresponds to expression: `(LeftBracket ~ MirOperand ~ SemiColon ~ Integer ~ RightBracket)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirRvalueRepeat , super :: super :: generics :: Seq5 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftBracket :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirOperand :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#SemiColon :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Integer :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightBracket :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirRvalueRepeat<'i, INHERITED> {
            #[doc = "A helper function to access [`Integer`]."]
            #[allow(non_snake_case)]
            pub fn r#Integer<'s>(&'s self) -> &'s super::super::rules::r#Integer<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`LeftBracket`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftBracket<'s>(&'s self) -> &'s super::super::rules::r#LeftBracket<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirOperand`]."]
            #[allow(non_snake_case)]
            pub fn r#MirOperand<'s>(&'s self) -> &'s super::super::rules::r#MirOperand<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RightBracket`]."]
            #[allow(non_snake_case)]
            pub fn r#RightBracket<'s>(&'s self) -> &'s super::super::rules::r#RightBracket<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.4.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`SemiColon`]."]
            #[allow(non_snake_case)]
            pub fn r#SemiColon<'s>(&'s self) -> &'s super::super::rules::r#SemiColon<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirRvalueRef , "Corresponds to expression: `(And ~ Region? ~ Mutability ~ MirPlace)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirRvalueRef , super :: super :: generics :: Seq4 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#And :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#Region :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Mutability :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirPlace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirRvalueRef<'i, INHERITED> {
            #[doc = "A helper function to access [`And`]."]
            #[allow(non_snake_case)]
            pub fn r#And<'s>(&'s self) -> &'s super::super::rules::r#And<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirPlace`]."]
            #[allow(non_snake_case)]
            pub fn r#MirPlace<'s>(&'s self) -> &'s super::super::rules::r#MirPlace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Mutability`]."]
            #[allow(non_snake_case)]
            pub fn r#Mutability<'s>(&'s self) -> &'s super::super::rules::r#Mutability<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Region`]."]
            #[allow(non_snake_case)]
            pub fn r#Region<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Region<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
        }
        :: pest_typed :: rule ! (r#MirRvalueRawPtr , "Corresponds to expression: `(And ~ kw_raw ~ PtrMutability ~ MirPlace)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirRvalueRawPtr , super :: super :: generics :: Seq4 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#And :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_raw :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#PtrMutability :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirPlace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirRvalueRawPtr<'i, INHERITED> {
            #[doc = "A helper function to access [`And`]."]
            #[allow(non_snake_case)]
            pub fn r#And<'s>(&'s self) -> &'s super::super::rules::r#And<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirPlace`]."]
            #[allow(non_snake_case)]
            pub fn r#MirPlace<'s>(&'s self) -> &'s super::super::rules::r#MirPlace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`PtrMutability`]."]
            #[allow(non_snake_case)]
            pub fn r#PtrMutability<'s>(&'s self) -> &'s super::super::rules::r#PtrMutability<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_raw`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_raw<'s>(&'s self) -> &'s super::super::rules::r#kw_raw<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirRvalueLen , "Corresponds to expression: `(kw_Len ~ LeftParen ~ MirPlace ~ RightParen)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirRvalueLen , super :: super :: generics :: Seq4 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_Len :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirPlace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirRvalueLen<'i, INHERITED> {
            #[doc = "A helper function to access [`LeftParen`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftParen<'s>(&'s self) -> &'s super::super::rules::r#LeftParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirPlace`]."]
            #[allow(non_snake_case)]
            pub fn r#MirPlace<'s>(&'s self) -> &'s super::super::rules::r#MirPlace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RightParen`]."]
            #[allow(non_snake_case)]
            pub fn r#RightParen<'s>(&'s self) -> &'s super::super::rules::r#RightParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_Len`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_Len<'s>(&'s self) -> &'s super::super::rules::r#kw_Len<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#PointerCoercion , "Corresponds to expression: `kw_Unsize`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#PointerCoercion , super :: super :: rules :: r#kw_Unsize :: < 'i , INHERITED > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#PointerCoercion<'i, INHERITED> {
            #[doc = "A helper function to access [`kw_Unsize`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_Unsize<'s>(&'s self) -> &'s super::super::rules::r#kw_Unsize<'i, INHERITED> {
                let res = &*self.content;
                res
            }
        }
        :: pest_typed :: rule ! (r#CoercionSource , "Corresponds to expression: `kw_Implicit`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#CoercionSource , super :: super :: rules :: r#kw_Implicit :: < 'i , INHERITED > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#CoercionSource<'i, INHERITED> {
            #[doc = "A helper function to access [`kw_Implicit`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_Implicit<'s>(&'s self) -> &'s super::super::rules::r#kw_Implicit<'i, INHERITED> {
                let res = &*self.content;
                res
            }
        }
        :: pest_typed :: rule ! (r#MirCastKind , "Corresponds to expression: `(kw_PtrToPtr | kw_IntToInt | kw_Transmute | (kw_PointerCoercion ~ LeftParen ~ PointerCoercion ~ Comma ~ CoercionSource ~ RightParen))`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirCastKind , super :: super :: generics :: Choice4 :: < super :: super :: rules :: r#kw_PtrToPtr :: < 'i , INHERITED > , super :: super :: rules :: r#kw_IntToInt :: < 'i , INHERITED > , super :: super :: rules :: r#kw_Transmute :: < 'i , INHERITED > , super :: super :: generics :: Seq6 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_PointerCoercion :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#PointerCoercion :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Comma :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#CoercionSource :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirCastKind<'i, INHERITED> {
            #[doc = "A helper function to access [`CoercionSource`]."]
            #[allow(non_snake_case)]
            pub fn r#CoercionSource<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#CoercionSource<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._3().map(|res| {
                        let res = &res.content.4.matched;
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`Comma`]."]
            #[allow(non_snake_case)]
            pub fn r#Comma<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Comma<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._3().map(|res| {
                        let res = &res.content.3.matched;
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`LeftParen`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftParen<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#LeftParen<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._3().map(|res| {
                        let res = &res.content.1.matched;
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`PointerCoercion`]."]
            #[allow(non_snake_case)]
            pub fn r#PointerCoercion<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#PointerCoercion<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._3().map(|res| {
                        let res = &res.content.2.matched;
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`RightParen`]."]
            #[allow(non_snake_case)]
            pub fn r#RightParen<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#RightParen<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._3().map(|res| {
                        let res = &res.content.5.matched;
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`kw_IntToInt`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_IntToInt<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_IntToInt<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_PointerCoercion`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_PointerCoercion<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_PointerCoercion<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._3().map(|res| {
                        let res = &res.content.0.matched;
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`kw_PtrToPtr`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_PtrToPtr<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_PtrToPtr<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_Transmute`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_Transmute<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_Transmute<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._2().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirRvalueCast , "Corresponds to expression: `(MirOperand ~ kw_as ~ Type ~ LeftParen ~ MirCastKind ~ RightParen)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirRvalueCast , super :: super :: generics :: Seq6 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirOperand :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_as :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Type :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirCastKind :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirRvalueCast<'i, INHERITED> {
            #[doc = "A helper function to access [`LeftParen`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftParen<'s>(&'s self) -> &'s super::super::rules::r#LeftParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirCastKind`]."]
            #[allow(non_snake_case)]
            pub fn r#MirCastKind<'s>(&'s self) -> &'s super::super::rules::r#MirCastKind<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.4.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirOperand`]."]
            #[allow(non_snake_case)]
            pub fn r#MirOperand<'s>(&'s self) -> &'s super::super::rules::r#MirOperand<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RightParen`]."]
            #[allow(non_snake_case)]
            pub fn r#RightParen<'s>(&'s self) -> &'s super::super::rules::r#RightParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.5.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Type`]."]
            #[allow(non_snake_case)]
            pub fn r#Type<'s>(&'s self) -> &'s super::super::rules::r#Type<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_as`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_as<'s>(&'s self) -> &'s super::super::rules::r#kw_as<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirBinOp , "Corresponds to expression: `(kw_Add | kw_Sub | kw_Mul | kw_Div | kw_Rem | kw_Lt | kw_Le | kw_Gt | kw_Ge | kw_Eq | kw_Ne | kw_Offset)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirBinOp , super :: super :: generics :: Choice12 :: < super :: super :: rules :: r#kw_Add :: < 'i , INHERITED > , super :: super :: rules :: r#kw_Sub :: < 'i , INHERITED > , super :: super :: rules :: r#kw_Mul :: < 'i , INHERITED > , super :: super :: rules :: r#kw_Div :: < 'i , INHERITED > , super :: super :: rules :: r#kw_Rem :: < 'i , INHERITED > , super :: super :: rules :: r#kw_Lt :: < 'i , INHERITED > , super :: super :: rules :: r#kw_Le :: < 'i , INHERITED > , super :: super :: rules :: r#kw_Gt :: < 'i , INHERITED > , super :: super :: rules :: r#kw_Ge :: < 'i , INHERITED > , super :: super :: rules :: r#kw_Eq :: < 'i , INHERITED > , super :: super :: rules :: r#kw_Ne :: < 'i , INHERITED > , super :: super :: rules :: r#kw_Offset :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirBinOp<'i, INHERITED> {
            #[doc = "A helper function to access [`kw_Add`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_Add<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_Add<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_Div`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_Div<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_Div<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._3().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_Eq`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_Eq<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_Eq<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._9().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_Ge`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_Ge<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_Ge<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._8().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_Gt`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_Gt<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_Gt<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._7().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_Le`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_Le<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_Le<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._6().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_Lt`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_Lt<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_Lt<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._5().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_Mul`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_Mul<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_Mul<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._2().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_Ne`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_Ne<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_Ne<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._10().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_Offset`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_Offset<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_Offset<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._11().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_Rem`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_Rem<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_Rem<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._4().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_Sub`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_Sub<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_Sub<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirRvalueBinOp , "Corresponds to expression: `(MirBinOp ~ LeftParen ~ MirOperand ~ Comma ~ MirOperand ~ RightParen)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirRvalueBinOp , super :: super :: generics :: Seq6 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirBinOp :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirOperand :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Comma :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirOperand :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirRvalueBinOp<'i, INHERITED> {
            #[doc = "A helper function to access [`Comma`]."]
            #[allow(non_snake_case)]
            pub fn r#Comma<'s>(&'s self) -> &'s super::super::rules::r#Comma<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`LeftParen`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftParen<'s>(&'s self) -> &'s super::super::rules::r#LeftParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirBinOp`]."]
            #[allow(non_snake_case)]
            pub fn r#MirBinOp<'s>(&'s self) -> &'s super::super::rules::r#MirBinOp<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirOperand`]."]
            #[allow(non_snake_case)]
            pub fn r#MirOperand<'s>(
                &'s self,
            ) -> (
                &'s super::super::rules::r#MirOperand<'i, INHERITED>,
                &'s super::super::rules::r#MirOperand<'i, INHERITED>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.2.matched;
                            res
                        },
                        {
                            let res = &res.content.4.matched;
                            res
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`RightParen`]."]
            #[allow(non_snake_case)]
            pub fn r#RightParen<'s>(&'s self) -> &'s super::super::rules::r#RightParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.5.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirNullOp , "Corresponds to expression: `(kw_SizeOf | kw_AlignOf)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirNullOp , super :: super :: generics :: Choice2 :: < super :: super :: rules :: r#kw_SizeOf :: < 'i , INHERITED > , super :: super :: rules :: r#kw_AlignOf :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirNullOp<'i, INHERITED> {
            #[doc = "A helper function to access [`kw_AlignOf`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_AlignOf<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_AlignOf<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_SizeOf`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_SizeOf<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_SizeOf<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirRvalueNullOp , "Corresponds to expression: `(MirNullOp ~ LeftParen ~ Type ~ RightParen)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirRvalueNullOp , super :: super :: generics :: Seq4 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirNullOp :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Type :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirRvalueNullOp<'i, INHERITED> {
            #[doc = "A helper function to access [`LeftParen`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftParen<'s>(&'s self) -> &'s super::super::rules::r#LeftParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirNullOp`]."]
            #[allow(non_snake_case)]
            pub fn r#MirNullOp<'s>(&'s self) -> &'s super::super::rules::r#MirNullOp<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RightParen`]."]
            #[allow(non_snake_case)]
            pub fn r#RightParen<'s>(&'s self) -> &'s super::super::rules::r#RightParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Type`]."]
            #[allow(non_snake_case)]
            pub fn r#Type<'s>(&'s self) -> &'s super::super::rules::r#Type<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirUnOp , "Corresponds to expression: `(kw_Neg | kw_Not | kw_PtrMetadata)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirUnOp , super :: super :: generics :: Choice3 :: < super :: super :: rules :: r#kw_Neg :: < 'i , INHERITED > , super :: super :: rules :: r#kw_Not :: < 'i , INHERITED > , super :: super :: rules :: r#kw_PtrMetadata :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirUnOp<'i, INHERITED> {
            #[doc = "A helper function to access [`kw_Neg`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_Neg<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_Neg<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_Not`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_Not<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_Not<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`kw_PtrMetadata`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_PtrMetadata<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_PtrMetadata<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._2().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirRvalueUnOp , "Corresponds to expression: `(MirUnOp ~ LeftParen ~ MirOperand ~ RightParen)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirRvalueUnOp , super :: super :: generics :: Seq4 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirUnOp :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirOperand :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirRvalueUnOp<'i, INHERITED> {
            #[doc = "A helper function to access [`LeftParen`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftParen<'s>(&'s self) -> &'s super::super::rules::r#LeftParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirOperand`]."]
            #[allow(non_snake_case)]
            pub fn r#MirOperand<'s>(&'s self) -> &'s super::super::rules::r#MirOperand<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirUnOp`]."]
            #[allow(non_snake_case)]
            pub fn r#MirUnOp<'s>(&'s self) -> &'s super::super::rules::r#MirUnOp<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RightParen`]."]
            #[allow(non_snake_case)]
            pub fn r#RightParen<'s>(&'s self) -> &'s super::super::rules::r#RightParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirRvalueDiscriminant , "Corresponds to expression: `(kw_discriminant ~ LeftParen ~ MirPlace ~ RightParen)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirRvalueDiscriminant , super :: super :: generics :: Seq4 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_discriminant :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirPlace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirRvalueDiscriminant<'i, INHERITED> {
            #[doc = "A helper function to access [`LeftParen`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftParen<'s>(&'s self) -> &'s super::super::rules::r#LeftParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirPlace`]."]
            #[allow(non_snake_case)]
            pub fn r#MirPlace<'s>(&'s self) -> &'s super::super::rules::r#MirPlace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RightParen`]."]
            #[allow(non_snake_case)]
            pub fn r#RightParen<'s>(&'s self) -> &'s super::super::rules::r#RightParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_discriminant`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_discriminant<'s>(&'s self) -> &'s super::super::rules::r#kw_discriminant<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirOperandsSeparatedByComma , "Corresponds to expression: `(MirOperand ~ (Comma ~ MirOperand)* ~ Comma?)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirOperandsSeparatedByComma , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirOperand :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Comma :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirOperand :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#Comma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirOperandsSeparatedByComma<'i, INHERITED> {
            #[doc = "A helper function to access [`Comma`]."]
            #[allow(non_snake_case)]
            pub fn r#Comma<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#Comma<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#Comma<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.0.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                        {
                            let res = &res.content.2.matched;
                            {
                                let res = res.as_ref().map(|res| res);
                                res
                            }
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`MirOperand`]."]
            #[allow(non_snake_case)]
            pub fn r#MirOperand<'s>(
                &'s self,
            ) -> (
                &'s super::super::rules::r#MirOperand<'i, INHERITED>,
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#MirOperand<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.0.matched;
                            res
                        },
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.1.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                    );
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirAggregateArray , "Corresponds to expression: `(LeftBracket ~ MirOperandsSeparatedByComma? ~ RightBracket)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirAggregateArray , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftBracket :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#MirOperandsSeparatedByComma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightBracket :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirAggregateArray<'i, INHERITED> {
            #[doc = "A helper function to access [`LeftBracket`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftBracket<'s>(&'s self) -> &'s super::super::rules::r#LeftBracket<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirOperandsSeparatedByComma`]."]
            #[allow(non_snake_case)]
            pub fn r#MirOperandsSeparatedByComma<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirOperandsSeparatedByComma<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`RightBracket`]."]
            #[allow(non_snake_case)]
            pub fn r#RightBracket<'s>(&'s self) -> &'s super::super::rules::r#RightBracket<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirAggregateTuple , "Corresponds to expression: `(LeftParen ~ MirOperandsSeparatedByComma? ~ RightParen)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirAggregateTuple , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#MirOperandsSeparatedByComma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirAggregateTuple<'i, INHERITED> {
            #[doc = "A helper function to access [`LeftParen`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftParen<'s>(&'s self) -> &'s super::super::rules::r#LeftParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirOperandsSeparatedByComma`]."]
            #[allow(non_snake_case)]
            pub fn r#MirOperandsSeparatedByComma<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirOperandsSeparatedByComma<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`RightParen`]."]
            #[allow(non_snake_case)]
            pub fn r#RightParen<'s>(&'s self) -> &'s super::super::rules::r#RightParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#PathOrLangItem , "Corresponds to expression: `(Path | LangItemWithArgs)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#PathOrLangItem , super :: super :: generics :: Choice2 :: < super :: super :: rules :: r#Path :: < 'i , INHERITED > , super :: super :: rules :: r#LangItemWithArgs :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#PathOrLangItem<'i, INHERITED> {
            #[doc = "A helper function to access [`LangItemWithArgs`]."]
            #[allow(non_snake_case)]
            pub fn r#LangItemWithArgs<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#LangItemWithArgs<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`Path`]."]
            #[allow(non_snake_case)]
            pub fn r#Path<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Path<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirAggregateAdtStructField , "Corresponds to expression: `(Identifier ~ Colon ~ MirOperand)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirAggregateAdtStructField , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Identifier :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Colon :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirOperand :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirAggregateAdtStructField<'i, INHERITED> {
            #[doc = "A helper function to access [`Colon`]."]
            #[allow(non_snake_case)]
            pub fn r#Colon<'s>(&'s self) -> &'s super::super::rules::r#Colon<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Identifier`]."]
            #[allow(non_snake_case)]
            pub fn r#Identifier<'s>(&'s self) -> &'s super::super::rules::r#Identifier<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirOperand`]."]
            #[allow(non_snake_case)]
            pub fn r#MirOperand<'s>(&'s self) -> &'s super::super::rules::r#MirOperand<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirAggregateAdtStructFieldsSeparatedByComma , "Corresponds to expression: `(MirAggregateAdtStructField ~ (Comma ~ MirAggregateAdtStructField)* ~ Comma?)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirAggregateAdtStructFieldsSeparatedByComma , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirAggregateAdtStructField :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Comma :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirAggregateAdtStructField :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#Comma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirAggregateAdtStructFieldsSeparatedByComma<'i, INHERITED> {
            #[doc = "A helper function to access [`Comma`]."]
            #[allow(non_snake_case)]
            pub fn r#Comma<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#Comma<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#Comma<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.0.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                        {
                            let res = &res.content.2.matched;
                            {
                                let res = res.as_ref().map(|res| res);
                                res
                            }
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`MirAggregateAdtStructField`]."]
            #[allow(non_snake_case)]
            pub fn r#MirAggregateAdtStructField<'s>(
                &'s self,
            ) -> (
                &'s super::super::rules::r#MirAggregateAdtStructField<'i, INHERITED>,
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#MirAggregateAdtStructField<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.0.matched;
                            res
                        },
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.1.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                    );
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirAggregateAdtStruct , "Corresponds to expression: `(PathOrLangItem ~ LeftBrace ~ MirAggregateAdtStructFieldsSeparatedByComma? ~ RightBrace)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirAggregateAdtStruct , super :: super :: generics :: Seq4 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#PathOrLangItem :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#MirAggregateAdtStructFieldsSeparatedByComma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirAggregateAdtStruct<'i, INHERITED> {
            #[doc = "A helper function to access [`LeftBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftBrace<'s>(&'s self) -> &'s super::super::rules::r#LeftBrace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirAggregateAdtStructFieldsSeparatedByComma`]."]
            #[allow(non_snake_case)]
            pub fn r#MirAggregateAdtStructFieldsSeparatedByComma<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<
                &'s super::super::rules::r#MirAggregateAdtStructFieldsSeparatedByComma<'i, INHERITED>,
            > {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`PathOrLangItem`]."]
            #[allow(non_snake_case)]
            pub fn r#PathOrLangItem<'s>(&'s self) -> &'s super::super::rules::r#PathOrLangItem<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RightBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#RightBrace<'s>(&'s self) -> &'s super::super::rules::r#RightBrace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirAggregateAdtTuple , "Corresponds to expression: `(Hash ~ LeftBracket ~ kw_Ctor ~ RightBracket ~ Path ~ LeftParen ~ MirOperandsSeparatedByComma? ~ RightParen)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirAggregateAdtTuple , super :: super :: generics :: Seq8 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Hash :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftBracket :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_Ctor :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightBracket :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Path :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#MirOperandsSeparatedByComma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirAggregateAdtTuple<'i, INHERITED> {
            #[doc = "A helper function to access [`Hash`]."]
            #[allow(non_snake_case)]
            pub fn r#Hash<'s>(&'s self) -> &'s super::super::rules::r#Hash<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`LeftBracket`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftBracket<'s>(&'s self) -> &'s super::super::rules::r#LeftBracket<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`LeftParen`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftParen<'s>(&'s self) -> &'s super::super::rules::r#LeftParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.5.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirOperandsSeparatedByComma`]."]
            #[allow(non_snake_case)]
            pub fn r#MirOperandsSeparatedByComma<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirOperandsSeparatedByComma<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = &res.content.6.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`Path`]."]
            #[allow(non_snake_case)]
            pub fn r#Path<'s>(&'s self) -> &'s super::super::rules::r#Path<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.4.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RightBracket`]."]
            #[allow(non_snake_case)]
            pub fn r#RightBracket<'s>(&'s self) -> &'s super::super::rules::r#RightBracket<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RightParen`]."]
            #[allow(non_snake_case)]
            pub fn r#RightParen<'s>(&'s self) -> &'s super::super::rules::r#RightParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.7.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_Ctor`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_Ctor<'s>(&'s self) -> &'s super::super::rules::r#kw_Ctor<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirAggregateAdtUnit , "Corresponds to expression: `PathOrLangItem`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirAggregateAdtUnit , super :: super :: rules :: r#PathOrLangItem :: < 'i , INHERITED > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirAggregateAdtUnit<'i, INHERITED> {
            #[doc = "A helper function to access [`PathOrLangItem`]."]
            #[allow(non_snake_case)]
            pub fn r#PathOrLangItem<'s>(&'s self) -> &'s super::super::rules::r#PathOrLangItem<'i, INHERITED> {
                let res = &*self.content;
                res
            }
        }
        :: pest_typed :: rule ! (r#MirAggregateRawPtr , "Corresponds to expression: `(TypePtr ~ kw_from ~ LeftParen ~ MirOperand ~ Comma ~ MirOperand ~ RightParen)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirAggregateRawPtr , super :: super :: generics :: Seq7 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#TypePtr :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_from :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirOperand :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Comma :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirOperand :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirAggregateRawPtr<'i, INHERITED> {
            #[doc = "A helper function to access [`Comma`]."]
            #[allow(non_snake_case)]
            pub fn r#Comma<'s>(&'s self) -> &'s super::super::rules::r#Comma<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.4.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`LeftParen`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftParen<'s>(&'s self) -> &'s super::super::rules::r#LeftParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirOperand`]."]
            #[allow(non_snake_case)]
            pub fn r#MirOperand<'s>(
                &'s self,
            ) -> (
                &'s super::super::rules::r#MirOperand<'i, INHERITED>,
                &'s super::super::rules::r#MirOperand<'i, INHERITED>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.3.matched;
                            res
                        },
                        {
                            let res = &res.content.5.matched;
                            res
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`RightParen`]."]
            #[allow(non_snake_case)]
            pub fn r#RightParen<'s>(&'s self) -> &'s super::super::rules::r#RightParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.6.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`TypePtr`]."]
            #[allow(non_snake_case)]
            pub fn r#TypePtr<'s>(&'s self) -> &'s super::super::rules::r#TypePtr<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_from`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_from<'s>(&'s self) -> &'s super::super::rules::r#kw_from<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirRvalueAggregate , "Corresponds to expression: `(MirAggregateArray | MirAggregateTuple | MirAggregateAdtStruct | MirAggregateAdtTuple | MirAggregateAdtUnit | MirAggregateRawPtr)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirRvalueAggregate , super :: super :: generics :: Choice6 :: < super :: super :: rules :: r#MirAggregateArray :: < 'i , INHERITED > , super :: super :: rules :: r#MirAggregateTuple :: < 'i , INHERITED > , super :: super :: rules :: r#MirAggregateAdtStruct :: < 'i , INHERITED > , super :: super :: rules :: r#MirAggregateAdtTuple :: < 'i , INHERITED > , super :: super :: rules :: r#MirAggregateAdtUnit :: < 'i , INHERITED > , super :: super :: rules :: r#MirAggregateRawPtr :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirRvalueAggregate<'i, INHERITED> {
            #[doc = "A helper function to access [`MirAggregateAdtStruct`]."]
            #[allow(non_snake_case)]
            pub fn r#MirAggregateAdtStruct<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirAggregateAdtStruct<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._2().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirAggregateAdtTuple`]."]
            #[allow(non_snake_case)]
            pub fn r#MirAggregateAdtTuple<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirAggregateAdtTuple<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._3().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirAggregateAdtUnit`]."]
            #[allow(non_snake_case)]
            pub fn r#MirAggregateAdtUnit<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirAggregateAdtUnit<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._4().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirAggregateArray`]."]
            #[allow(non_snake_case)]
            pub fn r#MirAggregateArray<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirAggregateArray<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirAggregateRawPtr`]."]
            #[allow(non_snake_case)]
            pub fn r#MirAggregateRawPtr<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirAggregateRawPtr<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._5().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirAggregateTuple`]."]
            #[allow(non_snake_case)]
            pub fn r#MirAggregateTuple<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirAggregateTuple<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirRvalue , "Corresponds to expression: `(PlaceHolder | MirRvalueCast | MirRvalueUse | MirRvalueRepeat | MirRvalueRef | MirRvalueRawPtr | MirRvalueLen | MirRvalueBinOp | MirRvalueNullOp | MirRvalueUnOp | MirRvalueDiscriminant | MirRvalueAggregate)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirRvalue , super :: super :: generics :: Choice12 :: < super :: super :: rules :: r#PlaceHolder :: < 'i , INHERITED > , super :: super :: rules :: r#MirRvalueCast :: < 'i , INHERITED > , super :: super :: rules :: r#MirRvalueUse :: < 'i , INHERITED > , super :: super :: rules :: r#MirRvalueRepeat :: < 'i , INHERITED > , super :: super :: rules :: r#MirRvalueRef :: < 'i , INHERITED > , super :: super :: rules :: r#MirRvalueRawPtr :: < 'i , INHERITED > , super :: super :: rules :: r#MirRvalueLen :: < 'i , INHERITED > , super :: super :: rules :: r#MirRvalueBinOp :: < 'i , INHERITED > , super :: super :: rules :: r#MirRvalueNullOp :: < 'i , INHERITED > , super :: super :: rules :: r#MirRvalueUnOp :: < 'i , INHERITED > , super :: super :: rules :: r#MirRvalueDiscriminant :: < 'i , INHERITED > , super :: super :: rules :: r#MirRvalueAggregate :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirRvalue<'i, INHERITED> {
            #[doc = "A helper function to access [`MirRvalueAggregate`]."]
            #[allow(non_snake_case)]
            pub fn r#MirRvalueAggregate<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirRvalueAggregate<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._11().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirRvalueBinOp`]."]
            #[allow(non_snake_case)]
            pub fn r#MirRvalueBinOp<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirRvalueBinOp<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._7().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirRvalueCast`]."]
            #[allow(non_snake_case)]
            pub fn r#MirRvalueCast<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirRvalueCast<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirRvalueDiscriminant`]."]
            #[allow(non_snake_case)]
            pub fn r#MirRvalueDiscriminant<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirRvalueDiscriminant<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._10().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirRvalueLen`]."]
            #[allow(non_snake_case)]
            pub fn r#MirRvalueLen<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirRvalueLen<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._6().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirRvalueNullOp`]."]
            #[allow(non_snake_case)]
            pub fn r#MirRvalueNullOp<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirRvalueNullOp<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._8().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirRvalueRawPtr`]."]
            #[allow(non_snake_case)]
            pub fn r#MirRvalueRawPtr<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirRvalueRawPtr<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._5().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirRvalueRef`]."]
            #[allow(non_snake_case)]
            pub fn r#MirRvalueRef<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirRvalueRef<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._4().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirRvalueRepeat`]."]
            #[allow(non_snake_case)]
            pub fn r#MirRvalueRepeat<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirRvalueRepeat<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._3().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirRvalueUnOp`]."]
            #[allow(non_snake_case)]
            pub fn r#MirRvalueUnOp<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirRvalueUnOp<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._9().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirRvalueUse`]."]
            #[allow(non_snake_case)]
            pub fn r#MirRvalueUse<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirRvalueUse<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._2().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`PlaceHolder`]."]
            #[allow(non_snake_case)]
            pub fn r#PlaceHolder<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#PlaceHolder<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirFnOperand , "Corresponds to expression: `((LeftParen ~ MirOperandCopy ~ RightParen) | (LeftParen ~ MirOperandMove ~ RightParen) | TypePath | LangItemWithArgs | MetaVariable)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirFnOperand , super :: super :: generics :: Choice5 :: < super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirOperandCopy :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirOperandMove :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: rules :: r#TypePath :: < 'i , INHERITED > , super :: super :: rules :: r#LangItemWithArgs :: < 'i , INHERITED > , super :: super :: rules :: r#MetaVariable :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirFnOperand<'i, INHERITED> {
            #[doc = "A helper function to access [`LangItemWithArgs`]."]
            #[allow(non_snake_case)]
            pub fn r#LangItemWithArgs<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#LangItemWithArgs<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._3().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`LeftParen`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftParen<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#LeftParen<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#LeftParen<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = res._0().map(|res| {
                                let res = &res.content.0.matched;
                                res
                            });
                            res
                        },
                        {
                            let res = res._1().map(|res| {
                                let res = &res.content.0.matched;
                                res
                            });
                            res
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`MetaVariable`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariable<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MetaVariable<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._4().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirOperandCopy`]."]
            #[allow(non_snake_case)]
            pub fn r#MirOperandCopy<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirOperandCopy<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| {
                        let res = &res.content.1.matched;
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`MirOperandMove`]."]
            #[allow(non_snake_case)]
            pub fn r#MirOperandMove<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirOperandMove<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| {
                        let res = &res.content.1.matched;
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`RightParen`]."]
            #[allow(non_snake_case)]
            pub fn r#RightParen<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#RightParen<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#RightParen<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = res._0().map(|res| {
                                let res = &res.content.2.matched;
                                res
                            });
                            res
                        },
                        {
                            let res = res._1().map(|res| {
                                let res = &res.content.2.matched;
                                res
                            });
                            res
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`TypePath`]."]
            #[allow(non_snake_case)]
            pub fn r#TypePath<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#TypePath<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._2().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirCall , "Corresponds to expression: `(MirFnOperand ~ LeftParen ~ MirOperandsSeparatedByComma? ~ RightParen)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirCall , super :: super :: generics :: Seq4 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirFnOperand :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#MirOperandsSeparatedByComma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirCall<'i, INHERITED> {
            #[doc = "A helper function to access [`LeftParen`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftParen<'s>(&'s self) -> &'s super::super::rules::r#LeftParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirFnOperand`]."]
            #[allow(non_snake_case)]
            pub fn r#MirFnOperand<'s>(&'s self) -> &'s super::super::rules::r#MirFnOperand<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirOperandsSeparatedByComma`]."]
            #[allow(non_snake_case)]
            pub fn r#MirOperandsSeparatedByComma<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirOperandsSeparatedByComma<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`RightParen`]."]
            #[allow(non_snake_case)]
            pub fn r#RightParen<'s>(&'s self) -> &'s super::super::rules::r#RightParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirRvalueOrCall , "Corresponds to expression: `(MirCall | MirRvalue)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirRvalueOrCall , super :: super :: generics :: Choice2 :: < super :: super :: rules :: r#MirCall :: < 'i , INHERITED > , super :: super :: rules :: r#MirRvalue :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirRvalueOrCall<'i, INHERITED> {
            #[doc = "A helper function to access [`MirCall`]."]
            #[allow(non_snake_case)]
            pub fn r#MirCall<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirCall<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirRvalue`]."]
            #[allow(non_snake_case)]
            pub fn r#MirRvalue<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirRvalue<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirTypeDecl , "Corresponds to expression: `(kw_type ~ Identifier ~ Assign ~ Type ~ SemiColon)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirTypeDecl , super :: super :: generics :: Seq5 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_type :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Identifier :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Assign :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Type :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#SemiColon :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirTypeDecl<'i, INHERITED> {
            #[doc = "A helper function to access [`Assign`]."]
            #[allow(non_snake_case)]
            pub fn r#Assign<'s>(&'s self) -> &'s super::super::rules::r#Assign<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Identifier`]."]
            #[allow(non_snake_case)]
            pub fn r#Identifier<'s>(&'s self) -> &'s super::super::rules::r#Identifier<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`SemiColon`]."]
            #[allow(non_snake_case)]
            pub fn r#SemiColon<'s>(&'s self) -> &'s super::super::rules::r#SemiColon<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.4.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Type`]."]
            #[allow(non_snake_case)]
            pub fn r#Type<'s>(&'s self) -> &'s super::super::rules::r#Type<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_type`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_type<'s>(&'s self) -> &'s super::super::rules::r#kw_type<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirLocalDecl , "Corresponds to expression: `(LabelWithColon? ~ kw_let ~ Mutability ~ MirPlaceLocal ~ Colon ~ Type ~ (Assign ~ MirRvalueOrCall)? ~ SemiColon)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirLocalDecl , super :: super :: generics :: Seq8 :: < (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#LabelWithColon :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_let :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Mutability :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirPlaceLocal :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Colon :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Type :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Assign :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirRvalueOrCall :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#SemiColon :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirLocalDecl<'i, INHERITED> {
            #[doc = "A helper function to access [`Assign`]."]
            #[allow(non_snake_case)]
            pub fn r#Assign<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Assign<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.6.matched;
                    {
                        let res = res.as_ref().map(|res| {
                            let res = &res.content.0.matched;
                            res
                        });
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`Colon`]."]
            #[allow(non_snake_case)]
            pub fn r#Colon<'s>(&'s self) -> &'s super::super::rules::r#Colon<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.4.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`LabelWithColon`]."]
            #[allow(non_snake_case)]
            pub fn r#LabelWithColon<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#LabelWithColon<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`MirPlaceLocal`]."]
            #[allow(non_snake_case)]
            pub fn r#MirPlaceLocal<'s>(&'s self) -> &'s super::super::rules::r#MirPlaceLocal<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirRvalueOrCall`]."]
            #[allow(non_snake_case)]
            pub fn r#MirRvalueOrCall<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirRvalueOrCall<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = &res.content.6.matched;
                    {
                        let res = res.as_ref().map(|res| {
                            let res = &res.content.1.matched;
                            res
                        });
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`Mutability`]."]
            #[allow(non_snake_case)]
            pub fn r#Mutability<'s>(&'s self) -> &'s super::super::rules::r#Mutability<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`SemiColon`]."]
            #[allow(non_snake_case)]
            pub fn r#SemiColon<'s>(&'s self) -> &'s super::super::rules::r#SemiColon<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.7.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Type`]."]
            #[allow(non_snake_case)]
            pub fn r#Type<'s>(&'s self) -> &'s super::super::rules::r#Type<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.5.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_let`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_let<'s>(&'s self) -> &'s super::super::rules::r#kw_let<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#UsePath , "Corresponds to expression: `(kw_use ~ Path ~ SemiColon)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#UsePath , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_use :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Path :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#SemiColon :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#UsePath<'i, INHERITED> {
            #[doc = "A helper function to access [`Path`]."]
            #[allow(non_snake_case)]
            pub fn r#Path<'s>(&'s self) -> &'s super::super::rules::r#Path<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`SemiColon`]."]
            #[allow(non_snake_case)]
            pub fn r#SemiColon<'s>(&'s self) -> &'s super::super::rules::r#SemiColon<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_use`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_use<'s>(&'s self) -> &'s super::super::rules::r#kw_use<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirDecl , "Corresponds to expression: `(MirTypeDecl | MirLocalDecl)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirDecl , super :: super :: generics :: Choice2 :: < super :: super :: rules :: r#MirTypeDecl :: < 'i , INHERITED > , super :: super :: rules :: r#MirLocalDecl :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirDecl<'i, INHERITED> {
            #[doc = "A helper function to access [`MirLocalDecl`]."]
            #[allow(non_snake_case)]
            pub fn r#MirLocalDecl<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirLocalDecl<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirTypeDecl`]."]
            #[allow(non_snake_case)]
            pub fn r#MirTypeDecl<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirTypeDecl<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirCallIgnoreRet , "Corresponds to expression: `(LabelWithColon? ~ PlaceHolder ~ Assign ~ MirCall)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirCallIgnoreRet , super :: super :: generics :: Seq4 :: < (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#LabelWithColon :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#PlaceHolder :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Assign :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirCall :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirCallIgnoreRet<'i, INHERITED> {
            #[doc = "A helper function to access [`Assign`]."]
            #[allow(non_snake_case)]
            pub fn r#Assign<'s>(&'s self) -> &'s super::super::rules::r#Assign<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`LabelWithColon`]."]
            #[allow(non_snake_case)]
            pub fn r#LabelWithColon<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#LabelWithColon<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`MirCall`]."]
            #[allow(non_snake_case)]
            pub fn r#MirCall<'s>(&'s self) -> &'s super::super::rules::r#MirCall<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`PlaceHolder`]."]
            #[allow(non_snake_case)]
            pub fn r#PlaceHolder<'s>(&'s self) -> &'s super::super::rules::r#PlaceHolder<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirDrop , "Corresponds to expression: `(LabelWithColon? ~ kw_drop ~ LeftParen ~ MirPlace ~ RightParen)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirDrop , super :: super :: generics :: Seq5 :: < (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#LabelWithColon :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_drop :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirPlace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirDrop<'i, INHERITED> {
            #[doc = "A helper function to access [`LabelWithColon`]."]
            #[allow(non_snake_case)]
            pub fn r#LabelWithColon<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#LabelWithColon<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`LeftParen`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftParen<'s>(&'s self) -> &'s super::super::rules::r#LeftParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirPlace`]."]
            #[allow(non_snake_case)]
            pub fn r#MirPlace<'s>(&'s self) -> &'s super::super::rules::r#MirPlace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RightParen`]."]
            #[allow(non_snake_case)]
            pub fn r#RightParen<'s>(&'s self) -> &'s super::super::rules::r#RightParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.4.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_drop`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_drop<'s>(&'s self) -> &'s super::super::rules::r#kw_drop<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#Label , "Corresponds to expression: `(Quote ~ LabelName)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Label , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Quote :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LabelName :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Label<'i, INHERITED> {
            #[doc = "A helper function to access [`LabelName`]."]
            #[allow(non_snake_case)]
            pub fn r#LabelName<'s>(&'s self) -> &'s super::super::rules::r#LabelName<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Quote`]."]
            #[allow(non_snake_case)]
            pub fn r#Quote<'s>(&'s self) -> &'s super::super::rules::r#Quote<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#LabelWithColon , "Corresponds to expression: `(Label ~ Colon)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#LabelWithColon , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Label :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Colon :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#LabelWithColon<'i, INHERITED> {
            #[doc = "A helper function to access [`Colon`]."]
            #[allow(non_snake_case)]
            pub fn r#Colon<'s>(&'s self) -> &'s super::super::rules::r#Colon<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Label`]."]
            #[allow(non_snake_case)]
            pub fn r#Label<'s>(&'s self) -> &'s super::super::rules::r#Label<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirControl , "Corresponds to expression: `(LabelWithColon? ~ (kw_break | kw_continue) ~ Label?)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirControl , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#LabelWithColon :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Choice2 :: < super :: super :: rules :: r#kw_break :: < 'i , INHERITED > , super :: super :: rules :: r#kw_continue :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#Label :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirControl<'i, INHERITED> {
            #[doc = "A helper function to access [`Label`]."]
            #[allow(non_snake_case)]
            pub fn r#Label<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Label<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`LabelWithColon`]."]
            #[allow(non_snake_case)]
            pub fn r#LabelWithColon<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#LabelWithColon<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`kw_break`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_break<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_break<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res._0().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`kw_continue`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_continue<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_continue<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res._1().map(|res| res);
                        res
                    }
                }
            }
        }
        :: pest_typed :: rule ! (r#MirStmtBlock , "Corresponds to expression: `(LeftBrace ~ MirStmt* ~ RightBrace)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirStmtBlock , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: rules :: r#MirStmt :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirStmtBlock<'i, INHERITED> {
            #[doc = "A helper function to access [`LeftBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftBrace<'s>(&'s self) -> &'s super::super::rules::r#LeftBrace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirStmt`]."]
            #[allow(non_snake_case)]
            pub fn r#MirStmt<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Vec<&'s super::super::rules::r#MirStmt<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res
                            .content
                            .iter()
                            .map(|res| {
                                let res = &res.matched;
                                res
                            })
                            .collect::<::pest_typed::re_exported::Vec<_>>();
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`RightBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#RightBrace<'s>(&'s self) -> &'s super::super::rules::r#RightBrace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirLoop , "Corresponds to expression: `(LabelWithColon? ~ kw_loop ~ MirStmtBlock)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirLoop , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#LabelWithColon :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_loop :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirStmtBlock :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirLoop<'i, INHERITED> {
            #[doc = "A helper function to access [`LabelWithColon`]."]
            #[allow(non_snake_case)]
            pub fn r#LabelWithColon<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#LabelWithColon<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`MirStmtBlock`]."]
            #[allow(non_snake_case)]
            pub fn r#MirStmtBlock<'s>(&'s self) -> &'s super::super::rules::r#MirStmtBlock<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_loop`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_loop<'s>(&'s self) -> &'s super::super::rules::r#kw_loop<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirSwitchValue , "Corresponds to expression: `(Bool | Integer | PlaceHolder)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirSwitchValue , super :: super :: generics :: Choice3 :: < super :: super :: rules :: r#Bool :: < 'i , INHERITED > , super :: super :: rules :: r#Integer :: < 'i , INHERITED > , super :: super :: rules :: r#PlaceHolder :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirSwitchValue<'i, INHERITED> {
            #[doc = "A helper function to access [`Bool`]."]
            #[allow(non_snake_case)]
            pub fn r#Bool<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Bool<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`Integer`]."]
            #[allow(non_snake_case)]
            pub fn r#Integer<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Integer<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`PlaceHolder`]."]
            #[allow(non_snake_case)]
            pub fn r#PlaceHolder<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#PlaceHolder<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._2().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirSwitchBody , "Corresponds to expression: `(MirStmtBlock | ((MirCallIgnoreRet | MirDrop | MirControl | MirAssign) ~ Comma) | MirLoop | MirSwitchInt)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirSwitchBody , super :: super :: generics :: Choice4 :: < super :: super :: rules :: r#MirStmtBlock :: < 'i , INHERITED > , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Choice4 :: < super :: super :: rules :: r#MirCallIgnoreRet :: < 'i , INHERITED > , super :: super :: rules :: r#MirDrop :: < 'i , INHERITED > , super :: super :: rules :: r#MirControl :: < 'i , INHERITED > , super :: super :: rules :: r#MirAssign :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Comma :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: rules :: r#MirLoop :: < 'i , INHERITED > , super :: super :: rules :: r#MirSwitchInt :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirSwitchBody<'i, INHERITED> {
            #[doc = "A helper function to access [`Comma`]."]
            #[allow(non_snake_case)]
            pub fn r#Comma<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Comma<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| {
                        let res = &res.content.1.matched;
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`MirAssign`]."]
            #[allow(non_snake_case)]
            pub fn r#MirAssign<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirAssign<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res
                        ._1()
                        .map(|res| {
                            let res = &res.content.0.matched;
                            {
                                let res = res._3().map(|res| res);
                                res
                            }
                        })
                        .flatten();
                    res
                }
            }
            #[doc = "A helper function to access [`MirCallIgnoreRet`]."]
            #[allow(non_snake_case)]
            pub fn r#MirCallIgnoreRet<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirCallIgnoreRet<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res
                        ._1()
                        .map(|res| {
                            let res = &res.content.0.matched;
                            {
                                let res = res._0().map(|res| res);
                                res
                            }
                        })
                        .flatten();
                    res
                }
            }
            #[doc = "A helper function to access [`MirControl`]."]
            #[allow(non_snake_case)]
            pub fn r#MirControl<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirControl<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res
                        ._1()
                        .map(|res| {
                            let res = &res.content.0.matched;
                            {
                                let res = res._2().map(|res| res);
                                res
                            }
                        })
                        .flatten();
                    res
                }
            }
            #[doc = "A helper function to access [`MirDrop`]."]
            #[allow(non_snake_case)]
            pub fn r#MirDrop<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirDrop<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res
                        ._1()
                        .map(|res| {
                            let res = &res.content.0.matched;
                            {
                                let res = res._1().map(|res| res);
                                res
                            }
                        })
                        .flatten();
                    res
                }
            }
            #[doc = "A helper function to access [`MirLoop`]."]
            #[allow(non_snake_case)]
            pub fn r#MirLoop<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirLoop<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._2().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirStmtBlock`]."]
            #[allow(non_snake_case)]
            pub fn r#MirStmtBlock<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirStmtBlock<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirSwitchInt`]."]
            #[allow(non_snake_case)]
            pub fn r#MirSwitchInt<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirSwitchInt<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._3().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirSwitchTarget , "Corresponds to expression: `(MirSwitchValue ~ RightArrow ~ MirSwitchBody)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirSwitchTarget , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirSwitchValue :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightArrow :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirSwitchBody :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirSwitchTarget<'i, INHERITED> {
            #[doc = "A helper function to access [`MirSwitchBody`]."]
            #[allow(non_snake_case)]
            pub fn r#MirSwitchBody<'s>(&'s self) -> &'s super::super::rules::r#MirSwitchBody<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirSwitchValue`]."]
            #[allow(non_snake_case)]
            pub fn r#MirSwitchValue<'s>(&'s self) -> &'s super::super::rules::r#MirSwitchValue<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RightArrow`]."]
            #[allow(non_snake_case)]
            pub fn r#RightArrow<'s>(&'s self) -> &'s super::super::rules::r#RightArrow<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirSwitchInt , "Corresponds to expression: `(kw_switchInt ~ LeftParen ~ MirOperand ~ RightParen ~ LeftBrace ~ MirSwitchTarget* ~ RightBrace)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirSwitchInt , super :: super :: generics :: Seq7 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_switchInt :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirOperand :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: rules :: r#MirSwitchTarget :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirSwitchInt<'i, INHERITED> {
            #[doc = "A helper function to access [`LeftBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftBrace<'s>(&'s self) -> &'s super::super::rules::r#LeftBrace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.4.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`LeftParen`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftParen<'s>(&'s self) -> &'s super::super::rules::r#LeftParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirOperand`]."]
            #[allow(non_snake_case)]
            pub fn r#MirOperand<'s>(&'s self) -> &'s super::super::rules::r#MirOperand<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirSwitchTarget`]."]
            #[allow(non_snake_case)]
            pub fn r#MirSwitchTarget<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Vec<&'s super::super::rules::r#MirSwitchTarget<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.5.matched;
                    {
                        let res = res
                            .content
                            .iter()
                            .map(|res| {
                                let res = &res.matched;
                                res
                            })
                            .collect::<::pest_typed::re_exported::Vec<_>>();
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`RightBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#RightBrace<'s>(&'s self) -> &'s super::super::rules::r#RightBrace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.6.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RightParen`]."]
            #[allow(non_snake_case)]
            pub fn r#RightParen<'s>(&'s self) -> &'s super::super::rules::r#RightParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_switchInt`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_switchInt<'s>(&'s self) -> &'s super::super::rules::r#kw_switchInt<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirAssign , "Corresponds to expression: `(LabelWithColon? ~ MirPlace ~ Assign ~ MirRvalueOrCall)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirAssign , super :: super :: generics :: Seq4 :: < (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#LabelWithColon :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirPlace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Assign :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirRvalueOrCall :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirAssign<'i, INHERITED> {
            #[doc = "A helper function to access [`Assign`]."]
            #[allow(non_snake_case)]
            pub fn r#Assign<'s>(&'s self) -> &'s super::super::rules::r#Assign<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`LabelWithColon`]."]
            #[allow(non_snake_case)]
            pub fn r#LabelWithColon<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#LabelWithColon<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`MirPlace`]."]
            #[allow(non_snake_case)]
            pub fn r#MirPlace<'s>(&'s self) -> &'s super::super::rules::r#MirPlace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MirRvalueOrCall`]."]
            #[allow(non_snake_case)]
            pub fn r#MirRvalueOrCall<'s>(&'s self) -> &'s super::super::rules::r#MirRvalueOrCall<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirStmt , "Corresponds to expression: `((MirCallIgnoreRet ~ SemiColon) | (MirDrop ~ SemiColon) | (MirControl ~ SemiColon) | (MirAssign ~ SemiColon) | MirLoop | MirSwitchInt)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirStmt , super :: super :: generics :: Choice6 :: < super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirCallIgnoreRet :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#SemiColon :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirDrop :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#SemiColon :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirControl :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#SemiColon :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirAssign :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#SemiColon :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: rules :: r#MirLoop :: < 'i , INHERITED > , super :: super :: rules :: r#MirSwitchInt :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirStmt<'i, INHERITED> {
            #[doc = "A helper function to access [`MirAssign`]."]
            #[allow(non_snake_case)]
            pub fn r#MirAssign<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirAssign<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._3().map(|res| {
                        let res = &res.content.0.matched;
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`MirCallIgnoreRet`]."]
            #[allow(non_snake_case)]
            pub fn r#MirCallIgnoreRet<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirCallIgnoreRet<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| {
                        let res = &res.content.0.matched;
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`MirControl`]."]
            #[allow(non_snake_case)]
            pub fn r#MirControl<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirControl<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._2().map(|res| {
                        let res = &res.content.0.matched;
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`MirDrop`]."]
            #[allow(non_snake_case)]
            pub fn r#MirDrop<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirDrop<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| {
                        let res = &res.content.0.matched;
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`MirLoop`]."]
            #[allow(non_snake_case)]
            pub fn r#MirLoop<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirLoop<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._4().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MirSwitchInt`]."]
            #[allow(non_snake_case)]
            pub fn r#MirSwitchInt<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirSwitchInt<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._5().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`SemiColon`]."]
            #[allow(non_snake_case)]
            pub fn r#SemiColon<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#SemiColon<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#SemiColon<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#SemiColon<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#SemiColon<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = res._0().map(|res| {
                                let res = &res.content.1.matched;
                                res
                            });
                            res
                        },
                        {
                            let res = res._1().map(|res| {
                                let res = &res.content.1.matched;
                                res
                            });
                            res
                        },
                        {
                            let res = res._2().map(|res| {
                                let res = &res.content.1.matched;
                                res
                            });
                            res
                        },
                        {
                            let res = res._3().map(|res| {
                                let res = &res.content.1.matched;
                                res
                            });
                            res
                        },
                    );
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MirBody , "Corresponds to expression: `(MirDecl* ~ MirStmt*)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MirBody , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: rules :: r#MirDecl :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: rules :: r#MirStmt :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MirBody<'i, INHERITED> {
            #[doc = "A helper function to access [`MirDecl`]."]
            #[allow(non_snake_case)]
            pub fn r#MirDecl<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Vec<&'s super::super::rules::r#MirDecl<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    {
                        let res = res
                            .content
                            .iter()
                            .map(|res| {
                                let res = &res.matched;
                                res
                            })
                            .collect::<::pest_typed::re_exported::Vec<_>>();
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`MirStmt`]."]
            #[allow(non_snake_case)]
            pub fn r#MirStmt<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Vec<&'s super::super::rules::r#MirStmt<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res
                            .content
                            .iter()
                            .map(|res| {
                                let res = &res.matched;
                                res
                            })
                            .collect::<::pest_typed::re_exported::Vec<_>>();
                        res
                    }
                }
            }
        }
        :: pest_typed :: rule ! (r#FnName , "Corresponds to expression: `(PlaceHolder | MetaVariable | Identifier)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#FnName , super :: super :: generics :: Choice3 :: < super :: super :: rules :: r#PlaceHolder :: < 'i , INHERITED > , super :: super :: rules :: r#MetaVariable :: < 'i , INHERITED > , super :: super :: rules :: r#Identifier :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#FnName<'i, INHERITED> {
            #[doc = "A helper function to access [`Identifier`]."]
            #[allow(non_snake_case)]
            pub fn r#Identifier<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Identifier<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._2().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`MetaVariable`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariable<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MetaVariable<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`PlaceHolder`]."]
            #[allow(non_snake_case)]
            pub fn r#PlaceHolder<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#PlaceHolder<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#FnParamsSeparatedByComma , "Corresponds to expression: `(FnParam ~ (Comma ~ FnParam)* ~ Comma?)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#FnParamsSeparatedByComma , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#FnParam :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Comma :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#FnParam :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#Comma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#FnParamsSeparatedByComma<'i, INHERITED> {
            #[doc = "A helper function to access [`Comma`]."]
            #[allow(non_snake_case)]
            pub fn r#Comma<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#Comma<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#Comma<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.0.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                        {
                            let res = &res.content.2.matched;
                            {
                                let res = res.as_ref().map(|res| res);
                                res
                            }
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`FnParam`]."]
            #[allow(non_snake_case)]
            pub fn r#FnParam<'s>(
                &'s self,
            ) -> (
                &'s super::super::rules::r#FnParam<'i, INHERITED>,
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#FnParam<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.0.matched;
                            res
                        },
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.1.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                    );
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#FnSig , "Corresponds to expression: `(kw_unsafe? ~ kw_pub? ~ kw_fn ~ FnName ~ LeftParen ~ FnParamsSeparatedByComma? ~ RightParen ~ FnRet?)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#FnSig , super :: super :: generics :: Seq8 :: < (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#kw_unsafe :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#kw_pub :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_fn :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#FnName :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#FnParamsSeparatedByComma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#FnRet :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#FnSig<'i, INHERITED> {
            #[doc = "A helper function to access [`FnName`]."]
            #[allow(non_snake_case)]
            pub fn r#FnName<'s>(&'s self) -> &'s super::super::rules::r#FnName<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`FnParamsSeparatedByComma`]."]
            #[allow(non_snake_case)]
            pub fn r#FnParamsSeparatedByComma<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#FnParamsSeparatedByComma<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = &res.content.5.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`FnRet`]."]
            #[allow(non_snake_case)]
            pub fn r#FnRet<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#FnRet<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.7.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`LeftParen`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftParen<'s>(&'s self) -> &'s super::super::rules::r#LeftParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.4.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RightParen`]."]
            #[allow(non_snake_case)]
            pub fn r#RightParen<'s>(&'s self) -> &'s super::super::rules::r#RightParen<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.6.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_fn`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_fn<'s>(&'s self) -> &'s super::super::rules::r#kw_fn<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_pub`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_pub<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_pub<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`kw_unsafe`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_unsafe<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_unsafe<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
        }
        :: pest_typed :: rule ! (r#FnBody , "Corresponds to expression: `(SemiColon | (LeftBrace ~ MirBody ~ RightBrace))`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#FnBody , super :: super :: generics :: Choice2 :: < super :: super :: rules :: r#SemiColon :: < 'i , INHERITED > , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MirBody :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#FnBody<'i, INHERITED> {
            #[doc = "A helper function to access [`LeftBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftBrace<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#LeftBrace<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| {
                        let res = &res.content.0.matched;
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`MirBody`]."]
            #[allow(non_snake_case)]
            pub fn r#MirBody<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MirBody<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| {
                        let res = &res.content.1.matched;
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`RightBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#RightBrace<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#RightBrace<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| {
                        let res = &res.content.2.matched;
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`SemiColon`]."]
            #[allow(non_snake_case)]
            pub fn r#SemiColon<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#SemiColon<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#FnRet , "Corresponds to expression: `(Arrow ~ (PlaceHolder | Type))`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#FnRet , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Arrow :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Choice2 :: < super :: super :: rules :: r#PlaceHolder :: < 'i , INHERITED > , super :: super :: rules :: r#Type :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#FnRet<'i, INHERITED> {
            #[doc = "A helper function to access [`Arrow`]."]
            #[allow(non_snake_case)]
            pub fn r#Arrow<'s>(&'s self) -> &'s super::super::rules::r#Arrow<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`PlaceHolder`]."]
            #[allow(non_snake_case)]
            pub fn r#PlaceHolder<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#PlaceHolder<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res._0().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`Type`]."]
            #[allow(non_snake_case)]
            pub fn r#Type<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Type<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res._1().map(|res| res);
                        res
                    }
                }
            }
        }
        :: pest_typed :: rule ! (r#Fn , "Corresponds to expression: `(FnSig ~ FnBody)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Fn , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#FnSig :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#FnBody :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Fn<'i, INHERITED> {
            #[doc = "A helper function to access [`FnBody`]."]
            #[allow(non_snake_case)]
            pub fn r#FnBody<'s>(&'s self) -> &'s super::super::rules::r#FnBody<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`FnSig`]."]
            #[allow(non_snake_case)]
            pub fn r#FnSig<'s>(&'s self) -> &'s super::super::rules::r#FnSig<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#Field , "Corresponds to expression: `(MetaVariable ~ Colon ~ Type)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Field , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MetaVariable :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Colon :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Type :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Field<'i, INHERITED> {
            #[doc = "A helper function to access [`Colon`]."]
            #[allow(non_snake_case)]
            pub fn r#Colon<'s>(&'s self) -> &'s super::super::rules::r#Colon<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MetaVariable`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariable<'s>(&'s self) -> &'s super::super::rules::r#MetaVariable<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Type`]."]
            #[allow(non_snake_case)]
            pub fn r#Type<'s>(&'s self) -> &'s super::super::rules::r#Type<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#FieldsSeparatedByComma , "Corresponds to expression: `(Field ~ (Comma ~ Field)* ~ Comma?)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#FieldsSeparatedByComma , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Field :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Comma :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Field :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#Comma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#FieldsSeparatedByComma<'i, INHERITED> {
            #[doc = "A helper function to access [`Comma`]."]
            #[allow(non_snake_case)]
            pub fn r#Comma<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#Comma<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#Comma<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.0.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                        {
                            let res = &res.content.2.matched;
                            {
                                let res = res.as_ref().map(|res| res);
                                res
                            }
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`Field`]."]
            #[allow(non_snake_case)]
            pub fn r#Field<'s>(
                &'s self,
            ) -> (
                &'s super::super::rules::r#Field<'i, INHERITED>,
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#Field<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.0.matched;
                            res
                        },
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.1.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                    );
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#Struct , "Corresponds to expression: `(kw_pub? ~ kw_struct ~ MetaVariable ~ LeftBrace ~ FieldsSeparatedByComma? ~ RightBrace)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Struct , super :: super :: generics :: Seq6 :: < (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#kw_pub :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_struct :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MetaVariable :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#FieldsSeparatedByComma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Struct<'i, INHERITED> {
            #[doc = "A helper function to access [`FieldsSeparatedByComma`]."]
            #[allow(non_snake_case)]
            pub fn r#FieldsSeparatedByComma<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#FieldsSeparatedByComma<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = &res.content.4.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`LeftBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftBrace<'s>(&'s self) -> &'s super::super::rules::r#LeftBrace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MetaVariable`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariable<'s>(&'s self) -> &'s super::super::rules::r#MetaVariable<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RightBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#RightBrace<'s>(&'s self) -> &'s super::super::rules::r#RightBrace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.5.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_pub`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_pub<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#kw_pub<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`kw_struct`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_struct<'s>(&'s self) -> &'s super::super::rules::r#kw_struct<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#EnumVariant , "Corresponds to expression: `((Identifier ~ LeftBrace ~ FieldsSeparatedByComma? ~ RightBrace) | (Identifier ~ LeftParen ~ Type ~ RightParen) | Identifier)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#EnumVariant , super :: super :: generics :: Choice3 :: < super :: super :: generics :: Seq4 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Identifier :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#FieldsSeparatedByComma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Seq4 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Identifier :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Type :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: rules :: r#Identifier :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#EnumVariant<'i, INHERITED> {
            #[doc = "A helper function to access [`FieldsSeparatedByComma`]."]
            #[allow(non_snake_case)]
            pub fn r#FieldsSeparatedByComma<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#FieldsSeparatedByComma<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res
                        ._0()
                        .map(|res| {
                            let res = &res.content.2.matched;
                            {
                                let res = res.as_ref().map(|res| res);
                                res
                            }
                        })
                        .flatten();
                    res
                }
            }
            #[doc = "A helper function to access [`Identifier`]."]
            #[allow(non_snake_case)]
            pub fn r#Identifier<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#Identifier<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#Identifier<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#Identifier<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = res._0().map(|res| {
                                let res = &res.content.0.matched;
                                res
                            });
                            res
                        },
                        {
                            let res = res._1().map(|res| {
                                let res = &res.content.0.matched;
                                res
                            });
                            res
                        },
                        {
                            let res = res._2().map(|res| res);
                            res
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`LeftBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftBrace<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#LeftBrace<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| {
                        let res = &res.content.1.matched;
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`LeftParen`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftParen<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#LeftParen<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| {
                        let res = &res.content.1.matched;
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`RightBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#RightBrace<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#RightBrace<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| {
                        let res = &res.content.3.matched;
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`RightParen`]."]
            #[allow(non_snake_case)]
            pub fn r#RightParen<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#RightParen<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| {
                        let res = &res.content.3.matched;
                        res
                    });
                    res
                }
            }
            #[doc = "A helper function to access [`Type`]."]
            #[allow(non_snake_case)]
            pub fn r#Type<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Type<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| {
                        let res = &res.content.2.matched;
                        res
                    });
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#EnumVariantsSeparatedByComma , "Corresponds to expression: `(EnumVariant ~ (Comma ~ EnumVariant)* ~ Comma?)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#EnumVariantsSeparatedByComma , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#EnumVariant :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Comma :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#EnumVariant :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#Comma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#EnumVariantsSeparatedByComma<'i, INHERITED> {
            #[doc = "A helper function to access [`Comma`]."]
            #[allow(non_snake_case)]
            pub fn r#Comma<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#Comma<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#Comma<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.0.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                        {
                            let res = &res.content.2.matched;
                            {
                                let res = res.as_ref().map(|res| res);
                                res
                            }
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`EnumVariant`]."]
            #[allow(non_snake_case)]
            pub fn r#EnumVariant<'s>(
                &'s self,
            ) -> (
                &'s super::super::rules::r#EnumVariant<'i, INHERITED>,
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#EnumVariant<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.0.matched;
                            res
                        },
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.1.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                    );
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#Enum , "Corresponds to expression: `(kw_enum ~ MetaVariable ~ LeftBrace ~ EnumVariantsSeparatedByComma? ~ RightBrace)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Enum , super :: super :: generics :: Seq5 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_enum :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MetaVariable :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#EnumVariantsSeparatedByComma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Enum<'i, INHERITED> {
            #[doc = "A helper function to access [`EnumVariantsSeparatedByComma`]."]
            #[allow(non_snake_case)]
            pub fn r#EnumVariantsSeparatedByComma<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#EnumVariantsSeparatedByComma<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`LeftBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftBrace<'s>(&'s self) -> &'s super::super::rules::r#LeftBrace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MetaVariable`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariable<'s>(&'s self) -> &'s super::super::rules::r#MetaVariable<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RightBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#RightBrace<'s>(&'s self) -> &'s super::super::rules::r#RightBrace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.4.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_enum`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_enum<'s>(&'s self) -> &'s super::super::rules::r#kw_enum<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#ImplKind , "Corresponds to expression: `(Path ~ kw_for)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#ImplKind , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Path :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_for :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#ImplKind<'i, INHERITED> {
            #[doc = "A helper function to access [`Path`]."]
            #[allow(non_snake_case)]
            pub fn r#Path<'s>(&'s self) -> &'s super::super::rules::r#Path<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_for`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_for<'s>(&'s self) -> &'s super::super::rules::r#kw_for<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#Impl , "Corresponds to expression: `(kw_impl ~ ImplKind? ~ Type ~ LeftBrace ~ Fn* ~ RightBrace)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Impl , super :: super :: generics :: Seq6 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_impl :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#ImplKind :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Type :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: rules :: r#Fn :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Impl<'i, INHERITED> {
            #[doc = "A helper function to access [`Fn`]."]
            #[allow(non_snake_case)]
            pub fn r#Fn<'s>(&'s self) -> ::pest_typed::re_exported::Vec<&'s super::super::rules::r#Fn<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.4.matched;
                    {
                        let res = res
                            .content
                            .iter()
                            .map(|res| {
                                let res = &res.matched;
                                res
                            })
                            .collect::<::pest_typed::re_exported::Vec<_>>();
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`ImplKind`]."]
            #[allow(non_snake_case)]
            pub fn r#ImplKind<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#ImplKind<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`LeftBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftBrace<'s>(&'s self) -> &'s super::super::rules::r#LeftBrace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RightBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#RightBrace<'s>(&'s self) -> &'s super::super::rules::r#RightBrace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.5.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Type`]."]
            #[allow(non_snake_case)]
            pub fn r#Type<'s>(&'s self) -> &'s super::super::rules::r#Type<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_impl`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_impl<'s>(&'s self) -> &'s super::super::rules::r#kw_impl<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#RustItem , "Corresponds to expression: `(Fn | Struct | Enum | Impl)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#RustItem , super :: super :: generics :: Choice4 :: < super :: super :: rules :: r#Fn :: < 'i , INHERITED > , super :: super :: rules :: r#Struct :: < 'i , INHERITED > , super :: super :: rules :: r#Enum :: < 'i , INHERITED > , super :: super :: rules :: r#Impl :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#RustItem<'i, INHERITED> {
            #[doc = "A helper function to access [`Enum`]."]
            #[allow(non_snake_case)]
            pub fn r#Enum<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Enum<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._2().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`Fn`]."]
            #[allow(non_snake_case)]
            pub fn r#Fn<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Fn<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`Impl`]."]
            #[allow(non_snake_case)]
            pub fn r#Impl<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Impl<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._3().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`Struct`]."]
            #[allow(non_snake_case)]
            pub fn r#Struct<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Struct<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#RustItems , "Corresponds to expression: `(LeftBrace ~ RustItem+ ~ RightBrace)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#RustItems , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: RepOnce :: < 'i , INHERITED , super :: super :: rules :: r#RustItem :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#RustItems<'i, INHERITED> {
            #[doc = "A helper function to access [`LeftBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftBrace<'s>(&'s self) -> &'s super::super::rules::r#LeftBrace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RightBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#RightBrace<'s>(&'s self) -> &'s super::super::rules::r#RightBrace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RustItem`]."]
            #[allow(non_snake_case)]
            pub fn r#RustItem<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Vec<&'s super::super::rules::r#RustItem<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res
                            .content
                            .iter()
                            .map(|res| {
                                let res = &res.matched;
                                res
                            })
                            .collect::<::pest_typed::re_exported::Vec<_>>();
                        res
                    }
                }
            }
        }
        :: pest_typed :: rule ! (r#PatternConfiguration , "Corresponds to expression: `(Identifier ~ MetaVariableAssignList)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#PatternConfiguration , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Identifier :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MetaVariableAssignList :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#PatternConfiguration<'i, INHERITED> {
            #[doc = "A helper function to access [`Identifier`]."]
            #[allow(non_snake_case)]
            pub fn r#Identifier<'s>(&'s self) -> &'s super::super::rules::r#Identifier<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MetaVariableAssignList`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariableAssignList<'s>(
                &'s self,
            ) -> &'s super::super::rules::r#MetaVariableAssignList<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#PatternExpression , "Corresponds to expression: `(PatternConfiguration ~ (Minus ~ PatternConfiguration)*)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#PatternExpression , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#PatternConfiguration :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Minus :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#PatternConfiguration :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#PatternExpression<'i, INHERITED> {
            #[doc = "A helper function to access [`Minus`]."]
            #[allow(non_snake_case)]
            pub fn r#Minus<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Vec<&'s super::super::rules::r#Minus<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res
                            .content
                            .iter()
                            .map(|res| {
                                let res = &res.matched;
                                {
                                    let res = &res.content.0.matched;
                                    res
                                }
                            })
                            .collect::<::pest_typed::re_exported::Vec<_>>();
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`PatternConfiguration`]."]
            #[allow(non_snake_case)]
            pub fn r#PatternConfiguration<'s>(
                &'s self,
            ) -> (
                &'s super::super::rules::r#PatternConfiguration<'i, INHERITED>,
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#PatternConfiguration<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.0.matched;
                            res
                        },
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.1.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                    );
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#PatternOperation , "Corresponds to expression: `PatternExpression`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#PatternOperation , super :: super :: rules :: r#PatternExpression :: < 'i , INHERITED > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#PatternOperation<'i, INHERITED> {
            #[doc = "A helper function to access [`PatternExpression`]."]
            #[allow(non_snake_case)]
            pub fn r#PatternExpression<'s>(&'s self) -> &'s super::super::rules::r#PatternExpression<'i, INHERITED> {
                let res = &*self.content;
                res
            }
        }
        :: pest_typed :: rule ! (r#RustItemsOrPatternOperation , "Corresponds to expression: `(RustItem | RustItems | PatternOperation)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#RustItemsOrPatternOperation , super :: super :: generics :: Choice3 :: < super :: super :: rules :: r#RustItem :: < 'i , INHERITED > , super :: super :: rules :: r#RustItems :: < 'i , INHERITED > , super :: super :: rules :: r#PatternOperation :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#RustItemsOrPatternOperation<'i, INHERITED> {
            #[doc = "A helper function to access [`PatternOperation`]."]
            #[allow(non_snake_case)]
            pub fn r#PatternOperation<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#PatternOperation<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = res._2().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`RustItem`]."]
            #[allow(non_snake_case)]
            pub fn r#RustItem<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#RustItem<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`RustItems`]."]
            #[allow(non_snake_case)]
            pub fn r#RustItems<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#RustItems<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#RPLPatternItem , "Corresponds to expression: `(Identifier ~ MetaVariableDeclList? ~ Assign ~ RustItemsOrPatternOperation ~ WhereBlock?)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#RPLPatternItem , super :: super :: generics :: Seq5 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Identifier :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#MetaVariableDeclList :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Assign :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RustItemsOrPatternOperation :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#WhereBlock :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#RPLPatternItem<'i, INHERITED> {
            #[doc = "A helper function to access [`Assign`]."]
            #[allow(non_snake_case)]
            pub fn r#Assign<'s>(&'s self) -> &'s super::super::rules::r#Assign<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Identifier`]."]
            #[allow(non_snake_case)]
            pub fn r#Identifier<'s>(&'s self) -> &'s super::super::rules::r#Identifier<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MetaVariableDeclList`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariableDeclList<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#MetaVariableDeclList<'i, INHERITED>>
            {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`RustItemsOrPatternOperation`]."]
            #[allow(non_snake_case)]
            pub fn r#RustItemsOrPatternOperation<'s>(
                &'s self,
            ) -> &'s super::super::rules::r#RustItemsOrPatternOperation<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`WhereBlock`]."]
            #[allow(non_snake_case)]
            pub fn r#WhereBlock<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#WhereBlock<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.4.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
        }
        :: pest_typed :: rule ! (r#MetaVariableWithDiagMessage , "Corresponds to expression: `(MetaVariable ~ Colon ~ String)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MetaVariableWithDiagMessage , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MetaVariable :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Colon :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#String :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MetaVariableWithDiagMessage<'i, INHERITED> {
            #[doc = "A helper function to access [`Colon`]."]
            #[allow(non_snake_case)]
            pub fn r#Colon<'s>(&'s self) -> &'s super::super::rules::r#Colon<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MetaVariable`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariable<'s>(&'s self) -> &'s super::super::rules::r#MetaVariable<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`String`]."]
            #[allow(non_snake_case)]
            pub fn r#String<'s>(&'s self) -> &'s super::super::rules::r#String<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#MetaVariableWithDiagMessageSeparatedByComma , "Corresponds to expression: `(MetaVariableWithDiagMessage ~ (Comma ~ MetaVariableWithDiagMessage)* ~ Comma?)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#MetaVariableWithDiagMessageSeparatedByComma , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MetaVariableWithDiagMessage :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Comma :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MetaVariableWithDiagMessage :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#Comma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#MetaVariableWithDiagMessageSeparatedByComma<'i, INHERITED> {
            #[doc = "A helper function to access [`Comma`]."]
            #[allow(non_snake_case)]
            pub fn r#Comma<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#Comma<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#Comma<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.0.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                        {
                            let res = &res.content.2.matched;
                            {
                                let res = res.as_ref().map(|res| res);
                                res
                            }
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`MetaVariableWithDiagMessage`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariableWithDiagMessage<'s>(
                &'s self,
            ) -> (
                &'s super::super::rules::r#MetaVariableWithDiagMessage<'i, INHERITED>,
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#MetaVariableWithDiagMessage<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.0.matched;
                            res
                        },
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.1.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                    );
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#diagMessageArg , "Corresponds to expression: `(LeftBrace ~ MetaVariable ~ RightBrace)`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#diagMessageArg , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftBrace :: < 'i , 0 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#MetaVariable :: < 'i , 0 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightBrace :: < 'i , 0 > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#diagMessageArg<'i, INHERITED> {
            #[doc = "A helper function to access [`LeftBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftBrace<'s>(&'s self) -> &'s super::super::rules::r#LeftBrace<'i, 0> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MetaVariable`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariable<'s>(&'s self) -> &'s super::super::rules::r#MetaVariable<'i, 0> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RightBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#RightBrace<'s>(&'s self) -> &'s super::super::rules::r#RightBrace<'i, 0> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#diagMessageText , "Corresponds to expression: `(!(\"\\\"\" | LeftBrace) ~ ANY)+`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#diagMessageText , super :: super :: generics :: RepOnce :: < 'i , 0 , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Negative :: < super :: super :: generics :: Choice2 :: < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_129 > , super :: super :: rules :: r#LeftBrace :: < 'i , 0 > , > > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#ANY , super :: super :: generics :: Skipped < 'i > , 0 >) , > > , super :: super :: generics :: Skipped :: < 'i > , true , Span , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#diagMessageText<'i, INHERITED> {}
        :: pest_typed :: rule ! (r#diagMessageInner , "Corresponds to expression: `(diagMessageArg | diagMessageText)+`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#diagMessageInner , super :: super :: generics :: RepOnce :: < 'i , 0 , super :: super :: generics :: Choice2 :: < super :: super :: rules :: r#diagMessageArg :: < 'i , 0 > , super :: super :: rules :: r#diagMessageText :: < 'i , 0 > , > > , super :: super :: generics :: Skipped :: < 'i > , true , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#diagMessageInner<'i, INHERITED> {
            #[doc = "A helper function to access [`diagMessageArg`]."]
            #[allow(non_snake_case)]
            pub fn r#diagMessageArg<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Vec<
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#diagMessageArg<'i, 0>>,
            > {
                let res = &*self.content;
                {
                    let res = res
                        .content
                        .iter()
                        .map(|res| {
                            let res = &res.matched;
                            {
                                let res = res._0().map(|res| res);
                                res
                            }
                        })
                        .collect::<::pest_typed::re_exported::Vec<_>>();
                    res
                }
            }
            #[doc = "A helper function to access [`diagMessageText`]."]
            #[allow(non_snake_case)]
            pub fn r#diagMessageText<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Vec<
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#diagMessageText<'i, 0>>,
            > {
                let res = &*self.content;
                {
                    let res = res
                        .content
                        .iter()
                        .map(|res| {
                            let res = &res.matched;
                            {
                                let res = res._1().map(|res| res);
                                res
                            }
                        })
                        .collect::<::pest_typed::re_exported::Vec<_>>();
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#diagMessage , "Corresponds to expression: `(\"\\\"\" ~ diagMessageInner ~ \"\\\"\")`. Atomic rule." "" , super :: super :: Rule , super :: super :: Rule :: r#diagMessage , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_130 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#diagMessageInner :: < 'i , 0 > , super :: super :: generics :: Skipped < 'i > , 0 >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Str :: < super :: super :: constant_wrappers :: r#w_131 > , super :: super :: generics :: Skipped < 'i > , 0 >) , > , super :: super :: generics :: Skipped :: < 'i > , true , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#diagMessage<'i, INHERITED> {
            #[doc = "A helper function to access [`diagMessageInner`]."]
            #[allow(non_snake_case)]
            pub fn r#diagMessageInner<'s>(&'s self) -> &'s super::super::rules::r#diagMessageInner<'i, 0> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#diagAttr , "Corresponds to expression: `(Identifier ~ ((LeftParen ~ diagAttrs? ~ RightParen) | (Assign ~ diagMessage))?)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#diagAttr , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Identifier :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: generics :: Choice2 :: < super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#diagAttrs :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Assign :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#diagMessage :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#diagAttr<'i, INHERITED> {
            #[doc = "A helper function to access [`Assign`]."]
            #[allow(non_snake_case)]
            pub fn r#Assign<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#Assign<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res
                            .as_ref()
                            .map(|res| {
                                let res = res._1().map(|res| {
                                    let res = &res.content.0.matched;
                                    res
                                });
                                res
                            })
                            .flatten();
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`Identifier`]."]
            #[allow(non_snake_case)]
            pub fn r#Identifier<'s>(&'s self) -> &'s super::super::rules::r#Identifier<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`LeftParen`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftParen<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#LeftParen<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res
                            .as_ref()
                            .map(|res| {
                                let res = res._0().map(|res| {
                                    let res = &res.content.0.matched;
                                    res
                                });
                                res
                            })
                            .flatten();
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`RightParen`]."]
            #[allow(non_snake_case)]
            pub fn r#RightParen<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#RightParen<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res
                            .as_ref()
                            .map(|res| {
                                let res = res._0().map(|res| {
                                    let res = &res.content.2.matched;
                                    res
                                });
                                res
                            })
                            .flatten();
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`diagAttrs`]."]
            #[allow(non_snake_case)]
            pub fn r#diagAttrs<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#diagAttrs<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res
                            .as_ref()
                            .map(|res| {
                                let res = res
                                    ._0()
                                    .map(|res| {
                                        let res = &res.content.1.matched;
                                        {
                                            let res = res.as_ref().map(|res| res);
                                            res
                                        }
                                    })
                                    .flatten();
                                res
                            })
                            .flatten();
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`diagMessage`]."]
            #[allow(non_snake_case)]
            pub fn r#diagMessage<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#diagMessage<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res
                            .as_ref()
                            .map(|res| {
                                let res = res._1().map(|res| {
                                    let res = &res.content.1.matched;
                                    res
                                });
                                res
                            })
                            .flatten();
                        res
                    }
                }
            }
        }
        :: pest_typed :: rule ! (r#diagAttrs , "Corresponds to expression: `(diagAttr ~ (Comma ~ diagAttr)* ~ Comma?)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#diagAttrs , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#diagAttr :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Comma :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#diagAttr :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#Comma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#diagAttrs<'i, INHERITED> {
            #[doc = "A helper function to access [`Comma`]."]
            #[allow(non_snake_case)]
            pub fn r#Comma<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#Comma<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#Comma<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.0.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                        {
                            let res = &res.content.2.matched;
                            {
                                let res = res.as_ref().map(|res| res);
                                res
                            }
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`diagAttr`]."]
            #[allow(non_snake_case)]
            pub fn r#diagAttr<'s>(
                &'s self,
            ) -> (
                &'s super::super::rules::r#diagAttr<'i, INHERITED>,
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#diagAttr<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.0.matched;
                            res
                        },
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.1.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                    );
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#diagItem , "Corresponds to expression: `(Identifier ~ (LeftParen ~ diagAttrs ~ RightParen)? ~ Assign ~ diagMessage)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#diagItem , super :: super :: generics :: Seq4 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Identifier :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#diagAttrs :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightParen :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Assign :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#diagMessage :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#diagItem<'i, INHERITED> {
            #[doc = "A helper function to access [`Assign`]."]
            #[allow(non_snake_case)]
            pub fn r#Assign<'s>(&'s self) -> &'s super::super::rules::r#Assign<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Identifier`]."]
            #[allow(non_snake_case)]
            pub fn r#Identifier<'s>(&'s self) -> &'s super::super::rules::r#Identifier<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`LeftParen`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftParen<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#LeftParen<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res.as_ref().map(|res| {
                            let res = &res.content.0.matched;
                            res
                        });
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`RightParen`]."]
            #[allow(non_snake_case)]
            pub fn r#RightParen<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#RightParen<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res.as_ref().map(|res| {
                            let res = &res.content.2.matched;
                            res
                        });
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`diagAttrs`]."]
            #[allow(non_snake_case)]
            pub fn r#diagAttrs<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#diagAttrs<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res.as_ref().map(|res| {
                            let res = &res.content.1.matched;
                            res
                        });
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`diagMessage`]."]
            #[allow(non_snake_case)]
            pub fn r#diagMessage<'s>(&'s self) -> &'s super::super::rules::r#diagMessage<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#diagItems , "Corresponds to expression: `(diagItem ~ (Comma ~ diagItem)* ~ Comma?)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#diagItems , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#diagItem :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Comma :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#diagItem :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#Comma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#diagItems<'i, INHERITED> {
            #[doc = "A helper function to access [`Comma`]."]
            #[allow(non_snake_case)]
            pub fn r#Comma<'s>(
                &'s self,
            ) -> (
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#Comma<'i, INHERITED>>,
                ::pest_typed::re_exported::Option<&'s super::super::rules::r#Comma<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.0.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                        {
                            let res = &res.content.2.matched;
                            {
                                let res = res.as_ref().map(|res| res);
                                res
                            }
                        },
                    );
                    res
                }
            }
            #[doc = "A helper function to access [`diagItem`]."]
            #[allow(non_snake_case)]
            pub fn r#diagItem<'s>(
                &'s self,
            ) -> (
                &'s super::super::rules::r#diagItem<'i, INHERITED>,
                ::pest_typed::re_exported::Vec<&'s super::super::rules::r#diagItem<'i, INHERITED>>,
            ) {
                let res = &*self.content;
                {
                    let res = (
                        {
                            let res = &res.content.0.matched;
                            res
                        },
                        {
                            let res = &res.content.1.matched;
                            {
                                let res = res
                                    .content
                                    .iter()
                                    .map(|res| {
                                        let res = &res.matched;
                                        {
                                            let res = &res.content.1.matched;
                                            res
                                        }
                                    })
                                    .collect::<::pest_typed::re_exported::Vec<_>>();
                                res
                            }
                        },
                    );
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#diagBlockItem , "Corresponds to expression: `(Identifier ~ Assign ~ LeftBrace ~ diagItems ~ MetaVariableWithDiagMessageSeparatedByComma? ~ RightBrace)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#diagBlockItem , super :: super :: generics :: Seq6 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Identifier :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Assign :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#diagItems :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < :: pest_typed :: re_exported :: Option :: < super :: super :: rules :: r#MetaVariableWithDiagMessageSeparatedByComma :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#diagBlockItem<'i, INHERITED> {
            #[doc = "A helper function to access [`Assign`]."]
            #[allow(non_snake_case)]
            pub fn r#Assign<'s>(&'s self) -> &'s super::super::rules::r#Assign<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`Identifier`]."]
            #[allow(non_snake_case)]
            pub fn r#Identifier<'s>(&'s self) -> &'s super::super::rules::r#Identifier<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`LeftBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftBrace<'s>(&'s self) -> &'s super::super::rules::r#LeftBrace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`MetaVariableWithDiagMessageSeparatedByComma`]."]
            #[allow(non_snake_case)]
            pub fn r#MetaVariableWithDiagMessageSeparatedByComma<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<
                &'s super::super::rules::r#MetaVariableWithDiagMessageSeparatedByComma<'i, INHERITED>,
            > {
                let res = &*self.content;
                {
                    let res = &res.content.4.matched;
                    {
                        let res = res.as_ref().map(|res| res);
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`RightBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#RightBrace<'s>(&'s self) -> &'s super::super::rules::r#RightBrace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.5.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`diagItems`]."]
            #[allow(non_snake_case)]
            pub fn r#diagItems<'s>(&'s self) -> &'s super::super::rules::r#diagItems<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#pattBlock , "Corresponds to expression: `(kw_patt ~ LeftBrace ~ UsePath* ~ RPLPatternItem* ~ RightBrace)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#pattBlock , super :: super :: generics :: Seq5 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_patt :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: rules :: r#UsePath :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: rules :: r#RPLPatternItem :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#pattBlock<'i, INHERITED> {
            #[doc = "A helper function to access [`LeftBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftBrace<'s>(&'s self) -> &'s super::super::rules::r#LeftBrace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RPLPatternItem`]."]
            #[allow(non_snake_case)]
            pub fn r#RPLPatternItem<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Vec<&'s super::super::rules::r#RPLPatternItem<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    {
                        let res = res
                            .content
                            .iter()
                            .map(|res| {
                                let res = &res.matched;
                                res
                            })
                            .collect::<::pest_typed::re_exported::Vec<_>>();
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`RightBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#RightBrace<'s>(&'s self) -> &'s super::super::rules::r#RightBrace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.4.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`UsePath`]."]
            #[allow(non_snake_case)]
            pub fn r#UsePath<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Vec<&'s super::super::rules::r#UsePath<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    {
                        let res = res
                            .content
                            .iter()
                            .map(|res| {
                                let res = &res.matched;
                                res
                            })
                            .collect::<::pest_typed::re_exported::Vec<_>>();
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`kw_patt`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_patt<'s>(&'s self) -> &'s super::super::rules::r#kw_patt<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#utilBlock , "Corresponds to expression: `(kw_util ~ LeftBrace ~ UsePath* ~ RPLPatternItem* ~ RightBrace)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#utilBlock , super :: super :: generics :: Seq5 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_util :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: rules :: r#UsePath :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: rules :: r#RPLPatternItem :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#utilBlock<'i, INHERITED> {
            #[doc = "A helper function to access [`LeftBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftBrace<'s>(&'s self) -> &'s super::super::rules::r#LeftBrace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RPLPatternItem`]."]
            #[allow(non_snake_case)]
            pub fn r#RPLPatternItem<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Vec<&'s super::super::rules::r#RPLPatternItem<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    {
                        let res = res
                            .content
                            .iter()
                            .map(|res| {
                                let res = &res.matched;
                                res
                            })
                            .collect::<::pest_typed::re_exported::Vec<_>>();
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`RightBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#RightBrace<'s>(&'s self) -> &'s super::super::rules::r#RightBrace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.4.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`UsePath`]."]
            #[allow(non_snake_case)]
            pub fn r#UsePath<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Vec<&'s super::super::rules::r#UsePath<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    {
                        let res = res
                            .content
                            .iter()
                            .map(|res| {
                                let res = &res.matched;
                                res
                            })
                            .collect::<::pest_typed::re_exported::Vec<_>>();
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`kw_util`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_util<'s>(&'s self) -> &'s super::super::rules::r#kw_util<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#diagBlock , "Corresponds to expression: `(kw_diag ~ LeftBrace ~ diagBlockItem* ~ RightBrace)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#diagBlock , super :: super :: generics :: Seq4 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_diag :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#LeftBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: rules :: r#diagBlockItem :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RightBrace :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#diagBlock<'i, INHERITED> {
            #[doc = "A helper function to access [`LeftBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#LeftBrace<'s>(&'s self) -> &'s super::super::rules::r#LeftBrace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RightBrace`]."]
            #[allow(non_snake_case)]
            pub fn r#RightBrace<'s>(&'s self) -> &'s super::super::rules::r#RightBrace<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.3.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`diagBlockItem`]."]
            #[allow(non_snake_case)]
            pub fn r#diagBlockItem<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Vec<&'s super::super::rules::r#diagBlockItem<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    {
                        let res = res
                            .content
                            .iter()
                            .map(|res| {
                                let res = &res.matched;
                                res
                            })
                            .collect::<::pest_typed::re_exported::Vec<_>>();
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`kw_diag`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_diag<'s>(&'s self) -> &'s super::super::rules::r#kw_diag<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#Block , "Corresponds to expression: `(pattBlock | utilBlock | diagBlock)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#Block , super :: super :: generics :: Choice3 :: < super :: super :: rules :: r#pattBlock :: < 'i , INHERITED > , super :: super :: rules :: r#utilBlock :: < 'i , INHERITED > , super :: super :: rules :: r#diagBlock :: < 'i , INHERITED > , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Expression , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#Block<'i, INHERITED> {
            #[doc = "A helper function to access [`diagBlock`]."]
            #[allow(non_snake_case)]
            pub fn r#diagBlock<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#diagBlock<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._2().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`pattBlock`]."]
            #[allow(non_snake_case)]
            pub fn r#pattBlock<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#pattBlock<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._0().map(|res| res);
                    res
                }
            }
            #[doc = "A helper function to access [`utilBlock`]."]
            #[allow(non_snake_case)]
            pub fn r#utilBlock<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Option<&'s super::super::rules::r#utilBlock<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = res._1().map(|res| res);
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#RPLHeader , "Corresponds to expression: `(kw_pattern ~ Identifier)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#RPLHeader , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#kw_pattern :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#Identifier :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#RPLHeader<'i, INHERITED> {
            #[doc = "A helper function to access [`Identifier`]."]
            #[allow(non_snake_case)]
            pub fn r#Identifier<'s>(&'s self) -> &'s super::super::rules::r#Identifier<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`kw_pattern`]."]
            #[allow(non_snake_case)]
            pub fn r#kw_pattern<'s>(&'s self) -> &'s super::super::rules::r#kw_pattern<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#RPLPattern , "Corresponds to expression: `(RPLHeader ~ Block*)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#RPLPattern , super :: super :: generics :: Seq2 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RPLHeader :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: generics :: Rep :: < 'i , INHERITED , super :: super :: rules :: r#Block :: < 'i , INHERITED > > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#RPLPattern<'i, INHERITED> {
            #[doc = "A helper function to access [`Block`]."]
            #[allow(non_snake_case)]
            pub fn r#Block<'s>(
                &'s self,
            ) -> ::pest_typed::re_exported::Vec<&'s super::super::rules::r#Block<'i, INHERITED>> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    {
                        let res = res
                            .content
                            .iter()
                            .map(|res| {
                                let res = &res.matched;
                                res
                            })
                            .collect::<::pest_typed::re_exported::Vec<_>>();
                        res
                    }
                }
            }
            #[doc = "A helper function to access [`RPLHeader`]."]
            #[allow(non_snake_case)]
            pub fn r#RPLHeader<'s>(&'s self) -> &'s super::super::rules::r#RPLHeader<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
        }
        :: pest_typed :: rule ! (r#main , "Corresponds to expression: `(SOI ~ RPLPattern ~ EOI)`. Normal rule." "" , super :: super :: Rule , super :: super :: Rule :: r#main , super :: super :: generics :: Seq3 :: < (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#SOI , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#RPLPattern :: < 'i , INHERITED > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , (:: pest_typed :: predefined_node :: Skipped < super :: super :: rules :: r#EOI :: < 'i > , super :: super :: generics :: Skipped < 'i > , INHERITED >) , > , super :: super :: generics :: Skipped :: < 'i > , INHERITED , Both , true);
        impl<'i, const INHERITED: ::core::primitive::usize> r#main<'i, INHERITED> {
            #[doc = "A helper function to access [`EOI`]."]
            #[allow(non_snake_case)]
            pub fn r#EOI<'s>(&'s self) -> &'s super::super::rules::r#EOI<'i> {
                let res = &*self.content;
                {
                    let res = &res.content.2.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`RPLPattern`]."]
            #[allow(non_snake_case)]
            pub fn r#RPLPattern<'s>(&'s self) -> &'s super::super::rules::r#RPLPattern<'i, INHERITED> {
                let res = &*self.content;
                {
                    let res = &res.content.1.matched;
                    res
                }
            }
            #[doc = "A helper function to access [`SOI`]."]
            #[allow(non_snake_case)]
            pub fn r#SOI<'s>(&'s self) -> &'s super::super::rules::r#SOI {
                let res = &*self.content;
                {
                    let res = &res.content.0.matched;
                    res
                }
            }
        }
        #[allow(unused_imports)]
        use super::super::unicode::*;
        ::pest_typed::rule_eoi!(EOI, super::super::Rule);
        pub use ::pest_typed::predefined_node::{ANY, NEWLINE, SOI};
    }
}
pub use rules_impl::rules;
#[doc = "Used generics."]
pub mod generics {
    use ::pest_typed::predefined_node;
    #[doc = r" Skipped content."]
    pub type Skipped<'i> = predefined_node::AtomicRepeat<
        ::pest_typed::choices::Choice2<super::rules::WHITESPACE<'i, 0>, super::rules::COMMENT<'i, 0>>,
    >;
    pub use pest_typed::choices::{Choice2, Choice3, Choice4, Choice5, Choice6, Choice10};
    pub use pest_typed::sequence::{Seq2, Seq3, Seq4, Seq5, Seq6, Seq7, Seq8};
    pub use predefined_node::{CharRange, Insens, Negative, PeekSlice1, PeekSlice2, Positive, Push, Skip, Str};
    pest_typed::choices!(
        Choice12, choice12, 12usize, T0, _0, T1, _1, T2, _2, T3, _3, T4, _4, T5, _5, T6, _6, T7, _7, T8, _8, T9, _9,
        T10, _10, T11, _11,
    );
    pest_typed::choices!(
        Choice14, choice14, 14usize, T0, _0, T1, _1, T2, _2, T3, _3, T4, _4, T5, _5, T6, _6, T7, _7, T8, _8, T9, _9,
        T10, _10, T11, _11, T12, _12, T13, _13,
    );
    pest_typed::choices!(
        Choice74, choice74, 74usize, T0, _0, T1, _1, T2, _2, T3, _3, T4, _4, T5, _5, T6, _6, T7, _7, T8, _8, T9, _9,
        T10, _10, T11, _11, T12, _12, T13, _13, T14, _14, T15, _15, T16, _16, T17, _17, T18, _18, T19, _19, T20, _20,
        T21, _21, T22, _22, T23, _23, T24, _24, T25, _25, T26, _26, T27, _27, T28, _28, T29, _29, T30, _30, T31, _31,
        T32, _32, T33, _33, T34, _34, T35, _35, T36, _36, T37, _37, T38, _38, T39, _39, T40, _40, T41, _41, T42, _42,
        T43, _43, T44, _44, T45, _45, T46, _46, T47, _47, T48, _48, T49, _49, T50, _50, T51, _51, T52, _52, T53, _53,
        T54, _54, T55, _55, T56, _56, T57, _57, T58, _58, T59, _59, T60, _60, T61, _61, T62, _62, T63, _63, T64, _64,
        T65, _65, T66, _66, T67, _67, T68, _68, T69, _69, T70, _70, T71, _71, T72, _72, T73, _73,
    );
    #[doc = r" Repeat arbitrary times."]
    pub type Rep<'i, const SKIP: ::core::primitive::usize, T> = predefined_node::Rep<T, Skipped<'i>, SKIP>;
    #[doc = r" Repeat at least once."]
    pub type RepOnce<'i, const SKIP: ::core::primitive::usize, T> = predefined_node::RepOnce<T, Skipped<'i>, SKIP>;
}
#[doc = "Re-export some types from rules to simplify the usage."]
pub use rules as pairs;
impl ::pest_typed::TypedParser<Rule> for Grammar {}
