use run_shadex::{
    execution::typechecking,
    parsing::{construct_node_graph, parse_whole_input},
};

fn main() {
    let input_text = include_str!("../examples/test_fv.shadex");
    let full_output = parse_whole_input(input_text.as_bytes()).unwrap().1;
    let constructed = construct_node_graph(full_output).unwrap();

    let mut typechecker = typechecking::NodeGraphFormalTypeAnalysis::analyze(&constructed);
    println!("{:#?}", constructed);
    println!("{:#?}", typechecker);
}
