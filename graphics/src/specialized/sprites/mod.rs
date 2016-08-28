use gfx;
use rayon::prelude::*;

use {Graphics, Frame, TextureBind};
use sprites::Sprite;
use color::PackedColor;
use layer::{LayerId, LayerOrder, LayerOcclusion, Layers};
use texture;
use camera;
use utils::*;
use types::*;
use specialized::dynamic_lights::OcclusionFormat;

pub const SPRITE_BUFFER_SIZE: usize = 2048;

#[doc(hidden)]
pub const RENDER_GLSLV_150: &'static [u8] = include_bytes!("render_150.glslv");
#[doc(hidden)]
pub const RENDER_GLSLF_150: &'static [u8] = include_bytes!("render_150.glslf");

#[doc(hidden)]
pub use self::defines::{SpriteInstance, LayerLocals, render_pipe};
mod defines {
    pub use types::*;
    pub use utils::*;
    pub use specialized::dynamic_lights::OcclusionFormat;
    pub use camera;

    gfx_defines! {
        vertex SpriteInstance {
            translate: [f32; 2] = "a_Translate",
            rotate: f32 = "a_Rotate",
            scale: [f32; 2] = "a_Scale",
            color: u32 = "a_Color",
            tex_coord_inf: [f32; 2] = "a_TexCoordInf",
            tex_coord_sup: [f32; 2] = "a_TexCoordSup",
        }

        constant LayerLocals {
            occlusion: f32 = "u_LayerOcclusion",
        }

        pipeline render_pipe {
            vertices: gfx::VertexBuffer<Position> = (),
            instances: gfx::InstanceBuffer<SpriteInstance> = (),
            camera: gfx::ConstantBuffer<camera::Locals> = "Camera",
            layer_locals: gfx::ConstantBuffer<LayerLocals> = "LayerLocals",
            color_sampler: gfx::TextureSampler<[f32; 4]> = "s_Color",
            normal_sampler: gfx::TextureSampler<[f32; 4]> = "s_Normal",
            color_target: gfx::BlendTarget<ColorFormat> =
                ("o_Color", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
            normal_target: gfx::RenderTarget<NormalFormat> = "o_Normal",
            occlusion_target: gfx::BlendTarget<OcclusionFormat> =
                ("o_Occlusion", gfx::state::RED, gfx::state::Blend::new(
                    gfx::state::Equation::Max,
                    gfx::state::Factor::One,
                    gfx::state::Factor::One,
                )), 
        }
    }
}

pub struct Renderer {
    bundle: SpriteBundle,
    mapping: MappingWritable<SpriteInstance>,
    layers: Layers<LayerEntities, LayerData>,
}

struct SpriteBundle {
    pso: PipelineState<render_pipe::Meta>,
    vertices: GpuBuffer<Position>,
    instances: GpuBuffer<SpriteInstance>,
    camera: GpuBuffer<camera::Locals>,
    layer_locals: GpuBuffer<LayerLocals>,
    texture_sampler: Sampler,
    color_target: RenderTargetView<ColorFormat>,
    normal_target: RenderTargetView<NormalFormat>,
    occlusion_target: RenderTargetView<OcclusionFormat>,
    slice: Slice,
}

impl SpriteBundle {
    fn encode(&mut self, encoder: &mut Encoder,
                         texture: &texture::Bind,
                         instance_count: u32) {
        let data = render_pipe::Data {
            vertices: self.vertices.clone(),
            instances: self.instances.clone(),
            camera: self.camera.clone(),
            layer_locals: self.layer_locals.clone(),
            color_sampler: (texture.color.clone(), self.texture_sampler.clone()),
            normal_sampler: (texture.normal.clone(), self.texture_sampler.clone()),
            color_target: self.color_target.clone(),
            normal_target: self.normal_target.clone(),
            occlusion_target: self.occlusion_target.clone(),
        };

        self.slice.instances = Some((instance_count, 0));
        encoder.draw(&self.slice, &self.pso, &data);
    }
}

struct LayerEntities {
    sprites: Vec<SpriteData>,
}

impl Default for LayerEntities {
    fn default() -> Self {
        LayerEntities {
            sprites: Vec::new(),
        }
    }
}

struct LayerData {
    sort_keys: Vec<SortKey>,
    occlusion: f32,
}

impl LayerData {
    pub fn new(index: u8, occlusion: LayerOcclusion) -> Self {
        LayerData {
            sort_keys: Vec::new(),
            occlusion: match occlusion {
                LayerOcclusion::Ignore => 0.,
                LayerOcclusion::Stack => (index + 1) as f32,
            },
        }
    }
}

impl Renderer {
    pub fn new(color_target: RenderTargetView<ColorFormat>,
               normal_target: RenderTargetView<NormalFormat>,
               occlusion_target: RenderTargetView<OcclusionFormat>,
               camera_locals: GpuBuffer<camera::Locals>,
               graphics: &mut Graphics) -> Self
    {
        use gfx::traits::*;

        let pso = graphics.factory
            .create_pipeline_simple(RENDER_GLSLV_150, RENDER_GLSLF_150, render_pipe::new())
            .expect("could not create sprite render pipeline");
            
        let layer_locals = graphics.factory.create_constant_buffer(1);
        
        let texture_sampler = graphics.factory.create_sampler(gfx::tex::SamplerInfo::new(
            gfx::tex::FilterMethod::Mipmap,
            gfx::tex::WrapMode::Tile,
        ));

        let (vertices, slice) = graphics.factory
            .create_vertex_buffer_with_slice(&HALF_QUAD_VERTICES, &HALF_QUAD_INDICES[..]);

        let (instances, mapping) = graphics.factory
            .create_buffer_persistent_writable(SPRITE_BUFFER_SIZE,
                                               gfx::BufferRole::Vertex,
                                               gfx::Bind::empty());
        
        let bundle = SpriteBundle {
            pso: pso,
            vertices: vertices,
            instances: instances,
            camera: camera_locals,
            layer_locals: layer_locals,
            texture_sampler: texture_sampler,
            color_target: color_target,
            normal_target: normal_target,
            occlusion_target: occlusion_target,
            slice: slice
        };

        Renderer {
            bundle: bundle,
            mapping: mapping,
            layers: Layers::new(),
        }
    }

