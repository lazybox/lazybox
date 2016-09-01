use std::cmp;
use gfx_core;
use conrod;
use conrod::widget::primitive::line::Cap as LineCap;

use {Graphics, Frame, Color, NormalizedColor};
use camera;
use specialized::polygons;
use specialized::polygons::triangulation::*;
use specialized::glyphs;
use specialized::images;
use types::*;
use utils::*;

pub use conrod::render::*;
pub type ImageMap = conrod::image::Map<TextureView<ColorFormat>>;

#[doc(hidden)]
pub const FORWARD_GLSLV_150: &'static [u8] = include_bytes!("forward_150.glslv");
#[doc(hidden)]
pub const FORWARD_GLSLF_150: &'static [u8] = include_bytes!("forward_150.glslf");

#[doc(hidden)]
pub use self::defines::forward_pipe;
mod defines {
    pub use types::*;
    pub use utils::*;

    gfx_defines! {
        pipeline forward_pipe {
            vertices: gfx::VertexBuffer<Position> = (),
            color_sampler: gfx::TextureSampler<[f32; 4]> = "s_Color",
            color_target: gfx::BlendTarget<ColorFormat> =
                ("o_Color", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
        }
    }
}

pub struct Renderer {
    polygons: polygons::Renderer,
    glyphs: glyphs::Renderer,
    images: images::Renderer,
    dpi_factor: f32,
    forward_bundle: Bundle<forward_pipe::Data<Resources>>,
}

impl Renderer {
    pub fn new(dpi_factor: f32, graphics: &mut Graphics) -> Self {
        use gfx::traits::*;

        let forward_pso = graphics.factory
            .create_pipeline_simple(FORWARD_GLSLV_150, FORWARD_GLSLF_150, forward_pipe::new())
            .expect("could not create forward pipeline");

        let (w, h, _, _) = graphics.output_color.get_dimensions();

        let (_, color_view, color_target) =
            graphics.factory.create_render_target::<ColorFormat>(w, h).unwrap();

        let conrod_camera_locals = graphics.factory.create_constant_buffer(1);
        let rusttype_camera_locals = graphics.factory.create_constant_buffer(1);

        let (pixel_w, pixel_h) = (w as f32 / dpi_factor,
                                  h as f32 / dpi_factor);
        Self::update_cameras(&conrod_camera_locals,
                             &rusttype_camera_locals,
                             pixel_w, pixel_h,
                             graphics);

        let scissor = gfx_core::target::Rect {
            x: 0,
            y: 0,
            w: w,
            h: h,
        };

        let polygons = polygons::Renderer::new(color_target.clone(),
                                               conrod_camera_locals.clone(),
                                               scissor.clone(),
                                               graphics);

        let glyphs = glyphs::Renderer::new(color_target.clone(),
                                           rusttype_camera_locals,
                                           scissor.clone(),
                                           graphics);

        let images = images::Renderer::new(color_target,
                                           conrod_camera_locals,
                                           scissor,
                                           graphics);

        let forward_bundle = {
            let (vertices, slice) = graphics.factory
                .create_vertex_buffer_with_slice(&QUAD_VERTICES, &QUAD_INDICES[..]);
            let linear_sampler = graphics.factory.create_sampler_linear();

            let data = forward_pipe::Data {
                vertices: vertices,
                color_sampler: (color_view, linear_sampler),
                color_target: graphics.output_color.clone(),
            };

            Bundle::new(slice, forward_pso, data)
        };

        Renderer {
            polygons: polygons,
            glyphs: glyphs,
            images: images,
            dpi_factor: dpi_factor,
            forward_bundle: forward_bundle,
        }
    }

    pub fn resize(&mut self, dpi_factor: f32, graphics: &mut Graphics) {
        use gfx::traits::*;

        let (w, h, _, _) = graphics.output_color.get_dimensions();

        let (_, color_view, color_target) =
            graphics.factory.create_render_target::<ColorFormat>(w, h).unwrap();

        let (pixel_w, pixel_h) = (w as f32 / dpi_factor,
                                  h as f32 / dpi_factor);
        Self::update_cameras(&self.polygons.camera(),
                             &self.glyphs.camera(),
                             pixel_w, pixel_h,
                             graphics);

        self.polygons.resize(color_target.clone());
        self.glyphs.resize(color_target.clone());
        self.images.resize(color_target);
        self.dpi_factor = dpi_factor;

        self.forward_bundle.data.color_sampler.0 = color_view;
        self.forward_bundle.data.color_target = graphics.output_color.clone();
    }

