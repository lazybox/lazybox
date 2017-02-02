#![feature(pub_restricted)]

extern crate glutin;
extern crate cgmath;
extern crate yaml_rust;
#[macro_use]
extern crate error_chain;

pub mod error;
pub mod state;
pub mod interaction;
#[macro_use]
pub mod macros;

pub use error::Error;
pub use state::InputState;
pub use interaction::{Interaction, InteractionBuilder, InterfaceBuilder, Action};
use interaction::Interface;
use error::Result;

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

    pub fn triggered_actions(&self, interface_name: &str) -> Option<&[Action]> {
        self.interaction
            .interface(interface_name)
            .map(Interface::triggered_actions)
    }

    pub fn load_interaction_profile(&mut self, path: &str) -> Result<()> {
        use std::fs::File;
        use std::io::prelude::*;

        let mut f = File::open(path)?;
        let mut s = String::new();
        f.read_to_string(&mut s)?;

        let docs = YamlLoader::load_from_str(&s)?;

        self.interaction.load_profile(&docs[0]).map_err(Error::from)
    }

    pub fn handle_event(&mut self, event: &Event) {
        let &mut Inputs { ref mut state, ref mut interaction } = self;

        match event {
            &Event::KeyboardInput(e_state, _, Some(key)) => {
                state.update_key(key, e_state);

                let input = interaction::Input::Key(e_state, key);
                interaction.trigger_input_actions(&input, state);
            }
            &Event::MouseMoved(x, y) => {
                state.update_mouse_position(Point2::new(x, y));
            }
            &Event::MouseInput(e_state, button) => {
                state.update_mouse_button(button, e_state);

                let input = interaction::Input::MouseButton(e_state, button);
                interaction.trigger_input_actions(&input, state);
            }
            &Event::Focused(focused) => {
                state.update_window_focus(focused);
            }
            _ => {}
        }
    }

    pub fn trigger_state_actions(&mut self) {
        let &mut Inputs { ref mut state, ref mut interaction } = self;

        interaction.trigger_state_actions(state);
    }

    pub fn clear_actions(&mut self) {
        self.interaction.clear_actions();
    }
}
