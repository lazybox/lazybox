use std::cmp;
use gfx_core;
use conrod;

use {Graphics, Frame, Color, NormalizedColor};
use camera;
use types::*;
use specialized::polygons;
use specialized::polygons::triangulation::*;
use specialized::glyphs;
use specialized::images;

pub use conrod::render::*;
pub type ImageMap = conrod::image::Map<TextureView<ColorFormat>>;

pub struct Renderer {
    polygons: polygons::Renderer,
    glyphs: glyphs::Renderer,
    images: images::Renderer,
    dpi_factor: f32,
}

impl Renderer {
    pub fn new(color_target: RenderTargetView<ColorFormat>,
               dpi_factor: f32,
               graphics: &mut Graphics) -> Self
    {
        use gfx::traits::*;
        let (w, h, _, _) = color_target.get_dimensions();

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

        Renderer {
            polygons: polygons,
            glyphs: glyphs,
            images: images,
            dpi_factor: dpi_factor,
        }
    }

    pub fn resize(&mut self,
                  color_target: RenderTargetView<ColorFormat>,
                  dpi_factor: f32,
                  graphics: &mut Graphics)
    {
        let (w, h, _, _) = color_target.get_dimensions();
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

    pub fn render<PW>(&mut self,
                      mut primitives: PW,
                      image_map: &ImageMap,
                      frame: &mut Frame)
        where PW: PrimitiveWalker
    {
        let (w, h, _, _) = self.polygons.color_target().get_dimensions();
        let (pixel_w, pixel_h) = (w as f32 / self.dpi_factor,
                                  h as f32 / self.dpi_factor);

        let mut render = Render {
            currently: Rendering::Nothing,
            polygons: &mut self.polygons.render(frame),
            glyphs: &mut self.glyphs,
            images: &mut self.images.render(frame),
            frame: frame,
            dpi_factor: self.dpi_factor,
        };

        while let Some(Primitive { index, kind, scizzor, rect }) = primitives.next_primitive() {
            let scissor = gfx_core::target::Rect {
                x: (scizzor.left() + (pixel_w / 2.) as f64) as u16,
                y: (scizzor.bottom() + (pixel_h / 2.) as f64) as u16,
                w: scizzor.w() as u16,
                h: scizzor.h() as u16,
            };

            use conrod::render::PrimitiveKind::*;
            match kind {
                Rectangle { color } => {
                    println!("rectangle");
                    render.add_rectangle(color, rect, scissor);
                }
                Polygon { color, points } => {
                    println!("polygon");
                    render.add_polygon(color, points, scissor);
                }
                Lines { color, cap, thickness, points } => {
                    println!("line");
                    render.add_lines(color, cap, thickness, points, scissor);
                }
                Image { color, source_rect } => {
                    if let Some(texture) = image_map.get(index) {
                        println!("image");
                        render.add_image(color, source_rect, texture.clone(), rect, scissor);
                    }
                }
                Text { color, text, font_id } => {
                    println!("text");
                    //render.add_text(color, text, font_id, scissor);
                }
                Other(_) => {
                    // TODO?
                }
            }
        }
        
        render.finish();
    }
}

struct Render<'a> {
    currently: Rendering,
    polygons: &'a mut polygons::Render<'a>,
    glyphs: &'a mut glyphs::Renderer,
    images: &'a mut images::Render<'a>,
    frame: &'a mut Frame<'a>,
    dpi_factor: f32,
}

impl<'a> Render<'a> {
    fn switch(&mut self, target: Rendering, scissor: gfx_core::target::Rect) {
        if self.currently != target {
            match self.currently {
                Rendering::Nothing => (),
                Rendering::Polygons => self.polygons.ensure_drawed(self.frame),
                Rendering::Glyphs => unimplemented!(), // TODO
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
        let flush = &mut |frame: &mut Frame| {
            self.images.before_flush(frame);
            // TODO: glyphs
            frame.flush();
        };

        let color = Self::conv_color(color);
        stream_polygon([rect.bottom_right(),
                        rect.top_right(),
                        rect.top_left(),
                        rect.bottom_left()]
                            .iter()
                            .map(|&[x, y]| [x as f32, y as f32]),
                        |triangles| self.polygons.add_slice(color, triangles, flush, self.frame));
    }

    fn add_polygon(&mut self,
                   color: conrod::Color,
                   points: &[conrod::Point],
                   scissor: GfxRect)
    {
        self.switch(Rendering::Polygons, scissor);
        let flush = &mut |frame: &mut Frame| {
            self.images.before_flush(frame);
            // TODO: glyphs
            frame.flush();
        };

        let color = Self::conv_color(color);
        stream_polygon(points.iter().map(|&[x, y]| [x as f32, y as f32]),
                       |triangles| self.polygons.add_slice(color, triangles, flush, self.frame));
    }

    fn add_lines(&mut self,
                 color: conrod::Color,
                 cap: conrod::LineCap,
                 thickness: conrod::Scalar,
                 points: &[conrod::Point],
                 scissor: GfxRect)
    {
        use cgmath::Point2;

        self.switch(Rendering::Polygons, scissor);
        let flush = &mut |frame: &mut Frame| {
            self.images.before_flush(frame);
            // TODO: glyphs
            frame.flush();
        };

        let color = Self::conv_color(color);        
        let resolution = match cap {
            conrod::LineCap::Flat => 2,
            conrod::LineCap::Round => cmp::max(thickness as u32, 2),
        };

        let mut points_iter = points.iter();

        if let Some(previous) = points_iter.next() {
            let mut previous = Point2::new(previous[0] as f32, previous[1] as f32);

            for point in points_iter {
                let current = Point2::new(point[0] as f32, point[1] as f32);

                stream_round_borders_line(
                    previous, current, resolution, thickness as f32 / 2.0,
                    |triangles| self.polygons.add_slice(color, triangles, flush, self.frame));

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
        let flush = &mut |frame: &mut Frame| {
            self.polygons.before_flush(frame);
            // TODO: glyphs
            frame.flush();
        };

        let color = color.map(Self::conv_color).unwrap_or(Color::white());
        let position_inf = [rect.left() as f32, rect.bottom() as f32];
        let position_sup = [rect.right() as f32, rect.top() as f32];

        let (tex_coord_inf, tex_coord_sup) = source_rect
            .map(|rect| ([rect.left() as f32, rect.bottom() as f32],
                         [rect.right() as f32, rect.top() as f32]))
            .unwrap_or(([0.0; 2],
                        [1.0; 2]));

        self.images.add(position_inf,
                        position_sup,
                        tex_coord_inf,
                        tex_coord_sup,
                        texture,
                        color,
                        flush,
                        self.frame); 
    }

    fn add_text(&mut self,
                color: conrod::Color,
                text: conrod::render::Text,
                font_id: conrod::text::font::Id,
                scissor: GfxRect)
    {
        self.switch(Rendering::Glyphs, scissor);
        let flush = &mut |frame: &mut Frame| {
            self.polygons.before_flush(frame);
            self.images.before_flush(frame);
            frame.flush();
        };

        let color = Self::conv_color(color);
        let font_id = font_id.index();
        for glyph in text.positioned_glyphs(self.dpi_factor) {
            self.glyphs.render(font_id, color, glyph.clone(), flush, self.frame);
        }
    }

    fn finish(mut self) {
        self.polygons.before_flush(self.frame);
        // TODO: glyphs
        self.images.before_flush(self.frame);
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
