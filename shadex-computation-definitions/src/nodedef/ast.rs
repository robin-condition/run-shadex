use std::{collections::HashMap, fmt::Debug};
pub mod full_untyped;
pub mod identifiers_linked;

pub mod mathy_ast;

pub trait ExpressionType: Debug {}

pub trait StatementType: Debug {}

pub trait ArgDefCollectionType: Debug {}

pub trait BodyType: Debug {}

pub trait CapturesInfoType: Debug {}

pub trait AnnotationType: Debug {}

pub mod linearize_untyped;

#[derive(Debug, Clone)]
pub enum ArithmeticOp {
    Add,
    Sub,
    Mult,
    Div,
    Eq,
    Leq,
    Geq,
}

#[derive(Debug, Clone)]
pub struct FourArithmeticExpression<Arg> {
    pub op: ArithmeticOp,
    pub left: Arg,
    pub right: Arg,
}

#[derive(Debug, Clone)]
pub struct CallExpression<FnType: ExpressionType, Arg: ExpressionType> {
    pub fn_expr: FnType,
    pub args: Vec<(String, Arg)>,
}

#[derive(Debug, Clone)]
pub struct MemberExpression<OwnerType: ExpressionType> {
    pub owner: OwnerType,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct LambdaExpression<
    ArgDefCollection: ArgDefCollectionType,
    Body: BodyType,
    Captures: CapturesInfoType,
> {
    pub args: ArgDefCollection,
    pub body: Body,
    pub caps: Captures,
}

#[derive(Debug, Clone)]
pub struct AnnotatedExpression<SourceType: ExpressionType, Annotation: AnnotationType> {
    pub src: SourceType,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone)]
pub struct StructExpression<FieldExpr: ExpressionType> {
    pub fields: Vec<(String, FieldExpr)>,
}

#[derive(Debug, Clone, Copy)]
pub struct LiteralExpression<Val: Clone + Copy> {
    pub v: Val,
}

#[derive(Debug, Clone, Copy)]
pub enum LiteralExpressionNumber {
    LiteralI32(LiteralExpression<i32>),
    LiteralU32(LiteralExpression<u32>),
    LiteralF32(LiteralExpression<f32>),
}

#[derive(Debug, Clone)]
pub struct IdentifierExpression {
    pub name: String,
}

pub trait Identifier: Debug {}

#[derive(Debug, Clone)]
pub struct AssignmentStatement<Id: Identifier, Expr: ExpressionType> {
    pub id: Id,
    pub rhs: Expr,
}

#[derive(Debug, Clone)]
pub struct DeclAssignmentStatement<Id: Identifier, Expr: ExpressionType> {
    pub id: Id,
    pub rhs: Expr,
}
