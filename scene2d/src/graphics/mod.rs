mod utils;

pub mod types;
pub mod camera;
pub mod color;
pub mod lights;
pub mod sprites;
#[doc(hidden)]
pub mod texture;
#[doc(hidden)]
pub mod layer;

pub mod combined;
pub mod specialized;

use {gfx, glutin, gfx_window_glutin, gfx_device_gl, image};
use glutin::{WindowBuilder, Window};
use self::types::*;
use {NormalizedColor, TextureBind};
use self::texture::TextureBinds;

pub struct Graphics {
    device: Device,
    factory: Factory,
    encoder: Encoder,
    output_color: OutputColor,
    output_depth: OutputDepth,
    texture_binds: TextureBinds,
}

impl Graphics {
    pub fn new(builder: WindowBuilder) -> (Window, Self) {
        let (window, device, mut factory, color, depth) =
            gfx_window_glutin::init::<ColorFormat, DepthFormat>(
                builder.with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2))));
        let encoder = factory.create_command_buffer().into();

        let graphics = Graphics {
            device: device,
            factory: factory,
            encoder: encoder,
            output_color: color,
            output_depth: depth,
            texture_binds: TextureBinds::new(),
        };

        (window, graphics)
    }

    pub fn resize(&mut self, window: &Window) {
        use gfx_core::format::Formatted;
        use gfx_core::memory::Typed;

        let (w, h) = window.get_inner_size_pixels().unwrap();
        let aa = window.get_pixel_format().multisampling
            .unwrap_or(0) as gfx::texture::NumSamples;
        let dim = (w as gfx::texture::Size, h as gfx::texture::Size, 1, aa.into());

        let (color, depth) = gfx_device_gl::create_main_targets_raw(
            dim,
            ColorFormat::get_format().0,
            DepthFormat::get_format().0);

        self.output_color = Typed::new(color);
        self.output_depth = Typed::new(depth);
    }

    pub fn draw<'a>(&'a mut self) -> Frame<'a> {
        Frame::new(self)
    }

    pub fn bind_textures(&mut self, color: TextureView<ColorFormat>,
                                    normal: TextureView<NormalFormat>) -> TextureBind {
        self.texture_binds.insert(texture::Bind {
            color: color,
            normal: normal,
        })
    }

    pub fn update_bound_color(&mut self, bind: TextureBind, color: TextureView<ColorFormat>) {
        self.texture_binds.get_mut(bind).color = color;
    }

    pub fn update_bound_normal(&mut self, bind: TextureBind, normal: TextureView<NormalFormat>) {
        self.texture_binds.get_mut(bind).normal = normal;
    }

    pub fn unbind_textures(&mut self, bind: TextureBind) {
        self.texture_binds.remove(bind);
    }

    pub fn load_texture<F>(&mut self, w: u16, h: u16, data: &[u8]) -> TextureView<F>
        where F: gfx::format::TextureFormat
    {
        use gfx::traits::*;

        let aa_mode = gfx::texture::AaMode::Single;
        let kind = gfx::texture::Kind::D2(w, h, aa_mode);
        let (_, view) = self.factory.create_texture_immutable_u8::<F>(kind, &[data]).unwrap();
        view
    }

    pub fn load_texture_from_image<F>(&mut self, path: &str) -> TextureView<F>
        where F: gfx::format::TextureFormat
    {
        let image = image::open(path).unwrap().to_rgba();
        let (w, h) = image.dimensions();
        self.load_texture::<F>(w as u16, h as u16, &image)
    }

    pub fn load_white_color(&mut self) -> TextureView<ColorFormat> {
        self.load_texture::<ColorFormat>(1, 1, &[255, 255, 255, 255])
    }

    pub fn load_flat_normal(&mut self) -> TextureView<NormalFormat> {
        self.load_texture::<NormalFormat>(1, 1, &[128, 128, 255, 255])
    }
}

pub struct Frame<'a> {
    pub graphics: &'a mut Graphics,
    pub should_flush: bool,
}

impl<'a> Frame<'a> {
    fn new(graphics: &'a mut Graphics) -> Self {
        Frame {
            graphics: graphics,
            should_flush: false,
        }
    }

    pub fn clear(&mut self, color: NormalizedColor) {
        self.graphics.encoder.clear(&self.graphics.output_color, color.to_array());
        self.should_flush();
    }

    pub fn should_flush(&mut self) {
        self.should_flush = true;
    }

    pub fn flush(&mut self) {
        self.graphics.encoder.flush(&mut self.graphics.device);
        self.should_flush = false;
    }

    pub fn ensure_flushed(&mut self) {
        if self.should_flush { self.flush(); }
    }

    pub fn present(mut self, window: &'a Window) {
        use gfx::traits::*;

        self.ensure_flushed();
        window.swap_buffers().unwrap();
        self.graphics.device.cleanup();
    }
}
