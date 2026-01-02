use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, BlendState, BufferBinding, BufferUsages, Color, ColorTargetState, Device,
    Extent3d, FragmentState, MultisampleState, Operations, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderStages,
    TextureFormat, TextureUsages, TextureView, VertexState, include_wgsl,
    util::TextureBlitter,
    wgt::{BufferDescriptor, CommandEncoderDescriptor, TextureDescriptor, TextureViewDescriptor},
};

use crate::execution::NodeExecutionOutput;

pub struct WGPURunner {
    pub dev: Device,
    pub queue: Queue,
}

impl WGPURunner {
    pub fn run_shader(&mut self, inf: &NodeExecutionOutput, dest_view: &TextureView) {
        // Texture size
        let out_tex_size = [1024u32, 1024u32];

        // Blitter
        let blitter = TextureBlitter::new(&self.dev, TextureFormat::Rgba8Unorm);

        // Create texture
        let out_tex = self.dev.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: out_tex_size[0],
                height: out_tex_size[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::COPY_SRC
                | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let out_view = out_tex.create_view(&TextureViewDescriptor::default());

        // Create shaders
        let vertex_module = self.dev.create_shader_module(include_wgsl!("./vert.wgsl"));
        let fragment_module = self.dev.create_shader_module(ShaderModuleDescriptor {
            label: Some("Fragment!"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(format!(
                include_str!("frag.wgsl_template"),
                inf.text, inf.name
            ))),
        });

        let bind_group_layout = self
            .dev
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let buffer = self.dev.create_buffer(&BufferDescriptor {
            label: None,
            size: 32,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = self.dev.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        let output_format = wgpu::TextureFormat::Rgba8Unorm;

        let pipeline_layout = self.dev.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render pipeline layout!"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = self.dev.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: Some(&pipeline_layout),
            cache: None,
            vertex: VertexState {
                module: &vertex_module,
                entry_point: None,
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[],
            },
            primitive: PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &fragment_module,
                entry_point: None,
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format: output_format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        // Populate uniform
        let var_name = [out_tex_size[0], out_tex_size[1]];
        let bytes = bytemuck::bytes_of(&var_name);
        self.queue
            .write_buffer(&buffer, 0, bytemuck::bytes_of(&var_name));

        // Do the render!
        let mut encoder = self
            .dev
            .create_command_encoder(&CommandEncoderDescriptor { label: None });

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &out_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: wgpu::LoadOp::Clear(Color::BLUE),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&pipeline);

        render_pass.set_bind_group(0, &bind_group, &[]);

        render_pass.draw(0..3, 0..1);
        drop(render_pass);

        //encoder.copy_texture_to_buffer(source, destination, copy_size);

        //encoder.map_buffer_on_submit(buffer, mode, bounds, callback);

        blitter.copy(&self.dev, &mut encoder, &out_view, &dest_view);

        self.queue.submit([encoder.finish()]);
    }
}
