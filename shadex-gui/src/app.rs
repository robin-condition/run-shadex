use egui::{Pos2, Rect, Vec2};
use visual_shadex_lib::{ViewState, make_constant_node, make_testing_vnode_graph, visual_graph::VisualNodeGraph};

pub struct App {
    vs: ViewState,
    graph: VisualNodeGraph
}

impl App {
    pub fn new<'a>(_ctx: &eframe::CreationContext<'a>) -> Self {
        Self {
            vs: ViewState {
                rect: Rect::from_min_size(Pos2::ZERO, Vec2::splat(200f32)),
            },
            graph: make_testing_vnode_graph()
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            visual_shadex_lib::visual_shadex_test(ui, &mut self.vs, &mut self.graph);
        });
    }
}
