#![feature(slice_patterns)]

extern crate lazybox_graphics as graphics;
extern crate lazybox_frameclock as frameclock;
extern crate glutin;
extern crate cgmath;
extern crate rayon;
extern crate rand;

use graphics::{Graphics, Camera, Color, NormalizedColor};
use graphics::combined::sprites::Renderer;
use graphics::layer::{LayerOrder, LayerOcclusion};
use graphics::sprites::{Sprite, SpriteTexture};
use graphics::lights::*;
use graphics::types::{ColorFormat, NormalFormat};
use glutin::{WindowBuilder, Event};
use cgmath::{Point2, Vector2, Rad};
use rayon::prelude::*;
use rand::Rng;
use frameclock::*;

fn main() {
    let builder = WindowBuilder::new()
        .with_title("Sprites".to_string())
        .with_dimensions(1080, 768);

    let (window, mut graphics) = Graphics::new(builder);
    let mut renderer = Renderer::new(&mut graphics);
    let ground_layer = renderer.push_layer(LayerOcclusion::Ignore);
    let first_layer = renderer.push_layer(LayerOcclusion::Stack);
    let platform_layer = renderer.push_layer(LayerOcclusion::Ignore);
    let second_layer = renderer.push_layer(LayerOcclusion::Stack);

    let mut camera = Camera::new(Point2::new(10., 10.), Vector2::new(20.0, 20.0), &window);

    let clear_color = NormalizedColor::from_srgb([0.1, 0.1, 0.2, 1.0]);
    let ambient_light = AmbientLight {
        color: LightColor::from_srgb([0.2, 0.2, 0.2]),
        intensity: 0.5,
    };

    let white_color = graphics.load_white_color();
    let flat_normal = graphics.load_flat_normal();
    let no_texture = SpriteTexture {
        bind: graphics.bind_textures(white_color, flat_normal),
        coord_inf: [0.; 2],
        coord_sup: [1.; 2],
    };
    let no_blend_color = Color::white();

    let ground_color = graphics
        .load_texture_from_image::<ColorFormat>("resources/slate_color.png");
    let ground_normal = graphics
        .load_texture_from_image::<NormalFormat>("resources/slate_normal.png");
    let ground_texture = SpriteTexture {
        bind: graphics.bind_textures(ground_color, ground_normal),
        coord_inf: [0.; 2],
        coord_sup: [3.; 2],
    };

    let ground_blend_color = no_blend_color;
    let platform_blend_color = Color::from_srgb([44, 41, 61, 200]);
    let object_blend_color = Color::from_srgb([135, 1, 15, 255]);

    let ground_sprites = [Sprite {
        position: Point2::new(10., 10.),
        size: Vector2::new(20., 20.),
        rotation: Rad(0.),
        texture: ground_texture,
        color: ground_blend_color,
    }];

    let object_sprite = |p, s, r| Sprite {
         position: p,
         size: Vector2::new(s, s),
         rotation: r,
         texture: no_texture.clone(),
         color: object_blend_color,
    };

    let first_sprites: Vec<_> = [
        [ 4.,  4.], [ 16.,  4.],
        [ 4.,  8.], [ 16.,  8.],
        [ 4., 12.], [ 16., 12.],
        [ 4., 16.], [ 16., 16.],
    ].iter()
     .map(|&[x, y]| object_sprite(Point2::new(x, y), 1.0, Rad(0.75)))
     .collect();

    let platform_sprites = [Sprite {
        position: Point2::new(10., 10.),
        size: Vector2::new(14., 14.),
        rotation: Rad(0.),
        texture: no_texture.clone(),
        color: platform_blend_color,
    }];

    let second_sprites: Vec<_> = [
        [  6., 6.], [  6., 14.],
        [  8., 6.], [  8., 14.],
        [ 12., 6.], [ 12., 14.],
        [ 14., 6.], [ 14., 14.],
    ].iter()
     .map(|&[x, y]| object_sprite(Point2::new(x, y), 0.5, Rad(0.)))
     .collect();

    let sprites = [
        (ground_layer, &ground_sprites[..]),
        (first_layer, &first_sprites[..]),
        (platform_layer, &platform_sprites[..]),
        (second_layer, &second_sprites[..]),
    ];

    let mut first_lights: Vec<_> = [
        ( 1., [  8.,  4.]),
        (-1., [ 12., 16.]),
    ].iter()
     .map(|&(d, p)| Light {
         center: p.into(),
         radius: 6.,
         source_radius: 0.5,
         source_layer: first_layer,
         color: LightColor::from_srgb([0.66, 0.84, 0.84]),
         intensity: 1.5,
     })
     .collect();

    let mut second_lights: Vec<_> = [
        ( 1., [  4., 12.]),
        (-1., [ 16.,  8.]),
    ].iter()
     .map(|&(d, p)| Light {
         center: p.into(),
         radius: 4.,
         source_radius: 0.1,
         source_layer: second_layer,
         color: LightColor::from_srgb([0.9, 0.6, 0.03]),
         intensity: 2.5,
     })
     .collect();

    let rng = &mut rand::thread_rng();
    let mut create_light = |c| Light {
        center: c,
        radius: rng.gen_range(0.5, 4.),
        source_radius: rng.gen_range(0.1, 0.2),
        source_layer: second_layer,
        color: LightColor {
            r: rng.gen_range(0., 1.),
            g: rng.gen_range(0., 1.),
            b: rng.gen_range(0., 1.),
        },
        intensity: 2.,
    };

    let mut user_lights = Vec::new();
    let mut world_position = Point2::new(10., 10.);

    let mut frameclock = FrameClock::start(1.);
    let mut fps_counter = FpsCounter::new(1.);

    'main: loop {
        let delta_time = frameclock.reset();
        if let Some(fps) = fps_counter.update(delta_time) {
            println!("{:.4} ms/frame, {} frame/s", 1000. / fps, fps as usize);
            println!("- {} lights", user_lights.len());
        }

        for event in window.poll_events() {
            match event {
                Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                Event::Closed => break 'main,
                Event::Resized(..) => {
                    graphics.resize(&window);
                    renderer.resize(&mut graphics);
                    camera.update_transform(&window);
                },
                Event::MouseInput(glutin::ElementState::Pressed, _) => {
                    user_lights.push(create_light(world_position));
                }
                Event::MouseMoved(x, y) => {
                    let w = Point2::new(x, y);
                    world_position = camera.window_point_to_world(w, &window);

                    user_lights.last_mut().map(|l| l.center = world_position); 
                }
                _ => {},
            }
        }

        {
            let access = renderer.access();
            rayon::join(
                || {
                    sprites.par_iter().for_each(|&(layer, sprites)| {
                        let mut queue = access.queue(layer);
                        for sprite in sprites {
                            queue.submit(sprite, LayerOrder(0));
                        }
                    });
                },
                || {
                    rayon::join(
                        || {
                            let mut queue = access.light_queue();
                            for light in &mut first_lights {
                                queue.submit(light.clone());
                            }
                            for light in &mut second_lights {
                                queue.submit(light.clone());
                            }
                        },
                        || {
                            let mut queue = access.light_queue();
                            for light in &user_lights {
                                queue.submit(light.clone());
                            }
                        },
                    );
                },
            );
        }

        let mut frame = graphics.draw();
        frame.clear(clear_color);
        renderer.submit(&camera, &ambient_light, &mut frame);
        frame.present(&window);
    }
}
