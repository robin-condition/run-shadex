use std::collections::HashMap;

use crate::nodedef::{
    ast::{FourArithmeticExpression, LiteralExpressionNumber},
    ir::{FieldId, InstrId, OpCode, ParamId, ParamInfo, TypeAnnotation, lstruct::LStructOpCode},
};

pub enum FnValueRef {
    InstrId(InstrId),
    Arg(ParamId),
}

impl ValueRefType for FnValueRef {}

pub struct ParamSupplier<TParam: ParamInfo> {
    param_infos: HashMap<ParamId, TParam>,
    params_names: HashMap<String, ParamId>,
    next_id: ParamId,
}

#[derive(Clone)]
pub struct FnBody<TArg: ValueRefType, Op: OpCode<TArg>, Typ: TypeAnnotation> {
    instrs: HashMap<InstrId, Instruction<TArg, Op, Typ>>,
    next_id: InstrId,

    pub first: Next,
    pub last: Prev,
}

impl<TArg: ValueRefType, Op: OpCode<TArg>> FnBody<TArg, Op> {
    pub fn new() -> Self {
        Self {
            instrs: HashMap::new(),
            next_id: InstrId(0),
            first: Next::End,
            last: Prev::Begin,
        }
    }
}

pub struct FnDef<TArg: ValueRefType, TParam: ParamInfo, Op: OpCode<TArg>, Typ: TypeAnnotation> {
    pub body: FnBody<TArg, Op, Typ>,
    pub params: ParamSupplier<TParam>,
}

pub enum LFunOpCode<TArg: ValueRefType, TParam: ParamInfo, Op: OpCode<TArg>, Typ: TypeAnnotation> {
    Struct(LStructOpCode<TArg>),
    FnCtor(FnDef<TArg, TParam, Op, Typ>),
    GlobalFn(String),
    CallFn(TArg, HashMap<String, TArg>),
}

impl<TArg: ValueRefType, TParam: ParamInfo, Op: OpCode<TArg>, Typ: TypeAnnotation> OpCode<TArg>
    for LFunOpCode<TArg, TParam, Op, Typ>
{
}
