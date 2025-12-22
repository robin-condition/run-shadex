use std::collections::HashMap;

use egui::{Rect, Shape, Ui, Vec2, epaint::CircleShape, layers, vec2};
use shadex_backend::nodegraph::{NodeGraph, NodeRef};

use crate::vnode_infos::{ConstantInfo, VisualNode};

mod helpers;
mod node_templates;
mod vnode_infos;

pub enum VisualNodeConfigurationEntry {}

pub struct VisualNodeConfigState {}

pub trait VNodePrototypeConfig {
    fn draw(ui: &mut Ui);
}

pub struct VisualNodeFrameLayout {
    pub name: String,
    pub size: Vec2,
}

pub struct VisualNodePrototype {
    pub name: String,
    pub size: Vec2,
    // This will change because they need to be configurable.
    // Prototypes will allow configuration prototypes,
    // and visual nodes will have configuration data.
    // Only with that configuration data can shadex types be known.
}

impl VisualNodePrototype {}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct VisualNodePrototypeRef(usize);

pub struct VisualNodeInfo {
    id: NodeRef,
    prototype: VisualNodePrototypeRef,
}

pub struct VisualPrototypeUniverse {
    prototypes: HashMap<usize, VisualNodePrototype>,
    next_id: usize,
}

pub struct VisualNodeGraph {
    universe: VisualPrototypeUniverse,
    visual_info: HashMap<NodeRef, VisualNodeInfo>,
    backend: NodeGraph,
}

pub struct SelectionState {}

pub trait RenderableObject {
    fn process(ui: &mut Ui, layer_zero_shapes: Vec<Shape>, list: layers::PaintList) {}
}

pub struct ViewState {
    pub rect: Rect,
    pub node: VisualNode,
}

pub fn make_constant_node() -> VisualNode {
    VisualNode {
        data: Box::new(ConstantInfo::new(2f32)),
        position: Vec2::ZERO,
    }
}

pub fn visual_shadex_test(ui: &mut egui::Ui, viewstate: &mut ViewState) {
    _ = ui.button("Hello from visual shadex lib!");
    let mut vrect = viewstate.rect;
    egui::containers::Scene::new().show(ui, &mut vrect, |ui| {
        _ = ui.button("WOOO!");
        viewstate.node.show_box(ui);
    });

    viewstate.rect = vrect;
}

pub fn render_basic_node(ui: &mut egui::Ui) {}
