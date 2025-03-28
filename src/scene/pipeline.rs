use iced::{Color, Rectangle};
use std::path::PathBuf;
use iced::wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}

impl Vertex {
    fn desc() -> iced::wgpu::VertexBufferLayout<'static> {
        iced::wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as iced::wgpu::BufferAddress,
            step_mode: iced::wgpu::VertexStepMode::Vertex,
            attributes: &[
                iced::wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: iced::wgpu::VertexFormat::Float32x2,
                },
                iced::wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as iced::wgpu::BufferAddress,
                    shader_location: 1,
                    format: iced::wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub struct Pipeline {
    pipeline: iced::wgpu::RenderPipeline,
    vertices: iced::wgpu::Buffer,
    uniform_buffer: iced::wgpu::Buffer,
    bind_group: iced::wgpu::BindGroup,
    image_path: Option<PathBuf>,
    texture: Option<iced::wgpu::Texture>,
    texture_view: Option<iced::wgpu::TextureView>,
    sampler: Option<iced::wgpu::Sampler>,
}

impl Pipeline {
    pub fn new(
        device: &iced::wgpu::Device,
        queue: &iced::wgpu::Queue,
        format: iced::wgpu::TextureFormat,
        color: Color,
    ) -> Self {
        // Create a full-screen quad
        let vertices = [
            Vertex { position: [-1.0, -1.0], uv: [0.0, 1.0] },
            Vertex { position: [1.0, -1.0], uv: [1.0, 1.0] },
            Vertex { position: [1.0, 1.0], uv: [1.0, 0.0] },
            Vertex { position: [-1.0, -1.0], uv: [0.0, 1.0] },
            Vertex { position: [1.0, 1.0], uv: [1.0, 0.0] },
            Vertex { position: [-1.0, 1.0], uv: [0.0, 0.0] },
        ];

        let vertex_buffer = device.create_buffer_init(&iced::wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: iced::wgpu::BufferUsages::VERTEX,
        });

