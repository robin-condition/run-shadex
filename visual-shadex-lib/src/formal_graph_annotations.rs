use shadex_backend::{
    nodegraph::{NodeGraph, TypedNodeGraph},
    typechecking::NodeGraphFormalTypeAnalysis,
};

pub struct FormalGraph {
    pub formal_graph: TypedNodeGraph,
    pub typecheck: Option<NodeGraphFormalTypeAnalysis>,
}
