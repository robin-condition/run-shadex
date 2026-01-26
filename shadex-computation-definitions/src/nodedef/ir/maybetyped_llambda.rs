use std::{collections::HashSet, fmt::Display};

use crate::nodedef::{
    ast::{
        ArithmeticOp, BoolValuedOp, LambdaExpression, LiteralExpressionNumber, MathOp,
        full_maybetyped::MaybetypedBody, typing::Type,
    },
    ir::{
        CaptureId, Instruction, OpCode, ParamInfo, TypeAnnotation, ValueRefType,
        ldumb::LDumbOpCode,
        lfun::{FnBody, FnDef, FnValueRef, LFunOpCode},
        llambda::{LLambdaOpCode, LambdaDef, LambdaValueRef},
        lstruct::LStructOpCode,
    },
};

pub mod opt;
pub mod type_propagation;

pub type MaybetypedLLambdaType = Type;

impl TypeAnnotation for MaybetypedLLambdaType {}

pub type MaybetypedLLambdaParamInfo = MaybetypedLLambdaType;
impl ParamInfo for MaybetypedLLambdaParamInfo {}

#[derive(Debug, Clone)]
pub struct MaybetypedLLambdaOpCode(
    pub LLambdaOpCode<LambdaValueRef, MaybetypedLLambdaParamInfo, Self, MaybetypedLLambdaType>,
);

pub mod interpreter;

impl
    From<
        LLambdaOpCode<
            LambdaValueRef,
            MaybetypedLLambdaParamInfo,
            MaybetypedLLambdaOpCode,
            MaybetypedLLambdaType,
        >,
    > for MaybetypedLLambdaOpCode
{
    fn from(
        value: LLambdaOpCode<
            LambdaValueRef,
            MaybetypedLLambdaParamInfo,
            MaybetypedLLambdaOpCode,
            MaybetypedLLambdaType,
        >,
    ) -> Self {
        Self(value)
    }
}

impl OpCode<LambdaValueRef> for MaybetypedLLambdaOpCode {}

pub type MaybetypedLLambdaInstr =
    Instruction<LambdaValueRef, MaybetypedLLambdaOpCode, MaybetypedLLambdaType>;

pub type MaybetypedLLambdaFDef = FnDef<
    LambdaValueRef,
    MaybetypedLLambdaParamInfo,
    MaybetypedLLambdaOpCode,
    MaybetypedLLambdaType,
>;

impl ValueRefType for LambdaValueRef {}

pub type MaybetypedLLambdaFBody =
    FnBody<LambdaValueRef, MaybetypedLLambdaOpCode, MaybetypedLLambdaType>;

pub type MaybetypedLLambdaLambdaDef = LambdaDef<
    LambdaValueRef,
    MaybetypedLLambdaParamInfo,
    MaybetypedLLambdaOpCode,
    MaybetypedLLambdaType,
>;

impl
    From<
        LFunOpCode<
            LambdaValueRef,
            MaybetypedLLambdaParamInfo,
            MaybetypedLLambdaOpCode,
            MaybetypedLLambdaType,
        >,
    > for MaybetypedLLambdaOpCode
{
    fn from(
        value: LFunOpCode<
            LambdaValueRef,
            MaybetypedLLambdaParamInfo,
            MaybetypedLLambdaOpCode,
            MaybetypedLLambdaType,
        >,
    ) -> Self {
        Self(LLambdaOpCode::Fn(value))
    }
}

impl From<LStructOpCode<LambdaValueRef>> for MaybetypedLLambdaOpCode {
    fn from(value: LStructOpCode<LambdaValueRef>) -> Self {
        Self(LLambdaOpCode::Fn(LFunOpCode::Struct(value)))
    }
}

impl From<LDumbOpCode<LambdaValueRef>> for MaybetypedLLambdaOpCode {
    fn from(value: LDumbOpCode<LambdaValueRef>) -> Self {
        Self(LLambdaOpCode::Fn(LFunOpCode::Struct(LStructOpCode::Dumb(
            value,
        ))))
    }
}

impl Display for LiteralExpressionNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LiteralExpressionNumber::LiteralI32(i) => write!(f, "{}i32", i.v),
            LiteralExpressionNumber::LiteralU32(u) => write!(f, "{}u32", u.v),
            LiteralExpressionNumber::LiteralF32(fl) => write!(f, "{}f32", fl.v),
        }
    }
}

