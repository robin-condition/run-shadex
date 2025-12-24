use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum U32Boundedness {
    Unbounded,
    Bounded(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    F32,
    I32,
    U32(U32Boundedness),
}

impl Display for PrimitiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimitiveType::F32 => write!(f, "f32"),
            PrimitiveType::I32 => write!(f, "i32"),
            PrimitiveType::U32(u32_boundedness) => match u32_boundedness {
                U32Boundedness::Unbounded => write!(f, "u32"),
                U32Boundedness::Bounded(bd) => write!(f, "[{}]", *bd),
            },
        }
    }
}

impl Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inputs.is_empty() {
            write!(f, "{}", self.output)
        } else {
            let mut res = write!(f, "(");
            let mut inps: Vec<(&String, &Box<ValueType>)> = self.inputs.iter().collect();
            inps.sort_by(|a, b| a.0.cmp(b.0));
            res = res.and_then(|_| write!(f, "{}: {}", inps[0].0, inps[0].1));
            for (n, v) in inps.into_iter().skip(1) {
                res = res.and_then(|_| write!(f, ", {}: {}", n, v));
            }
            res = res.and_then(|_| write!(f, " -> {})", self.output));
            res
            //write!(f, "({} -> {})", self.inputs.iter().collect::<Vec<(&String, &Box<ValueType>)>>(), self.output)
        }
    }
}

impl ValueType {
    pub fn primitive(prim: PrimitiveType) -> ValueType {
        ValueType {
            inputs: HashMap::new(),
            output: prim,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueType {
    pub inputs: HashMap<String, Box<ValueType>>,
    pub output: PrimitiveType,
}

pub type MaybeValueType = Result<ValueType, TypeError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeError {
    pub message: String,
}
