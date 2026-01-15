use crate::nodedef::ast::{
    ExpressionType, FourArithmeticExpression, Identifier, LiteralExpressionNumber,
};

#[derive(Debug, Clone)]
pub enum ArithmeticOrLiteralOrId<Expr: ExpressionType, Id> {
    Arithmetic(FourArithmeticExpression<Expr>),
    Literal(LiteralExpressionNumber),
    Id(Id),
}
