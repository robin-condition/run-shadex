use std::collections::HashMap;

use crate::nodedef::{
    ast::{FourArithmeticExpression, LiteralExpressionNumber},
    ir::{
        InstrId, Instruction, Next, OpCode, ParamId, ParamInfo, Prev, TypeAnnotation, ValueRefType,
        ldumb::LDumbOpCode, lstruct::LStructOpCode,
    },
};

#[derive(Clone, Copy, Debug)]
pub enum FnValueRef {
    InstrId(InstrId),
    Arg(ParamId),
}

impl ValueRefType for FnValueRef {}

#[derive(Debug, Clone)]
pub struct ParamSupplier<TParam: ParamInfo> {
    pub param_infos: HashMap<ParamId, TParam>,
    pub params_names: HashMap<String, ParamId>,
    next_id: ParamId,
}

impl<TParam: ParamInfo> ParamSupplier<TParam> {
    pub fn new() -> Self {
        Self {
            param_infos: HashMap::new(),
            params_names: HashMap::new(),
            next_id: ParamId(0),
        }
    }

    pub fn add_param(&mut self, name: String, info: TParam) -> ParamId {
        let id = self.next_id;
        self.next_id = ParamId(id.0 + 1);
        self.param_infos.insert(id, info);
        self.params_names.insert(name, id);
        id
    }
}

#[derive(Clone, Debug)]
pub struct FnBody<TArg: ValueRefType, Op: OpCode<TArg>, Typ: TypeAnnotation> {
    pub instrs: HashMap<InstrId, Instruction<TArg, Op, Typ>>,
    next_id: InstrId,

    pub returned: Option<TArg>,

    pub first: Next,
    pub last: Prev,
}

impl<TArg: ValueRefType, Op: OpCode<TArg>, Typ: TypeAnnotation> FnBody<TArg, Op, Typ> {
    pub fn new() -> Self {
        Self {
            returned: None,
            instrs: HashMap::new(),
            next_id: InstrId(0),
            first: Next::End,
            last: Prev::Begin,
        }
    }

    fn apply_prev_to_next(&mut self, prev: Prev, new_next: Next) {
        match prev {
            Prev::Begin => {
                self.first = new_next;
            }
            Prev::Instr(instr_id) => {
                self.instrs.get_mut(&instr_id).as_mut().unwrap().next = new_next;
            }
        }
    }

    fn prev_to_next(&self, prev: Prev) -> Next {
        match prev {
            Prev::Begin => self.first,
            Prev::Instr(instr_id) => self.instrs.get(&instr_id).unwrap().next,
        }
    }

    pub fn append_instr(&mut self, op: Op, typ: Typ) -> InstrId {
        let new_id = self.next_id;
        self.next_id = InstrId(self.next_id.0 + 1);
        let next_to_fix = self.apply_prev_to_next(self.last, Next::Instr(new_id));
        let old_last = self.last;
        self.last = Prev::Instr(new_id);
        let new_instr = Instruction {
            prev: old_last,
            next: Next::End,
            op,
            typ,
            phantom: std::marker::PhantomData,
        };
        self.instrs.insert(new_id, new_instr);
        new_id
    }
}

pub struct FnBodyIter<'a, TArg: ValueRefType, Op: OpCode<TArg>, Typ: TypeAnnotation> {
    bd: &'a FnBody<TArg, Op, Typ>,
    next: Next,
}

impl<'a, TArg: ValueRefType, Op: OpCode<TArg>, Typ: TypeAnnotation> Iterator
    for FnBodyIter<'a, TArg, Op, Typ>
{
    type Item = (InstrId, &'a Op);

    fn next(&mut self) -> Option<Self::Item> {
        match self.next {
            Next::End => {
                return None;
            }
            Next::Instr(instr_id) => {
                let next_next = self.bd.prev_to_next(Prev::Instr(instr_id));
                let op = &self.bd.instrs.get(&instr_id).unwrap().op;
                self.next = next_next;
                return Some((instr_id, op));
            }
        }
    }
}

impl<'a, TArg: ValueRefType, Op: OpCode<TArg>, Typ: TypeAnnotation> IntoIterator
    for &'a FnBody<TArg, Op, Typ>
{
    type Item = (InstrId, &'a Op);

    type IntoIter = FnBodyIter<'a, TArg, Op, Typ>;

    fn into_iter(self) -> Self::IntoIter {
        FnBodyIter {
            bd: self,
            next: self.first,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FnDef<TArg: ValueRefType, TParam: ParamInfo, Op: OpCode<TArg>, Typ: TypeAnnotation> {
    pub body: FnBody<TArg, Op, Typ>,
    pub params: ParamSupplier<TParam>,
}

#[derive(Debug, Clone)]
pub enum LFunOpCode<TArg: ValueRefType, TParam: ParamInfo, Op: OpCode<TArg>, Typ: TypeAnnotation> {
    Struct(LStructOpCode<TArg>),
    FnCtor(FnDef<TArg, TParam, Op, Typ>),
    GlobalFn(String),
    CallFn(TArg, HashMap<String, TArg>),
}

impl<TArg: ValueRefType, TParam: ParamInfo, Op: OpCode<TArg>, Typ: TypeAnnotation>
    From<LStructOpCode<TArg>> for LFunOpCode<TArg, TParam, Op, Typ>
{
    fn from(value: LStructOpCode<TArg>) -> Self {
        Self::Struct(value)
    }
}

impl<TArg: ValueRefType, TParam: ParamInfo, Op: OpCode<TArg>, Typ: TypeAnnotation>
    From<LDumbOpCode<TArg>> for LFunOpCode<TArg, TParam, Op, Typ>
{
    fn from(value: LDumbOpCode<TArg>) -> Self {
        Self::Struct(LStructOpCode::Dumb(value))
    }
}

impl<TArg: ValueRefType, TParam: ParamInfo, Op: OpCode<TArg>, Typ: TypeAnnotation> OpCode<TArg>
    for LFunOpCode<TArg, TParam, Op, Typ>
{
}
