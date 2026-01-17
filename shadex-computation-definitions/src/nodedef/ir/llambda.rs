use std::collections::HashMap;

use crate::nodedef::ir::{
    CaptureId, OpCode, ParamInfo, TypeAnnotation, ValueRefType,
    ldumb::LDumbOpCode,
    lfun::{FnDef, FnValueRef, LFunOpCode},
    lstruct::LStructOpCode,
};

#[derive(Clone, Copy, Debug)]
pub enum LambdaValueRef {
    FnValueRef(FnValueRef),
    CaptureRef(CaptureId),
}

#[derive(Debug, Clone)]
pub struct CapturesInfo<TArg: ValueRefType> {
    pub captures: HashMap<CaptureId, TArg>,
    next_id: CaptureId,
}

impl<TArg: ValueRefType> CapturesInfo<TArg> {
    pub fn new() -> Self {
        Self {
            captures: HashMap::new(),
            next_id: CaptureId(0),
        }
    }

    pub fn add_capture(&mut self, arg: TArg) -> CaptureId {
        let id = self.next_id;
        self.next_id = CaptureId(id.0 + 1);

        self.captures.insert(id, arg);
        id
    }
}

#[derive(Debug, Clone)]
pub struct LambdaDef<TArg: ValueRefType, TParam: ParamInfo, Op: OpCode<TArg>, Typ: TypeAnnotation> {
    pub fn_def: FnDef<TArg, TParam, Op, Typ>,
    pub captures_info: CapturesInfo<TArg>,
}

#[derive(Debug, Clone)]
pub enum LLambdaOpCode<TArg: ValueRefType, TParam: ParamInfo, Op: OpCode<TArg>, Typ: TypeAnnotation>
{
    Fn(LFunOpCode<TArg, TParam, Op, Typ>),
    ConstructLambda(LambdaDef<TArg, TParam, Op, Typ>),
}

impl<TArg: ValueRefType, TParam: ParamInfo, Op: OpCode<TArg>, Typ: TypeAnnotation>
    From<LFunOpCode<TArg, TParam, Op, Typ>> for LLambdaOpCode<TArg, TParam, Op, Typ>
{
    fn from(value: LFunOpCode<TArg, TParam, Op, Typ>) -> Self {
        Self::Fn(value)
    }
}

impl<TArg: ValueRefType, TParam: ParamInfo, Op: OpCode<TArg>, Typ: TypeAnnotation>
    From<LStructOpCode<TArg>> for LLambdaOpCode<TArg, TParam, Op, Typ>
{
    fn from(value: LStructOpCode<TArg>) -> Self {
        Self::Fn(LFunOpCode::Struct(value))
    }
}

impl<TArg: ValueRefType, TParam: ParamInfo, Op: OpCode<TArg>, Typ: TypeAnnotation>
    From<LDumbOpCode<TArg>> for LLambdaOpCode<TArg, TParam, Op, Typ>
{
    fn from(value: LDumbOpCode<TArg>) -> Self {
        Self::Fn(LFunOpCode::Struct(LStructOpCode::Dumb(value)))
    }
}

impl<TArg: ValueRefType, TParam: ParamInfo, Op: OpCode<TArg>, Typ: TypeAnnotation>
    OpCode<LambdaValueRef> for LLambdaOpCode<TArg, TParam, Op, Typ>
{
}
