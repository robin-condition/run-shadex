mod vnode_infos;
use egui::ahash::HashMap;
pub use vnode_infos::{ConstantInfo, VisualNode, VisualNodeInfo};

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct VNodeId(usize);

pub struct VNodeOutputRef {
    pub source: VNodeId,
    pub output_ind: usize
}
pub struct VNodeInputRef {
    pub dest: VNodeId,
    pub input_ind: usize
}

pub struct VisualNodeGraph {
    nodes: HashMap<VNodeId, VisualNode>,
    next_id: VNodeId
}

impl Default for VisualNodeGraph {
    fn default() -> Self {
        Self { nodes: Default::default(), next_id: VNodeId(0usize) }
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

    pub fn show(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        // Make a placeholder for the lines below all the nodes
        let lines_id = ui.painter().add(egui::Shape::Noop);

        // Draw and update all the nodes
        for n in &mut self.nodes {
            changed = n.1.show_box(ui).inner | changed;
        }

        // Compute all the lines and draw them

        changed
    }
}