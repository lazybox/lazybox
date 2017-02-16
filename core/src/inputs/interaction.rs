use std::collections::{HashMap, HashSet};
use winit::{VirtualKeyCode, MouseButton, ElementState};
use yaml_rust::Yaml;
use inputs::state::InputState;
use inputs::error::{ErrorKind, Result};

#[derive(Clone, Eq, PartialEq, Hash)]
#[doc(hidden)]
pub enum Input {
    Key(ElementState, VirtualKeyCode),
    MouseButton(ElementState, MouseButton),
}

#[doc(hidden)]
pub enum Condition {
    KeyHeld(VirtualKeyCode),
    MouseButtonHeld(MouseButton),
}

impl Condition {
    pub fn evaluate(&self, state: &InputState) -> bool {
        match self {
            &Condition::KeyHeld(ref key) => state.is_key_held(key),
            &Condition::MouseButtonHeld(ref button) => state.is_mouse_button_held(button),
        }
    }
}

pub type Action = &'static str;

#[doc(hidden)]
pub struct ConditionalAction {
    action: Action,
    condition: Condition,
}

impl ConditionalAction {
    pub fn may_trigger(&self, state: &InputState) -> Option<Action> {
        if self.condition.evaluate(state) {
            Some(self.action)
        } else {
            None
        }
    }
}

#[doc(hidden)]
pub struct Rules {
    by_input: HashMap<Input, Action>,
    others: Vec<ConditionalAction>,
}

impl Rules {
    pub fn new() -> Self {
        Rules {
            by_input: HashMap::new(),
            others: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.by_input.clear();
        self.others.clear();
    }
}

pub struct InteractionBuilder {
    interfaces: HashMap<&'static str, Interface>,
}

impl InteractionBuilder {
    pub fn new() -> Self {
        InteractionBuilder { interfaces: HashMap::new() }
    }

    pub fn interface(mut self, name: &'static str, builder: InterfaceBuilder) -> Self {
        self.interfaces.insert(name, builder.build());
        self
    }

    #[doc(hidden)]
    pub fn build(self) -> Interaction {
        Interaction { interfaces: self.interfaces }
    }
}

pub struct InterfaceBuilder {
    actions: HashSet<Action>,
}

impl InterfaceBuilder {
    pub fn new() -> Self {
        InterfaceBuilder { actions: HashSet::new() }
    }

    pub fn action(mut self, a: Action) -> Self {
        self.actions.insert(a);
        self
    }

    fn build(self) -> Interface {
        Interface {
            actions: self.actions,
            rules: Rules::new(),
            triggered_actions: Vec::new(),
        }
    }
}

pub struct Interaction {
    interfaces: HashMap<&'static str, Interface>,
}

impl Interaction {
    #[must_use]
    pub fn load_profile(&mut self, profile: &Yaml) -> Result<()> {
        for (&name, interface) in &mut self.interfaces {
            if let Some(rules) = profile[name]["rules"].as_vec() {
                interface.load_rules(rules)?;
            } else {
                bail!(ErrorKind::RulesFormat)
            }
        }

        Ok(())
    }

    pub fn interface(&self, name: &str) -> Option<&Interface> {
        self.interfaces.get(name)
    }

    pub fn trigger_input_actions(&mut self, input: &Input, state: &InputState) {
        // TODO: enable/disable interface dispatch
        for (_, interface) in &mut self.interfaces {
            interface.trigger_input_actions(input, state);
        }
    }

    pub fn trigger_state_actions(&mut self, state: &InputState) {
        // TODO: enable/disable interface dispatch
        for (_, interface) in &mut self.interfaces {
            interface.trigger_state_actions(state);
        }
    }

    pub fn clear_actions(&mut self) {
        for (_, interface) in &mut self.interfaces {
            interface.clear_actions();
        }
    }
}

#[doc(hidden)]
pub struct Interface {
    actions: HashSet<Action>,
    rules: Rules,
    triggered_actions: Vec<Action>,
}

impl Interface {
    fn load_rules(&mut self, rules: &[Yaml]) -> Result<()> {
        self.rules.clear();
        for rule in rules {
            let action = match rule["action"].as_str() {
                Some(action) => action,
                None => bail!(ErrorKind::InterfaceFormat),
            };

            let when = match rule["when"].as_str() {
                Some(when) => when,
                None => bail!(ErrorKind::InterfaceFormat),
            };


            if let Some(action) = self.actions.get(action) {
                use self::WhenParse::*;

                match WhenParse::from_str(when) {
                    Some(Input(input)) => {
                        self.rules.by_input.insert(input, action);
                    }
                    Some(Condition(condition)) => {
                        self.rules.others.push(ConditionalAction {
                            action: action,
                            condition: condition,
                        });
                    }
                    None => bail!(ErrorKind::ConditionFormat),
                }
            } else {
                bail!(ErrorKind::UnknownInterface);
            }
        }

        Ok(())
    }

