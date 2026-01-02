use eframe::wgpu::{self, Extent3d, TextureDescriptor, wgt::TextureViewDescriptor};
use egui::{Pos2, Rect, Vec2};
use shadex_backend::execution::WGPURunner;
use visual_shadex_lib::{
    InteractionState, TextureViewInfo, ViewState, graph_state::NodeGraphState, make_constant_node,
    make_testing_graph_state, visual_graph::VisualNodeGraph,
};

pub struct App {
    vs: ViewState,
    graph: NodeGraphState,
    mode: InteractionState,
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
            vs: ViewState {
                rect: Rect::from_min_size(Pos2::ZERO, Vec2::splat(300f32)),
            },
            graph: make_testing_graph_state(),
            mode: InteractionState {
                dragging: Default::default(),
                prev_mouse_pos: Default::default(),
            },
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
        egui::CentralPanel::default().show(ctx, |ui| {
            visual_shadex_lib::visual_shadex_test(
                ui,
                &mut self.vs,
                &mut self.graph,
                &mut self.mode,
                &mut self.runner,
                &mut self.output_tex,
            );
        });
    }
}
