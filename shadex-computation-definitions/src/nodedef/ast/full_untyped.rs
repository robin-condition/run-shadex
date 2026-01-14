use std::collections::HashMap;

use crate::nodedef::ast::{
    AnnotatedExpression, AnnotationType, ArgDefCollectionType, AssignmentStatement, BodyType,
    CallExpression, CapturesInfoType, ExpressionType, FourArithmeticExpression, Identifier,
    LambdaExpression, LiteralExpression, MemberExpression, StructExpression,
};

impl ArgDefCollectionType for Vec<String> {}

#[derive(Debug)]
pub struct BlockStatement {}

impl BodyType for BlockStatement {}

impl CapturesInfoType for () {}

#[derive(Debug)]
pub enum UntypedExpression {
    Arithmetic(FourArithmeticExpression<Box<UntypedExpression>>),
    Lambda(LambdaExpression<Vec<String>, UntypedBody, ()>),
    Call(CallExpression<Box<UntypedExpression>, UntypedExpression>),
    LiteralI32(LiteralExpression<i32>),
    LiteralU32(LiteralExpression<u32>),
    LiteralF32(LiteralExpression<f32>),
    MemberAccess(MemberExpression<Box<UntypedExpression>>),
    ScopedIdentifier(ScopedIdentifier),
    StructConstructor(StructExpression<UntypedExpression>),
    AnnotatedExpression(AnnotatedExpression<Box<UntypedExpression>, String>),
}
impl ExpressionType for UntypedExpression {}
impl ExpressionType for Box<UntypedExpression> {}

#[derive(Debug)]
pub enum UntypedStatement {
    //Assignment(AssignmentStatement<String, UntypedExpression>),
    DeclAssignment(AssignmentStatement<String, UntypedExpression>),
}

impl Identifier for String {}
impl AnnotationType for String {}

#[derive(Debug)]
pub enum ScopedIdentifier {
    InScope(Box<ScopedIdentifier>, String),
    Scopeless(String),
}

#[derive(Debug)]
pub struct UntypedBody {
    pub stmts: Vec<UntypedStatement>,
    pub end_expr: Option<Box<UntypedExpression>>,
}

impl BodyType for UntypedBody {}

#[derive(Debug)]
pub struct GlobalUntypedExprDefs {
    pub map: HashMap<String, UntypedExpression>,
    pub names: Vec<String>,
}
