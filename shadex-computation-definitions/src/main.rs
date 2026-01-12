use nom::Parser;
use shadex_computation_definitions::nodedef::{
    parsing::{parse_expr, parse_term},
    semantic_analysis::free_variables,
};

fn main() {
    let unit = parse_expr()
        .parse_complete(include_str!("../examples/wip_testing.nodedef").as_bytes())
        .unwrap()
        .1;
    let fvs = free_variables(&unit);
    println!("{:?}", unit);
    let v: Vec<_> = fvs.iter().collect();
    println!("{:?}", v);
}
