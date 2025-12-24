use egui::{Pos2, Rect, Vec2};
use visual_shadex_lib::{
    InteractionState, ViewState, graph_state::NodeGraphState, make_constant_node,
    make_testing_graph_state, visual_graph::VisualNodeGraph,
};

pub struct App {
    vs: ViewState,
    graph: NodeGraphState,
    mode: InteractionState,
}

impl App {
    pub fn new<'a>(_ctx: &eframe::CreationContext<'a>) -> Self {
        Self {
            vs: ViewState {
                rect: Rect::from_min_size(Pos2::ZERO, Vec2::splat(300f32)),
            },
            graph: make_testing_graph_state(),
            mode: InteractionState {
                dragging: Default::default(),
                prev_mouse_pos: Default::default(),
            },
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            visual_shadex_lib::visual_shadex_test(
                ui,
                &mut self.vs,
                &mut self.graph,
                &mut self.mode,
            );
        });
    }
}
