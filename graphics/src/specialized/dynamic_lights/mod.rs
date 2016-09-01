use gfx;

use {Graphics, Frame};
use lights::*;
use camera;
use utils::*;
use types::*;

pub const LIGHT_BUFFER_SIZE: usize = 256;
pub const SMALL_SHADOW_MAP_SIZE: u16 = 64;
pub const SMALL_SHADOW_MAP_COUNT: u16 = 256;
pub const BIG_SHADOW_MAP_SIZE: u16 = 256;
pub const BIG_SHADOW_MAP_COUNT: u16 = 32;

#[doc(hidden)]
pub const SHADOW_MAP_GLSLV_150: &'static [u8] = include_bytes!("shadow_map_150.glslv");
#[doc(hidden)]
pub const SHADOW_MAP_GLSLG_150: &'static [u8] = include_bytes!("shadow_map_150.glslg");
#[doc(hidden)]
pub const SHADOW_MAP_GLSLF_150: &'static [u8] = include_bytes!("shadow_map_150.glslf");
#[doc(hidden)]
pub const RENDER_GLSLV_150: &'static [u8] = include_bytes!("render_150.glslv");
#[doc(hidden)]
pub const RENDER_GLSLF_150: &'static [u8] = include_bytes!("render_150.glslf");

#[doc(hidden)]
pub const RATIO_VERTICES: [Ratio; 2] = [
    Ratio { ratio: 0. },
    Ratio { ratio: 1. },
];

pub type OcclusionFormat = (gfx::format::R16, gfx::format::Unorm);
pub type LightFormat = (gfx::format::R16_G16_B16_A16, gfx::format::Float);

#[doc(hidden)]
pub use self::defines::{Ratio, LightInstance, shadow_map_pipe, render_pipe};
mod defines {
    pub use super::*;
    pub use types::*;
    pub use utils::*;
    pub use camera;

    gfx_defines! {
        vertex Ratio {
            ratio: f32 = "a_Ratio",
        }

        constant LightInstance {
            color_intensity: [f32; 4] = "color_intensity",
            center: [f32; 2] = "center",
            radius: f32 = "radius",
            source_radius: f32 = "source_radius",
            occlusion_threshold: f32 = "occlusion_threshold",
            shadow_map_index: u32 = "shadow_map_index",
            padding: [f32; 2] = "padding",
        }

        pipeline shadow_map_pipe {
            vertices: gfx::VertexBuffer<Ratio> = (),
            camera: gfx::ConstantBuffer<camera::Locals> = "Camera",
            lights: gfx::ConstantBuffer<LightInstance> = "Lights",
            occlusion_sampler: gfx::TextureSampler<f32> = "s_Occlusion",
            shadow_target: gfx::RenderTarget<f32> = "o_ShadowMap",
        }

        pipeline render_pipe {
            vertices: gfx::VertexBuffer<Position> = (),
            camera: gfx::ConstantBuffer<camera::Locals> = "Camera",
            lights: gfx::ConstantBuffer<LightInstance> = "Lights",
            shadow_map_sampler: gfx::TextureSampler<f32> = "s_ShadowMap",
            normal_sampler: gfx::TextureSampler<[f32; 4]> = "s_Normal",
            light_target: gfx::BlendTarget<LightFormat> =
                ("o_Light", gfx::state::MASK_ALL, gfx::preset::blend::ADD),
        }
    }
}

pub struct Renderer {
    small_shadow_maps_view: ShaderResourceView<f32>,
    small_shadow_maps_target: RenderTargetView<f32>,
    big_shadow_maps_view: ShaderResourceView<f32>,
    big_shadow_maps_target: RenderTargetView<f32>,
    shadow_map_bundle: Bundle<shadow_map_pipe::Data<Resources>>,
    render_bundle: Bundle<render_pipe::Data<Resources>>,
    mapping: MappingWritable<LightInstance>,
}

