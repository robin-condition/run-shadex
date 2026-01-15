use std::{collections::HashMap, rc::Rc};

use rpds::HashTrieMap;

use crate::nodedef::ast::{
    AnnotatedExpression, AnnotationType, ArgDefCollectionType, AssignmentStatement, BodyType,
    CallExpression, ExpressionType, FourArithmeticExpression, Identifier, LambdaExpression,
    LiteralExpression, LiteralExpressionNumber, MemberExpression, StructExpression,
    full_untyped::{GlobalUntypedExprDefs, ScopedIdentifier, UntypedBody, UntypedExpression},
    mathy_ast::ArithmeticOrLiteralOrId,
};

#[derive(Debug, Clone)]
pub struct LambdaArgDef {
    pub name: String,
    pub id: IdentifierReference,
}

pub struct IdentifierOwningScope {
    pub defs: HashMap<String, IdentifierReference>,
    pub defs_vec: HashMap<IdentifierReference, Rc<NameLinkedExpression>>,
}

impl IdentifierOwningScope {
    pub fn new() -> IdentifierOwningScope {
        IdentifierOwningScope {
            defs: HashMap::new(),
            defs_vec: HashMap::new(),
        }
    }
}

pub struct IdentifierExprIdDispatcher {
    pub next_id: usize,
}

impl IdentifierExprIdDispatcher {
    pub fn new() -> IdentifierExprIdDispatcher {
        Self { next_id: 0 }
    }

