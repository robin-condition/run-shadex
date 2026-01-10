use eframe::wgpu::{self, Extent3d, TextureDescriptor, wgt::TextureViewDescriptor};
use egui::{Pos2, Rect, Vec2};
use shadex_backend::execution::WGPURunner;
use visual_shadex_lib::{
    GraphUIState, InteractionState, TextureViewInfo, ViewState, graph_state::NodeGraphState,
    visual_graph::VisualNodeGraph,
};

use crate::example_programs::EXAMPLE_PROGRAMS;

pub struct App {
    graph_ui_state: GraphUIState,
    runner: WGPURunner,
    output_tex: TextureViewInfo,
}

impl App {
    pub fn new<'a>(ctx: &eframe::CreationContext<'a>) -> Self {
        let rstate = ctx.wgpu_render_state.as_ref().unwrap();
        let texture = rstate.device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: 1024u32,
                height: 1024u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&TextureViewDescriptor::default());

        let egui_id = rstate.renderer.write().register_native_texture(
            &rstate.device,
            &view,
            wgpu::FilterMode::Linear,
        );
        Self {
            graph_ui_state: Default::default(),
            runner: WGPURunner {
                dev: rstate.device.clone(),
                queue: rstate.queue.clone(),
            },
            output_tex: TextureViewInfo {
                tex_handle: texture,
                tex_view: view,
                size: [1024u32, 1024u32],
                egui_texture_id: egui_id,
            },
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5f32);
        let load = &mut false;
        egui::SidePanel::left("global_left_panel").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("Examples to load");
                for (name, content) in EXAMPLE_PROGRAMS {
                    if ui.button(name).clicked() {
                        self.graph_ui_state = serde_json::from_str(content).unwrap();
                        *load = true;
                    }
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            //let text = serde_json::to_string(&self.graph_ui_state).unwrap_or("None".to_string());
            //ui.label(text);
            visual_shadex_lib::visual_shadex_test(
                ui,
                &mut self.graph_ui_state,
                &mut self.runner,
                &mut self.output_tex,
                *load,
            );
        });
    }
}
