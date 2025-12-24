use shadex_backend::{
    execution::typechecking,
    parsing::{construct_node_graph, parse_whole_input, type_parsing::parse_type_universe},
};

fn main() {
    let types_text = include_str!("../../examples/typeland.shadextypes");
    let universe = parse_type_universe(types_text).unwrap();

    let input_text = include_str!("../../examples/test_fv.shadex");
    let full_output = parse_whole_input(input_text.as_bytes()).unwrap().1;
    let constructed = construct_node_graph(universe, full_output).unwrap();

    let typechecker = typechecking::NodeGraphFormalTypeAnalysis::analyze(&constructed);
    println!("{:#?}", constructed);
    println!("{:#?}", typechecker);
}
