use rpds::HashTrieMap;
use wgsl_parse::{
    SyntaxNode,
    span::{Span, Spanned},
    syntax::{
        AssignmentStatement, BinaryExpression, Declaration, FormalParameter, Ident, Statement,
        TranslationUnit, TypeExpression,
    },
};

use crate::nodedef::ir::{
    InstrId, lfun,
    llambda::LambdaValueRef,
    maybetyped_llambda::{
        MaybetypedLLambdaFBody, MaybetypedLLambdaFDef, MaybetypedLLambdaInstr,
        interpreter::{
            LambdaValue, SpecificScalar, apply_math_op_to_spec_scals, specific_scalar_from_lit_val,
        },
    },
};

pub struct NameEmitter {
    next_id: usize,
}

impl NameEmitter {
    pub fn new() -> Self {
        Self { next_id: 0 }
    }
    pub fn next_name(&mut self) -> String {
        let id = self.next_id;
        self.next_id += 1;
        format!("id{}", id)
    }
}

pub struct FunctionCallInformation {
    pub id: String,
    pub expects_context_argument: Option<()>,
    pub expects_globals: Vec<String>,
}

pub struct ShaderCtx {
    pub functions: Vec<wgsl_parse::syntax::Function>,
    pub types: Vec<wgsl_parse::syntax::Struct>,
    pub naming_ctx: NameEmitter,
}

#[derive(Clone)]
pub enum CompilationValueSource {
    ConstantScalar(SpecificScalar),
    Buffer(),
    LocalVariable(String),
}

pub struct CompilationValueSources {
    pub data_sources: HashTrieMap<LambdaValueRef, CompilationValueSource>,
}

impl CompilationValueSources {
    pub fn with(self, vref: LambdaValueRef, vsrc: CompilationValueSource) -> Self {
        Self {
            data_sources: self.data_sources.insert(vref, vsrc),
        }
    }
}

impl LambdaValue {
    pub fn turn_into_shader(&self) -> ShaderCtx {
        todo!()
    }
}

fn specific_scalar_to_expr(sc: SpecificScalar) -> wgsl_parse::syntax::LiteralExpression {
    match sc {
        SpecificScalar::F32(v) => wgsl_parse::syntax::LiteralExpression::F32(v),
        SpecificScalar::I32(v) => wgsl_parse::syntax::LiteralExpression::I32(v),
        SpecificScalar::U32(v) => wgsl_parse::syntax::LiteralExpression::U32(v),
        SpecificScalar::U8(v) => todo!(), //wgsl_parse::syntax::LiteralExpression::(v),
        SpecificScalar::U1(v) => wgsl_parse::syntax::LiteralExpression::Bool(v),
    }
}

fn local_var_to_expr(sc: String) -> wgsl_parse::syntax::Ident {
    wgsl_parse::syntax::Ident::new(sc)
}

fn comp_src_to_expr(src: CompilationValueSource) -> wgsl_parse::syntax::Expression {
    match src {
        CompilationValueSource::ConstantScalar(specific_scalar) => {
            wgsl_parse::syntax::Expression::Literal(specific_scalar_to_expr(specific_scalar))
        }
        CompilationValueSource::Buffer() => todo!(),
        CompilationValueSource::LocalVariable(v) => {
            wgsl_parse::syntax::Expression::TypeOrIdentifier(TypeExpression::new(
                local_var_to_expr(v),
            ))
        }
    }
}

