use std::collections::HashMap;

use crate::nodedef::ast::{
    AnnotatedExpression, AnnotationType, ArgDefCollectionType, AssignmentStatement, BodyType,
    CallExpression, CapturesInfoType, ExpressionType, FourArithmeticExpression, Identifier,
    LambdaExpression, LiteralExpression, LiteralExpressionNumber, MemberExpression,
    StructExpression, mathy_ast::ArithmeticOrLiteralOrId, typing::Type,
};

impl ArgDefCollectionType for Vec<(String, Type)> {}

#[derive(Debug)]
pub struct BlockStatement {}

impl BodyType for BlockStatement {}

impl CapturesInfoType for Option<Vec<Type>> {}

#[derive(Debug)]
pub struct MaybetypedExpression {
    pub shape: MaybetypedExpressionShape,
    pub typ: Type,
}

impl MaybetypedExpression {
    pub fn typeless(shape: MaybetypedExpressionShape) -> Self {
        Self {
            shape,
            typ: Type::Unknown,
        }
    }
}

pub type MaybetypedLambdaExpressionShape =
    LambdaExpression<Vec<(String, Type)>, MaybetypedBody, Option<Vec<Type>>>;

#[derive(Debug)]
pub enum MaybetypedExpressionShape {
    Arithmetic(ArithmeticOrLiteralOrId<Box<MaybetypedExpression>, ScopedIdentifier>),
    Lambda(MaybetypedLambdaExpressionShape),
    Call(CallExpression<Box<MaybetypedExpression>, MaybetypedExpression>),
    MemberAccess(MemberExpression<Box<MaybetypedExpression>>),
    StructConstructor(StructExpression<MaybetypedExpression>),
    AnnotatedExpression(AnnotatedExpression<Box<MaybetypedExpression>, String>),
}
impl ExpressionType for MaybetypedExpression {}
impl ExpressionType for Box<MaybetypedExpression> {}

#[derive(Debug)]
pub enum MaybetypedStatement {
    //Assignment(AssignmentStatement<String, UntypedExpression>),
    DeclAssignment(AssignmentStatement<String, MaybetypedExpression>),
}

impl Identifier for String {}
impl AnnotationType for String {}

#[derive(Debug)]
pub enum ScopedIdentifier {
    InScope(Box<ScopedIdentifier>, String),
    Scopeless(String),
}

impl Identifier for ScopedIdentifier {}

#[derive(Debug)]
pub struct MaybetypedBody {
    pub stmts: Vec<MaybetypedStatement>,
    pub end_expr: Option<Box<MaybetypedExpression>>,
}

impl BodyType for MaybetypedBody {}

#[derive(Debug)]
pub struct GlobalMaybetypedExprDefs {
    pub map: HashMap<String, MaybetypedExpression>,
    pub names: Vec<String>,
}
