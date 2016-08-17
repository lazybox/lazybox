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
        use cgmath::Point2;

        let &mut Renderer { ref mut polygons,
                            ref mut glyphs,
                            ref mut images,
                            dpi_factor } = self;

        let (w, h, _, _) = polygons.color_target().get_dimensions();
        let (pixel_w, pixel_h) = (w as f32 / dpi_factor,
                                  h as f32 / dpi_factor);

        let polygons = &mut polygons.render(frame);
        let images = &mut images.render(frame);

        let conv_color = |c: conrod::Color| {
            Color::from(NormalizedColor::from_srgb(c.to_fsa()))
        };
        
        let flush_polygons = |glyphs: &mut glyphs::Renderer,
                              images: &mut images::Render,
                              frame: &mut Frame| {
            images.before_flush(frame);
            glyphs.ensure_flushed(&mut |frame| frame.flush(), frame);
            frame.ensure_flushed();
        };

        let flush_glyphs = |polygons: &mut polygons::Render,
                            images: &mut images::Render,
                            frame: &mut Frame| {
            polygons.before_flush(frame);
            images.before_flush(frame);
            frame.flush();
        };

        let flush_images = |polygons: &mut polygons::Render,
                            glyphs: &mut glyphs::Renderer,
                            frame: &mut Frame| {
            polygons.before_flush(frame);
            glyphs.ensure_flushed(&mut |frame| frame.flush(), frame);
            frame.ensure_flushed();
        };

        let currently = &mut Rendering::Nothing;
        let mut switch = |target,
                          scissor,
                          polygons: &mut polygons::Render,
                          glyphs: &mut glyphs::Renderer,
                          images: &mut images::Render,
                          frame: &mut Frame| {
            if *currently != target {
                match *currently {
                    Rendering::Nothing => (),
                    Rendering::Polygons => polygons.ensure_drawed(frame),
                    Rendering::Glyphs => {
                        let flush = &mut |frame: &mut Frame| flush_glyphs(polygons, images, frame);
                        glyphs.ensure_flushed(flush, frame);
                    }
                    Rendering::Images => images.ensure_drawed(frame),
                }
                *currently = target;
            }

            match target {
                Rendering::Nothing => (),
                Rendering::Polygons => *polygons.scissor_mut() = scissor,
                Rendering::Glyphs => *glyphs.scissor_mut() = scissor,
                Rendering::Images => *images.scissor_mut() = scissor,
            }
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

                    switch(Rendering::Polygons, scissor, polygons, glyphs, images, frame);
                    let flush = &mut |frame: &mut Frame| flush_polygons(glyphs, images, frame);

                    let color = conv_color(color);
                    stream_polygon([rect.bottom_right(),
                                    rect.top_right(),
                                    rect.top_left(),
                                    rect.bottom_left()]
                                        .iter()
                                        .map(|&[x, y]| [x as f32, y as f32]),
                                    |triangles| polygons.add_slice(color, triangles, flush, frame));
                }
                Polygon { color, points } => {
                    println!("polygon");

                    switch(Rendering::Polygons, scissor, polygons, glyphs, images, frame);
                    let flush = &mut |frame: &mut Frame| flush_polygons(glyphs, images, frame);

                    let color = conv_color(color);
                    stream_polygon(points.iter().map(|&[x, y]| [x as f32, y as f32]),
                                    |triangles| polygons.add_slice(color, triangles, flush, frame));
                }
                Lines { color, cap, thickness, points } => {
                    println!("line");

                    switch(Rendering::Polygons, scissor, polygons, glyphs, images, frame);
                    let flush = &mut |frame: &mut Frame| flush_polygons(glyphs, images, frame);

                    let color = conv_color(color);            
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
                                |triangles| polygons.add_slice(color, triangles, flush, frame));

                            previous = current;
                        }
                    }
                }
                Image { color, source_rect } => {
                    if let Some(texture_view) = image_map.get(index) {
                        println!("image");

                        switch(Rendering::Images, scissor, polygons, glyphs, images, frame);
                        let flush = &mut |frame: &mut Frame| flush_images(polygons, glyphs, frame);

                        let color = color.map(|c| conv_color(c)).unwrap_or(Color::white());
                        let position_inf = [rect.left() as f32, rect.bottom() as f32];
                        let position_sup = [rect.right() as f32, rect.top() as f32];

                        let tex_coord_inf;
                        let tex_coord_sup;
                        if let Some(rect) = source_rect {
                            tex_coord_inf = [rect.left() as f32, rect.bottom() as f32];
                            tex_coord_sup = [rect.right() as f32, rect.top() as f32];
                        } else {
                            tex_coord_inf = [0.0; 2];
                            tex_coord_sup = [1.0; 2];
                        }

                        images.add(position_inf,
                                    position_sup,
                                    tex_coord_inf,
                                    tex_coord_sup,
                                    texture_view.clone(),
                                    color,
                                    flush,
                                    frame); 
                    }
                }
                Text { color, text, font_id } => {
                    println!("text");

                    switch(Rendering::Glyphs, scissor, polygons, glyphs, images, frame);
                    let flush = &mut |frame: &mut Frame| flush_glyphs(polygons, images, frame);

                    let color = conv_color(color);
                    let font_id = font_id.index();
                    for glyph in text.positioned_glyphs(dpi_factor) {
                        glyphs.render(font_id, color, glyph.clone(), flush, frame);
                    }
                }
                Other(_) => {
                    // TODO?
                }
            }
        }

        println!("end");
        polygons.before_flush(frame);
        images.before_flush(frame);
        glyphs.ensure_flushed(&mut |frame| frame.flush(), frame);
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Rendering {
    Nothing,
    Polygons,
    Glyphs,
    Images,
}
