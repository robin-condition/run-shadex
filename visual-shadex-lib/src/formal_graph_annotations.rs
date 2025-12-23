use shadex_backend::{execution::typechecking::NodeGraphFormalTypeAnalysis, nodegraph::NodeGraph};

pub struct FormalGraph {
    pub formal_graph: NodeGraph,
    pub typecheck: Option<NodeGraphFormalTypeAnalysis>,
}
