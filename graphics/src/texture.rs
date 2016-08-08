use types::*;

#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct TextureBind(pub(crate) u32);

impl TextureBind {
    fn new(index: usize) -> Self { TextureBind(index as u32) }
    fn index(self) -> usize { self.0 as usize }
}

pub(crate) struct TextureBinds {
    binds: Vec<Option<Bind>>,
    available: Vec<usize>,
}

pub(crate) struct Bind {
    pub color: TextureView<ColorFormat>,
    pub normal: TextureView<NormalFormat>,
}

impl TextureBinds {
    pub fn new() -> Self {
        TextureBinds {
            binds: Vec::new(),
            available: Vec::new(),
        }
    }

    pub fn insert(&mut self, bind: Bind) -> TextureBind {
        match self.available.pop() {
            Some(index) => {
                self.binds[index] = Some(bind);
                TextureBind::new(index)
            }
            None => {
                let index = self.binds.len();
                self.binds.push(Some(bind));
                TextureBind::new(index)
            }
        }
    }

    pub fn remove(&mut self, bind: TextureBind) -> Bind {
        let index = bind.index();
        let texture = self.binds[index].take().expect("invalid TextureBind");
        self.available.push(index);
        texture
    }

    pub fn get(&self, bind: TextureBind) -> &Bind {
        self.binds[bind.index()].as_ref().expect("invalid TextureBind")
    }

    pub fn get_mut(&mut self, bind: TextureBind) -> &mut Bind {
        self.binds[bind.index()].as_mut().expect("invalid TextureBind")
    }
}