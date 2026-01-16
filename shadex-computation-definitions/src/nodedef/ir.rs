use std::collections::HashMap;

use crate::{
    Value,
    nodedef::ast::{FourArithmeticExpression, LiteralExpressionNumber},
};

pub mod ldumb;
pub mod lfun;
pub mod llambda;
pub mod lstruct;

#[derive(Clone, Copy, Debug)]
pub struct InstrId(usize);

#[derive(Clone, Copy, Debug)]
pub struct ParamId(usize);

#[derive(Clone, Copy, Debug)]
pub struct CaptureId(usize);

#[derive(Clone, Copy, Debug)]
pub struct FieldId(usize);

pub trait ValueRefType {}

pub trait ParamInfo {}

pub trait LangTypeSpec {}

pub enum Next {
    End,
    Instr(InstrId),
}

pub enum Prev {
    Begin,
    Instr(InstrId),
}

pub trait TypeAnnotation {}

pub struct Instruction<TArg: ValueRefType, Op: OpCode<TArg>, Typ: TypeAnnotation> {
    pub prev: Prev,
    pub next: Next,
    pub op: Op,
    pub typ: Typ,
}

pub trait OpCode<TArg: ValueRefType> {}
