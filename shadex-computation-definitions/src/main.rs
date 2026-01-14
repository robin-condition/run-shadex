use nom::Parser;
use shadex_computation_definitions::nodedef::{
    parsing::{parse_expr, parse_global_def_file_specific, parse_term},
    semantic_analysis::free_variables,
};

fn main() {
    let unit = parse_global_def_file_specific(
        include_str!("../examples/wip_testing.nodedef"), //    .as_bytes()
    )
    .unwrap();
    let fvs = free_variables(&unit.get("test").unwrap());
    println!("{:?}", unit);
    let v: Vec<_> = fvs.iter().collect();
    println!("{:?}", v);
    println!("{:?}", unit.get("vector").unwrap())
}
