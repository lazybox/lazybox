use std::collections::{HashMap, HashSet};
use events::*;
use glutin::{VirtualKeyCode, MouseButton, ElementState};
use yaml_rust::Yaml;
use state::InputState;

#[derive(Clone, Eq, PartialEq, Hash)]
pub(crate) enum Input {
    Key(ElementState, VirtualKeyCode),
    MouseButton(ElementState, MouseButton),
}

pub(crate) enum Condition {
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

pub(crate) struct ConditionalAction {
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

pub(crate) struct Rules {
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
        InteractionBuilder {
            interfaces: HashMap::new(),
        }
    }

    pub fn interface(mut self, name: &'static str, builder: InterfaceBuilder) -> Self {
        self.interfaces.insert(name, builder.build());
        self
    }

    pub(crate) fn build(self) -> Interaction {
        Interaction {
            interfaces: self.interfaces,
        }
    }
}

pub struct InterfaceBuilder {
    actions: HashSet<Action>,
    dispatch: fn(Action, &EventDispatcher),
}

impl InterfaceBuilder {
    pub fn new<E: ActionEvent>() -> Self {
        InterfaceBuilder {
            actions: HashSet::new(),
            dispatch: E::dispatch,
        }
    }

    pub fn action(mut self, a: Action) -> Self {
        self.actions.insert(a);
        self
    }

    fn build(self) -> Interface {
        Interface {
            actions: self.actions,
            rules: Rules::new(),
            dispatch: self.dispatch,
        }
    }
}

pub trait ActionEvent: Event {
    fn dispatch(action: &'static str, dispatcher: &EventDispatcher);
}

pub struct Interaction {
    interfaces: HashMap<&'static str, Interface>,
}

impl Interaction {
    pub fn load_profile(&mut self, profile: &Yaml) {
        let wrong_fmt = "wrong interaction profile format";

        for (&name, interface) in &mut self.interfaces {
            let rules = profile[name]["rules"].as_vec().expect(wrong_fmt);

            interface.load_rules(rules);
        }
    }

    pub(crate) fn dispatch_input_actions(&self, input: &Input,
                                     state: &InputState,
                                     dispatcher: &EventDispatcher)
    {
        // TODO: enable/disable interface dispatch
        for (_, interface) in &self.interfaces {
            interface.dispatch_input_actions(input, state, dispatcher);
        }
    }

    pub(crate) fn dispatch_state_actions(&self, state: &InputState, dispatcher: &EventDispatcher) {
        // TODO: enable/disable interface dispatch
        for (_, interface) in &self.interfaces {
            interface.dispatch_state_actions(state, dispatcher);
        }
    }
}

pub(crate) struct Interface {
    actions: HashSet<Action>,
    rules: Rules,
    dispatch: fn(Action, &EventDispatcher),
}

impl Interface {
    fn load_rules(&mut self, rules: &[Yaml]) {
        let wrong_fmt = "wrong interface rule format";

        self.rules.clear();
        for rule in rules {
            let action = rule["action"].as_str().expect(wrong_fmt);
            let when = rule["when"].as_str().expect(wrong_fmt);

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
                    None => panic!("could not parse interface rule condition"),
                }
            } else {
                println!("unknown interface action is ignored");
            }
        }
    }

    fn dispatch_input_actions(&self, input: &Input,
                                     _state: &InputState,
                                     dispatcher: &EventDispatcher)
    {
        if let Some(action) = self.rules.by_input.get(input) {
            (self.dispatch)(action, dispatcher);
        }
    }

    fn dispatch_state_actions(&self, state: &InputState, dispatcher: &EventDispatcher) {
        for ca in &self.rules.others {
            if let Some(action) = ca.may_trigger(state) {
                (self.dispatch)(action, dispatcher);
            }
        }
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
        use glutin::ElementState::*;

        let mut split = s.split('.');
        match split.next() {
            Some("Key") => {
                split.next().and_then(|state| {
                    split.next().and_then(|k| key_from_str(k)).and_then(|key| {
                        match state {
                            "Pressed" => Some(Input(Key(Pressed, key))),
                            "Released" => Some(Input(Key(Released, key))),
                            "Held" => Some(Condition(KeyHeld(key))),
                            _ => None
                        }
                    })
                })
            }
            Some("MouseButton") => {
                split.next().and_then(|state| {
                    split.next().and_then(|b| mouse_button_from_str(b)).and_then(|button| {
                        match state {
                            "Pressed" => Some(Input(MouseButton(Pressed, button))),
                            "Released" => Some(Input(MouseButton(Released, button))),
                            "Held" => Some(Condition(MouseButtonHeld(button))),
                            _ => None
                        }
                    })
                })
            }
            _ => None
        }
    }

    pub fn to_string(&self) -> String {
        use self::WhenParse::*;
        use self::Input::*;
        use self::Condition::*;
        use glutin::ElementState::*;

        match self {
            &Input(Key(Pressed, key)) => "Key.Pressed.".to_string() + key_to_str(key),
            &Input(Key(Released, key)) => "Key.Released.".to_string() + key_to_str(key),
            &Condition(KeyHeld(key)) => "Key.Held.".to_string() + key_to_str(key),
            &Input(MouseButton(Pressed, button)) =>
                "MouseButton.Pressed.".to_string() + mouse_button_to_str(button),
            &Input(MouseButton(Released, button)) =>
                "MouseButton.Released.".to_string() + mouse_button_to_str(button),
            &Condition(MouseButtonHeld(button)) =>
                "MouseButton.Held.".to_string() + mouse_button_to_str(button),
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
    key_to_str, key_from_str, ::glutin::VirtualKeyCode,
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
    mouse_button_to_str, mouse_button_from_str, ::glutin::MouseButton,
    Left,
    Right,
    Middle
    _
}