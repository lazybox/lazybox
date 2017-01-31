#[macro_use] extern crate lazybox_inputs as inputs;
extern crate lazybox_frameclock as frameclock;
extern crate glutin;

use inputs::Inputs;
use frameclock::FrameClock;
use glutin::Window;

inputs_interaction! {
    "spaceship" => ActionIterator, Action { 
        Shoot => "Shoot", 
        LaunchBomb => "LaunchBomb",
    }
}

pub struct ActionListener;

impl ActionListener {
    fn process(&self, inputs: &Inputs) {
        let iterator = ActionIterator::new(inputs).expect("invalid interface");

        for action in iterator {
            match action {
                Action::Shoot => println!("I am shooting !"),
                Action::LaunchBomb => println!("I have launched a bomb."),
            }
        }
    }
}

fn main() {
    let mut inputs = Inputs::new(build_interaction());
    inputs.load_interaction_profile("interaction/profile.yml");

    let mut action_listener = ActionListener;

    let window = Window::new().unwrap();

    let mut frameclock = FrameClock::start(1. / 60.);
    'main: loop {
        frameclock.reset();

        for event in window.poll_events() {
            match event {
                glutin::Event::Closed => break 'main,
                _ => ()
            }

            inputs.handle_event(&event);
        }

        for _ in frameclock.drain_updates() {
            inputs.update_state_actions();
            action_listener.process(&inputs);
            inputs.clear_actions();
        }
    }
}