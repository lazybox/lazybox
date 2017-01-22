pub mod pool;
pub mod common;
pub mod mapping;

pub use self::pool::*;
pub use self::common::*;
pub use self::mapping::*;

use gfx;
use types::*;

pub fn create_vertex_upload_pair<T: Copy>(factory: &mut Factory, size: usize)
                                          -> (GpuBuffer<T>, GpuBuffer<T>) {
    use gfx::traits::*;
    let vertex = factory.create_buffer(size,
                                       gfx::buffer::Role::Vertex,
                                       gfx::memory::Usage::Data,
                                       gfx::TRANSFER_DST).unwrap();
    let upload = factory.create_upload_buffer(size).unwrap();
    return (vertex, upload);
}

pub fn create_constant_upload_pair<T: Copy>(factory: &mut Factory, size: usize)
                                            -> (GpuBuffer<T>, GpuBuffer<T>) {
    use gfx::traits::*;
    let constant = factory.create_buffer(size,
                                         gfx::buffer::Role::Constant,
                                         gfx::memory::Usage::Data,
                                         gfx::TRANSFER_DST).unwrap();
    let upload = factory.create_upload_buffer(size).unwrap();
    return (constant, upload);
}
