use std::collections::{HashMap, HashSet};

use rpds::HashTrieMap;

use crate::nodedef::{
    ast::{
        AssignmentStatement, FourArithmeticExpression, LambdaExpression,
        full_maybetyped::{
            GlobalMaybetypedExprDefs, MaybetypedBody, MaybetypedExpression,
            MaybetypedExpressionShape, MaybetypedLambdaExpressionShape, MaybetypedStatement,
            ScopedIdentifier,
        },
        mathy_ast::ArithmeticOrLiteralOrId,
        typing::Type,
    },
    ir::{
        FieldId, InstrId,
        ldumb::LDumbOpCode,
        lfun::{FnBody, FnValueRef, LFunOpCode, ParamSupplier},
        llambda::{CapturesInfo, LLambdaOpCode, LambdaDef, LambdaValueRef},
        lstruct::{LStructOpCode, StructCtor},
        maybetyped_llambda::{
            MaybetypedLLambdaFBody, MaybetypedLLambdaFDef, MaybetypedLLambdaOpCode,
        },
    },
};

impl MaybetypedLambdaExpressionShape {
    fn create_body_after_args(
        &self,
        cap_and_args_ctx: &HashTrieMap<String, LambdaValueRef>,
    ) -> MaybetypedLLambdaFBody {
        let mut val_ctx = cap_and_args_ctx.clone();
        let mut body = FnBody::new();
        for stmt in &self.body.stmts {
            stmt.emit(&mut val_ctx, &mut body);
        }
        if let Some(b) = &self.body.end_expr {
            let ret = b.emit(&mut val_ctx, &mut body);
            body.returned = Some(LambdaValueRef::FnValueRef(FnValueRef::InstrId(ret)));
        }
        body
    }

    pub fn create_fn_def(
        &self,
        cap_ctx: &HashTrieMap<String, LambdaValueRef>,
    ) -> MaybetypedLLambdaFDef {
        let mut param_supplier = ParamSupplier::new();
        let mut ctx = cap_ctx.clone();
        for n in &self.args {
            let id = param_supplier.add_param(n.0.clone(), n.1.clone());
            ctx = ctx.insert(n.0.clone(), LambdaValueRef::FnValueRef(FnValueRef::Arg(id)));
        }
        let bd = self.create_body_after_args(&ctx);
        MaybetypedLLambdaFDef {
            body: bd,
            params: param_supplier,
        }
    }
}

impl MaybetypedExpression {
    pub fn emit(
        &self,
        ctx: &mut HashTrieMap<String, LambdaValueRef>,
        body: &mut MaybetypedLLambdaFBody,
    ) -> InstrId {
        match &self.shape {
            MaybetypedExpressionShape::Arithmetic(a) => match a {
                ArithmeticOrLiteralOrId::Arithmetic(e) => {
                    let lhs = e.left.emit(ctx, body);
                    let rhs = e.right.emit(ctx, body);
                    body.append_instr(
                        LDumbOpCode::Arithmetic(FourArithmeticExpression {
                            op: e.op.clone(),
                            left: LambdaValueRef::FnValueRef(FnValueRef::InstrId(lhs)),
                            right: LambdaValueRef::FnValueRef(FnValueRef::InstrId(rhs)),
                        })
                        .into(),
                        Type::Unknown,
                    )
                }
                ArithmeticOrLiteralOrId::Literal(l) => {
                    body.append_instr(LDumbOpCode::ConstantNum(*l).into(), Type::Unknown)
                }
                ArithmeticOrLiteralOrId::Id(name) => match name {
                    ScopedIdentifier::InScope(_scope, n) => {
                        body.append_instr(LFunOpCode::GlobalFn(n.clone()).into(), Type::Unknown)
                    }
                    ScopedIdentifier::Scopeless(n) => body.append_instr(
                        LDumbOpCode::Copy(*ctx.get(n).unwrap()).into(),
                        Type::Unknown,
                    ),
                },
            },
            MaybetypedExpressionShape::Lambda(e) => {
                let mut captured_ctx = HashTrieMap::new();
                let mut capture_inf = CapturesInfo::new();

                for c in ctx.iter() {
                    let new_captured_id = capture_inf.add_capture(*c.1);
                    captured_ctx = captured_ctx
                        .insert(c.0.clone(), LambdaValueRef::CaptureRef(new_captured_id));
                }

                let lfn = e.create_fn_def(&captured_ctx);

                body.append_instr(
                    LLambdaOpCode::ConstructLambda(LambdaDef {
                        fn_def: lfn,
                        captures_info: capture_inf,
                    })
                    .into(),
                    Type::Unknown,
                )
            }
            MaybetypedExpressionShape::MemberAccess(e) => {
                let owner = e.owner.emit(ctx, body);
                body.append_instr(
                    LStructOpCode::MemberAccess(
                        LambdaValueRef::FnValueRef(FnValueRef::InstrId(owner)),
                        FieldId(e.name.clone()),
                    )
                    .into(),
                    Type::Unknown,
                )
            }
            MaybetypedExpressionShape::StructConstructor(e) => {
                let flds: HashMap<_, _> = e
                    .fields
                    .iter()
                    .map(|(s, e1)| {
                        (
                            s.clone(),
                            LambdaValueRef::FnValueRef(FnValueRef::InstrId(e1.emit(ctx, body))),
                        )
                    })
                    .collect();
                body.append_instr(
                    LStructOpCode::ConstructStruct(StructCtor::from_map(&flds)).into(),
                    Type::Unknown,
                )
            }
            MaybetypedExpressionShape::Call(e) => {
                let fn_ref = e.fn_expr.emit(ctx, body);
                let mut arg_vals = HashMap::new();
                for (n, v) in &e.args {
                    arg_vals.insert(
                        n.clone(),
                        LambdaValueRef::FnValueRef(FnValueRef::InstrId(v.emit(ctx, body))),
                    );
                }
                body.append_instr(
                    LFunOpCode::CallFn(
                        LambdaValueRef::FnValueRef(FnValueRef::InstrId(fn_ref)),
                        arg_vals,
                    )
                    .into(),
                    Type::Unknown,
                )
            }

            MaybetypedExpressionShape::AnnotatedExpression(annotated_expression) => todo!(),
        }
    }
}

impl MaybetypedStatement {
    pub fn emit(
        &self,
        ctx: &mut HashTrieMap<String, LambdaValueRef>,
        body: &mut MaybetypedLLambdaFBody,
    ) {
        match self {
            MaybetypedStatement::DeclAssignment(s) => {
                let id = s.id.clone();
                let res = s.rhs.emit(ctx, body);

                *ctx = ctx.insert(id, LambdaValueRef::FnValueRef(FnValueRef::InstrId(res)));
            }
        }
    }
}

impl GlobalMaybetypedExprDefs {
    pub fn emit(&self) -> (HashTrieMap<String, LambdaValueRef>, MaybetypedLLambdaFBody) {
        let mut body = MaybetypedLLambdaFBody::new();
        let mut ctx = HashTrieMap::new();
        for n in &self.names {
            let id = self.map.get(n).unwrap().emit(&mut ctx, &mut body);
            ctx = ctx.insert(
                n.clone(),
                LambdaValueRef::FnValueRef(FnValueRef::InstrId(id)),
            );
        }
        (ctx, body)
    }
}
