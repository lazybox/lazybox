extern crate time;

pub use self::time::*;

pub struct FrameClock {
    mark: f64,
    timestep: f64,
    accumulator: f64,
}

impl FrameClock {
    pub fn start(timestep: f64) -> Self {
        FrameClock {
            mark: precise_time_s(),
            timestep: timestep,
            accumulator: 0.,
        }
    }

    pub fn reset(&mut self) -> f64 {
        let now = precise_time_s();
        let delta = now - self.mark;
        self.mark = now;
        self.accumulator += delta;
        delta
    }

    pub fn drain_updates(&mut self) -> UpdatesDrain {
        UpdatesDrain { clock: self }
    }
}

pub struct UpdatesDrain<'a> {
    clock: &'a mut FrameClock,
}

impl<'a> Iterator for UpdatesDrain<'a> {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        if self.clock.accumulator >= self.clock.timestep {
            self.clock.accumulator -= self.clock.timestep;
            Some(())
        } else {
            None
        }
    }
}

pub struct FpsCounter {
    accumulator: f64,
    frames: f64,
    frequency: f64,
}

impl FpsCounter {
    pub fn new(sample_frequency: f64) -> Self {
        FpsCounter {
            accumulator: 0.,
            frames: 0.,
            frequency: sample_frequency,
        }
    }

    pub fn update(&mut self, delta: f64) -> Option<f64> {
        self.accumulator += delta;
        self.frames += 1.;

        if self.accumulator >= self.frequency {
            let fps = self.frames / self.accumulator;
            self.accumulator = 0.;
            self.frames = 0.;
            Some(fps)
        } else {
            None
        }
    }
}
