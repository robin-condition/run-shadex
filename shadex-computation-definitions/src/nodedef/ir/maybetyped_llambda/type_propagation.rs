use std::collections::HashMap;

use rpds::HashTrieMap;

use crate::nodedef::{
    ast::typing::{CapturesType, FunctionType, LambdaType, Type},
    ir::{
        llambda::LambdaValueRef,
        maybetyped_llambda::{
            MaybetypedLLambdaFBody, MaybetypedLLambdaFDef, MaybetypedLLambdaOpCode,
        },
    },
};

impl MaybetypedLLambdaFDef {
    pub fn propagate_types(&mut self, mut ctx: HashTrieMap<LambdaValueRef, Type>) -> FunctionType {
        let mut arg_types = HashMap::new();
        for a in &self.params.params_names {
            let t = self.params.param_infos.get(a.1).unwrap();

            ctx = ctx.insert(
                LambdaValueRef::FnValueRef(crate::nodedef::ir::lfun::FnValueRef::Arg(*a.1)),
                t.clone(),
            );
            arg_types.insert(a.0.clone(), t.clone());
        }

        let ret_type = self.body.propagate_types(ctx);

        let fn_type = FunctionType(arg_types, Box::new(ret_type));
        fn_type
    }
}

impl MaybetypedLLambdaOpCode {
    pub fn assess_type(&mut self, ctx: HashTrieMap<LambdaValueRef, Type>) -> Type {
        match &mut self.0 {
            crate::nodedef::ir::llambda::LLambdaOpCode::Fn(lfun_op_code) => match lfun_op_code {
                crate::nodedef::ir::lfun::LFunOpCode::Struct(lstruct_op_code) => {
                    match lstruct_op_code {
                        crate::nodedef::ir::lstruct::LStructOpCode::Dumb(ldumb_op_code) => {
                            match ldumb_op_code {
                                crate::nodedef::ir::ldumb::LDumbOpCode::Nop => Type::Unknown,
                                crate::nodedef::ir::ldumb::LDumbOpCode::ConstantNum(
                                    literal_expression_number,
                                ) => match literal_expression_number {
                                    crate::nodedef::ast::LiteralExpressionNumber::LiteralI32(_) => {
                                        todo!()
                                    }
                                    crate::nodedef::ast::LiteralExpressionNumber::LiteralU32(_) => {
                                        Type::U32
                                    }
                                    crate::nodedef::ast::LiteralExpressionNumber::LiteralF32(_) => {
                                        Type::F32
                                    }
                                },
                                crate::nodedef::ir::ldumb::LDumbOpCode::Arithmetic(e) => {
                                    let ltype = ctx.get(&e.left).unwrap();
                                    let rtype = ctx.get(&e.right).unwrap();
                                    let bool_ret = match &e.op {
                                        crate::nodedef::ast::MathOp::Arith(_) => false,
                                        crate::nodedef::ast::MathOp::Comp(_) => true,
                                    };
                                    match (ltype, rtype) {
                                        (&Type::F32, &Type::F32) => {
                                            if bool_ret {
                                                Type::Bool
                                            } else {
                                                Type::F32
                                            }
                                        }
                                        (&Type::U32, &Type::U32) => {
                                            if bool_ret {
                                                Type::Bool
                                            } else {
                                                Type::U32
                                            }
                                        }
                                        _ => panic!("Unsupported type combination"),
                                    }
                                }
                                crate::nodedef::ir::ldumb::LDumbOpCode::Copy(src) => {
                                    ctx.get(src).unwrap().clone()
                                }
                            }
                        }
                        crate::nodedef::ir::lstruct::LStructOpCode::MemberAccess(_, field_id) => {
                            todo!()
                        }
                        crate::nodedef::ir::lstruct::LStructOpCode::ConstructStruct(
                            struct_ctor,
                        ) => todo!(),
                    }
                }
                crate::nodedef::ir::lfun::LFunOpCode::FnCtor(fn_def) => {
                    Type::Function(fn_def.propagate_types(ctx))
                }
                crate::nodedef::ir::lfun::LFunOpCode::GlobalFn(name) => match name.as_str() {
                    "sel" => Type::Function(FunctionType(
                        [
                            ("cond".to_string(), Type::Bool),
                            ("then".to_string(), Type::F32),
                            ("else".to_string(), Type::F32),
                        ]
                        .into(),
                        Box::new(Type::F32),
                    )),
                    "tex" => Type::Function(FunctionType(
                        [
                            ("w".to_string(), Type::U32),
                            ("h".to_string(), Type::U32),
                            ("d".to_string(), Type::U32),
                            ("v".to_string(), Type::Unknown),
                        ]
                        .into(),
                        Box::new(Type::Texture),
                    )),
                    _ => Type::Unknown,
                }, // This is todo. Needs parametric polymorphism to do it right
                crate::nodedef::ir::lfun::LFunOpCode::CallFn(func, hash_map) => {
                    let f_type = ctx.get(func).unwrap();
                    match f_type {
                        Type::Lambda(LambdaType {
                            captures: _,
                            function: function_type,
                        })
                        | Type::Function(function_type) => (*function_type.1).clone(),
                        _ => Type::Unknown,
                    }
                }
            },
            crate::nodedef::ir::llambda::LLambdaOpCode::ConstructLambda(lambda_def) => {
                let mut cap_types_saved = HashMap::new();
                let mut cap_types = HashTrieMap::new();
                for c in &lambda_def.captures_info.captures {
                    let src_typ = ctx.get(c.1).unwrap();

                    cap_types = cap_types.insert(LambdaValueRef::CaptureRef(*c.0), src_typ.clone());
                    cap_types_saved.insert(*c.0, src_typ.clone());
                }

                let mut cap_types_vec: Vec<_> = cap_types_saved.iter().collect();
                cap_types_vec.sort_by(|a, b| a.0.0.cmp(&b.0.0));
                let res = cap_types_vec.into_iter().map(|(_, b)| b.clone());

                let fn_type = lambda_def.fn_def.propagate_types(cap_types);
                Type::Lambda(LambdaType {
                    captures: CapturesType(res.collect()),
                    function: fn_type,
                })
            }
        }
    }
}

impl MaybetypedLLambdaFBody {
    pub fn propagate_types(&mut self, mut ctx: HashTrieMap<LambdaValueRef, Type>) -> Type {
        let ids: Vec<_> = (self as &Self).into_iter().map(|(a, _)| a).collect();
        for id in ids {
            let instr = self.instrs.get_mut(&id).unwrap();
            let t = instr.op.assess_type(ctx.clone());
            instr.typ = t.clone();

            ctx = ctx.insert(
                LambdaValueRef::FnValueRef(crate::nodedef::ir::lfun::FnValueRef::InstrId(id)),
                t,
            );
        }

        self.returned
            .map_or(Type::Unit, |f| ctx.get(&f).unwrap().clone())
    }
}
