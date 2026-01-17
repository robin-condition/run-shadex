use crate::nodedef::{
    ast::{FourArithmeticExpression, LiteralExpressionNumber},
    ir::ValueRefType,
};

#[derive(Clone, Debug)]
pub enum LDumbOpCode<TArg: ValueRefType> {
    Nop,
    ConstantNum(LiteralExpressionNumber),
    Arithmetic(FourArithmeticExpression<TArg>),
    Copy(TArg),
}

impl<TArg: ValueRefType> From<LiteralExpressionNumber> for LDumbOpCode<TArg> {
    fn from(value: LiteralExpressionNumber) -> Self {
        Self::ConstantNum(value)
    }
}

impl<TArg: ValueRefType> From<FourArithmeticExpression<TArg>> for LDumbOpCode<TArg> {
    fn from(value: FourArithmeticExpression<TArg>) -> Self {
        Self::Arithmetic(value)
    }
}

impl<TArg: ValueRefType> From<TArg> for LDumbOpCode<TArg> {
    fn from(value: TArg) -> Self {
        Self::Copy(value)
    }
}
