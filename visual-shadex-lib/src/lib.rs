use std::{collections::HashMap, hash::Hash};

use egui::{
    Color32, Pos2, Rect, Sense, Shape, Ui, Vec2, emath::TSTransform, epaint::CircleShape, layers,
    pos2, vec2,
};
use shadex_backend::{
    execution::WGPURunner,
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

pub struct TextureViewInfo {
    pub tex_handle: wgpu::Texture,
    pub tex_view: wgpu::TextureView,
    pub size: [u32; 2],
    pub egui_texture_id: egui::TextureId,
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
        position: Vec2::ZERO,
        input_ports: Vec::new(),
        output_ports: Vec::new(),
    }
}

pub fn make_add_node() -> VisualNode {
    VisualNode {
        data: Box::new(AddInfo::new()),
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
    runner: &mut WGPURunner,
    output_view: &mut TextureViewInfo,
) {
    let mut executor = shadex_backend::execution::Executor::default();
    let text = if let Ok(graph) = &graphstate.formal_graph {
        let res = executor.run(&graph.formal_graph, &graph.typecheck);
        if let Ok(prog) = res {
            //runner.run_shader(&prog, &output_view.tex_view);
            prog.text
        } else {
            "No compilation".to_string()
        }
    } else {
        "No compilation".to_string()
    };
    _ = ui.code(text);

    let mut vrect = viewstate.rect;

    ui.painter().image(
        output_view.egui_texture_id,
        //Rect::from_min_size(Pos2::ZERO, vec2(128f32, 128f32)),
        ui.available_rect_before_wrap(),
        Rect::from_min_max(Pos2::ZERO, pos2(1f32, 1f32)),
        Color32::WHITE,
    );

    egui::containers::Scene::new()
        .zoom_range(0.0..=5f32)
        .show(ui, &mut vrect, |ui| {
            _ = ui.button("WOOO!");
            let change = graphstate.show(ui, mode);
            if change {
                let mut executor = shadex_backend::execution::Executor::default();
                let text = if let Ok(graph) = &graphstate.formal_graph {
                    let res = executor.run(&graph.formal_graph, &graph.typecheck);
                    if let Ok(prog) = res {
                        log::info!("Executing.");
                        runner.run_shader(&prog, &output_view.tex_view);
                    }
                };
            }
        });

    viewstate.rect = vrect;
}
