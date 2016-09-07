//! type aliases

use gfx;
use gfx_core;
use gfx_device_gl;

pub type Resources = gfx_device_gl::Resources;
pub type CommandBuffer = gfx_device_gl::CommandBuffer;
pub type Encoder = gfx::Encoder<Resources, CommandBuffer>;
pub type Device = gfx_device_gl::Device;
pub type Factory = gfx_device_gl::Factory;

pub use gfx::format::TextureFormat;
pub type ColorFormat = gfx::format::Srgba8;
pub type NormalFormat = gfx::format::Rgba8;
pub type DepthFormat = (gfx::format::D16, gfx::format::Unorm);
pub type OutputColor = gfx::handle::RenderTargetView<Resources, ColorFormat>;
pub type OutputDepth = gfx::handle::DepthStencilView<Resources, DepthFormat>;

pub type PipelineState<T> = gfx::pso::PipelineState<Resources, T>;
pub type GpuBuffer<T> = gfx::handle::Buffer<Resources, T>;
pub type Texture<T> = gfx::handle::Texture<Resources, T>;
pub type Sampler = gfx::handle::Sampler<Resources>;
pub type ShaderResourceView<T> = gfx::handle::ShaderResourceView<Resources, T>;
pub type TextureView<F: TextureFormat> = ShaderResourceView<F::View>;
pub type RenderTargetView<T> = gfx::handle::RenderTargetView<Resources, T>;
pub type Slice = gfx::Slice<Resources>;

pub type MappingWritable<T> = gfx::mapping::Writable<Resources, T>;
pub type Bundle<T> = gfx::pso::bundle::Bundle<Resources, T>;

pub type GfxRect = gfx_core::target::Rect;