    pub fn get_id(&mut self) -> IdentifierReference {
        let id_to_use = self.next_id;
        self.next_id += 1;
        IdentifierReference { id: id_to_use }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct IdentifierReference {
    pub id: usize,
}

#[derive(Debug, Clone)]
pub enum GlobalReference {
    Scoped(String),
}

impl ArgDefCollectionType for Vec<LambdaArgDef> {}

#[derive(Debug, Clone)]
pub enum IdRefLocalOrGlobal {
    Id(IdentifierReference),
    Glob(GlobalReference),
}

#[derive(Debug, Clone)]
pub enum NameLinkedExpression {
    Arithmetic(ArithmeticOrLiteralOrId<Box<NameLinkedExpression>, IdRefLocalOrGlobal>),
    Lambda(LambdaExpression<Vec<LambdaArgDef>, NameLinkedBody, ()>),
    Call(CallExpression<Box<NameLinkedExpression>, NameLinkedExpression>),
    MemberAccess(MemberExpression<Box<NameLinkedExpression>>),
    StructConstructor(StructExpression<NameLinkedExpression>),
    AnnotatedExpression(AnnotatedExpression<Box<NameLinkedExpression>, String>),
}
impl ExpressionType for NameLinkedExpression {}
impl ExpressionType for Box<NameLinkedExpression> {}

#[derive(Debug, Clone)]
pub enum NameLinkedStatement {
    //Assignment(AssignmentStatement<String, NameLinkedExpression>),
    DeclAssignment(AssignmentStatement<String, NameLinkedExpression>),
}

#[derive(Debug, Clone)]
pub struct NameLinkedBody {
    pub stmts: Vec<NameLinkedStatement>,
    pub name_to_id_map: HashMap<String, IdentifierReference>,
    pub id_to_ind_map: HashMap<IdentifierReference, usize>,
    pub end_expr: Option<Box<NameLinkedExpression>>,
}

impl BodyType for NameLinkedBody {}

#[derive(Debug)]
pub struct GlobalNameLinkedExprDefs {
    pub map: HashMap<IdentifierReference, NameLinkedExpression>,
    pub name_to_id: HashMap<String, IdentifierReference>,
}

fn from_untyped_body(
    ctx: &HashTrieMap<String, IdentifierReference>,
    id_provider: &mut IdentifierExprIdDispatcher,
    body: &UntypedBody,
) -> NameLinkedBody {
    let mut name_meanings = ctx.clone();
    let mut new_name_meanings = HashTrieMap::new();
    let mut stmts_ind_map = HashTrieMap::new();
    let mut stmts_list = Vec::new();

    for s in &body.stmts {
        match s {
            super::full_untyped::UntypedStatement::DeclAssignment(s) => {
                let id = id_provider.get_id();
                let expr = from_untyped(&name_meanings, id_provider, &s.rhs);
                name_meanings = name_meanings.insert(s.id.clone(), id);
                new_name_meanings = new_name_meanings.insert(s.id.clone(), id);
                stmts_list.push(NameLinkedStatement::DeclAssignment(AssignmentStatement {
                    id: s.id.clone(),
                    rhs: expr,
                }));
                let ind = stmts_list.len() - 1;
                stmts_ind_map = stmts_ind_map.insert(id, ind);
            }
        }
    }

    let end_expr = body
        .end_expr
        .as_ref()
        .map(|f| Box::new(from_untyped(&name_meanings, id_provider, f)));

    NameLinkedBody {
        stmts: stmts_list,
        name_to_id_map: new_name_meanings
            .into_iter()
            .map(|f| (f.0.clone(), f.1.clone()))
            .collect(),
        id_to_ind_map: stmts_ind_map
            .into_iter()
            .map(|f| (f.0.clone(), f.1.clone()))
            .collect(),
        end_expr: end_expr,
    }
}

fn from_untyped(
    ctx: &HashTrieMap<String, IdentifierReference>,
    id_provider: &mut IdentifierExprIdDispatcher,
    expr: &UntypedExpression,
) -> NameLinkedExpression {
    match expr {
        UntypedExpression::Arithmetic(ArithmeticOrLiteralOrId::Arithmetic(e)) => {
            NameLinkedExpression::Arithmetic(ArithmeticOrLiteralOrId::Arithmetic(
                FourArithmeticExpression {
                    op: e.op.clone(),
                    left: Box::new(from_untyped(ctx, id_provider, &e.left)),
                    right: Box::new(from_untyped(ctx, id_provider, &e.right)),
                },
            ))
        }
        UntypedExpression::Lambda(e) => {
            let mut arg_names = Vec::new();
            let mut arg_map = ctx.clone();
            for i in &e.args {
                let id = id_provider.get_id();
                arg_map = arg_map.insert(i.clone(), id);
                arg_names.push(LambdaArgDef {
                    name: i.clone(),
                    id,
                });
            }

            NameLinkedExpression::Lambda(LambdaExpression {
                args: arg_names,
                body: from_untyped_body(&arg_map, id_provider, &e.body),
                caps: (),
            })
        }
        UntypedExpression::Call(e) => NameLinkedExpression::Call(CallExpression {
            fn_expr: Box::new(from_untyped(ctx, id_provider, &e.fn_expr)),
            args: e
                .args
                .iter()
                .map(|(a, b)| (a.clone(), from_untyped(ctx, id_provider, b)))
                .collect(),
        }),
        UntypedExpression::Arithmetic(ArithmeticOrLiteralOrId::Literal(e)) => {
            NameLinkedExpression::Arithmetic(ArithmeticOrLiteralOrId::Literal(e.clone()))
        }
        UntypedExpression::MemberAccess(e) => {
            NameLinkedExpression::MemberAccess(MemberExpression {
                owner: Box::new(from_untyped(ctx, id_provider, &e.owner)),
                name: e.name.clone(),
            })
        }
        UntypedExpression::Arithmetic(ArithmeticOrLiteralOrId::Id(e)) => match e {
            ScopedIdentifier::InScope(scope, id) => {
                NameLinkedExpression::Arithmetic(ArithmeticOrLiteralOrId::Id(
                    IdRefLocalOrGlobal::Glob(GlobalReference::Scoped(id.clone())),
                ))
            }
            ScopedIdentifier::Scopeless(id) => NameLinkedExpression::Arithmetic(
                ArithmeticOrLiteralOrId::Id(IdRefLocalOrGlobal::Id(*ctx.get(id).unwrap())),
            ),
        },
        UntypedExpression::StructConstructor(e) => {
            NameLinkedExpression::StructConstructor(StructExpression {
                fields: e
                    .fields
                    .iter()
                    .map(|(a, b)| (a.clone(), from_untyped(ctx, id_provider, b)))
                    .collect(),
            })
        }
        UntypedExpression::AnnotatedExpression(e) => {
            NameLinkedExpression::AnnotatedExpression(AnnotatedExpression {
                src: Box::new(from_untyped(ctx, id_provider, &e.src)),
                annotations: e.annotations.clone(),
            })
        }
    }
}

pub fn from_untyped_global(unit: &GlobalUntypedExprDefs) -> GlobalNameLinkedExprDefs {
    let mut id_provider = IdentifierExprIdDispatcher::new();
    let mut map = HashTrieMap::new();
    let mut expr_map = HashTrieMap::new();
    // let mut new_map = GlobalNameLinkedExprDefs {
    //     map: HashMap::new(),
    // };
    for i in &unit.names {
        let generated_id = id_provider.get_id();
        let expr = unit.map.get(i).unwrap();
        let process = from_untyped(&map, &mut id_provider, expr);
        map = map.insert(i.clone(), generated_id);
        expr_map = expr_map.insert(generated_id, process);
    }

    GlobalNameLinkedExprDefs {
        name_to_id: map
            .into_iter()
            .map(|(a, b)| (a.clone(), b.clone()))
            .collect(),
        map: expr_map
            .into_iter()
            .map(|(a, b)| (a.clone(), b.clone()))
            .collect(),
    }
}
