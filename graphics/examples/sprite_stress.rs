extern crate lazybox_graphics as graphics;
extern crate lazybox_frameclock as frameclock;
extern crate glutin;
extern crate cgmath;
extern crate rayon;
extern crate rand;

use graphics::{Graphics, Camera, Color, NormalizedColor};
use graphics::layer::{LayerOcclusion, LayerOrder};
use graphics::sprites::{Sprite, SpriteTexture};
use graphics::combined::sprites::Renderer;
use graphics::lights::*;
use graphics::types::ColorFormat;
use glutin::{WindowBuilder, Event};
use cgmath::{Point2, Vector2, Rad};
use rayon::prelude::*;
use rand::Rng;
use frameclock::*;

fn main() {
    let builder = WindowBuilder::new()
        .with_title("Sprites Stress".to_string())
        .with_dimensions(1080, 768)
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2)));

    let (window, mut graphics) = Graphics::new(builder);
    let mut renderer = Renderer::new(&mut graphics);
    let ground_layer = renderer.push_layer(LayerOcclusion::Ignore);
    let smiley_layer = renderer.push_layer(LayerOcclusion::Ignore);
    let sky_layer = renderer.push_layer(LayerOcclusion::Ignore);

    let camera = Camera::new(Point2::new(10., 10.), Vector2::new(20.0, 20.0), &window);

    let clear_color = NormalizedColor::from_srgb([0.1, 0.1, 0.2, 1.0]);
    let ambient_light = AmbientLight {
        color: LightColor::from_srgb([1.0, 0.8, 0.8]),
        intensity: 0.9,
    };

    let flat_normal = graphics.load_flat_normal();
    let texture = |graphics: &mut Graphics, color| SpriteTexture {
        bind: graphics.bind_textures(color, flat_normal.clone()),
        coord_inf: [0.; 2],
        coord_sup: [1.; 2],
    };

    let ground_color = graphics.load_texture_from_image::<ColorFormat>("resources/slate_color.png");
    let ground_texture = texture(&mut graphics, ground_color);
    let ground_blend_color = Color::white();

    let happy_color = graphics.load_texture_from_image::<ColorFormat>("resources/happy.png");
    let happy_texture = texture(&mut graphics, happy_color);
    let happy_blend_color = Color::from_srgb([185, 125, 25, 255]);

    let unhappy_color = graphics.load_texture_from_image::<ColorFormat>("resources/unhappy.png");
    let unhappy_texture = texture(&mut graphics, unhappy_color);
    let unhappy_blend_color = Color::from_srgb([235, 25, 65, 255]);

    let cloud_color = graphics.load_texture_from_image::<ColorFormat>("resources/cloud.png");
    let cloud_texture = texture(&mut graphics, cloud_color);
    let cloud_blend_color = Color::from_srgb([85, 85, 215, 255]);

    let rng = &mut rand::thread_rng();
    let random_position = |rng: &mut rand::ThreadRng|
        Point2::new(rng.gen_range(0., 20.), rng.gen_range(0., 20.));
    let random_rotation = |rng: &mut rand::ThreadRng|
        Rad(rng.gen_range(0., 2. * ::std::f32::consts::PI));
    let random_size = |rng: &mut rand::ThreadRng| {
        let s = rng.gen_range(0.1, 1.0);
        Vector2::new(s, s)
    };

    let mut grounds = Vec::with_capacity(100);
    for x in 0..10 {
        for y in 0..10 {
            let p = Point2::new(1. + (2. * x as f32), 1. + (2. * y as f32));
            grounds.push(Sprite {
                position: p,
                size: Vector2::new(2., 2.),
                rotation: Rad(0.),
                texture: ground_texture.clone(),
                color: ground_blend_color
            });
        }
    }

    let happies: Vec<_> = (0..500)
        .map(|_| Sprite {
            position: random_position(rng),
            size: random_size(rng),
            rotation: random_rotation(rng),
            texture: happy_texture.clone(),
            color: happy_blend_color
        })
        .collect();

    let unhappies: Vec<_> = (0..500)
        .map(|_| Sprite {
            position: random_position(rng),
            size: random_size(rng),
            rotation: random_rotation(rng),
            texture: unhappy_texture.clone(),
            color: unhappy_blend_color
        })
        .collect();

    let clouds: Vec<_> = (0..50)
        .map(|_| Sprite {
            position: random_position(rng),
            size: random_size(rng) * 2.,
            rotation: Rad(0.),
            texture: cloud_texture.clone(),
            color: cloud_blend_color
        })
        .collect();

    let sprites = [
        (ground_layer, LayerOrder(0), grounds),
        (smiley_layer, LayerOrder(1), happies),
        (smiley_layer, LayerOrder(0), unhappies),
        (sky_layer, LayerOrder(0), clouds),
    ];

    let mut frameclock = FrameClock::start(1.);
    let mut fps_counter = FpsCounter::new(1.);

    'main: loop {
        let delta_time = frameclock.reset();
        if let Some(fps) = fps_counter.update(delta_time) {
            println!("{:.4} ms/frame, {} frame/s", 1000. / fps, fps as usize);
        }

        for event in window.poll_events() {
            match event {
                Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                Event::Closed => break 'main,
                _ => {},
            }
        }

        {
            let access = renderer.access();
            sprites.par_iter()
                .weight_max()
                .for_each(|&(layer, order, ref sprites)| {
                    sprites.par_iter()
                        .weight(5.0)
                        .for_each(|sprite| access.queue(layer).submit(sprite, order));
                });
        }

        let mut frame = graphics.draw();
        frame.clear(clear_color);
        renderer.submit(&camera, &ambient_light, &mut frame);
        frame.present(&window);
    }
}
