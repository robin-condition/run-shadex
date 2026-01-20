use std::{
    collections::{HashMap, hash_map},
    ops::{Add, Deref, Div, Mul, Sub},
};

use rpds::HashTrieMap;

use crate::nodedef::{
    ast::{ArithmeticOp, BoolValuedOp, MathOp},
    ir::{
        CaptureId, FieldId, ParamId,
        llambda::{CapturesInfo, LambdaDef, LambdaValueRef},
        untyped_llambda::{
            UntypedLLambdaFBody, UntypedLLambdaInstr, UntypedLLambdaLambdaDef,
            UntypedLLambdaOpCode,
            interpreter::execution_types::{ConstantBool, ConstantF32, ConstantI32, ConstantU8},
        },
    },
};

pub struct Interpreter {}

pub mod execution_types;

#[derive(Clone, Copy)]
pub enum SpecificScalar {
    F32(f32),
    I32(i32),
    U32(u32),
    U8(u8),
    U1(bool),
}

#[derive(Clone)]
pub struct LambdaValue {
    pub def: UntypedLLambdaLambdaDef,
    pub captured_values: HashTrieMap<CaptureId, Value>,
}

#[derive(Clone)]
pub struct StructValue {
    pub fields: HashTrieMap<FieldId, Value>,
}

#[derive(Clone, Copy)]
pub enum SpecificVector {
    F32x3([f32; 3]),
    I32x3([i32; 3]),
    U8x3([u8; 3]),
    U1x3([bool; 3]),
}

#[derive(Clone)]
pub enum SpecialFn {
    MakeTex,
    Select,
}

#[derive(Clone)]
pub enum Value {
    Scalar(SpecificScalar),
    Vector(SpecificVector),
    Function(LambdaValue),
    Struct(StructValue),

    Tex(usize, usize, usize, Vec<f32>),
    SpecialFn(SpecialFn),
}

impl ArithmeticOp {
    pub fn apply<T: Add<Output = T> + Mul<Output = T> + Div<Output = T> + Sub<Output = T>>(
        self,
        l: T,
        r: T,
    ) -> T {
        match self {
            ArithmeticOp::Add => l + r,
            ArithmeticOp::Sub => l - r,
            ArithmeticOp::Mult => l * r,
            ArithmeticOp::Div => l / r,
        }
    }
}

impl BoolValuedOp {
    pub fn apply<T: PartialOrd + PartialEq>(self, l: T, r: T) -> bool {
        match self {
            BoolValuedOp::Eq => l == r,
            BoolValuedOp::Leq => l <= r,
            BoolValuedOp::Geq => l >= r,
        }
    }
}

fn apply_math_op<
    T: Add<Output = T> + Mul<Output = T> + Div<Output = T> + Sub<Output = T> + PartialEq + PartialOrd,
>(
    op: MathOp,
    l: T,
    r: T,
    to_val: impl Fn(T) -> SpecificScalar,
) -> Value {
    match op {
        MathOp::Arith(arithmetic_op) => Value::Scalar(to_val(arithmetic_op.apply(l, r))),
        MathOp::Comp(bool_valued_op) => {
            Value::Scalar(SpecificScalar::U1(bool_valued_op.apply(l, r)))
        }
    }
}

