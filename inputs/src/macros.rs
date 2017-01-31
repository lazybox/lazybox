#[macro_export]
macro_rules! inputs_interaction {
    ( $($interface:expr => $iterator:ident, $event:ident { $($variant:ident => $action:expr,)* })* ) => {
        $(
            #[derive(Copy, Clone, PartialEq, Eq, Debug)]
            pub enum $event {
                $($variant,)*
            }

            impl<'a> From<&'a str> for $event {
                fn from(name: &'a str) -> $event {
                    match name {
                        $(x if x == $action => { $event::$variant }),*
                        _ => { panic!("event variant not handled"); }
                    }
                }
            }

            pub struct $iterator<'a> {
                inner: ::std::slice::Iter<'a, $crate::Action>,
            }

            impl<'a> $iterator<'a> {
                pub fn new(inputs: &'a $crate::Inputs) -> Option<Self> {
                    inputs.triggered_actions($interface)
                          .map(|actions| $iterator { inner: actions.iter() })
                }
            }

            impl<'a> Iterator for $iterator<'a> {
                type Item = $event;

                fn next(&mut self) -> Option<Self::Item> {
                    self.inner.next()
                              .map(|name| $event::from(&**name))
                }
            }
        )*

        pub fn build_interaction() -> $crate::InteractionBuilder {
            $crate::InteractionBuilder::new()
            $(
                .interface($interface, $crate::InterfaceBuilder::new()
                                            $(.action($action))*)
            )*
        }
    }
}