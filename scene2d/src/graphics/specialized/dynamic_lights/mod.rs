use gfx;

use {Graphics, Frame};
use graphics::types::*;
use graphics::utils::*;
use graphics::camera;
use graphics::lights::*;

pub const CATEGORY_COUNT: usize = 2;
pub const SHADOW_MAP_SIZES: [u16; CATEGORY_COUNT] = [64, 256];
pub const SHADOW_MAP_COUNTS: [u16; CATEGORY_COUNT] = [256, 32];

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
pub const MAP_POSITION_VERTICES: [MapPosition; 2] = [
    MapPosition { pos: -1. },
    MapPosition { pos:  1. },
];

pub type OcclusionFormat = (gfx::format::R16, gfx::format::Unorm);
pub type LightFormat = (gfx::format::R16_G16_B16_A16, gfx::format::Float);

#[doc(hidden)]
pub use self::defines::{MapPosition, LightInstance, shadow_map_pipe, render_pipe};
mod defines {
    use gfx;
    pub use super::*;
    pub use graphics::types::*;
    pub use graphics::utils::*;
    pub use graphics::camera;

    gfx_defines! {
        vertex MapPosition {
            pos: f32 = "a_MapPosition",
        }

        constant LightInstance {
            color_intensity: [f32; 4] = "color_intensity",
            center: [f32; 2] = "center",
            radius: f32 = "radius",
            source_radius: f32 = "source_radius",
            occlusion_threshold: f32 = "occlusion_threshold",
            padding_1: f32 = "padding_1",
            padding_2: [f32; 2] = "padding_2",
        }

        pipeline shadow_map_pipe {
            vertices: gfx::VertexBuffer<MapPosition> = (),
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
    lights: [GpuBuffer<LightInstance>; CATEGORY_COUNT],
    uploads: [GpuBuffer<LightInstance>; CATEGORY_COUNT],
    shadow_maps_views: [ShaderResourceView<f32>; CATEGORY_COUNT],
    shadow_maps_targets: [RenderTargetView<f32>; CATEGORY_COUNT],
    shadow_map_bundle: Bundle<shadow_map_pipe::Data<Resources>>,
    render_bundle: Bundle<render_pipe::Data<Resources>>,
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

        let (small_lights, small_upload) =
            create_constant_upload_pair(&mut graphics.factory, SHADOW_MAP_COUNTS[0] as usize);

        let (_, small_shadow_maps_view, small_shadow_maps_target) =
            Self::create_shadow_maps(SHADOW_MAP_SIZES[0], SHADOW_MAP_COUNTS[0], &mut graphics.factory);

        let (big_lights, big_upload) =
            create_constant_upload_pair(&mut graphics.factory, SHADOW_MAP_COUNTS[1] as usize);

        let (_, big_shadow_maps_view, big_shadow_maps_target) =
            Self::create_shadow_maps(SHADOW_MAP_SIZES[1], SHADOW_MAP_COUNTS[1], &mut graphics.factory);

        let linear_sampler = graphics.factory.create_sampler_linear();

        let shadow_map_bundle = {
            let (vertices, slice) = graphics.factory
                .create_vertex_buffer_with_slice(&MAP_POSITION_VERTICES, gfx::IndexBuffer::Auto);

            let data = shadow_map_pipe::Data {
                vertices: vertices,
                camera: camera_locals.clone(),
                lights: small_lights.clone(),
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
                lights: small_lights.clone(),
                shadow_map_sampler: (small_shadow_maps_view.clone(), linear_sampler.clone()),
                normal_sampler: (normal_view, linear_sampler),
                light_target: light_target,
            };

            Bundle::new(slice, render_pso, data)
        };

        Renderer {
            lights: [small_lights, big_lights],
            uploads: [small_upload, big_upload],
            shadow_maps_views: [small_shadow_maps_view, big_shadow_maps_view],
            shadow_maps_targets: [small_shadow_maps_target, big_shadow_maps_target],
            shadow_map_bundle: shadow_map_bundle,
            render_bundle: render_bundle,
        }
    }

    fn create_shadow_maps(size: u16, count: u16, factory: &mut Factory)
                          -> (Texture<gfx::format::R32>, TextureView<f32>, RenderTargetView<f32>) {
        use gfx::traits::*;
        use gfx::{texture, memory, format};

        let kind = texture::Kind::D1Array(size, count);
        let bind = gfx::SHADER_RESOURCE | gfx::RENDER_TARGET;
        let usage = memory::Usage::Data;
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

    pub fn render(&mut self, layer_count: u8, _: &mut Frame) -> Render {
        Render {
            lights: &self.lights,
            writers: [MappingWriter::new(&self.uploads[0]),
                      MappingWriter::new(&self.uploads[1])],
            shadow_maps_views: &self.shadow_maps_views,
            shadow_maps_targets: &self.shadow_maps_targets,
            shadow_map_bundle: &mut self.shadow_map_bundle,
            render_bundle: &mut self.render_bundle,

            offsets: [0; CATEGORY_COUNT],
            layer_count: layer_count as f32,
        }
    }
}

pub struct Render<'a> {
    lights: &'a [GpuBuffer<LightInstance>; CATEGORY_COUNT],
    writers: [MappingWriter<'a, LightInstance>; CATEGORY_COUNT],
    shadow_maps_views: &'a [ShaderResourceView<f32>; CATEGORY_COUNT],
    shadow_maps_targets: &'a [RenderTargetView<f32>; CATEGORY_COUNT],
    shadow_map_bundle: &'a mut Bundle<shadow_map_pipe::Data<Resources>>,
    render_bundle: &'a mut Bundle<render_pipe::Data<Resources>>,

    offsets: [usize; CATEGORY_COUNT],
    layer_count: f32,
}

impl<'a> Render<'a> {
    pub fn add_small(&mut self, light: Light, frame: &mut Frame) {
        self.add(0, light, frame);
    }

    pub fn add_big(&mut self, light: Light, frame: &mut Frame) {
        self.add(1, light, frame);
    }

    pub fn add(&mut self, category: usize, light: Light, frame: &mut Frame) {
        let c = category;

        if self.offsets[c] == SHADOW_MAP_COUNTS[c] as usize {
            self.before_category_flush(c, frame);
            frame.flush();
        }

        let mut writer = self.writers[c].acquire(&mut frame.graphics.factory);
        writer[self.offsets[c]] = LightInstance {
            color_intensity: [
                light.color.r,
                light.color.g,
                light.color.b,
                light.intensity
            ],
            center: [light.center.x, light.center.y],
            radius: light.radius,
            source_radius: light.source_radius,
            occlusion_threshold: ((light.source_layer.0) as f32 + 0.5) / self.layer_count,
            padding_1: 0.,
            padding_2: [0.; 2],
        };
        self.offsets[c] += 1;
    }

    pub fn before_flush(&mut self, frame: &mut Frame) {
        for c in 0..CATEGORY_COUNT {
            if self.offsets[c] > 0 {
                self.before_category_flush(c, frame);
            }
        }
    }

    fn before_category_flush(&mut self, category: usize, frame: &mut Frame) {
        let c = category;
        let &mut Graphics { ref mut encoder, .. } = frame.graphics;

        let instances = Some((self.offsets[c] as u32, 0));
        self.shadow_map_bundle.slice.instances = instances;
        self.shadow_map_bundle.data.lights = self.lights[c].clone();
        self.shadow_map_bundle.data.shadow_target = self.shadow_maps_targets[c].clone();
        self.render_bundle.slice.instances = instances;
        self.render_bundle.data.lights = self.lights[c].clone();
        self.render_bundle.data.shadow_map_sampler.0 = self.shadow_maps_views[c].clone();

        encoder.copy_buffer(self.writers[c].buffer(), &self.lights[c],
                            0, 0, self.offsets[c]).unwrap();
        self.shadow_map_bundle.encode(encoder);
        self.render_bundle.encode(encoder);
        self.offsets[c] = 0;
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