    fn update_cameras(conrod_camera_locals: &GpuBuffer<camera::Locals>,
                      rusttype_camera_locals: &GpuBuffer<camera::Locals>,
                      pixel_w: f32, pixel_h: f32,
                      graphics: &mut Graphics)
    {
        let (w, h) = (pixel_w, pixel_h);

        // conrod (origin middle, pixel unit, y upwards) to gl.
        let conrod_locals = camera::Locals {
            translate: [0., 0.],
            scale: [2. / w, 2. / h],
        };
        graphics.encoder.update_constant_buffer(conrod_camera_locals, &conrod_locals);

        // rusttype (origin top-left, pixel unit, y downwards) to gl.
        let rusttype_locals = camera::Locals {
            translate: [-w / 2., -h / 2.],
            scale: [2. / w, -2. / h],
        };
        graphics.encoder.update_constant_buffer(rusttype_camera_locals, &rusttype_locals);
    }

    fn color_target(&self) -> &RenderTargetView<ColorFormat> {
        self.polygons.color_target()
    }

    pub fn changed<PW>(&mut self,
                       mut primitives: PW,
                       image_map: &ImageMap,
                       frame: &mut Frame)
        where PW: PrimitiveWalker
    {
        let (w, h, _, _) = self.color_target().get_dimensions();
        let (pixel_w, pixel_h) = (w as f32 / self.dpi_factor,
                                  h as f32 / self.dpi_factor);

        frame.graphics.encoder.clear(self.color_target(), [0.0; 4]);
        let mut render = Render {
            currently: Rendering::Nothing,
            polygons: self.polygons.render(frame),
            glyphs: &mut self.glyphs,
            images: self.images.render(frame),
            frame: frame,
            dpi_factor: self.dpi_factor,
        };

        while let Some(Primitive { index, kind, scizzor, rect }) = primitives.next_primitive() {
            // FIXME: the scissor is meant for the current primitive,
            // but it will be applied to the whole batch 
            let scissor = gfx_core::target::Rect {
                x: (scizzor.left() + (pixel_w / 2.) as f64) as u16,
                y: (scizzor.bottom() + (pixel_h / 2.) as f64) as u16,
                w: scizzor.w() as u16,
                h: scizzor.h() as u16,
            };

            use conrod::render::PrimitiveKind::*;
            match kind {
                Rectangle { color } => {
                    render.add_rectangle(color, rect, scissor);
                }
                Polygon { color, points } => {
                    render.add_polygon(color, points, scissor);
                }
                Lines { color, cap, thickness, points } => {
                    render.add_lines(color, cap, thickness, points, scissor);
                }
                Image { color, source_rect } => {
                    if let Some(texture) = image_map.get(index) {
                        render.add_image(color, source_rect, texture.clone(), rect, scissor);
                    }
                }
                Text { color, text, font_id } => {
                    render.add_text(color, text, font_id, scissor);
                }
                Other(_) => {
                    // TODO?
                }
            }
        }

        render.finish();
    }

    pub fn render(&mut self, frame: &mut Frame) {
        self.forward_bundle.encode(&mut frame.graphics.encoder);
        frame.should_flush();
    }
}

struct Render<'a, 'b: 'a, 'c, 'd: 'c> {
    currently: Rendering,
    polygons: polygons::Render<'b>,
    glyphs: &'a mut glyphs::Renderer,
    images: images::Render<'b>,
    frame: &'c mut Frame<'d>,
    dpi_factor: f32,
}

impl<'a, 'b: 'a, 'c, 'd: 'c> Render<'a, 'b, 'c, 'd> {
    fn switch(&mut self, target: Rendering, scissor: GfxRect) {
        if self.currently != target {
            match self.currently {
                Rendering::Nothing => (),
                Rendering::Polygons => self.polygons.ensure_drawed(self.frame),
                Rendering::Glyphs => {
                    self.polygons.before_flush(self.frame);
                    self.images.before_flush(self.frame);
                    self.glyphs.before_flush(self.frame);
                    self.frame.ensure_flushed();
                }
                Rendering::Images => self.images.ensure_drawed(self.frame),
            }

            self.currently = target;
        }

        match target {
            Rendering::Nothing => (),
            Rendering::Polygons => *self.polygons.scissor_mut() = scissor,
            Rendering::Glyphs => *self.glyphs.scissor_mut() = scissor,
            Rendering::Images => *self.images.scissor_mut() = scissor,
        }
    }