impl Renderer {
    pub fn new(occlusion_view: ShaderResourceView<f32>,
               normal_view: ShaderResourceView<[f32; 4]>,
               light_target: RenderTargetView<LightFormat>,
               camera_locals: GpuBuffer<camera::Locals>,
               graphics: &mut Graphics) -> Self {
        use gfx::traits::*;

        let shadow_map_pso = {
            let vertex = graphics.factory.create_shader_vertex(SHADOW_MAP_GLSLV_150)
                .unwrap_or_else(|e| panic!("could not create vertex shader: {}", e));
            let geometry = graphics.factory.create_shader_geometry(SHADOW_MAP_GLSLG_150)
                .unwrap_or_else(|e| panic!("could not create geometry shader: {}", e));
            let pixel = graphics.factory.create_shader_pixel(SHADOW_MAP_GLSLF_150)
                .unwrap_or_else(|e| panic!("could not create pixel shader: {}", e));
            let program = graphics.factory
                .create_program(&gfx::ShaderSet::Geometry(vertex, geometry, pixel))
                .expect("could not link shadow map program");
            graphics.factory.create_pipeline_from_program(&program,
                                                          gfx::Primitive::LineList,
                                                          gfx::state::Rasterizer::new_fill(),
                                                          shadow_map_pipe::new())
                .expect("could not create shadow map pipeline")
        };

        let render_pso = graphics.factory
            .create_pipeline_simple(RENDER_GLSLV_150,
                                    RENDER_GLSLF_150,
                                    render_pipe::new())
            .expect("could not create light render pipeline");

        let (_, small_shadow_maps_view, small_shadow_maps_target) =
            Self::create_shadow_maps(SMALL_SHADOW_MAP_SIZE, SMALL_SHADOW_MAP_COUNT, &mut graphics.factory);

        let (_, big_shadow_maps_view, big_shadow_maps_target) =
            Self::create_shadow_maps(BIG_SHADOW_MAP_SIZE, BIG_SHADOW_MAP_COUNT, &mut graphics.factory);

        let (lights, mapping) = graphics.factory
            .create_buffer_persistent_writable(LIGHT_BUFFER_SIZE,
                                               gfx::buffer::Role::Constant,
                                               gfx::Bind::empty());

        let linear_sampler = graphics.factory.create_sampler_linear();

        let shadow_map_bundle = {
            let (vertices, slice) = graphics.factory
                .create_vertex_buffer_with_slice(&RATIO_VERTICES, gfx::IndexBuffer::Auto);

            let data = shadow_map_pipe::Data {
                vertices: vertices,
                camera: camera_locals.clone(),
                lights: lights.clone(),
                occlusion_sampler: (occlusion_view, linear_sampler.clone()),
                shadow_target: small_shadow_maps_target.clone(),
            };

            Bundle::new(slice, shadow_map_pso, data)
        };

        let render_bundle = {
            let (vertices, slice) = graphics.factory
                .create_vertex_buffer_with_slice(&QUAD_VERTICES, &QUAD_INDICES[..]);

            let data = render_pipe::Data {
                vertices: vertices,
                camera: camera_locals,
                lights: lights,
                shadow_map_sampler: (small_shadow_maps_view.clone(), linear_sampler.clone()),
                normal_sampler: (normal_view, linear_sampler),
                light_target: light_target,
            };

            Bundle::new(slice, render_pso, data)
        };

        Renderer {
            small_shadow_maps_view: small_shadow_maps_view,
            small_shadow_maps_target: small_shadow_maps_target,
            big_shadow_maps_view: big_shadow_maps_view,
            big_shadow_maps_target: big_shadow_maps_target,
            shadow_map_bundle: shadow_map_bundle,
            render_bundle: render_bundle,
            mapping: mapping,
        }
    }

    fn create_shadow_maps(size: u16, count: u16, factory: &mut Factory)
                          -> (Texture<gfx::format::R32>, TextureView<f32>, RenderTargetView<f32>) {
        use gfx::traits::*;
        use gfx::{texture, memory, format};

        let kind = texture::Kind::D2Array(size, 1, count, texture::AaMode::Single);
        let bind = gfx::SHADER_RESOURCE | gfx::RENDER_TARGET;
        let usage = memory::Usage::GpuOnly;
        let channel = format::ChannelType::Float;
        let texture = factory.create_texture(kind, 1, bind, usage, Some(channel)).unwrap();
        let swizzle = format::Swizzle::new();
        let view = factory.view_texture_as_shader_resource::<f32>(&texture, (0, 0), swizzle).unwrap();
        let target = factory.view_texture_as_render_target(&texture, 0, None).unwrap();
        (texture, view, target)
    }

    pub fn resize(&mut self,
                  occlusion_view: ShaderResourceView<f32>,
                  normal_view: ShaderResourceView<[f32; 4]>,
                  light_target: RenderTargetView<LightFormat>)
    {
        self.shadow_map_bundle.data.occlusion_sampler.0 = occlusion_view;
        self.render_bundle.data.normal_sampler.0 = normal_view;
        self.render_bundle.data.light_target = light_target;
    }

    pub fn light_target(&self) -> &RenderTargetView<LightFormat> {
        &self.render_bundle.data.light_target
    }

    pub fn render(&mut self,
                  radius_factor: f32,
                  layer_count: u8,
                  _: &mut Frame) -> Render {
        Render {
            renderer: self,
            offset: 0,
            shadow_map_index: 0,
            radius_factor: radius_factor,
            layer_count: layer_count as f32,
        }
    }
}

pub struct Render<'a> {
    renderer: &'a mut Renderer,
    offset: usize,
    shadow_map_index: u32,
    radius_factor: f32,
    layer_count: f32,
}

impl<'a> Render<'a> {
    pub fn add(&mut self, light: Light, frame: &mut Frame) {
        // TODO: lights category
        if self.offset == LIGHT_BUFFER_SIZE {
            self.flush(frame);
        }

        // TODO: call `write()` less often :/
        self.renderer.mapping.write().set(self.offset, LightInstance {
            color_intensity: [
                light.color.r,
                light.color.g,
                light.color.b,
                light.intensity
            ],
            center: light.center.into(),
            radius: light.radius,
            source_radius: light.source_radius,
            occlusion_threshold: ((light.source_layer.0) as f32 + 0.5) / self.layer_count,
            shadow_map_index: self.shadow_map_index,
            padding: [0.; 2],
        });
        self.offset += 1;
        self.shadow_map_index += 1;
    }

    pub fn ensure_flushed(&mut self, frame: &mut Frame) {
        if self.offset > 0 {
            self.flush(frame);
        }
    }

    fn flush(&mut self, frame: &mut Frame) {
        let &mut Graphics { ref mut encoder, ref mut device, .. } = frame.graphics;

        let instances = Some((self.offset as u32, 0));
        self.renderer.shadow_map_bundle.slice.instances = instances;
        self.renderer.render_bundle.slice.instances = instances;

        self.renderer.shadow_map_bundle.encode(encoder);
        self.renderer.render_bundle.encode(encoder);
        encoder.flush(device);

        self.offset = 0;
        self.shadow_map_index = 0;
    }
}

pub struct Queue<'a> {
    #[doc(hidden)] pub buffer: Guard<'a, Vec<Light>>,
}

impl<'a> Queue<'a> {
    pub fn submit(&mut self, light: Light) {
        self.buffer.push(light);
    }
}
