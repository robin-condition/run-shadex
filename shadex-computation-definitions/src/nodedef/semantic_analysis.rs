use rpds::HashTrieSet;

use crate::nodedef::ast::full_untyped::{UntypedBody, UntypedExpression, UntypedStatement};

#[derive(Debug)]
pub enum PrimitiveType {
    F32,
    I32,
    U32,
}

#[derive(Debug)]
pub enum TypeInfo {
    Vector(usize, PrimitiveType),
    Primitive(PrimitiveType),
    Complex(Box<ClosureContextType>),
}

#[derive(Debug)]
pub struct FieldInfo {
    pub name: String,
    pub typeinfo: TypeInfo,
}

#[derive(Debug)]
pub struct ClosureContextType {
    pub fields: Vec<FieldInfo>,
}

fn set_union(s1: HashTrieSet<String>, s2: HashTrieSet<String>) -> HashTrieSet<String> {
    let mut res = s1;
    for i in s2.into_iter() {
        res = res.insert(i.clone());
    }
    res
}

pub fn body_fvs(body: &UntypedBody) -> HashTrieSet<String> {
    let mut to_ret = HashTrieSet::new();
    for s in body.stmts.iter().rev() {
        to_ret = statement_fvs(s, to_ret);
    }
    to_ret
}

pub fn statement_fvs(stmt: &UntypedStatement, belows: HashTrieSet<String>) -> HashTrieSet<String> {
    match stmt {
        UntypedStatement::Assignment(s) => {
            println!("{}", s.id);
            set_union(belows, free_variables(&s.rhs).insert(s.id.clone()))
        }
        UntypedStatement::DeclAssignment(s) => {
            set_union(belows.remove(&s.id), free_variables(&s.rhs))
        }
    }
}

pub fn free_variables(expr: &UntypedExpression) -> HashTrieSet<String> {
    match expr {
        UntypedExpression::Arithmetic(e) => {
            set_union(free_variables(&e.left), free_variables(&e.right))
        }
        UntypedExpression::Lambda(e) => {
            let mut fvs = body_fvs(&e.body);
            for a in &e.args {
                fvs = fvs.remove(&a.name);
            }
            fvs
        }
        UntypedExpression::Call(e) => {
            let mut res = free_variables(&e.fn_expr);
            for i in &e.args {
                res = set_union(res, free_variables(&i.1));
            }
            res
        }
        UntypedExpression::LiteralI32(_) => HashTrieSet::new(),
        UntypedExpression::LiteralU32(_) => HashTrieSet::new(),
        UntypedExpression::LiteralF32(_) => HashTrieSet::new(),
        UntypedExpression::MemberAccess(e) => free_variables(&e.owner),
        UntypedExpression::ScopedIdentifier(e) => match e {
            crate::nodedef::ast::full_untyped::ScopedIdentifier::InScope(_, _) => {
                HashTrieSet::new()
            }
            crate::nodedef::ast::full_untyped::ScopedIdentifier::Scopeless(name) => {
                HashTrieSet::new().insert(name.clone())
            }
        },
    }
}
