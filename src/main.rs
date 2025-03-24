use iced::advanced::Shell;
use iced::event::Status;
use iced::mouse;
use iced::mouse::Cursor;
use iced::widget::shader::wgpu;
use iced::widget::shader::Event;
use iced::widget::{column, row, shader, slider, text};
use iced::{Alignment, Element, Length, Rectangle, Sandbox, Settings, Size};
use image::io::Reader as ImageReader;
use std::env;
use std::path::Path;
use std::path::PathBuf;

const ZOOM_MIN: f32 = 1.0;
const ZOOM_DEFAULT: f32 = 1.0;
const ZOOM_MAX: f32 = 17.0;

const ZOOM_PIXELS_FACTOR: f32 = 200.0;
const ZOOM_WHEEL_SCALE: f32 = 0.2;

const ITERS_MIN: u32 = 20;
const ITERS_DEFAULT: u32 = 20;
const ITERS_MAX: u32 = 200;

const CENTER_DEFAULT: [f32; 2] = [0.0, 0.0];

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Uniforms {
    resolution: [f32; 2],
    center: [f32; 2],
    scale: [f32; 2],
}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            resolution: [0.0, 0.0],
            center: [0.0, 0.0],
            scale: [0.0, 0.0],
        }
    }
}

struct FragmentShaderPipeline {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    texture: wgpu::Texture,
    texture_bind_group: wgpu::BindGroup,
}

impl FragmentShaderPipeline {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Vertex shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "shader/vert.wgsl"
            ))),
        });

        let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fragment shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "shader/frag.wgsl"
            ))),
        });
        

        // Load and create texture
        let img = ImageReader::open("assets/test.jpeg")
            .unwrap()
            .decode()
            .unwrap();
        let img = img.to_rgba8();
        let dimensions = img.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(
            &wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("Image texture"),
                view_formats: &[],
            },
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
            ],
            label: Some("texture_bind_group"),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("shader_quad uniform buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniform bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("FragmentShaderPipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
                push_constant_ranges: &[],
            })),
            vertex: wgpu::VertexState {
                module: &vertex_shader,
                entry_point: "main",
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader,
                entry_point: "main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shader_quad uniform bind group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Upload image data
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &img,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        Self {
            pipeline,
            uniform_buffer,
            uniform_bind_group,
            texture,
            texture_bind_group,
        }
    }

    fn update(&mut self, queue: &wgpu::Queue, uniforms: &Uniforms) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(uniforms));
    }

    fn render(
        &self,
        target: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        viewport: Rectangle<u32>,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("fill color test"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_viewport(
            viewport.x as f32,
            viewport.y as f32,
            viewport.width as f32,
            viewport.height as f32,
            0.0,
            1.0,
        );
        pass.set_bind_group(0, &self.texture_bind_group, &[]);
        pass.set_bind_group(1, &self.uniform_bind_group, &[]);

        pass.draw(0..6, 0..1);
    }
}

#[derive(Debug, Clone, Copy)]
struct Controls {
    max_iter: u32,
    zoom: f32,
    center: [f32; 2],
}

impl Controls {
    fn scale(&self) -> f32 {
        1.0 / 2.0_f32.powf(self.zoom) / ZOOM_PIXELS_FACTOR
    }
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            max_iter: ITERS_DEFAULT,
            zoom: ZOOM_DEFAULT,
            center: CENTER_DEFAULT,
        }
    }
}

#[derive(Debug)]
struct FragmentShaderPrimitive {
    controls: Controls,
}

impl FragmentShaderPrimitive {
    fn new(controls: Controls) -> Self {
        Self { controls }
    }
}

impl shader::Primitive for FragmentShaderPrimitive {
    fn prepare(
        &self,
        format: wgpu::TextureFormat,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _bounds: Rectangle,
        target_size: Size<u32>,
        _scale_factor: f32,
        storage: &mut shader::Storage,
    ) {
        if !storage.has::<FragmentShaderPipeline>() {
            storage.store(FragmentShaderPipeline::new(device, queue, format));
        }

        let pipeline = storage.get_mut::<FragmentShaderPipeline>().unwrap();

        pipeline.update(
            queue,
            &Uniforms {
                resolution: [target_size.width as f32, target_size.height as f32],
                center: self.controls.center,
                scale: [self.controls.scale(), 0.0],
            },
        );
    }

