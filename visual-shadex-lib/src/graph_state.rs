use std::collections::HashMap;

use shadex_backend::{nodegraph::NodeGraph, typechecking::NodeGraphFormalTypeAnalysis};

use crate::{
    InteractionState, formal_graph_annotations::FormalGraph, visual_graph::VisualNodeGraph,
};

pub struct NodeGraphState {
    pub visual_graph: VisualNodeGraph,
    pub formal_graph: Result<FormalGraph, ()>,
}

impl NodeGraphState {
    pub fn new() -> Self {
        let ngraph = NodeGraph::new();
        let typecheck = NodeGraphFormalTypeAnalysis::analyze(&ngraph);
        NodeGraphState {
            visual_graph: VisualNodeGraph::default(),
            formal_graph: Ok(FormalGraph {
                formal_graph: ngraph,
                typecheck,
                vnode_to_fnode: HashMap::new(),
            }),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, mode: &mut InteractionState) -> bool {
        let changed = self.visual_graph.show(
            ui,
            mode,
            self.formal_graph.as_ref().map(|f| Some(f)).unwrap_or(None),
        );
        if changed {
            self.formal_graph = self.visual_graph.to_formal();
        }

        changed
    }
}