fn emit_statements(
    iid: &InstrId,
    statement: &MaybetypedLLambdaInstr,
    shader: &mut ShaderCtx,
    outputs: &mut Vec<Spanned<Statement>>,
    mut ctx: CompilationValueSources,
) -> CompilationValueSources {
    let selfref = LambdaValueRef::FnValueRef(lfun::FnValueRef::InstrId(*iid));
    match &statement.op.0 {
        crate::nodedef::ir::llambda::LLambdaOpCode::Fn(lfun_op_code) => match lfun_op_code {
            lfun::LFunOpCode::Struct(lstruct_op_code) => match lstruct_op_code {
                crate::nodedef::ir::lstruct::LStructOpCode::Dumb(ldumb_op_code) => {
                    match ldumb_op_code {
                        crate::nodedef::ir::ldumb::LDumbOpCode::Nop => {
                            return ctx;
                        }
                        crate::nodedef::ir::ldumb::LDumbOpCode::ConstantNum(
                            literal_expression_number,
                        ) => {
                            let val = specific_scalar_from_lit_val(literal_expression_number);
                            let val_src = CompilationValueSource::ConstantScalar(val);
                            ctx = ctx.with(selfref, val_src);
                            return ctx;
                        }
                        crate::nodedef::ir::ldumb::LDumbOpCode::Arithmetic(
                            four_arithmetic_expression,
                        ) => {
                            let op = four_arithmetic_expression.op.clone();

                            let left = &ctx.data_sources[&four_arithmetic_expression.left];
                            let right = &ctx.data_sources[&four_arithmetic_expression.right];

                            match (left, right) {
                                (
                                    CompilationValueSource::ConstantScalar(l),
                                    CompilationValueSource::ConstantScalar(r),
                                ) => {
                                    let val = apply_math_op_to_spec_scals(op, *l, *r);
                                    let valsrc = CompilationValueSource::ConstantScalar(val);
                                    ctx = ctx.with(selfref, valsrc);
                                    return ctx;
                                }
                                (_, CompilationValueSource::Buffer()) => {
                                    panic!("Can't math buffers.")
                                }
                                (CompilationValueSource::Buffer(), _) => {
                                    panic!("Can't math buffers.")
                                }
                                (
                                    CompilationValueSource::ConstantScalar(_)
                                    | CompilationValueSource::LocalVariable(_),
                                    CompilationValueSource::ConstantScalar(_)
                                    | CompilationValueSource::LocalVariable(_),
                                ) => {
                                    let l = comp_src_to_expr(left.clone());
                                    let r = comp_src_to_expr(right.clone());
                                    let expr =
                                        wgsl_parse::syntax::Expression::Binary(BinaryExpression {
                                            operator: match op {
                                                crate::nodedef::ast::MathOp::Arith(
                                                    arithmetic_op,
                                                ) => match arithmetic_op {
                                                    crate::nodedef::ast::ArithmeticOp::Add => wgsl_parse::syntax::BinaryOperator::Addition,
                                                    crate::nodedef::ast::ArithmeticOp::Sub => wgsl_parse::syntax::BinaryOperator::Subtraction,
                                                    crate::nodedef::ast::ArithmeticOp::Mult => wgsl_parse::syntax::BinaryOperator::Multiplication,
                                                    crate::nodedef::ast::ArithmeticOp::Div => wgsl_parse::syntax::BinaryOperator::Division,
                                                },
                                                crate::nodedef::ast::MathOp::Comp(
                                                    bool_valued_op,
                                                ) => match bool_valued_op {
                                                    crate::nodedef::ast::BoolValuedOp::Eq => wgsl_parse::syntax::BinaryOperator::Equality,
                                                    crate::nodedef::ast::BoolValuedOp::Leq => wgsl_parse::syntax::BinaryOperator::LessThanEqual,
                                                    crate::nodedef::ast::BoolValuedOp::Geq => wgsl_parse::syntax::BinaryOperator::GreaterThanEqual,
                                                },
                                            },
                                            left: Spanned::new(l, Span::default()),
                                            right: Spanned::new(r, Span::default()),
                                        });
                                    let new_id = shader.naming_ctx.next_name();
                                    let id_expr = local_var_to_expr(new_id.clone());
                                    let decl_statement = Statement::Declaration(Declaration {
                                        attributes: Vec::new(),
                                        kind: wgsl_parse::syntax::DeclarationKind::Let,
                                        ident: id_expr,
                                        ty: None, // TODO
                                        initializer: Some(Spanned::new(expr, Span::default())),
                                    });
                                    outputs.push(Spanned::new(decl_statement, Span::default()));
                                    let comp_src = CompilationValueSource::LocalVariable(new_id);

                                    ctx = ctx.with(selfref, comp_src);
                                    return ctx;
                                }
                            }

                            todo!()
                        }
                        crate::nodedef::ir::ldumb::LDumbOpCode::Copy(cp) => {
                            let src = ctx.data_sources[cp].clone();
                            ctx = ctx.with(selfref, src);
                            return ctx;
                        }
                    }
                }
                crate::nodedef::ir::lstruct::LStructOpCode::MemberAccess(_, field_id) => todo!(),
                crate::nodedef::ir::lstruct::LStructOpCode::ConstructStruct(struct_ctor) => todo!(),
            },
            lfun::LFunOpCode::FnCtor(fn_def) => todo!(),
            lfun::LFunOpCode::GlobalFn(_) => todo!(),
            lfun::LFunOpCode::CallFn(_, hash_map) => todo!(),
        },
        crate::nodedef::ir::llambda::LLambdaOpCode::ConstructLambda(lambda_def) => todo!(),
    }
}

impl MaybetypedLLambdaFBody {
    pub fn emit_body(
        &self,
        shader: &mut ShaderCtx,
        statements: &mut Vec<Spanned<Statement>>,
        mut ctxes: CompilationValueSources,
    ) -> FunctionCallInformation {
        //for s in self {
        //    ctxes = emit_statements(s.1, shader, statements, ctxes);
        //}
        //fndef.parameters.push(FormalParameter {});
        todo!()
    }
}

impl MaybetypedLLambdaFDef {}