    pub fn resize(&mut self,
                  color_target: RenderTargetView<ColorFormat>,
                  normal_target: RenderTargetView<NormalFormat>,
                  occlusion_target: RenderTargetView<OcclusionFormat>)
    {
        self.bundle.color_target = color_target;
        self.bundle.normal_target = normal_target;
        self.bundle.occlusion_target = occlusion_target;
    }

    pub fn color_target(&self) -> &RenderTargetView<ColorFormat> {
        &self.bundle.color_target
    }

    pub fn normal_target(&self) -> &RenderTargetView<NormalFormat> {
        &self.bundle.normal_target
    }

    pub fn occlusion_target(&self) -> &RenderTargetView<OcclusionFormat> {
        &self.bundle.occlusion_target
    }

    pub fn camera(&self) -> &GpuBuffer<camera::Locals> {
        &self.bundle.camera
    }

    pub fn push_layer(&mut self, occlusion: LayerOcclusion) -> LayerId {
        let index = self.layers.count();
        self.layers.push(LayerData::new(index, occlusion))
    }

    pub fn layer_count(&self) -> u8 {
        self.layers.count()
    }

    pub fn queue(&self, id: LayerId) -> Queue {
        Queue { buffer: self.layers.get(id) }
    }
    
    pub fn submit(&mut self, frame: &mut Frame) {
        let &mut Graphics { ref mut encoder,
                            ref mut device,
                            ref texture_binds,
                            .. } = frame.graphics;

        let layer_count = self.layer_count() as f32;
        self.layers.vec.par_iter_mut()
            .weight_max()
            .for_each(|mut layer| {
                let (mut buffers, data) = layer.access();
                Self::preprocess_layer(&mut buffers, data);
            });

        for layer in &mut self.layers.vec {
            let (mut buffers, data) = layer.access();

            let layer_locals = LayerLocals {
                occlusion: data.occlusion / layer_count,
            };
            encoder.update_constant_buffer(&self.bundle.layer_locals, &layer_locals).unwrap();

            let bundle = &mut self.bundle;
            let mut flush = |texture_id, instance_count| {
                let texture = texture_binds.get(texture_id);
                bundle.encode(encoder, texture, instance_count);
                encoder.flush(device);
            };

            let mut current_texture = None;
            let mut i = 0;
            for key in &data.sort_keys {
                let (texture, buffer_index, index) = key.into();

                if let Some(current) = current_texture {
                    if i == SPRITE_BUFFER_SIZE || current != texture {
                        flush(current, i as u32);
                        i = 0;
                    }
                }

                self.mapping.write().set(i, buffers[buffer_index].sprites[index].1);
                current_texture = Some(texture);
                i += 1;
            }

            if let Some(current) = current_texture {
                flush(current, i as u32);
            }

            for entities in buffers.iter_mut() {
                entities.sprites.clear();
            }
        }
    }