    fn trigger_input_actions(&mut self, input: &Input, _state: &InputState) {
        if let Some(action) = self.rules.by_input.get(input) {
            self.triggered_actions.push(action);
        }
    }

    fn trigger_state_actions(&mut self, state: &InputState) {
        for ca in &self.rules.others {
            if let Some(action) = ca.may_trigger(state) {
                self.triggered_actions.push(action);
            }
        }
    }

    pub fn triggered_actions(&self) -> &[Action] {
        &self.triggered_actions
    }

    fn clear_actions(&mut self) {
        self.triggered_actions.clear();
    }
}

enum WhenParse {
    Input(Input),
    Condition(Condition),
}

impl WhenParse {
    pub fn from_str(s: &str) -> Option<Self> {
        use self::WhenParse::*;
        use self::Input::*;
        use self::Condition::*;
        use winit::ElementState::*;

        let mut split = s.split('.');
        match split.next() {
            Some("Key") => {
                split.next().and_then(|state| {
                    split.next().and_then(|k| key_from_str(k)).and_then(|key| match state {
                        "Pressed" => Some(Input(Key(Pressed, key))),
                        "Released" => Some(Input(Key(Released, key))),
                        "Held" => Some(Condition(KeyHeld(key))),
                        _ => None,
                    })
                })
            }
            Some("MouseButton") => {
                split.next().and_then(|state| {
                    split.next()
                        .and_then(|b| mouse_button_from_str(b))
                        .and_then(|button| match state {
                            "Pressed" => Some(Input(MouseButton(Pressed, button))),
                            "Released" => Some(Input(MouseButton(Released, button))),
                            "Held" => Some(Condition(MouseButtonHeld(button))),
                            _ => None,
                        })
                })
            }
            _ => None,
        }
    }
}

macro_rules! enum_to_str {
    ( $name:ident, $enum_path:path, $($variant:pat),* ) => {
        #[allow(dead_code)]
        fn $name(v: $enum_path) -> &'static str {
            use $enum_path::*;
            match v {
                $( $variant => stringify!($variant), )*
            }
        }
    }
}

macro_rules! enum_from_str {
    ( $name:ident, $enum_path:path, $($variant:ident),* ) => {
        #[allow(dead_code)]
        fn $name(s: &str) -> Option<$enum_path> {
            use $enum_path::*;
            match s {
                $( stringify!($variant) => Some($variant), )*
                _ => None
            }
        }
    }
}

macro_rules! enum_str_conv {
    ( $to:ident, $from:ident, $enum_path:path, $($variant:tt),* ) => {
        enum_to_str! { $to, $enum_path, $($variant),* }
        enum_from_str! { $from, $enum_path, $($variant),* }
    };

    ( $to:ident, $from:ident, $enum_path:path, $($variant:tt),* _ ) => {
        enum_to_str! { $to, $enum_path, $($variant,)* _ }
        enum_from_str! { $from, $enum_path, $($variant),* }
    }
}

enum_str_conv! {
    key_to_str, key_from_str, ::winit::VirtualKeyCode,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Key0,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Escape,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    Snapshot,
    Scroll,
    Pause,
    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,
    Left,
    Up,
    Right,
    Down,
    Back,
    Return,
    Space,
    Compose,
    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    AbntC1,
    AbntC2,
    Add,
    Apostrophe,
    Apps,
    At,
    Ax,
    Backslash,
    Calculator,
    Capital,
    Colon,
    Comma,
    Convert,
    Decimal,
    Divide,
    Equals,
    Grave,
    Kana,
    Kanji,
    LAlt,
    LBracket,
    LControl,
    LMenu,
    LShift,
    LWin,
    Mail,
    MediaSelect,
    MediaStop,
    Minus,
    Multiply,
    Mute,
    MyComputer,
    NavigateForward,
    NavigateBackward,
    NextTrack,
    NoConvert,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    OEM102,
    Period,
    PlayPause,
    Power,
    PrevTrack,
    RAlt,
    RBracket,
    RControl,
    RMenu,
    RShift,
    RWin,
    Semicolon,
    Slash,
    Sleep,
    Stop,
    Subtract,
    Sysrq,
    Tab,
    Underline,
    Unlabeled,
    VolumeDown,
    VolumeUp,
    Wake,
    WebBack,
    WebFavorites,
    WebForward,
    WebHome,
    WebRefresh,
    WebSearch,
    WebStop,
    Yen
}

enum_str_conv! {
    mouse_button_to_str, mouse_button_from_str, ::winit::MouseButton,
    Left,
    Right,
    Middle
    _
}
