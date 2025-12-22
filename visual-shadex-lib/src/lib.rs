use std::collections::HashMap;

use egui::{Rect, Shape, Ui, Vec2, epaint::CircleShape, layers, vec2};
use shadex_backend::nodegraph::{NodeGraph, NodeRef};

use crate::visual_graph::{ConstantInfo, VNodeInputRef, VNodeOutputRef, VisualNode, VisualNodeGraph};

mod helpers;
mod node_templates;
pub mod visual_graph;


pub struct ViewState {
    pub rect: Rect,
}

pub enum InteractionState {
    NotDraggingLine,
    DraggingLineFromInputPort(VNodeInputRef, Option<VNodeOutputRef>),
    DraggingLineFromOutputPort(Option<VNodeInputRef>, VNodeOutputRef)
}

pub fn make_constant_node() -> VisualNode {
    VisualNode {
        data: Box::new(ConstantInfo::new(2f32)),
        position: Vec2::ZERO,
        input_sources: Vec::new(),
    }
}

pub fn make_testing_vnode_graph() -> VisualNodeGraph {
    let mut graph = VisualNodeGraph::default();
    graph.add_node(make_constant_node());
    graph
}

pub fn visual_shadex_test(ui: &mut egui::Ui, viewstate: &mut ViewState, graph: &mut VisualNodeGraph) {
    _ = ui.button("Hello from visual shadex lib!");
    let mut vrect = viewstate.rect;
    egui::containers::Scene::new().show(ui, &mut vrect, |ui| {
        _ = ui.button("WOOO!");
        graph.show(ui);
    });

    viewstate.rect = vrect;
}

