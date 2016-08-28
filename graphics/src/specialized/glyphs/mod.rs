pub mod cache;

use gfx;
use gfx_core;
use rusttype::{PositionedGlyph, Point};

use {Graphics, Frame, Color};
use color::PackedColor;
use types::*;
use utils::*;
use camera;
use self::cache::*;

const GLYPH_BUFFER_SIZE: usize = 1024;

#[doc(hidden)]
pub const RENDER_GLSLV_150: &'static [u8] = include_bytes!("render_150.glslv");
#[doc(hidden)]
pub const RENDER_GLSLF_150: &'static [u8] = include_bytes!("render_150.glslf");

#[doc(hidden)]
pub use self::defines::{GlyphInstance, pipe};
mod defines {
    pub use types::*;
    pub use utils::*;
    pub use camera;

    gfx_defines! {
        vertex GlyphInstance {
            translate_inf: [f32; 2] = "a_TranslateInf",
            translate_sup: [f32; 2] = "a_TranslateSup",
            tex_coord_inf: [f32; 2] = "a_TexCoordInf",
            tex_coord_sup: [f32; 2] = "a_TexCoordSup",
            color: u32 = "a_Color",
        }

        pipeline pipe {
            vertices: gfx::VertexBuffer<Position> = (),
            instances: gfx::InstanceBuffer<GlyphInstance> = (),
            camera: gfx::ConstantBuffer<camera::Locals> = "Camera",
            glyph_sampler: gfx::TextureSampler<f32> = "s_Glyph",
            scissor: gfx::pso::target::Scissor = (),
            color_target: gfx::BlendTarget<ColorFormat> =
                ("o_Color", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
        }
    }
}

pub struct Renderer {
    bundle: Bundle<pipe::Data<Resources>>,
    mapping: MappingWritable<GlyphInstance>,
    cache: GlyphCache,
    queue: Vec<(usize, PackedColor, PositionedGlyph<'static>)>,
}

impl Renderer {
    pub fn new(color_target: RenderTargetView<ColorFormat>,
               camera_locals: GpuBuffer<camera::Locals>,
               scissor: gfx_core::target::Rect,
               graphics: &mut Graphics) -> Self
    {
        use gfx::traits::*;

        let pso = graphics.factory
            .create_pipeline_simple(RENDER_GLSLV_150, RENDER_GLSLF_150, pipe::new())
            .expect("could not create glyphs pipeline");
        
        let (vertices, slice) = graphics.factory
            .create_vertex_buffer_with_slice(&QUAD_VERTICES, &QUAD_INDICES[..]);
        
        let (instances, mapping) = graphics.factory
            .create_buffer_persistent_writable(GLYPH_BUFFER_SIZE,
                                               gfx::BufferRole::Vertex,
                                               gfx::Bind::empty());

        let linear_sampler = graphics.factory.create_sampler_linear();

        // TODO: think about the cache size
        let (cache, glyph_view) = GlyphCache::new(512, 512, graphics);

        let data = pipe::Data {
            vertices: vertices,
            instances: instances,
            camera: camera_locals,
            glyph_sampler: (glyph_view, linear_sampler),
            scissor: scissor,
            color_target: color_target,
        };

        Renderer {
            bundle: Bundle::new(slice, pso, data),
            mapping: mapping,
            cache: cache,
            queue: Vec::with_capacity(GLYPH_BUFFER_SIZE),
        }
    }

    pub fn resize(&mut self, color_target: RenderTargetView<ColorFormat>) {
        self.bundle.data.color_target = color_target;
    }

    pub fn camera(&self) -> &GpuBuffer<camera::Locals> {
        &self.bundle.data.camera
    }

    pub fn scissor_mut(&mut self) -> &mut gfx_core::target::Rect {
        &mut self.bundle.data.scissor
    }

    pub fn render<F>(&mut self,
                     font_id: usize,
                     color: Color,
                     glyph: PositionedGlyph<'static>,
                     before_flush: &mut F,
                     frame: &mut Frame)
        where F: FnMut(&mut Frame)
    {
        if self.queue.len() == GLYPH_BUFFER_SIZE {
            before_flush(frame);
            self.draw(frame);
            frame.flush();
        }

        self.cache.queue_glyph(font_id, glyph.clone());
        self.queue.push((font_id, PackedColor::from(color), glyph));
    }

    pub fn before_flush(&mut self, frame: &mut Frame) {
        if self.queue.len() > 0 {
            self.draw(frame);
            frame.should_flush();
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let &mut Graphics { ref mut encoder, .. } = frame.graphics;

        self.cache.cache_queued(encoder).unwrap();

        let conv_point = |p: Point<i32>| [p.x as f32, p.y as f32];

        let mut writer = self.mapping.write();
        let mut i = 0;
        for (font_id, color, glyph) in self.queue.drain(..) {
            if let Ok(Some((uv, screen))) = self.cache.rect_for(font_id, &glyph) {
                let instance = GlyphInstance {
                    translate_inf: conv_point(screen.min),
                    translate_sup: conv_point(screen.max),
                    tex_coord_inf: [uv.min.x, uv.min.y],
                    tex_coord_sup: [uv.max.x, uv.max.y],
                    color: color.0,
                };

                writer.set(i, instance);
                i += 1;                
            }
        }

        self.bundle.slice.instances = Some((i as u32, 0));
        self.bundle.encode(encoder);
    }
}
