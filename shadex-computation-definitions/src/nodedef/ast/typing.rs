use std::{collections::HashMap, fmt::Display, rc::Rc};

#[derive(Debug, Clone)]
pub struct CapturesType(pub Vec<Type>);

impl Display for CapturesType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Capture<")?;
        for t in &self.0 {
            write!(f, "{}, ", t)?;
        }
        write!(f, ">")
    }
}

#[derive(Debug, Clone)]
pub struct FunctionType(pub HashMap<String, Type>, pub Box<Type>);

impl Display for FunctionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        for a in &self.0 {
            write!(f, "{}: {}, ", a.0, a.1)?;
        }
        write!(f, ") -> {}", self.1)
    }
}

#[derive(Debug, Clone)]
pub struct LambdaType {
    pub captures: CapturesType,
    pub function: FunctionType,
}

impl Display for LambdaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Lambda<{}, {}>", self.captures, self.function)
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    U32,
    F32,
    Bool,
    Lambda(LambdaType),
    Unknown,
    Unit,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::U32 => write!(f, "u32"),
            Type::F32 => write!(f, "f32"),
            Type::Unknown => write!(f, "unknown"),
            Type::Unit => write!(f, "unit"),
            Type::Bool => write!(f, "bool"),
            Type::Lambda(lambda_type) => lambda_type.fmt(f),
        }
    }
}
