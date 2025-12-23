mod vnode_infos;
use std::any;

use egui::{Color32, Pos2, Stroke, ahash::HashMap};
pub use vnode_infos::{VisualNode, VisualNodeInfo, add::AddInfo, constant::ConstantInfo};

use crate::{DraggingState, InteractionState, helpers::draw_line};

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct VNodeId(usize);

#[derive(Clone, Copy)]
pub struct VNodeOutputRef {
    pub source: VNodeId,
    pub output_ind: usize,
}

#[derive(Clone, Copy)]
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

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        mode: &mut InteractionState,
    ) -> bool {
        let mut changed = false;

        // Make a placeholder for the lines below all the nodes
        let lines_shape_id = ui.painter().add(egui::Shape::Noop);

        let mut any_drag_stopped = false;

        // Make sure that if nothing is hovered, it will be treated as a line-to-cursor.
        match &mode.dragging {
            DraggingState::DraggingLineFromInputPort(inp, _) => mode.dragging = DraggingState::DraggingLineFromInputPort(*inp, None),
            DraggingState::DraggingLineFromOutputPort(_, outp) => mode.dragging = DraggingState::DraggingLineFromOutputPort(None, *outp),
            _ => ()
        };

        let mut mouse_pos = None;

        // Draw and update all the nodes
        for n in &mut self.nodes {
            changed =
                n.1.show_box(ui, *n.0, mode, &mut any_drag_stopped, &mut mouse_pos)
                    .inner
                    | changed;
        }

        if any_drag_stopped {
            if let crate::DraggingState::DraggingLineFromInputPort(inp, Some(outp)) = &mode.dragging
            {
                let node = self.get_node_mut(&inp.dest);
                if node.input_ports.len() > inp.input_ind {
                    node.input_ports[inp.input_ind].input_source = Some(*outp);
                }
            }
            if let crate::DraggingState::DraggingLineFromOutputPort(Some(inp), outp) =
                &mode.dragging
            {
                let node = self.get_node_mut(&inp.dest);
                if node.input_ports.len() > inp.input_ind {
                    node.input_ports[inp.input_ind].input_source = Some(*outp);
                }
            }

            mode.dragging = crate::DraggingState::NotDraggingLine;
        }

        // Compute all the lines and draw them
        let mut line_vec = Vec::new();

        let mut to_clear = Vec::new();

        for (nref, n) in &self.nodes {
            for (ind, inp) in n.input_ports.iter().enumerate() {
                if let Some(outp) = inp.input_source {
                    if self.get_node(&outp.source).output_ports.len() <= outp.output_ind {
                        to_clear.push(VNodeInputRef {
                            dest: *nref,
                            input_ind: ind,
                        });
                    }

                    let dest_pos = inp.pos;
                    let source_pos = self.get_node(&outp.source).output_ports[outp.output_ind].pos;
                    line_vec.push(draw_line(source_pos, dest_pos, 100));
                }
            }
        }

        for inp in to_clear {
            self.get_node_mut(&inp.dest).input_ports[inp.input_ind].input_source = None;
        }

        let mouse_pos = mouse_pos.unwrap_or_default();

        match &mode.dragging {
            crate::DraggingState::DraggingLineFromInputPort(vnode_input_ref, vnode_output_ref) => {
                Some((Some(*vnode_input_ref), *vnode_output_ref))
            }
            crate::DraggingState::DraggingLineFromOutputPort(vnode_input_ref, vnode_output_ref) => {
                Some((*vnode_input_ref, Some(*vnode_output_ref)))
            }
            _ => None,
        }
        .map(|(inp, outp)| {
            let ipos = inp
                .map(|i| self.get_node(&i.dest).input_ports[i.input_ind].pos)
                .unwrap_or(mouse_pos);
            let opos = outp
                .map(|o| self.get_node(&o.source).output_ports[o.output_ind].pos)
                .unwrap_or(mouse_pos);

            line_vec.push(draw_line(opos, ipos, 100));
        });

        ui.painter().set(lines_shape_id, line_vec);

        changed
    }
}
