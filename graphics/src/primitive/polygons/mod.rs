pub mod triangulation;

use gfx;
use gfx_core;

use {Graphics, Frame, Color};
use camera;
use color::PackedColor;
use utils::GpuBufferMapping;
use types::*;

pub type Triangle = [[f32; 2]; 3];
pub const VERTEX_BUFFER_TRIANGLES: usize = 512;
pub const VERTEX_BUFFER_SIZE: usize = VERTEX_BUFFER_TRIANGLES * 3;

#[doc(hidden)]
pub const RENDER_GLSLV_150: &'static [u8] = include_bytes!("render_150.glslv");
#[doc(hidden)]
pub const RENDER_GLSLF_150: &'static [u8] = include_bytes!("render_150.glslf");

#[doc(hidden)]
pub use self::defines::{Vertex, pipe};
mod defines {
    pub use types::*;
    pub use camera;

    gfx_defines! {
        vertex Vertex {
            position: [f32; 2] = "a_Position",
            color: u32 = "a_Color",
        }

        pipeline pipe {
            vertices: gfx::VertexBuffer<Vertex> = (),
            camera: gfx::ConstantBuffer<camera::Locals> = "Camera",
            scissor: gfx::pso::target::Scissor = (),
            color_target: gfx::RenderTarget<ColorFormat> = "o_Color",
        }
    }
}

pub struct Renderer {
    pso: PipelineState<pipe::Meta>,
    data: pipe::Data<Resources>,
}

impl Renderer {
    pub fn new(color_target: RenderTargetView<ColorFormat>,
               camera_locals: GpuBuffer<camera::Locals>,
               scissor: gfx_core::target::Rect,
               graphics: &mut Graphics) -> Self {
        use gfx::traits::*;

        let pso = graphics.factory
            .create_pipeline_simple(RENDER_GLSLV_150, RENDER_GLSLF_150, pipe::new())
            .expect("could not create polygon render pipeline");

        let vertices = graphics.factory
        	.create_buffer_dynamic(VERTEX_BUFFER_SIZE, gfx::BufferRole::Vertex, gfx::Bind::empty())
            .expect("could not create polygon buffer");

        Renderer {
            pso: pso,
            data: pipe::Data {
                vertices: vertices,
                camera: camera_locals,
                scissor: scissor,
                color_target: color_target,
            },
        }
    }

    pub fn resize(&mut self, color_target: RenderTargetView<ColorFormat>) {
        self.data.color_target = color_target;
    }

    pub fn camera(&self) -> &GpuBuffer<camera::Locals> {
        &self.data.camera
    }

    pub fn scissor_mut(&mut self) -> &mut gfx_core::target::Rect {
        &mut self.data.scissor
    }

    pub fn render(&mut self, frame: &mut Frame) -> Render {
        Render {
            mapping: GpuBufferMapping::new(&self.data.vertices, &frame.graphics.factory),
            offset: 0,
            renderer: self, 
        }
    }
}

pub struct Render<'a> {
    renderer: &'a mut Renderer,
    mapping: GpuBufferMapping<'a, Vertex>,
    offset: usize,
}

impl<'a> Render<'a> {
    pub fn add(&mut self, color: Color, triangle: &Triangle, frame: &mut Frame) {
        let color = PackedColor::from(color).0;

        for &p in triangle {
            if self.offset == VERTEX_BUFFER_SIZE {
                self.flush(frame);
            }

            self.mapping.set(self.offset, Vertex {
                position: p,
                color: color,
            });
            self.offset += 1;
        }
    }

    pub fn add_slice(&mut self, color: Color, triangles: &[Triangle], frame: &mut Frame) {
        for t in triangles {
            self.add(color, t, frame);
        }
    }

    pub fn ensure_flushed(&mut self, frame: &mut Frame) {
        if self.offset > 0 {
            self.flush(frame);
        }
    }

    fn flush(&mut self, frame: &mut Frame) {
        let &mut Graphics { ref mut encoder, ref mut device, .. } = frame.graphics;

        let slice = gfx::Slice {
            start: 0,
            end: self.offset as u32,
            buffer: gfx::IndexBuffer::Auto,
            base_vertex: 0,
            instances: None,
        };

        self.mapping.ensure_unmapped();
        encoder.draw(&slice, &self.renderer.pso, &self.renderer.data);
        encoder.flush(device);

        self.offset = 0;
    }

    pub fn scissor_mut(&mut self) -> &mut gfx_core::target::Rect {
        self.renderer.scissor_mut()
    }
}