        let uniform_buffer = device.create_buffer(&iced::wgpu::BufferDescriptor {
            label: Some("Color uniform buffer"),
            size: std::mem::size_of::<[f32; 4]>() as u64,
            usage: iced::wgpu::BufferUsages::UNIFORM | iced::wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        queue.write_buffer(
            &uniform_buffer,
            0,
            bytemuck::cast_slice(&[color.r, color.g, color.b, color.a]),
        );

        let bind_group_layout = device.create_bind_group_layout(&iced::wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind group layout"),
            entries: &[
                iced::wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: iced::wgpu::ShaderStages::FRAGMENT,
                    ty: iced::wgpu::BindingType::Buffer {
                        ty: iced::wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                iced::wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: iced::wgpu::ShaderStages::FRAGMENT,
                    ty: iced::wgpu::BindingType::Sampler(iced::wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                iced::wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: iced::wgpu::ShaderStages::FRAGMENT,
                    ty: iced::wgpu::BindingType::Texture {
                        sample_type: iced::wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: iced::wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&iced::wgpu::BindGroupDescriptor {
            label: Some("Bind group"),
            layout: &bind_group_layout,
            entries: &[
                iced::wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                iced::wgpu::BindGroupEntry {
                    binding: 2,
                    resource: iced::wgpu::BindingResource::Sampler(&device.create_sampler(&iced::wgpu::SamplerDescriptor::default())),
                },
                iced::wgpu::BindGroupEntry {
                    binding: 3,
                    resource: iced::wgpu::BindingResource::TextureView(&device.create_texture(&iced::wgpu::TextureDescriptor {
                        label: Some("Placeholder texture"),
                        size: iced::wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: iced::wgpu::TextureDimension::D2,
                        format: iced::wgpu::TextureFormat::Rgba8Unorm,
                        usage: iced::wgpu::TextureUsages::TEXTURE_BINDING,
                        view_formats: &[],
                    }).create_view(&iced::wgpu::TextureViewDescriptor::default())),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&iced::wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(iced::wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: iced::wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "../shaders/image.wgsl"
            ))),
        });

        let pipeline = device.create_render_pipeline(&iced::wgpu::RenderPipelineDescriptor {
            label: Some("Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: iced::wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: iced::wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(iced::wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(iced::wgpu::ColorTargetState {
                    format,
                    blend: Some(iced::wgpu::BlendState::REPLACE),
                    write_mask: iced::wgpu::ColorWrites::ALL,
                })],
                compilation_options: iced::wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: iced::wgpu::PrimitiveState {
                topology: iced::wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: iced::wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: iced::wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: iced::wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            vertices: vertex_buffer,
            uniform_buffer,
            bind_group,
            image_path: None,
            texture: None,
            texture_view: None,
            sampler: None,
        }
    }



    pub fn set_image_path(&mut self, path: PathBuf) {
        self.image_path = Some(path);
    }

    pub fn load_texture(&mut self, device: &iced::wgpu::Device, queue: &iced::wgpu::Queue) {
        if let Some(path) = &self.image_path {
            let img = image::open(path).unwrap();
            let rgba = img.to_rgba8();
            let dimensions = rgba.dimensions();

            let texture_size = iced::wgpu::Extent3d {
                width: dimensions.0,
                height: dimensions.1,
                depth_or_array_layers: 1,
            };

            let texture = device.create_texture(
                &iced::wgpu::TextureDescriptor {
                    label: Some("Image texture"),
                    size: texture_size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: iced::wgpu::TextureDimension::D2,
                    format: iced::wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: iced::wgpu::TextureUsages::TEXTURE_BINDING | iced::wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                },
            );

            queue.write_texture(
                iced::wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: iced::wgpu::Origin3d::ZERO,
                    aspect: iced::wgpu::TextureAspect::All,
                },
                &rgba,
                iced::wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * dimensions.0),
                    rows_per_image: Some(dimensions.1),
                },
                texture_size,
            );

            let texture_view = texture.create_view(&iced::wgpu::TextureViewDescriptor::default());
            let sampler = device.create_sampler(&iced::wgpu::SamplerDescriptor {
                label: Some("Image sampler"),
                address_mode_u: iced::wgpu::AddressMode::ClampToEdge,
                address_mode_v: iced::wgpu::AddressMode::ClampToEdge,
                address_mode_w: iced::wgpu::AddressMode::ClampToEdge,
                mag_filter: iced::wgpu::FilterMode::Linear,
                min_filter: iced::wgpu::FilterMode::Linear,
                mipmap_filter: iced::wgpu::FilterMode::Linear,
                ..Default::default()
            });

            self.texture = Some(texture);
            self.texture_view = Some(texture_view);
            self.sampler = Some(sampler);

            // Update bind group with texture and sampler
            let bind_group_layout = device.create_bind_group_layout(&iced::wgpu::BindGroupLayoutDescriptor {
                label: Some("Bind group layout"),
                entries: &[
                    iced::wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: iced::wgpu::ShaderStages::FRAGMENT,
                        ty: iced::wgpu::BindingType::Buffer {
                            ty: iced::wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    iced::wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: iced::wgpu::ShaderStages::FRAGMENT,
                        ty: iced::wgpu::BindingType::Sampler(iced::wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    iced::wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: iced::wgpu::ShaderStages::FRAGMENT,
                        ty: iced::wgpu::BindingType::Texture {
                            sample_type: iced::wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: iced::wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

            self.bind_group = device.create_bind_group(&iced::wgpu::BindGroupDescriptor {
                label: Some("Bind group"),
                layout: &bind_group_layout,
                entries: &[
                    iced::wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.uniform_buffer.as_entire_binding(),
                    },
                    iced::wgpu::BindGroupEntry {
                        binding: 2,
                        resource: iced::wgpu::BindingResource::Sampler(self.sampler.as_ref().unwrap()),
                    },
                    iced::wgpu::BindGroupEntry {
                        binding: 3,
                        resource: iced::wgpu::BindingResource::TextureView(self.texture_view.as_ref().unwrap()),
                    },
                ],
            });
        }
    }

    pub fn render(
        &self,
        target: &iced::wgpu::TextureView,
        encoder: &mut iced::wgpu::CommandEncoder,
        clip_bounds: Rectangle<u32>,
    ) {
        let mut render_pass = encoder.begin_render_pass(&iced::wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(iced::wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: iced::wgpu::Operations {
                    load: iced::wgpu::LoadOp::Clear(iced::wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: iced::wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertices.slice(..));
        render_pass.set_scissor_rect(
            clip_bounds.x,
            clip_bounds.y,
            clip_bounds.width,
            clip_bounds.height,
        );
        render_pass.draw(0..6, 0..1);
    }
}
