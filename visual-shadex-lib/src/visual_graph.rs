mod vnode_infos;
use std::{any, collections::HashMap};

use egui::{Color32, Pos2, Stroke};
use shadex_backend::{
    nodegraph::{FallibleNodeTypeRc, Node, NodeGraph, ValueRef},
    typechecking::{NodeGraphFormalTypeAnalysis, typetypes::TypeError},
};
pub use vnode_infos::{VisualNode, VisualNodeInfo, add::AddInfo, constant::ConstantInfo};

use crate::{
    DraggingState, InteractionState,
    formal_graph_annotations::{FormalGraph, MappedNodeAnnotation},
    helpers::draw_line,
    visual_graph::vnode_infos::INITIALIZATIONS,
};

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
        formal_graph: Option<&FormalGraph>,
    ) -> bool {
        let mut changed = false;

        // Make a placeholder for the lines below all the nodes
        let lines_shape_id = ui.painter().add(egui::Shape::Noop);

        let mut any_drag_stopped = false;

        // Make sure that if nothing is hovered, it will be treated as a line-to-cursor.
        match &mode.dragging {
            DraggingState::DraggingLineFromInputPort(inp, _) => {
                mode.dragging = DraggingState::DraggingLineFromInputPort(*inp, None)
            }
            DraggingState::DraggingLineFromOutputPort(_, outp) => {
                mode.dragging = DraggingState::DraggingLineFromOutputPort(None, *outp)
            }
            _ => (),
        };

        let mut mouse_pos = None;

        let mut node_to_del = None;

        // Draw and update all the nodes
        for n in &mut self.nodes {
            let mut deleted = false;
            changed =
                n.1.show_box(
                    ui,
                    *n.0,
                    mode,
                    &mut any_drag_stopped,
                    &mut mouse_pos,
                    &mut deleted,
                    formal_graph,
                )
                .inner
                    | changed;
            if deleted {
                node_to_del = Some(*n.0);
            }
        }

        if any_drag_stopped {
            if let crate::DraggingState::DraggingLineFromInputPort(inp, Some(outp)) = &mode.dragging
            {
                let node = self.get_node_mut(&inp.dest);
                if node.input_ports.len() > inp.input_ind {
                    node.input_ports[inp.input_ind].input_source = Some(*outp);
                }
                changed = true;
            }
            if let crate::DraggingState::DraggingLineFromOutputPort(Some(inp), outp) =
                &mode.dragging
            {
                let node = self.get_node_mut(&inp.dest);
                if node.input_ports.len() > inp.input_ind {
                    node.input_ports[inp.input_ind].input_source = Some(*outp);
                }
                changed = true;
            }

            mode.dragging = crate::DraggingState::NotDraggingLine;
        }

        // Delete nodes marked for deletion
        if let Some(id) = node_to_del {
            // Remove the node
            self.nodes.remove(&id);

            // Remove any edges from the node.
            for n in self.nodes.iter_mut() {
                for i in n.1.input_ports.iter_mut() {
                    if let Some(src) = i.input_source {
                        if src.source == id {
                            i.input_source = None;
                        }
                    }
                }
            }
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

        mode.prev_mouse_pos = mouse_pos
            .or(ui.response().interact_pointer_pos())
            .unwrap_or(mode.prev_mouse_pos);

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

        ui.response().context_menu(|ui| {
            let pos = mode.prev_mouse_pos;
            for (n, v) in &INITIALIZATIONS {
                if ui.button(*n).clicked() {
                    self.add_node(VisualNode {
                        data: v(),
                        position: pos.to_vec2(),
                        formal_type: None,
                        input_ports: Vec::new(),
                        output_ports: Vec::new(),
                    });
                }
            }
        });

        changed
    }

    pub fn to_formal(&self) -> Result<FormalGraph, ()> {
        let mut nodegraph = NodeGraph::<MappedNodeAnnotation>::new();
        let mut vnode_to_fnode = HashMap::new();

        for n in &self.nodes {
            let new_id = nodegraph.add_node(Node {
                annotation: MappedNodeAnnotation {
                    type_info: n.1.formal_type.clone().unwrap_or_else(|| {
                        Err(TypeError {
                            message: "No computed node type".to_string(),
                        })
                    }),
                    source_node: n.0.clone(),
                },
                inputs: [None].repeat(n.1.input_ports.len()),
                extra_data: None,
            });

            vnode_to_fnode.insert(*n.0, new_id);
        }

        for n in &self.nodes {
            let f_id = vnode_to_fnode.get(n.0).unwrap();

            let fref = nodegraph.get_node_mut(*f_id).unwrap();

            for inp_p in n.1.input_ports.iter().enumerate() {
                if let Some(src) = inp_p.1.input_source {
                    let src_node_id = vnode_to_fnode.get(&src.source).unwrap();
                    let src_node_output_ind = src.output_ind;
                    let node_output_ref = ValueRef {
                        node: *src_node_id,
                        output_index: src_node_output_ind,
                    };

                    fref.inputs[inp_p.0] = Some(node_output_ref);
                }
            }
        }

        let typecheck = NodeGraphFormalTypeAnalysis::analyze(&nodegraph);

        Ok(FormalGraph {
            formal_graph: nodegraph,
            typecheck,
            vnode_to_fnode,
        })
    }
}
