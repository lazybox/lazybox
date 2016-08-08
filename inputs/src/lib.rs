#![feature(pub_restricted)]

pub extern crate lazybox_events as events;
extern crate glutin;
extern crate cgmath;
extern crate yaml_rust;

pub mod state;
pub mod interaction;
#[macro_use] pub mod macros;

pub use state::InputState;
pub use interaction::{Interaction, InteractionBuilder, InterfaceBuilder, ActionEvent};

use events::EventDispatcher;
use glutin::Event;
use cgmath::Point2;
use yaml_rust::YamlLoader;

pub struct Inputs {
    state: InputState,
    interaction: Interaction,
}

impl Inputs {
    pub fn new(interaction: InteractionBuilder) -> Self {
        Inputs {
            state: InputState::new(),
            interaction: interaction.build(),
        }
    }

    pub fn state(&self) -> &InputState {
        &self.state
    }

    pub fn interaction_mut(&mut self) -> &mut Interaction {
        &mut self.interaction
    }

    pub fn load_interaction_profile(&mut self, path: &str) {
        use std::fs::File;
        use std::io::prelude::*;

        let mut f = File::open(path).unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();

        let docs = YamlLoader::load_from_str(&s).unwrap();
        self.interaction.load_profile(&docs[0]);
    }

    pub fn handle_event(&mut self, event: &Event, dispatcher: &EventDispatcher) {
        let &mut Inputs { ref mut state, ref interaction } = self;

        match event {
            &Event::KeyboardInput(e_state, _, Some(key)) => {
                state.update_key(key, e_state);

                let input = interaction::Input::Key(e_state, key);
                interaction.dispatch_input_actions(&input, state, dispatcher);
            }
            &Event::MouseMoved(x, y) => {
                state.update_mouse_position(Point2::new(x, y));
            }
            &Event::MouseInput(e_state, button) => {
                state.update_mouse_button(button, e_state);

                let input = interaction::Input::MouseButton(e_state, button);
                interaction.dispatch_input_actions(&input, state, dispatcher);
            }
            &Event::Focused(focused) => {
                state.update_window_focus(focused);
            }
            _ => {}
        }
    }

    pub fn dispatch_state_actions(&mut self, dispatcher: &EventDispatcher) {
        let &mut Inputs { ref mut state, ref interaction } = self;
        
        interaction.dispatch_state_actions(state, dispatcher);
    }
}