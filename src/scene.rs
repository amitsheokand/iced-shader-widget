mod pipeline;

use pipeline::Pipeline;

use crate::wgpu;

use iced::mouse;
use iced::widget::shader::{self, Viewport};
use iced::Rectangle;
use std::path::PathBuf;
use iced::Color;

#[derive(Clone)]
pub struct Scene {
    image_path: Option<PathBuf>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            image_path: None,
        }
    }

    pub fn set_image_path(&mut self, path: PathBuf) {
        self.image_path = Some(path);
    }
}

impl<Message> shader::Program<Message> for Scene {
    type State = ();
    type Primitive = Primitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        Primitive::new(bounds, self.image_path.clone())
    }
}

#[derive(Debug)]
pub struct Primitive {
    image_path: Option<PathBuf>,
}

impl Primitive {
    pub fn new(_bounds: Rectangle, image_path: Option<PathBuf>) -> Self {
        Self {
            image_path,
        }
    }
}

impl shader::Primitive for Primitive {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        storage: &mut shader::Storage,
        _bounds: &Rectangle,
        _viewport: &Viewport,
    ) {
        if !storage.has::<Pipeline>() {
            storage.store(Pipeline::new(
                device,
                queue,
                format,
                Color::WHITE,
            ));
        }

        if let Some(pipeline) = storage.get_mut::<Pipeline>() {
            if let Some(path) = &self.image_path {
                pipeline.set_image_path(path.clone());
                pipeline.load_texture(device, queue);
            }
        }
    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        storage: &shader::Storage,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        if let Some(pipeline) = storage.get::<Pipeline>() {
            pipeline.render(target, encoder, *clip_bounds);
        }
    }
}
