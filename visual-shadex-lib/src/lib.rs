use std::{collections::HashMap, hash::Hash};

use egui::{
    Pos2, Rect, Sense, Shape, Ui, Vec2, emath::TSTransform, epaint::CircleShape, layers, vec2,
};
use shadex_backend::{
    nodegraph::{NodeGraph, NodeRef},
    typechecking::NodeGraphFormalTypeAnalysis,
};

use crate::{
    formal_graph_annotations::FormalGraph,
    graph_state::NodeGraphState,
    visual_graph::{
        AddInfo, ConstantInfo, VNodeInputRef, VNodeOutputRef, VisualNode, VisualNodeGraph,
    },
};

pub mod graph_state;

pub mod formal_graph_annotations;
mod helpers;
mod node_templates;
pub mod visual_graph;

pub struct ViewState {
    pub rect: Rect,
}

pub struct InteractionState {
    pub dragging: DraggingState,
    pub prev_mouse_pos: Pos2,
}

pub enum DraggingState {
    NotDraggingLine,
    DraggingLineFromInputPort(VNodeInputRef, Option<VNodeOutputRef>),
    DraggingLineFromOutputPort(Option<VNodeInputRef>, VNodeOutputRef),
}

impl Default for DraggingState {
    fn default() -> Self {
        Self::NotDraggingLine
    }
}

impl DraggingState {
    pub fn hover_inputs(&self) -> bool {
        match self {
            Self::DraggingLineFromInputPort(_, _) => false,
            _ => true,
        }
    }
    pub fn hover_outputs(&self) -> bool {
        match self {
            Self::DraggingLineFromOutputPort(_, _) => false,
            _ => true,
        }
    }
}

pub fn make_constant_node() -> VisualNode {
    VisualNode {
        data: Box::new(ConstantInfo::new(2f32)),
        formal_type: None,
        position: Vec2::ZERO,
        input_ports: Vec::new(),
        output_ports: Vec::new(),
    }
}

pub fn make_add_node() -> VisualNode {
    VisualNode {
        data: Box::new(AddInfo::new()),
        formal_type: None,
        position: Vec2::ZERO,
        input_ports: Vec::new(),
        output_ports: Vec::new(),
    }
}

pub fn make_testing_vnode_graph() -> VisualNodeGraph {
    let graph = VisualNodeGraph::default();
    //graph.add_node(make_constant_node());
    //graph.add_node(make_add_node());
    graph
}

pub fn make_testing_graph_state() -> NodeGraphState {
    NodeGraphState::new()
}

pub fn visual_shadex_test(
    ui: &mut egui::Ui,
    viewstate: &mut ViewState,
    graphstate: &mut NodeGraphState,
    mode: &mut InteractionState,
) {
    _ = ui.button("Hello from visual shadex lib!");
    let mut vrect = viewstate.rect;
    egui::containers::Scene::new().show(ui, &mut vrect, |ui| {
        _ = ui.button("WOOO!");
        graphstate.show(ui, mode);
    });

    viewstate.rect = vrect;
}
