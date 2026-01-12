use crate::nodedef::ast::{
    ArgDefType, AssignmentStatement, BodyType, CallExpression, CapturesInfoType, ExpressionType,
    FourArithmeticExpression, Identifier, LambdaExpression, LiteralExpression, MemberExpression,
    StructExpression,
};

#[derive(Debug)]
pub struct ArgName {
    pub name: String,
}

impl ArgDefType for ArgName {}

#[derive(Debug)]
pub struct BlockStatement {}

impl BodyType for BlockStatement {}

impl CapturesInfoType for () {}

#[derive(Debug)]
pub enum UntypedExpression {
    Arithmetic(FourArithmeticExpression<Box<UntypedExpression>>),
    Lambda(LambdaExpression<ArgName, UntypedBody, ()>),
    Call(CallExpression<Box<UntypedExpression>, UntypedExpression>),
    LiteralI32(LiteralExpression<i32>),
    LiteralU32(LiteralExpression<u32>),
    LiteralF32(LiteralExpression<f32>),
    MemberAccess(MemberExpression<Box<UntypedExpression>>),
    ScopedIdentifier(ScopedIdentifier),
    StructConstructor(StructExpression<UntypedExpression>),
}
impl ExpressionType for UntypedExpression {}
impl ExpressionType for Box<UntypedExpression> {}

#[derive(Debug)]
pub enum UntypedStatement {
    Assignment(AssignmentStatement<String, UntypedExpression>),
    DeclAssignment(AssignmentStatement<String, UntypedExpression>),
}

impl Identifier for String {}

#[derive(Debug)]
pub enum ScopedIdentifier {
    InScope(Box<ScopedIdentifier>, String),
    Scopeless(String),
}

#[derive(Debug)]
pub struct UntypedBody {
    pub stmts: Vec<UntypedStatement>,
}

impl BodyType for UntypedBody {}
