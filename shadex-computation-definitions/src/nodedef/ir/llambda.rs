use std::collections::HashMap;

use crate::nodedef::ir::{
    CaptureId, FnDef, OpCode, ParamInfo, TypeAnnotation, ValueRefType, lfun::LFunOpCode,
};

pub enum LambdaValueRef {
    FnValueRef(FnValueRef),
    CaptureRef(CaptureId),
}

pub struct CapturesInfo<TArg: ValueRefType, TParam: ParamInfo, Op: OpCode<TArg>> {
    captures: HashMap<CaptureId, TArg>,
    next_id: CaptureId,
}

pub struct LambdaDef<TArg: ValueRefType, TParam: ParamInfo, Op: OpCode<TArg>, Typ: TypeAnnotation> {
    pub fn_def: FnDef<TArg, TParam, Op, Typ>,
    pub captures_info: CapturesInfo<TArg, TParam, Op>,
}

pub enum LLambdaOpCode<TArg: ValueRefType, TParam: ParamInfo, Op: OpCode<TArg>, Typ: TypeAnnotation>
{
    Fn(LFunOpCode<TArg, TParam, Op, Typ>),
    ConstructLambda(LambdaDef<LambdaValueRef, TParam, LLambdaOpCode>),
}

impl OpCode<LambdaValueRef> for LLambdaOpCode {}
