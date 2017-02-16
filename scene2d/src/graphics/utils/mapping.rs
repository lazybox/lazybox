use gfx;

use graphics::types::*;

pub struct MappingWriter<'a, T: 'a + Copy> {
    buffer: &'a gfx::handle::Buffer<Resources, T>,
    writer: Option<gfx::mapping::Writer<'a, Resources, T>>,
}

impl<'a, T: 'a + Copy> MappingWriter<'a, T> {
    pub fn new(buffer: &'a gfx::handle::Buffer<Resources, T>) -> Self {
        MappingWriter {
            buffer: buffer,
            writer: None,
        }
    }

    pub fn buffer(&self) -> &'a gfx::handle::Buffer<Resources, T> {
        self.buffer
    }

    pub fn acquire(&mut self, factory: &mut Factory)
                   -> &mut gfx::mapping::Writer<'a, Resources, T> {
        use gfx::Factory;
        if self.writer.is_none() {
            self.writer = Some(factory.write_mapping(self.buffer).unwrap());
        }
        self.writer.as_mut().unwrap()
    }

    pub fn release(&mut self) {
        self.writer.take();
    }
}
