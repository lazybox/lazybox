use lazybox::Engine;

pub struct Runner {
    engine: Engine,
}

impl Runner {
    pub fn new() -> Self {
        Runner {
            engine: Engine::new()
        }
    }

    pub fn run(self) {
        self.engine.run()
    }
}