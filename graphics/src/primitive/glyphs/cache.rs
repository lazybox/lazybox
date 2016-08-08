use gfx;
use rusttype::gpu_cache::Cache;
pub use rusttype::gpu_cache::{CacheWriteErr, CacheReadErr};
use rusttype::{PositionedGlyph, Rect};

use Graphics;
use types::*;

pub const SCALE_TOLERANCE: f32 = 0.1;
pub const POSITION_TOLERANCE: f32 = 0.1;

pub type GlyphSurface = gfx::format::R8;
pub type GlyphChannel = gfx::format::Unorm;
pub type GlyphFormat = (GlyphSurface, GlyphChannel);

pub struct GlyphCache {
    cache: Cache,
    texture: Texture<GlyphSurface>,
}

impl GlyphCache {
    pub fn new(width: u32, height: u32,
               graphics: &mut Graphics) -> (Self, ShaderResourceView<f32>) {
        use gfx::traits::*;

        let cache = Cache::new(width, height, SCALE_TOLERANCE, POSITION_TOLERANCE);

        let (texture, texture_view) = {
            let kind = gfx::tex::Kind::D2(width as u16, height as u16, gfx::tex::AaMode::Single);
            let bind = gfx::SHADER_RESOURCE;
            let usage = gfx::Usage::Dynamic;
            let channel = gfx::format::ChannelType::Unorm;
            let texture = graphics.factory
                .create_texture(kind, 1, bind, usage, Some(channel))
                .unwrap();
            let swizzle = gfx::format::Swizzle::new();
            let view = graphics.factory
                .view_texture_as_shader_resource::<GlyphFormat>(&texture, (0, 0), swizzle)
                .unwrap();

            (texture, view)
        };

        let cache = GlyphCache {
            cache: cache,
            texture: texture,
        };

        (cache, texture_view)
    }

    pub fn queue_glyph(&mut self, font_id: usize, glyph: PositionedGlyph) {
        self.cache.queue_glyph(font_id, glyph);
    }

    pub fn cache_queued(&mut self, encoder: &mut Encoder) -> Result<(), CacheWriteErr> {
        let &mut GlyphCache { ref mut cache, ref texture } = self;

        cache.cache_queued(|rect, data| {
            let info = gfx::tex::ImageInfoCommon {
                xoffset: rect.min.x as u16,
                yoffset: rect.min.y as u16,
                zoffset: 0,
                width: rect.width() as u16,
                height: rect.height() as u16,
                depth: 0,
                format: (),
                mipmap: 0,
            };

            encoder.update_texture::<_, GlyphFormat>(texture, None, info, data).unwrap();
        })
    }

    pub fn rect_for(&self, font_id: usize, glyph: &PositionedGlyph
                    ) -> Result<Option<(Rect<f32>, Rect<i32>)>, CacheReadErr> {
        self.cache.rect_for(font_id, glyph)
    }
}