    fn add_rectangle(&mut self,
                     color: conrod::Color,
                     rect: conrod::Rect,
                     scissor: GfxRect)
    {
        self.switch(Rendering::Polygons, scissor);
        let &mut Render { ref mut polygons,
                          ref mut glyphs,
                          ref mut images,
                          ref mut frame, .. } = self;

        let before_flush = &mut |frame: &mut Frame| {
            images.before_flush(frame);
            glyphs.before_flush(frame);
        };

        let color = Self::conv_color(color);
        stream_polygon([rect.bottom_right(),
                        rect.top_right(),
                        rect.top_left(),
                        rect.bottom_left()]
                            .iter()
                            .map(|&[x, y]| [x as f32, y as f32]),
                        |triangles| polygons.add_slice(color, triangles, before_flush, frame));
    }

    fn add_polygon(&mut self,
                   color: conrod::Color,
                   points: &[conrod::Point],
                   scissor: GfxRect)
    {
        self.switch(Rendering::Polygons, scissor);
        let &mut Render { ref mut polygons,
                          ref mut glyphs,
                          ref mut images,
                          ref mut frame, .. } = self;

        let before_flush = &mut |frame: &mut Frame| {
            images.before_flush(frame);
            glyphs.before_flush(frame);
        };

        let color = Self::conv_color(color);
        stream_polygon(points.iter().map(|&[x, y]| [x as f32, y as f32]),
                       |triangles| polygons.add_slice(color, triangles, before_flush, frame));
    }

    fn add_lines(&mut self,
                 color: conrod::Color,
                 cap: LineCap,
                 thickness: conrod::Scalar,
                 points: &[conrod::Point],
                 scissor: GfxRect)
    {
        use cgmath::Point2;

        self.switch(Rendering::Polygons, scissor);
        let &mut Render { ref mut polygons,
                          ref mut glyphs,
                          ref mut images,
                          ref mut frame, .. } = self;

        let before_flush = &mut |frame: &mut Frame| {
            images.before_flush(frame);
            glyphs.before_flush(frame);
        };

        let color = Self::conv_color(color);        
        let resolution = match cap {
            LineCap::Flat => 2,
            LineCap::Round => cmp::max(thickness as u32, 2),
        };

        let mut points_iter = points.iter();

        if let Some(previous) = points_iter.next() {
            let mut previous = Point2::new(previous[0] as f32, previous[1] as f32);

            for point in points_iter {
                let current = Point2::new(point[0] as f32, point[1] as f32);

                stream_round_borders_line(
                    previous, current, resolution, thickness as f32 / 2.0,
                    |triangles| polygons.add_slice(color, triangles, before_flush, frame));

                previous = current;
            }
        }
    }

    fn add_image(&mut self,
                 color: Option<conrod::Color>,
                 source_rect: Option<conrod::Rect>,
                 texture: TextureView<ColorFormat>,
                 rect: conrod::Rect,
                 scissor: GfxRect)
    {
        self.switch(Rendering::Images, scissor);
        let &mut Render { ref mut polygons,
                          ref mut glyphs,
                          ref mut images,
                          ref mut frame, .. } = self;

        let before_flush = &mut |frame: &mut Frame| {
            polygons.before_flush(frame);
            glyphs.before_flush(frame);
        };

        let color = color.map(Self::conv_color).unwrap_or(Color::white());
        let position_inf = [rect.left() as f32, rect.bottom() as f32];
        let position_sup = [rect.right() as f32, rect.top() as f32];

        let (tex_coord_inf, tex_coord_sup) = source_rect
            .map(|rect| ([rect.left() as f32, rect.bottom() as f32],
                         [rect.right() as f32, rect.top() as f32]))
            .unwrap_or(([0.0; 2],
                        [1.0; 2]));

        images.add(position_inf,
                        position_sup,
                        tex_coord_inf,
                        tex_coord_sup,
                        texture,
                        color,
                        before_flush,
                        frame); 
    }

    fn add_text(&mut self,
                color: conrod::Color,
                text: conrod::render::Text,
                font_id: conrod::text::font::Id,
                scissor: GfxRect)
    {
        self.switch(Rendering::Glyphs, scissor);
        let &mut Render { ref mut polygons,
                          ref mut glyphs,
                          ref mut images,
                          ref mut frame,
                          dpi_factor, .. } = self;

        let before_flush = &mut |frame: &mut Frame| {
            polygons.before_flush(frame);
            images.before_flush(frame);
        };

        let color = Self::conv_color(color);
        let font_id = font_id.index();
        for glyph in text.positioned_glyphs(dpi_factor) {
            glyphs.render(font_id, color, glyph.clone(), before_flush, frame);
        }
    }

    fn finish(mut self) {
        self.polygons.before_flush(self.frame);
        self.images.before_flush(self.frame);
        self.glyphs.before_flush(self.frame);
    }

    fn conv_color(c: conrod::Color) -> Color {
        Color::from(NormalizedColor::from_srgb(c.to_fsa()))
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Rendering {
    Nothing,
    Polygons,
    Glyphs,
    Images,
}
