
pub struct App {

}

impl App {
    pub fn new<'a>(_ctx: &eframe::CreationContext<'a>) -> Self {
        Self {}
    }
}

impl eframe::App for App {
    
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            visual_shadex_lib::visual_shadex_test(ui);
        });
    }
}