impl Display for LambdaValueRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LambdaValueRef::FnValueRef(FnValueRef::Arg(a)) => write!(f, "arg{}", a.0),
            LambdaValueRef::FnValueRef(FnValueRef::InstrId(i)) => write!(f, "${}", i.0),
            LambdaValueRef::CaptureRef(capture_id) => write!(f, "cap{}", capture_id.0),
        }
    }
}

impl Display for MathOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MathOp::Arith(ArithmeticOp::Add) => write!(f, "+"),
            MathOp::Arith(ArithmeticOp::Sub) => write!(f, "-"),
            MathOp::Arith(ArithmeticOp::Mult) => write!(f, "*"),
            MathOp::Arith(ArithmeticOp::Div) => write!(f, "/"),
            MathOp::Comp(BoolValuedOp::Eq) => write!(f, "=="),
            MathOp::Comp(BoolValuedOp::Leq) => write!(f, "<="),
            MathOp::Comp(BoolValuedOp::Geq) => write!(f, ">="),
        }
    }
}

impl Display for LDumbOpCode<LambdaValueRef> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LDumbOpCode::Nop => write!(f, "nop"),
            LDumbOpCode::ConstantNum(num) => num.fmt(f),
            LDumbOpCode::Arithmetic(e) => write!(f, "{} {} {}", e.left, e.op, e.right),
            LDumbOpCode::Copy(v) => v.fmt(f),
        }
    }
}

impl Display for LStructOpCode<LambdaValueRef> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LStructOpCode::Dumb(ldumb_op_code) => ldumb_op_code.fmt(f),
            LStructOpCode::MemberAccess(owner, field_id) => write!(f, "{}.{}", owner, field_id.0),
            LStructOpCode::ConstructStruct(struct_ctor) => {
                write!(f, "(")?;
                for i in &struct_ctor.field_names {
                    write!(f, "{}: {},", i.0, struct_ctor.field_infs.get(i.1).unwrap())?;
                }
                write!(f, ")")
            }
        }
    }
}

impl
    FnDef<
        LambdaValueRef,
        MaybetypedLLambdaParamInfo,
        MaybetypedLLambdaOpCode,
        MaybetypedLLambdaType,
    >
{
    fn fmt_with_prefix(
        &self,
        line_prefix: &str,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "(")?;
        for a in &self.params.params_names {
            let t = self.params.param_infos.get(a.1).unwrap();
            write!(f, "{}/{}: {}, ", a.0, a.1, t)?;
        }
        writeln!(f, ") => {{")?;
        self.body
            .fmt_with_prefix(("  ".to_string() + line_prefix).as_str(), f)?;
        write!(f, "{}}}", line_prefix)?;
        Ok(())
    }
}

// Display for
impl
    LFunOpCode<
        LambdaValueRef,
        MaybetypedLLambdaParamInfo,
        MaybetypedLLambdaOpCode,
        MaybetypedLLambdaType,
    >
{
    fn fmt_with_prefix(
        &self,
        line_prefix: &str,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            LFunOpCode::Struct(lstruct_op_code) => lstruct_op_code.fmt(f),
            LFunOpCode::FnCtor(fn_def) => fn_def.fmt_with_prefix(line_prefix, f),
            LFunOpCode::GlobalFn(n) => write!(f, "glob::{}", n),
            LFunOpCode::CallFn(func, hash_map) => {
                write!(f, "call {} (", func)?;
                for (a, v) in hash_map {
                    write!(f, "{}: {}, ", a, v)?;
                }
                write!(f, ")")
            }
        }
    }
}

impl FnBody<LambdaValueRef, MaybetypedLLambdaOpCode, MaybetypedLLambdaType> {
    pub fn fmt_with_prefix(
        &self,
        prefix: &str,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        for l in self {
            let t = &l.1.typ;
            write!(f, "{}{}: {} = ", prefix, l.0, t)?;
            l.1.op.fmt_with_prefix(prefix, f)?;
            writeln!(f)?;
        }
        if let Some(r) = &self.returned {
            writeln!(f, "{}RET {}", prefix, r)?;
        }
        Ok(())
    }
}