    fn render(
        &self,
        storage: &shader::Storage,
        target: &wgpu::TextureView,
        _target_size: Size<u32>,
        viewport: Rectangle<u32>,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let pipeline = storage.get::<FragmentShaderPipeline>().unwrap();
        pipeline.render(target, encoder, viewport);
    }
}

#[derive(Debug, Clone)]
enum Message {
    PanningDelta([f32; 2]),
    ZoomDelta([f32; 2], Rectangle, f32),
}

enum MouseInteraction {
    Idle,
    Panning([f32; 2]),
}

impl Default for MouseInteraction {
    fn default() -> Self {
        MouseInteraction::Idle
    }
}

struct FragmentShaderProgram {
    controls: Controls,
}

impl FragmentShaderProgram {
    fn new() -> Self {
        Self {
            controls: Controls::default(),
        }
    }
}

impl shader::Program<Message> for FragmentShaderProgram {
    type State = MouseInteraction;
    type Primitive = FragmentShaderPrimitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        _bounds: Rectangle,
    ) -> Self::Primitive {
        FragmentShaderPrimitive::new(self.controls)
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
        _shell: &mut Shell<'_, Message>,
    ) -> (Status, Option<Message>) {
        if let Event::Mouse(mouse::Event::WheelScrolled { delta }) = event {
            if let Some(pos) = cursor.position_in(bounds) {
                let pos = [pos.x, pos.y];
                let delta = match delta {
                    mouse::ScrollDelta::Lines { x: _, y } => y,
                    mouse::ScrollDelta::Pixels { x: _, y } => y,
                };
                return (
                    Status::Captured,
                    Some(Message::ZoomDelta(pos, bounds, delta)),
                );
            }
        }

        match state {
            MouseInteraction::Idle => match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    if let Some(pos) = cursor.position_over(bounds) {
                        *state = MouseInteraction::Panning([pos.x, pos.y]);
                        return (Status::Captured, None);
                    }
                }
                _ => {}
            },
            MouseInteraction::Panning(prev_pos) => match event {
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    *state = MouseInteraction::Idle;
                }
                Event::Mouse(mouse::Event::CursorMoved { position }) => {
                    let pos = [position.x, position.y];
                    let delta = [pos[0] - prev_pos[0], pos[1] - prev_pos[1]];
                    *state = MouseInteraction::Panning(pos);
                    return (Status::Captured, Some(Message::PanningDelta(delta)));
                }
                _ => {}
            },
        };

        (Status::Ignored, None)
    }
}

struct FragmentShaderApp {
    program: FragmentShaderProgram,
}

fn control<'a>(
    label: &'static str,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    row![text(label), control.into()].spacing(10).into()
}

impl Sandbox for FragmentShaderApp {
    type Message = Message;

    fn new() -> Self {
        Self {
            program: FragmentShaderProgram::new(),
        }
    }

    fn title(&self) -> String {
        String::from("Fragment Shader Widget - Iced")
    }

    fn view(&self) -> Element<'_, Message> {
        let shader = shader(&self.program)
            .width(Length::Fill)
            .height(Length::Fill);

        shader.into()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::PanningDelta(delta) => {
                let scale = self.program.controls.scale();
                self.program.controls.center[0] -= 2.0 * delta[0] * scale;
                self.program.controls.center[1] -= 2.0 * delta[1] * scale;
            }
            Message::ZoomDelta(pos, bounds, delta) => {
                let delta = delta * ZOOM_WHEEL_SCALE;
                let prev_scale = self.program.controls.scale();
                let prev_zoom = self.program.controls.zoom;
                self.program.controls.zoom = (prev_zoom + delta).max(ZOOM_MIN).min(ZOOM_MAX);

                let vec = [
                    pos[0] - bounds.width as f32 * 0.5,
                    pos[1] - bounds.height as f32 * 0.5,
                ];
                let new_scale = self.program.controls.scale();
                self.program.controls.center[0] += vec[0] * (prev_scale - new_scale) * 2.0;
                self.program.controls.center[1] += vec[1] * (prev_scale - new_scale) * 2.0;
            }
        }
    }
}

fn main() -> iced::Result {
    FragmentShaderApp::run(Settings::default())
}
