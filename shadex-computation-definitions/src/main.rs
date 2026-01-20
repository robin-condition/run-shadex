use std::collections::HashMap;

use image::Rgb;
use nom::Parser;
use rpds::HashTrieMap;
use shadex_computation_definitions::nodedef::{
    ast::identifiers_linked,
    ir::{
        lfun::FnValueRef,
        llambda::LambdaValueRef,
        untyped_llambda::{
            UntypedLLambdaOpCode,
            interpreter::{Interpreter, LambdaValue, Value},
        },
    },
    parsing::{parse_expr, parse_global_def_file_specific, parse_term},
    semantic_analysis::free_variables,
};

fn main() {
    let unit = parse_global_def_file_specific(
        include_str!("../examples/interpreter_test.nodedef"), //    .as_bytes()
    )
    .unwrap();

    let mut emitted = unit.emit();
    emitted.1.remove_unnecessary_captures_in_children();

    let interp = Interpreter::new();
    let val = emitted.0.get("entry").unwrap();
    let instr_id = match val {
        LambdaValueRef::FnValueRef(FnValueRef::InstrId(id)) => *id,
        _ => panic!(),
    };

    let lambda_ctor = emitted.1.instrs.get(&instr_id).unwrap();
    let lambda = interp.interpret(match &lambda_ctor.op.0 {
        shadex_computation_definitions::nodedef::ir::llambda::LLambdaOpCode::ConstructLambda(lambda_def) => &lambda_def.fn_def.body,
        _ => panic!()
    }, &HashMap::new(), &HashMap::new()).unwrap();
    let as_tex = match lambda {
        Value::Tex(a, b, c, t) => (a, b, c, t),
        _ => panic!(),
    };

    let vec = as_tex
        .3
        .into_iter()
        .map(|f| (f.clamp(0f32, 1f32) * 256f32).clamp(0f32, 255f32) as u8)
        .collect();

    let img =
        image::ImageBuffer::<Rgb<u8>, Vec<u8>>::from_vec(as_tex.0 as u32, as_tex.1 as u32, vec)
            .unwrap();

    img.save("results/test.png").unwrap();
}