impl MaybetypedLLambdaOpCode {
    fn fmt_with_prefix(
        &self,
        line_prefix: &str,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match &self.0 {
            LLambdaOpCode::Fn(fun) => fun.fmt_with_prefix(line_prefix, f),
            LLambdaOpCode::ConstructLambda(lambda_def) => {
                write!(f, "caps{{")?;
                for c in &lambda_def.captures_info.captures {
                    write!(f, "{} -> {}, ", c.0, c.1)?;
                }
                write!(f, "}} ")?;
                lambda_def.fn_def.fmt_with_prefix(line_prefix, f)
            }
        }
    }
}

impl MaybetypedLLambdaFBody {
    fn report_used_captures(&self) -> HashSet<CaptureId> {
        let mut set = HashSet::new();
        for s in self {
            match &s.1.op.0 {
                LLambdaOpCode::Fn(LFunOpCode::GlobalFn(_)) => {}
                LLambdaOpCode::Fn(LFunOpCode::FnCtor(_)) => {}
                LLambdaOpCode::Fn(LFunOpCode::CallFn(c, args)) => {
                    if let LambdaValueRef::CaptureRef(c) = c {
                        set.insert(*c);
                    }

                    for v in args {
                        if let LambdaValueRef::CaptureRef(c) = v.1 {
                            set.insert(*c);
                        }
                    }
                }
                LLambdaOpCode::Fn(LFunOpCode::Struct(LStructOpCode::MemberAccess(owner, _))) => {
                    if let LambdaValueRef::CaptureRef(c) = owner {
                        set.insert(*c);
                    }
                }
                LLambdaOpCode::Fn(LFunOpCode::Struct(LStructOpCode::ConstructStruct(ctor))) => {
                    for v in &ctor.field_infs {
                        if let LambdaValueRef::CaptureRef(c) = v.1 {
                            set.insert(*c);
                        }
                    }
                }
                LLambdaOpCode::Fn(LFunOpCode::Struct(LStructOpCode::Dumb(LDumbOpCode::Nop))) => {}
                LLambdaOpCode::Fn(LFunOpCode::Struct(LStructOpCode::Dumb(
                    LDumbOpCode::ConstantNum(_),
                ))) => {}
                LLambdaOpCode::Fn(LFunOpCode::Struct(LStructOpCode::Dumb(LDumbOpCode::Copy(
                    s,
                )))) => {
                    if let LambdaValueRef::CaptureRef(c) = s {
                        set.insert(*c);
                    }
                }
                LLambdaOpCode::Fn(LFunOpCode::Struct(LStructOpCode::Dumb(
                    LDumbOpCode::Arithmetic(s),
                ))) => {
                    if let LambdaValueRef::CaptureRef(c) = s.left {
                        set.insert(c);
                    }
                    if let LambdaValueRef::CaptureRef(c) = s.right {
                        set.insert(c);
                    }
                }
                LLambdaOpCode::ConstructLambda(l) => {
                    for i in &l.captures_info.captures {
                        if let LambdaValueRef::CaptureRef(c) = i.1 {
                            set.insert(*c);
                        }
                    }
                }
            }
        }
        if let Some(r) = &self.returned {
            if let LambdaValueRef::CaptureRef(c) = r {
                set.insert(*c);
            }
        }
        set
    }
}

impl MaybetypedLLambdaOpCode {
    pub fn remove_unnecessary_captures_here(&mut self) {
        match &mut self.0 {
            LLambdaOpCode::ConstructLambda(l) => {
                l.fn_def.body.remove_unnecessary_captures_in_children();

                // Now collect the captures in use.
                let res = l.fn_def.body.report_used_captures();
                let mut to_rem = Vec::new();
                for c in &l.captures_info.captures {
                    if !res.contains(c.0) {
                        to_rem.push(*c.0);
                    }
                }

                // Remove the unused ones
                for c in to_rem {
                    l.captures_info.captures.remove(&c);
                }
            }
            _ => {}
        }
    }
}

impl MaybetypedLLambdaFBody {
    pub fn remove_unnecessary_captures_in_children(&mut self) {
        let ids: Vec<_> = self.into_iter().map(|(a, _)| a).collect();
        for id in ids {
            self.instrs
                .get_mut(&id)
                .unwrap()
                .op
                .remove_unnecessary_captures_here();
        }
    }
}

impl Display for MaybetypedLLambdaFBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_with_prefix("", f)
    }
}
