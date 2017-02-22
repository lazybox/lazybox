use std::path::Path;
use context::{Context, modules, processors};
use core::{State, Commit, Settings};
use core::processor::Scheduler;
use core::time::{FrameClock, FpsCounter};
use game::*;

pub struct Engine {
    mcx: modules::Context,
    state: State<Context>,
    processors: Scheduler<Context>,
    frameclock: FrameClock,
    fps_counter: FpsCounter,
}

impl Engine {
    pub fn new<G: Game>(game: G) -> Self {
        let settings = Self::create_settings(game.config_path());
        let context = Self::create_context(&settings);

        let (frameclock, fps_counter) = Self::create_frameclock(&settings);

        let mut state_builder = StateBuilder::new();
        Self::create_data_module(&game, &mut state_builder);
        game.modules(&mut state_builder);
        game.interfaces(&mut state_builder);


        let mut scheduler_builder = SchedulerBuilder::new();
        game.processes(&mut scheduler_builder);

        Engine {
            mcx: context,
            state: state_builder.build(),
            processors: scheduler_builder.build(),
            frameclock: frameclock,
            fps_counter: fps_counter,
        }
    }

    fn create_data_module<G: Game>(game: &G, state: &mut StateBuilder) {
        let mut builder = DataModuleBuilder::new(state);
        game.data_components(&mut builder);

        builder.build();
    }

    fn create_settings(settings_path: &Path) -> Settings {
        let mut settings = Settings::new_with_string(include_str!("config/default2d.yml")).unwrap();
        settings.override_with(settings_path).unwrap();

        settings
    }

    fn create_context(settings: &Settings) -> modules::Context {
        modules::Context {}
    }

    pub fn run(mut self) {
        'main: loop {
            let dt = self.frameclock.reset();
            if let Some(fps) = self.fps_counter.update(dt) {
                println!("{:.4} ms/frame, {} frame/s", 1000. / fps, fps as usize);
            }

            for _ in self.frameclock.drain_updates() {
                self.processors
                    .fixed_update(&mut self.state, &mut self.mcx, &processors::Context {});
            }


        }
    }

    fn create_frameclock(settings: &Settings) -> (FrameClock, FpsCounter) {
        let time = &settings["time"];

        let update_frequency =
            time["update_frequency"].as_i64().expect("update_frequency should be in Hz");
        let fps_frequency =
            time["fps_frequency"].as_f64().expect("fps frequency should be a float");

        let time_step = 1. / update_frequency as f64;

        (FrameClock::start(time_step), FpsCounter::new(1. / fps_frequency))
    }

    pub fn commit<F: FnOnce(&State<Context>, Commit<Context>)>(&mut self, f: F) {
        let mut update = self.state.update();
        update.commit(&mut self.mcx, f);
    }
}
