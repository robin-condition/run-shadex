use std::{collections::HashMap, hash::Hash};

use egui::{
    Color32, Pos2, Rect, Sense, Shape, Ui, Vec2, emath::TSTransform, epaint::CircleShape, layers,
    pos2, vec2,
};
use serde::{Deserialize, Serialize, Serializer};
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

#[derive(Serialize, Deserialize)]
pub struct ViewState {
    pub rect: Rect,
}

impl Default for ViewState {
    fn default() -> Self {
        ViewState {
            rect: Rect::from_min_size(Pos2::ZERO, Vec2::splat(300f32)),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
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

#[derive(Serialize, Deserialize)]
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

pub fn visual_shadex_test(
    ui: &mut egui::Ui,
    state: &mut GraphUIState,
    runner: &mut WGPURunner,
    output_view: &mut TextureViewInfo,
    force_render: bool,
) {
    let GraphUIState {
        view_state: viewstate,
        graph_state: graphstate,
        interaction_state: mode,
    } = state;

    /*
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
    */

    let mut vrect = viewstate.rect;

    ui.painter().image(
        output_view.egui_texture_id,
        //Rect::from_min_size(Pos2::ZERO, vec2(128f32, 128f32)),
        ui.available_rect_before_wrap(),
        Rect::from_min_max(Pos2::ZERO, pos2(1f32, 1f32)),
        Color32::WHITE,
    );

    let change = &mut false;

    egui::containers::Scene::new()
        .zoom_range(0.0..=1.0f32)
        .show(ui, &mut vrect, |ui| {
            *change = graphstate.show(ui, mode);
            if *change || force_render {
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

#[derive(Serialize, Deserialize, Default)]
pub struct GraphUIState {
    pub view_state: ViewState,
    pub graph_state: NodeGraphState,

    #[serde(skip)]
    pub interaction_state: InteractionState,
}
