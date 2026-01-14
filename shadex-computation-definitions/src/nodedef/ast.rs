use std::fmt::Debug;
pub mod full_untyped;

pub trait ExpressionType: Debug {}

pub trait StatementType: Debug {}

pub trait ArgDefType: Debug {}

pub trait BodyType: Debug {}

pub trait CapturesInfoType: Debug {}

pub trait AnnotationType: Debug {}

#[derive(Debug)]
pub enum ArithmeticOp {
    Add,
    Sub,
    Mult,
    Div,
    Eq,
    Leq,
    Geq,
}

#[derive(Debug)]
pub struct FourArithmeticExpression<Arg: ExpressionType> {
    pub op: ArithmeticOp,
    pub left: Arg,
    pub right: Arg,
}

#[derive(Debug)]
pub struct CallExpression<FnType: ExpressionType, Arg: ExpressionType> {
    pub fn_expr: FnType,
    pub args: Vec<(String, Arg)>,
}

#[derive(Debug)]
pub struct MemberExpression<OwnerType: ExpressionType> {
    pub owner: OwnerType,
    pub name: String,
}

#[derive(Debug)]
pub struct LambdaExpression<ArgDef: ArgDefType, Body: BodyType, Captures: CapturesInfoType> {
    pub args: Vec<ArgDef>,
    pub body: Body,
    pub caps: Captures,
}

#[derive(Debug)]
pub struct AnnotatedExpression<SourceType: ExpressionType, Annotation: AnnotationType> {
    pub src: SourceType,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug)]
pub struct StructExpression<FieldExpr: ExpressionType> {
    pub fields: Vec<(String, FieldExpr)>,
}

#[derive(Debug, Clone, Copy)]
pub struct LiteralExpression<Val: Clone + Copy> {
    pub v: Val,
}

#[derive(Debug)]
pub struct IdentifierExpression {
    pub name: String,
}

pub trait Identifier: Debug {}

#[derive(Debug)]
pub struct AssignmentStatement<Id: Identifier, Expr: ExpressionType> {
    pub id: Id,
    pub rhs: Expr,
}

#[derive(Debug)]
pub struct DeclAssignmentStatement<Id: Identifier, Expr: ExpressionType> {
    pub id: Id,
    pub rhs: Expr,
}