    fn preprocess_layer(buffers: &mut [LayerEntities], data: &mut LayerData) {
        use quickersort::sort;

        data.sort_keys.clear();

        for (buffer_index, buffer) in buffers.iter_mut().enumerate() {
            for (index, &(texture, _, order)) in buffer.sprites.iter().enumerate() {
                data.sort_keys.push(SortKey::from(order, texture, buffer_index, index));
            }
        }

        sort(&mut data.sort_keys);
    }
}

type SpriteData = (TextureBind, SpriteInstance, LayerOrder);

pub struct Queue<'a> {
    buffer: Guard<'a, LayerEntities>,
}

impl<'a> Queue<'a> {
    pub fn submit(&mut self, sprite: &Sprite, order: LayerOrder) {
        let instance = SpriteInstance {
            translate: sprite.position.into(),
            scale: sprite.size.into(),
            rotate: sprite.rotation.s,
            color: PackedColor::from(sprite.color).0,
            tex_coord_inf: sprite.texture.coord_inf,
            tex_coord_sup: sprite.texture.coord_sup,
        };

        self.buffer.sprites.push((sprite.texture.bind, instance, order));
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
struct SortKey(u32);
const ORDER_BITS: u32 = 5; // ~> 32 sub layers
const TEXTURE_BITS: u32 = 9; // ~> 512 textures
const BUFFER_BITS: u32 = 4; // ~> 16 threads 
const INDEX_BITS: u32 = 14; // ~> 16_384 sprites per thread
const ORDER_SHIFT: u32 = 32 - ORDER_BITS;
const TEXTURE_SHIFT: u32 = ORDER_SHIFT - TEXTURE_BITS;
const TEXTURE_MASK: u32 = 0xFFFFFFFF >> ORDER_BITS;
const BUFFER_SHIFT: u32 = TEXTURE_SHIFT - BUFFER_BITS;
const BUFFER_MASK: u32 = TEXTURE_MASK >> TEXTURE_BITS;
const INDEX_MASK: u32 = BUFFER_MASK >> BUFFER_BITS;

impl SortKey {
    fn from(order: LayerOrder, texture: TextureBind, buffer: usize, index: usize) -> Self {
        let order = order.0 as u32;
        let texture = texture.0 as u32;
        let buffer = buffer as u32;
        let index = index as u32;
        let order_bits = order << ORDER_SHIFT;
        let texture_bits = (texture << TEXTURE_SHIFT) & TEXTURE_MASK;
        let buffer_bits = (buffer << BUFFER_SHIFT) & BUFFER_MASK;
        let index_bits = index & INDEX_MASK;
        SortKey(order_bits | texture_bits | buffer_bits | index_bits)
    }

    fn into(&self) -> (TextureBind, usize, usize) {
        let _order = self.0 >> ORDER_SHIFT;
        let texture = (self.0 & TEXTURE_MASK) >> TEXTURE_SHIFT;
        let buffer = (self.0 & BUFFER_MASK) >> BUFFER_SHIFT;
        let index = self.0 & INDEX_MASK;
        (TextureBind(texture), buffer as usize, index as usize) 
    }
}
