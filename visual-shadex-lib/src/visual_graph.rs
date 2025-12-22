mod vnode_infos;
use egui::ahash::HashMap;
pub use vnode_infos::{VisualNode, VisualNodeInfo, add::AddInfo, constant::ConstantInfo};

use crate::InteractionState;

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct VNodeId(usize);

#[derive(Clone)]
pub struct VNodeOutputRef {
    pub source: VNodeId,
    pub output_ind: usize,
}
pub struct VNodeInputRef {
    pub dest: VNodeId,
    pub input_ind: usize,
}

pub struct VisualNodeGraph {
    nodes: HashMap<VNodeId, VisualNode>,
    next_id: VNodeId,
}

impl Default for VisualNodeGraph {
    fn default() -> Self {
        Self {
            nodes: Default::default(),
            next_id: VNodeId(0usize),
        }
    }
}

impl VisualNodeGraph {
    pub fn add_node(&mut self, node: VisualNode) -> VNodeId {
        let id = self.next_id;
        self.next_id = VNodeId(self.next_id.0 + 1);
        self.nodes.insert(id, node);
        id
    }

    pub fn get_node(&self, id: &VNodeId) -> &VisualNode {
        self.nodes.get(id).unwrap()
    }

    pub fn get_node_mut(&mut self, id: &VNodeId) -> &mut VisualNode {
        self.nodes.get_mut(id).unwrap()
    }

    pub fn show(&mut self, ui: &mut egui::Ui, mode: &mut InteractionState) -> bool {
        let mut changed = false;

        // Make a placeholder for the lines below all the nodes
        let lines_shape_id = ui.painter().add(egui::Shape::Noop);

        // Draw and update all the nodes
        for n in &mut self.nodes {
            changed = n.1.show_box(ui, mode).inner | changed;
        }

        // Compute all the lines and draw them
        let mut line_vec = Vec::new();

        for (_, n) in &self.nodes {}

        ui.painter().set(lines_shape_id, line_vec);

        changed
    }
}
