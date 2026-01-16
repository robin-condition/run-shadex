use std::collections::HashMap;

use crate::nodedef::ir::{FieldId, ValueRefType, ldumb::LDumbOpCode};

pub struct StructCtor<TArg: ValueRefType> {
    field_infs: HashMap<FieldId, TArg>,
    next_field_id: FieldId,
}

pub enum LStructOpCode<TArg: ValueRefType> {
    Dumb(LDumbOpCode<TArg>),
    MemberAccess(TArg, FieldId),
    ConstructStruct(StructCtor<TArg>),
}
