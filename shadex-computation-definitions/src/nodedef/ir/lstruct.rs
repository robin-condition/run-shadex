use std::collections::HashMap;

use crate::nodedef::ir::{FieldId, ValueRefType, ldumb::LDumbOpCode};

#[derive(Debug, Clone)]
pub struct StructCtor<TArg: ValueRefType> {
    pub field_infs: HashMap<FieldId, TArg>,
    pub field_names: HashMap<String, FieldId>,
    next_field_id: FieldId,
}

impl<TArg: ValueRefType> StructCtor<TArg> {
    pub fn from_map(map: &HashMap<String, TArg>) -> Self {
        let mut stct = Self {
            field_infs: HashMap::new(),
            field_names: HashMap::new(),
            next_field_id: FieldId(String::new()),
        };
        for (k, v) in map {
            let id = FieldId(k.clone());
            //let id = stct.next_field_id;
            //stct.next_field_id = FieldId(id.0 + 1);
            stct.field_infs.insert(id.clone(), v.clone());
            stct.field_names.insert(k.clone(), id);
        }
        stct
    }
}

#[derive(Clone, Debug)]
pub enum LStructOpCode<TArg: ValueRefType> {
    Dumb(LDumbOpCode<TArg>),
    MemberAccess(TArg, FieldId),
    ConstructStruct(StructCtor<TArg>),
}

impl<TArg: ValueRefType> From<LDumbOpCode<TArg>> for LStructOpCode<TArg> {
    fn from(value: LDumbOpCode<TArg>) -> Self {
        Self::Dumb(value)
    }
}
