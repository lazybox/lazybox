use gfx;
use gfx_core;

use {Graphics, Frame, Color};
use camera;
use color::PackedColor;
use types::*;
pub use utils::*;

const IMAGE_BUFFER_SIZE: usize = 128;

#[doc(hidden)]
pub const RENDER_GLSLV_150: &'static [u8] = include_bytes!("render_150.glslv");
#[doc(hidden)]
pub const RENDER_GLSLF_150: &'static [u8] = include_bytes!("render_150.glslf");

#[doc(hidden)]
pub use self::defines::{ImageInstance, render_pipe};
mod defines {
    pub use types::*;
    pub use utils::*;
    pub use camera;

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
                ("o_Color", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
        }
    }
}

pub struct Renderer {
    pso: PipelineState<render_pipe::Meta>,
    vertices: GpuBuffer<Position>,
    instances: GpuBuffer<ImageInstance>,
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

        let instances = graphics.factory
        	.create_buffer_dynamic(IMAGE_BUFFER_SIZE, gfx::BufferRole::Vertex, gfx::Bind::empty())
            .unwrap();
        
        let linear_sampler = graphics.factory.create_sampler_linear();

        Renderer {
            pso: pso,
            vertices: vertices,
            instances: instances,
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

    pub fn render(&mut self, frame: &mut Frame) -> Render {
        Render {
            mapping: GpuBufferMapping::new(&self.instances, &frame.graphics.factory),
            offset: 0,
            current_texture: None,
            renderer: self,
        }
    }
}

pub struct Render<'a> {
    renderer: &'a mut Renderer,
    mapping: GpuBufferMapping<'a, ImageInstance>,
    offset: usize,
    current_texture: Option<TextureView<ColorFormat>>,
}

impl<'a> Render<'a> {
    pub fn add(&mut self,
               position_inf: [f32; 2],
               position_sup: [f32; 2],
               tex_coord_inf: [f32; 2],
               tex_coord_sup: [f32; 2],
               texture_view: TextureView<ColorFormat>,
               color: Color,
               frame: &mut Frame)
    {
        if let Some(current) = self.current_texture.take() {
            if self.offset == IMAGE_BUFFER_SIZE || current != texture_view {
                self.flush(current, frame);
            }
        }

        let instance = ImageInstance {
            translate_inf: position_inf,
            translate_sup: position_sup,
            tex_coord_inf: tex_coord_inf,
            tex_coord_sup: tex_coord_sup,
            color: PackedColor::from(color).0,
        };

        self.mapping.set(self.offset, instance);
        self.offset += 1;
        self.current_texture = Some(texture_view);
    }

    pub fn ensure_flushed(&mut self, frame: &mut Frame) {
        if let Some(current) = self.current_texture.take() {
            self.flush(current, frame);
        }
    }

    fn flush(&mut self,
             texture_view: TextureView<ColorFormat>,
             frame: &mut Frame)
    {
        let &mut Graphics { ref mut encoder, ref mut device, .. } = frame.graphics;

        let data = render_pipe::Data {
            vertices: self.renderer.vertices.clone(),
            instances: self.renderer.instances.clone(),
            camera: self.renderer.camera_locals.clone(),
            image_sampler: (texture_view, self.renderer.sampler.clone()),
            scissor: self.renderer.scissor.clone(),
            color_target: self.renderer.color_target.clone(),
        };

        self.renderer.slice.instances = Some((self.offset as u32, 0));

        self.mapping.ensure_unmapped();
        encoder.draw(&self.renderer.slice, &self.renderer.pso, &data);
        encoder.flush(device);

        self.offset = 0;
    }
    
    pub fn scissor_mut(&mut self) -> &mut gfx_core::target::Rect {
        self.renderer.scissor_mut()
    }
}