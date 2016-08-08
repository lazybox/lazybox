use types::*;

pub struct GpuBufferMapping<'a, T: 'static + Copy> {
    buffer: GpuBuffer<T>,
    mapping: Option<MappingWritable<'a, T>>,
    factory: Factory,
}

impl<'a, T: 'static + Copy> GpuBufferMapping<'a, T> {
    pub fn new(buffer: &GpuBuffer<T>, factory: &Factory) -> Self {
        GpuBufferMapping {
            buffer: buffer.clone(),
            mapping: None,
            factory: factory.clone(),
        }
    }

    pub fn ensure_unmapped(&mut self) {
        self.mapping.take();
    }

    pub fn set(&mut self, index: usize, value: T) {
        let mut mapping = self.take_mapping();
        mapping.set(index, value);
        self.mapping = Some(mapping);
    }

    fn take_mapping(&mut self) -> MappingWritable<'a, T> {
        use std::mem;
        use gfx::traits::*;

        self.mapping.take().unwrap_or_else(|| unsafe {
            mem::transmute::<_, MappingWritable<'a, T>>(
                self.factory.map_buffer_writable(&self.buffer))
        })
    }
}