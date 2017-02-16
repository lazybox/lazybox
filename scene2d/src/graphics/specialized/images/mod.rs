use {gfx, gfx_core};

use {Graphics, Frame, Color, PackedColor};
use graphics::types::*;
use graphics::utils::*;
use graphics::camera;

const IMAGE_BUFFER_SIZE: usize = 128;

#[doc(hidden)]
pub const RENDER_GLSLV_150: &'static [u8] = include_bytes!("render_150.glslv");
#[doc(hidden)]
pub const RENDER_GLSLF_150: &'static [u8] = include_bytes!("render_150.glslf");

#[doc(hidden)]
pub use self::defines::{ImageInstance, render_pipe};
mod defines {
    use gfx;
    pub use graphics::types::*;
    pub use graphics::utils::*;
    pub use graphics::camera;

    gfx_defines! {
        vertex ImageInstance {
            translate_inf: [f32; 2] = "a_TranslateInf",
            translate_sup: [f32; 2] = "a_TranslateSup",
            tex_coord_inf: [f32; 2] = "a_TexCoordInf",
            tex_coord_sup: [f32; 2] = "a_TexCoordSup",
            color: u32 = "a_Color",
        }

        pipeline render_pipe {
            vertices: gfx::VertexBuffer<Position> = (),
            instances: gfx::InstanceBuffer<ImageInstance> = (),
            camera: gfx::ConstantBuffer<camera::Locals> = "Camera",
            image_sampler: gfx::TextureSampler<[f32; 4]> = "s_Image",
            scissor: gfx::pso::target::Scissor = (),
            color_target: gfx::BlendTarget<ColorFormat> =
                ("Target0", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
        }
    }
}

pub struct Renderer {
    pso: PipelineState<render_pipe::Meta>,
    vertices: GpuBuffer<Position>,
    instances: GpuBuffer<ImageInstance>,
    upload: GpuBuffer<ImageInstance>,
    camera_locals: GpuBuffer<camera::Locals>,
    sampler: Sampler,
    scissor: gfx_core::target::Rect,
    color_target: RenderTargetView<ColorFormat>,
    slice: Slice,
}

impl Renderer {
    pub fn new(color_target: RenderTargetView<ColorFormat>,
               camera_locals: GpuBuffer<camera::Locals>,
               scissor: gfx_core::target::Rect,
               graphics: &mut Graphics) -> Self
    {
        use gfx::traits::*;

        let pso = graphics.factory
            .create_pipeline_simple(RENDER_GLSLV_150, RENDER_GLSLF_150, render_pipe::new())
            .expect("could not create image render pipeline");

        let (vertices, slice) = graphics.factory
            .create_vertex_buffer_with_slice(&QUAD_VERTICES, &QUAD_INDICES[..]);

        let (instances, upload) =
            create_vertex_upload_pair(&mut graphics.factory, IMAGE_BUFFER_SIZE);

        let linear_sampler = graphics.factory.create_sampler_linear();

        Renderer {
            pso: pso,
            vertices: vertices,
            instances: instances,
            upload: upload,
            camera_locals: camera_locals,
            sampler: linear_sampler,
            scissor: scissor,
            color_target: color_target,
            slice: slice,
        }
    }

    pub fn resize(&mut self, color_target: RenderTargetView<ColorFormat>) {
        self.color_target = color_target;
    }

    pub fn scissor_mut(&mut self) -> &mut gfx_core::target::Rect {
        &mut self.scissor
    }

    pub fn render(&mut self, _: &mut Frame) -> Render {
        Render {
            pso: &self.pso,
            vertices: &self.vertices,
            instances: &self.instances,
            writer: MappingWriter::new(&self.upload),
            camera_locals: &self.camera_locals,
            sampler: &self.sampler,
            scissor: &mut self.scissor,
            color_target: &self.color_target,
            slice: &mut self.slice,

            start: 0,
            end: 0,
            current_texture: None,
        }
    }
}

pub struct Render<'a> {
    pso: &'a PipelineState<render_pipe::Meta>,
    vertices: &'a GpuBuffer<Position>,
    instances: &'a GpuBuffer<ImageInstance>,
    writer: MappingWriter<'a, ImageInstance>,
    camera_locals: &'a GpuBuffer<camera::Locals>,
    sampler: &'a Sampler,
    scissor: &'a mut gfx_core::target::Rect,
    color_target: &'a RenderTargetView<ColorFormat>,
    slice: &'a mut Slice,

    start: usize,
    end: usize,
    current_texture: Option<TextureView<ColorFormat>>,
}

impl<'a> Render<'a> {
    pub fn add<F>(&mut self,
                  position_inf: [f32; 2],
                  position_sup: [f32; 2],
                  tex_coord_inf: [f32; 2],
                  tex_coord_sup: [f32; 2],
                  texture_view: TextureView<ColorFormat>,
                  color: Color,
                  before_flush: &mut F,
                  frame: &mut Frame)
        where F: FnMut(&mut Frame)
    {
        if self.end == IMAGE_BUFFER_SIZE {
            before_flush(frame);
            self.before_flush(frame);
            frame.flush();
        } else if let Some(current) = self.current_texture.take() {
            if current != texture_view {
                self.draw(current, &mut frame.graphics.encoder);
                frame.should_flush();
            }
        }

        let instance = ImageInstance {
            translate_inf: position_inf,
            translate_sup: position_sup,
            tex_coord_inf: tex_coord_inf,
            tex_coord_sup: tex_coord_sup,
            color: PackedColor::from(color).0,
        };

        self.writer.acquire(&mut frame.graphics.factory)[self.end] = instance;
        self.end += 1;
        self.current_texture = Some(texture_view);
    }

    pub fn before_flush(&mut self, frame: &mut Frame) {
        self.ensure_drawed(frame);
        self.start = 0;
        self.end = 0;
        self.writer.release();
    }

    pub fn ensure_drawed(&mut self, frame: &mut Frame) {
        if let Some(texture) = self.current_texture.take() {
            self.draw(texture, &mut frame.graphics.encoder);
            frame.should_flush();
        }
    }

    fn draw(&mut self,
            texture_view: TextureView<ColorFormat>,
            encoder: &mut Encoder)
    {
        let data = render_pipe::Data {
            vertices: self.vertices.clone(),
            instances: self.instances.clone(),
            camera: self.camera_locals.clone(),
            image_sampler: (texture_view, self.sampler.clone()),
            scissor: self.scissor.clone(),
            color_target: self.color_target.clone(),
        };

        let count = self.end - self.start;
        self.slice.instances = Some((count as gfx::InstanceCount,
                                     self.start as gfx::VertexCount));
        encoder.copy_buffer(self.writer.buffer(), self.instances,
                            self.start, self.start, count).unwrap();
        encoder.draw(self.slice, self.pso, &data);
        self.start = self.end;
    }

    pub fn scissor_mut(&mut self) -> &mut gfx_core::target::Rect {
        self.scissor
    }
}
