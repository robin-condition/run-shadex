use nom::Parser;
use shadex_computation_definitions::nodedef::{
    ast::identifiers_linked,
    parsing::{parse_expr, parse_global_def_file_specific, parse_term},
    semantic_analysis::free_variables,
};

fn main() {
    let unit = parse_global_def_file_specific(
        include_str!("../examples/wip_testing.nodedef"), //    .as_bytes()
    )
    .unwrap();
    let fvs = free_variables(&unit.map.get("test").unwrap());
    println!("{:?}", unit);
    let v: Vec<_> = fvs.iter().collect();
    println!("{:?}", v);
    println!("{:?}", unit.map.get("vector").unwrap());

    //let id_linked = identifiers_linked::from_untyped_global(&unit);

    println!("----");

    let mut emitted = unit.emit();
    emitted.1.remove_unnecessary_captures_in_children();

    println!("{}", emitted.1);

    //let id = id_linked.name_to_id.get("vector").unwrap();

    //println!("{:?}", id_linked.map.get(id).unwrap());

    //println!("{:?}", emitted.1);
}
