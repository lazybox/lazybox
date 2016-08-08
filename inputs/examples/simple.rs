#[macro_use] extern crate lazybox_inputs as inputs;
extern crate lazybox_events as events;
extern crate lazybox_frameclock as frameclock;
extern crate glutin;

use inputs::Inputs;
use events::{EventDispatcher, EventReceiver};
use frameclock::FrameClock;
use glutin::Window;

inputs_interaction! {
    "spaceship" => Action { "Shoot", "LaunchBomb" }
}

pub struct ActionListener {
    receiver: EventReceiver<Action>,
}

impl ActionListener {
    pub fn new(dispatcher: &EventDispatcher) -> Self {
        let receiver = dispatcher.listen_to::<Action>();

        ActionListener {
            receiver: receiver,
        }
    }

    pub fn handle_events(&mut self) {
        self.receiver.handle_with(|action| println!("{:?}", action));
    }
}

fn main() {
    let mut inputs = Inputs::new(build_interaction());
    inputs.load_interaction_profile("interaction/profile.yml");

    let event_dispatcher = EventDispatcher::new();
    let mut action_listener = ActionListener::new(&event_dispatcher);

    let window = Window::new().unwrap();

    let mut frameclock = FrameClock::start(1. / 60.);
    'main: loop {
        frameclock.reset();

        for event in window.poll_events() {
            match event {
                glutin::Event::Closed => break 'main,
                _ => ()
            }

            inputs.handle_event(&event, &event_dispatcher);
        }

        for _ in frameclock.drain_updates() {
            inputs.dispatch_state_actions(&event_dispatcher);
            action_listener.handle_events();
        }
    }
}