pub struct JIT;

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    fn interpret_special_fn(&self, spec: SpecialFn, args: &HashMap<String, Value>) -> Value {
        match spec {
            SpecialFn::MakeTex => {
                let width = match args.get("w").unwrap() {
                    Value::Scalar(SpecificScalar::U32(w)) => *w,
                    _ => panic!(),
                } as usize;
                let height = match args.get("h").unwrap() {
                    Value::Scalar(SpecificScalar::U32(w)) => *w,
                    _ => panic!(),
                } as usize;
                let depth = match args.get("d").unwrap() {
                    Value::Scalar(SpecificScalar::U32(w)) => *w,
                    _ => panic!(),
                } as usize;
                let eval = match args.get("v").unwrap() {
                    Value::Function(lam) => lam.clone(),
                    _ => panic!(),
                };

                let mut tex = vec![0f32; width * height * depth];
                let layer_size = width * depth;

                let param_ids = [
                    eval.def
                        .fn_def
                        .params
                        .params_names
                        .get("x")
                        .unwrap()
                        .clone(),
                    eval.def
                        .fn_def
                        .params
                        .params_names
                        .get("y")
                        .unwrap()
                        .clone(),
                ];

                for y in 0..height {
                    let starting_pos = layer_size * y;
                    let ending_pos = starting_pos + layer_size;

                    // This could be parallelized

                    let editable = &mut tex[starting_pos..ending_pos];
                    for x in 0..width {
                        let starting_ind = x * depth;
                        let ending_depth = starting_ind + depth;

                        let mut argctx = HashMap::new();
                        argctx.insert(
                            param_ids[0],
                            Value::Scalar(SpecificScalar::F32(x as f32 / width as f32)),
                        );
                        argctx.insert(
                            param_ids[1],
                            Value::Scalar(SpecificScalar::F32(y as f32 / height as f32)),
                        );

                        let component_evaluator = match self.interpret_lambda(&eval, argctx) {
                            Value::Function(myfun) => myfun,
                            _ => panic!(),
                        };

                        for c in 0..depth {
                            let mut ctx2 = HashMap::new();
                            ctx2.insert(
                                component_evaluator
                                    .def
                                    .fn_def
                                    .params
                                    .params_names
                                    .get("c")
                                    .unwrap()
                                    .clone(),
                                Value::Scalar(SpecificScalar::U32(c as u32)),
                            );
                            let brightness = match self.interpret_lambda(&component_evaluator, ctx2)
                            {
                                Value::Scalar(SpecificScalar::F32(f)) => f,
                                _ => panic!(),
                            };
                            editable[starting_ind + c] = brightness;
                        }
                    }
                }

                Value::Tex(width, height, depth, tex)
            }
            SpecialFn::Select => {
                let cond = match args.get("cond").unwrap() {
                    Value::Scalar(SpecificScalar::U1(b)) => *b,
                    _ => panic!(),
                };
                let a = args.get("then").unwrap();
                let b = args.get("else").unwrap();
                (if cond { a } else { b }).clone()
            }
        }
    }

    fn interpret_instr(
        &self,
        op: &UntypedLLambdaOpCode,
        ctx: &mut HashMap<LambdaValueRef, Value>,
    ) -> Value {
        match &op.0 {
            crate::nodedef::ir::llambda::LLambdaOpCode::Fn(lfun_op_code) => match lfun_op_code {
                crate::nodedef::ir::lfun::LFunOpCode::Struct(lstruct_op_code) => {
                    match lstruct_op_code {
                        crate::nodedef::ir::lstruct::LStructOpCode::Dumb(ldumb_op_code) => {
                            match ldumb_op_code {
                                crate::nodedef::ir::ldumb::LDumbOpCode::Nop => panic!("No op!"),
                                crate::nodedef::ir::ldumb::LDumbOpCode::ConstantNum(
                                    literal_expression_number,
                                ) => match literal_expression_number {
                                    crate::nodedef::ast::LiteralExpressionNumber::LiteralI32(l) => {
                                        Value::Scalar(SpecificScalar::I32(l.v))
                                    }
                                    crate::nodedef::ast::LiteralExpressionNumber::LiteralU32(l) => {
                                        Value::Scalar(SpecificScalar::U32(l.v))
                                    }
                                    crate::nodedef::ast::LiteralExpressionNumber::LiteralF32(l) => {
                                        Value::Scalar(SpecificScalar::F32(l.v))
                                    }
                                },
                                crate::nodedef::ir::ldumb::LDumbOpCode::Arithmetic(
                                    four_arithmetic_expression,
                                ) => {
                                    let left = ctx.get(&four_arithmetic_expression.left).unwrap();
                                    let right = ctx.get(&four_arithmetic_expression.right).unwrap();
                                    let op = four_arithmetic_expression.op.clone();

                                    match (left, right) {
                                        (
                                            Value::Scalar(SpecificScalar::F32(l)),
                                            Value::Scalar(SpecificScalar::F32(r)),
                                        ) => apply_math_op(op, *l, *r, SpecificScalar::F32),
                                        (
                                            Value::Scalar(SpecificScalar::I32(l)),
                                            Value::Scalar(SpecificScalar::I32(r)),
                                        ) => apply_math_op(op, *l, *r, SpecificScalar::I32),
                                        (
                                            Value::Scalar(SpecificScalar::U32(l)),
                                            Value::Scalar(SpecificScalar::U32(r)),
                                        ) => apply_math_op(op, *l, *r, SpecificScalar::U32),
                                        (
                                            Value::Scalar(SpecificScalar::U8(l)),
                                            Value::Scalar(SpecificScalar::U8(r)),
                                        ) => apply_math_op(op, *l, *r, SpecificScalar::U8),
                                        _ => panic!("Can't do math on these things"),
                                    }
                                }
                                crate::nodedef::ir::ldumb::LDumbOpCode::Copy(v) => {
                                    ctx.get(v).unwrap().clone()
                                }
                            }
                        }
                        crate::nodedef::ir::lstruct::LStructOpCode::MemberAccess(
                            struct_expr,
                            field_id,
                        ) => {
                            let struc = ctx.get(struct_expr).unwrap();
                            match struc {
                                Value::Struct(struct_value) => {
                                    struct_value.fields.get(field_id).unwrap().clone()
                                }
                                _ => panic!("Expected struct value"),
                            }
                        }
                        crate::nodedef::ir::lstruct::LStructOpCode::ConstructStruct(
                            struct_ctor,
                        ) => {
                            let mut fld_ctx = HashTrieMap::new();
                            for fld in &struct_ctor.field_names {
                                let fld_valref = struct_ctor.field_infs.get(fld.1).unwrap();
                                let fld_val = ctx.get(fld_valref).unwrap().clone();
                                fld_ctx = fld_ctx.insert(fld.1.clone(), fld_val);
                            }
                            Value::Struct(StructValue { fields: fld_ctx })
                        }
                    }
                }
                crate::nodedef::ir::lfun::LFunOpCode::FnCtor(fn_def) => {
                    Value::Function(LambdaValue {
                        def: LambdaDef {
                            fn_def: fn_def.clone(),
                            captures_info: CapturesInfo::new(),
                        },
                        captured_values: HashTrieMap::new(),
                    })
                }
                crate::nodedef::ir::lfun::LFunOpCode::GlobalFn(globfn) => match globfn.as_str() {
                    "tex" => Value::SpecialFn(SpecialFn::MakeTex),
                    "sel" => Value::SpecialFn(SpecialFn::Select),
                    _ => panic!("Special function not implemented"),
                },
                crate::nodedef::ir::lfun::LFunOpCode::CallFn(func, arg_map) => {
                    let funcval = ctx.get(func).unwrap();
                    match funcval {
                        Value::Function(lambda_value) => {
                            let mut argctx = HashMap::new();
                            for arg in arg_map {
                                let param_id = lambda_value
                                    .def
                                    .fn_def
                                    .params
                                    .params_names
                                    .get(arg.0)
                                    .unwrap()
                                    .clone();
                                let arg_val = ctx.get(arg.1).unwrap().clone();
                                argctx.insert(param_id, arg_val);
                            }
                            self.interpret_lambda(lambda_value, argctx)
                        }
                        Value::SpecialFn(spec_fn) => {
                            let mut args = HashMap::new();
                            for a in arg_map {
                                args.insert(a.0.clone(), ctx.get(a.1).unwrap().clone());
                            }
                            self.interpret_special_fn(spec_fn.clone(), &args)
                        }
                        _ => panic!("Wanted a function value"),
                    }
                }
            },
            crate::nodedef::ir::llambda::LLambdaOpCode::ConstructLambda(lambda_def) => {
                let mut caps = HashTrieMap::new();
                for (i, v) in &lambda_def.captures_info.captures {
                    caps = caps.insert(i.clone(), ctx.get(v).unwrap().clone());
                }
                Value::Function(LambdaValue {
                    def: lambda_def.clone(),
                    captured_values: caps,
                })
            }
        }
    }

    fn interpret_lambda(
        &self,
        lambda_value: &LambdaValue,
        argctx: HashMap<ParamId, Value>,
    ) -> Value {
        let cap_map = lambda_value
            .captured_values
            .iter()
            .map(|(a, b)| (a.clone(), b.clone()))
            .collect();
        self.interpret(&lambda_value.def.fn_def.body, &argctx, &cap_map)
            .expect("Function must return the value")
    }

    pub fn interpret(
        &self,
        bd: &UntypedLLambdaFBody,
        arg_ctx: &HashMap<ParamId, Value>,
        cap_ctx: &HashMap<CaptureId, Value>,
    ) -> Option<Value> {
        let mut val_body: HashMap<LambdaValueRef, Value> = HashMap::new();
        for i in arg_ctx {
            val_body.insert(
                LambdaValueRef::FnValueRef(crate::nodedef::ir::lfun::FnValueRef::Arg(*i.0)),
                i.1.clone(),
            );
        }
        for c in cap_ctx {
            val_body.insert(LambdaValueRef::CaptureRef(*c.0), c.1.clone());
        }

        // Now can actually interpret.
        for inst in bd {
            let v = self.interpret_instr(inst.1, &mut val_body);
            val_body.insert(
                LambdaValueRef::FnValueRef(crate::nodedef::ir::lfun::FnValueRef::InstrId(inst.0)),
                v,
            );
        }

        bd.returned
            .as_ref()
            .map(|r| val_body.get(r).unwrap().clone())
    }
}

impl JIT {
    pub fn new() -> Self {
        Self
    }

    pub fn to_wgsl() -> wgsl_parse::syntax::TranslationUnit {
        todo!()
    }
}
