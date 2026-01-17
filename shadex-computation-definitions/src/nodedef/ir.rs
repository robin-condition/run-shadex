use std::{collections::HashMap, fmt::Display, marker::PhantomData};

use crate::{
    Value,
    nodedef::ast::{FourArithmeticExpression, LiteralExpressionNumber},
};

pub mod ldumb;
pub mod lfun;
pub mod llambda;
pub mod lstruct;

pub mod untyped_llambda;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct InstrId(usize);

impl Display for InstrId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.0)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct ParamId(usize);

impl Display for ParamId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "arg{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct CaptureId(usize);

impl Display for CaptureId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cap{}", self.0)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
//pub struct FieldId(usize);
pub struct FieldId(pub String);

pub trait ValueRefType: Clone {}

pub trait ParamInfo {}

pub trait LangTypeSpec {}

#[derive(Clone, Copy, Debug)]
pub enum Next {
    End,
    Instr(InstrId),
}

#[derive(Clone, Copy, Debug)]
pub enum Prev {
    Begin,
    Instr(InstrId),
}

pub trait TypeAnnotation {}

#[derive(Clone, Debug)]
pub struct Instruction<TArg: ValueRefType, Op: OpCode<TArg>, Typ: TypeAnnotation> {
    pub prev: Prev,
    pub next: Next,
    pub op: Op,
    pub typ: Typ,
    phantom: PhantomData<TArg>,
}

pub trait OpCode<TArg: ValueRefType> {}
