use {Graphics, Frame, Camera};
use camera;
use layer::{LayerId, LayerOcclusion};
use lights::*;
use specialized::sprites;
use specialized::dynamic_lights::{self as lights, OcclusionFormat};
use types::*;
use utils::*;

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
            light_sampler: gfx::TextureSampler<[f32; 4]> = "s_Light",
            color_target: gfx::BlendTarget<ColorFormat> =
                ("o_Color", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
        }
    }
}

pub struct Renderer {
    sprites: sprites::Renderer,
    lights: lights::Renderer,
    forward_bundle: Bundle<forward_pipe::Data<Resources>>,
    light_queues: Pool<Vec<Light>>,
}

impl Renderer {
    pub fn new(graphics: &mut Graphics) -> Self {
        use gfx::traits::*;

        let forward_pso = graphics.factory
            .create_pipeline_simple(FORWARD_GLSLV_150, FORWARD_GLSLF_150, forward_pipe::new())
            .expect("could not create forward pipeline");

        let (w, h, _, _) = graphics.output_color.get_dimensions();

        let (_, color_view, color_target) =
            graphics.factory.create_render_target::<ColorFormat>(w, h).unwrap();

        let (_, normal_view, normal_target) =
            graphics.factory.create_render_target::<NormalFormat>(w, h).unwrap();

        let (_, occlusion_view, occlusion_target) =
            graphics.factory.create_render_target::<OcclusionFormat>(w, h).unwrap();

        let (_, light_view, light_target) =
            graphics.factory.create_render_target::<lights::LightFormat>(w, h).unwrap();

        let camera_locals = graphics.factory.create_constant_buffer(1);

        let sprites = sprites::Renderer::new(color_target,
                                             normal_target,
                                             occlusion_target,
                                             camera_locals.clone(),
                                             graphics);

        let lights = lights::Renderer::new(occlusion_view,
                                           normal_view,
                                           light_target,
                                           camera_locals,
                                           graphics);

        let linear_sampler = graphics.factory.create_sampler_linear();

        let forward_bundle = {
            let (vertices, slice) = graphics.factory
                .create_vertex_buffer_with_slice(&QUAD_VERTICES, &QUAD_INDICES[..]);

            let data = forward_pipe::Data {
                vertices: vertices,
                color_sampler: (color_view, linear_sampler.clone()),
                light_sampler: (light_view, linear_sampler.clone()),
                color_target: graphics.output_color.clone(),
            };

            Bundle::new(slice, forward_pso, data)
        };

        Renderer {
            sprites: sprites,
            lights: lights,
            forward_bundle: forward_bundle,
            light_queues: Pool::new(),
        }
    }

    pub fn resize(&mut self, graphics: &mut Graphics) {
        use gfx::traits::*;

        let (w, h, _, _) = graphics.output_color.get_dimensions();

        let (_, color_view, color_target) =
            graphics.factory.create_render_target::<ColorFormat>(w, h).unwrap();

        let (_, normal_view, normal_target) =
            graphics.factory.create_render_target::<NormalFormat>(w, h).unwrap();

        let (_, occlusion_view, occlusion_target) =
            graphics.factory.create_render_target::<OcclusionFormat>(w, h).unwrap();

        let (_, light_view, light_target) =
            graphics.factory.create_render_target::<lights::LightFormat>(w, h).unwrap();

        self.sprites.resize(color_target, normal_target, occlusion_target);
        self.lights.resize(occlusion_view, normal_view, light_target);

        self.forward_bundle.data.color_sampler.0 = color_view;
        self.forward_bundle.data.light_sampler.0 = light_view;
        self.forward_bundle.data.color_target = graphics.output_color.clone();
    }

    pub fn push_layer(&mut self, occlusion: LayerOcclusion) -> LayerId {
       self.sprites.push_layer(occlusion)
    }

    pub fn access(&self) -> Access {
        Access {
            sprites: self.sprites.access(),
            lights: &self.light_queues,
        }
    }

    pub fn submit(&mut self, camera: &Camera, ambient: &AmbientLight, frame: &mut Frame) {
        self.update_camera(camera, &mut frame.graphics);

        {
            let encoder = &mut frame.graphics.encoder;

            encoder.clear(&self.sprites.color_target(), [0.0; 4]);
            encoder.clear(&self.sprites.normal_target(), [0.5, 0.5, 1.0, 0.0]);
            encoder.clear(&self.sprites.occlusion_target(), 0.0);
        }

        self.sprites.submit(frame);
        self.submit_lights(camera, ambient, frame);

        self.forward_bundle.encode(&mut frame.graphics.encoder);
        frame.should_flush();
    }

    fn submit_lights(&mut self, camera: &Camera, ambient: &AmbientLight, frame: &mut Frame) {
        let layer_count = self.sprites.layer_count();
        frame.graphics.encoder.clear(&self.lights.light_target(), [
            ambient.color.r * ambient.intensity,
            ambient.color.g * ambient.intensity,
            ambient.color.b * ambient.intensity,
            0.0
        ]);

        let max_scale = camera.scale.x.max(camera.scale.y);

        let mut render = self.lights.render(max_scale, layer_count, frame);
        for buffer in self.light_queues.availables().iter_mut() {
            for light in buffer.drain(..) {
                render.add(light, frame);
            }
        }

        // TODO: ensure_drawed
        render.ensure_flushed(frame);
    }

    fn update_camera(&mut self, camera: &Camera, graphics: &mut Graphics) {
        let locals = camera::Locals {
            translate: camera.translate.into(),
            scale: camera.scale.into(),
        };
        graphics.encoder.update_constant_buffer(&self.sprites.camera(), &locals);
    }
}

#[derive(Clone, Copy)]
pub struct Access<'a> {
    sprites: sprites::Access<'a>,
    lights: &'a Pool<Vec<Light>>,
}

impl<'a> Access<'a> {
    pub fn queue(&self, id: LayerId) -> sprites::Queue {
        self.sprites.queue(id)
    }

    pub fn light_queue(&self) -> lights::Queue {
        lights::Queue { buffer: self.lights.get() }
    }
}
