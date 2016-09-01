pub mod triangulation;

use gfx;
use gfx_core;

use {Graphics, Frame, Color};
use camera;
use color::PackedColor;
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
    mapping: MappingWritable<Vertex>,
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

        let (vertices, mapping) = graphics.factory
            .create_buffer_persistent_writable(VERTEX_BUFFER_SIZE,
                                               gfx::buffer::Role::Vertex,
                                               gfx::Bind::empty());

        Renderer {
            pso: pso,
            data: pipe::Data {
                vertices: vertices,
                camera: camera_locals,
                scissor: scissor,
                color_target: color_target,
            },
            mapping: mapping,
        }
    }

    pub fn resize(&mut self, color_target: RenderTargetView<ColorFormat>) {
        self.data.color_target = color_target;
    }

    pub fn color_target(&self) -> &RenderTargetView<ColorFormat> {
        &self.data.color_target
    }

    pub fn camera(&self) -> &GpuBuffer<camera::Locals> {
        &self.data.camera
    }

    pub fn scissor_mut(&mut self) -> &mut gfx_core::target::Rect {
        &mut self.data.scissor
    }

    pub fn render(&mut self, _: &mut Frame) -> Render {
        Render {
            renderer: self, 
            start: 0,
            end: 0,
        }
    }
}

pub struct Render<'a> {
    renderer: &'a mut Renderer,
    start: usize,
    end: usize,
}

impl<'a> Render<'a> {
    pub fn add<F>(&mut self,
                  color: Color,
                  triangle: &Triangle,
                  before_flush: &mut F,
                  frame: &mut Frame)
        where F: FnMut(&mut Frame)
    {
        if self.end == VERTEX_BUFFER_SIZE {
            before_flush(frame);
            self.before_flush(frame);
            frame.flush();
        }

        let color = PackedColor::from(color).0;
        // TODO: call `write()` less often :/
        let mut writer = self.renderer.mapping.write();
        for &p in triangle {
            let vertex = Vertex { position: p, color: color };
            writer.set(self.end, vertex);
            self.end += 1;
        }
    }

    pub fn add_slice<F>(&mut self,
                        color: Color,
                        triangles: &[Triangle],
                        before_flush: &mut F,
                        frame: &mut Frame)
        where F: FnMut(&mut Frame)
    {
        for t in triangles {
            self.add(color, t, before_flush, frame);
        }
    }

    pub fn before_flush(&mut self, frame: &mut Frame) {
        self.ensure_drawed(frame);
        self.start = 0;
        self.end = 0;
    }

    pub fn ensure_drawed(&mut self, frame: &mut Frame) {
        if self.end > self.start {
            self.draw(&mut frame.graphics.encoder);
            frame.should_flush();
        }
    }

    fn draw(&mut self, encoder: &mut Encoder) {
        let slice = gfx::Slice {
            start: self.start as gfx::VertexCount,
            end: self.end as gfx::VertexCount,
            buffer: gfx::IndexBuffer::Auto,
            base_vertex: 0,
            instances: None,
        };

        encoder.draw(&slice, &self.renderer.pso, &self.renderer.data);
        self.start = self.end;
    }

    pub fn scissor_mut(&mut self) -> &mut gfx_core::target::Rect {
        self.renderer.scissor_mut()
    }
}
