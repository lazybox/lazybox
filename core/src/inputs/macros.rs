#[macro_export]
macro_rules! input_interfaces {
    ( $($interface:ident => { $($variant:ident,)* })* ) => {
        $(
            pub mod $interface {
                #[derive(Copy, Clone, PartialEq, Eq, Debug)]
                pub enum Action {
                    $($variant,)*
                }

                impl<'a> From<&'a str> for Action {
                    fn from(name: &'a str) -> Action {
                        match name {
                            $(x if x == stringify!($variant) => { Action::$variant }),*
                            _ => { panic!("event variant not handled"); }
                        }
                    }
                }

                pub struct ActionIterator<'a> {
                    inner: ::std::slice::Iter<'a, $crate::Action>,
                }

                impl<'a> ActionIterator<'a> {
                    pub fn new(inputs: &'a $crate::Inputs) -> Option<Self> {
                        inputs.triggered_actions(stringify!($interface))
                            .map(|actions| ActionIterator { inner: actions.iter() })
                    }
                }

                impl<'a> Iterator for ActionIterator<'a> {
                    type Item = Action;

                    fn next(&mut self) -> Option<Self::Item> {
                        self.inner.next()
                                .map(|name| Action::from(&**name))
                    }
                }
            }
           
        )*

        pub fn build_interaction() -> $crate::InteractionBuilder {
            $crate::InteractionBuilder::new()
            $(
                .interface(stringify!($interface), $crate::InterfaceBuilder::new()
                                            $(.action(stringify!($variant)))*)
            )*
        }
    }
}