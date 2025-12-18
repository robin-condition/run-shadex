use egui::{Pos2, Rect, Vec2};
use visual_shadex_lib::ViewState;

pub struct App {
    vs: ViewState,
}

impl App {
    pub fn new<'a>(_ctx: &eframe::CreationContext<'a>) -> Self {
        Self {
            vs: ViewState {
                rect: Rect::from_min_size(Pos2::ZERO, Vec2::splat(200f32)),
            },
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            visual_shadex_lib::visual_shadex_test(ui, &mut self.vs);
        });
    }
}
