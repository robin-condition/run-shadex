use nom::Parser;
use shadex_computation_definitions::nodedef::parsing::{parse_expr, parse_term};

fn main() {
    let unit = parse_expr()
        .parse_complete(include_str!("../examples/wip_testing.nodedef").as_bytes())
        .unwrap()
        .1;
    println!("{:?}", unit);
}
