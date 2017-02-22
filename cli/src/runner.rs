use std::path::Path;

use lazybox::Engine;

pub struct Runner {
    engine: Engine,
}

impl Runner {
    pub fn new(config_path: &Path) -> Self {
        Runner { engine: Engine::new(config_path) }
    }

    pub fn run(self) {
        self.engine.run()
    }
}
