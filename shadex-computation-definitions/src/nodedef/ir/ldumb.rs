use crate::nodedef::{
    ast::{FourArithmeticExpression, LiteralExpressionNumber},
    ir::ValueRefType,
};

pub enum LDumbOpCode<TArg: ValueRefType> {
    Nop,
    ConstantNum(LiteralExpressionNumber),
    Arithmetic(FourArithmeticExpression<TArg>),
    Copy(TArg),
}
