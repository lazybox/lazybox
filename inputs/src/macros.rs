#[macro_export]
macro_rules! inputs_interaction {
    ( $($interface:expr => $event:ident { $($action:expr),* })* ) => {
        $(
            #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
            pub struct $event(pub &'static str);

            impl $crate::events::Event for $event {}

            impl $crate::ActionEvent for $event {
                fn dispatch(action: &'static str,
                            dispatcher: &$crate::events::EventDispatcher) {
                    dispatcher.dispatch($event(action));
                }
            }
        )*

        pub fn build_interaction() -> $crate::InteractionBuilder {
            $crate::InteractionBuilder::new()
            $(
                .interface($interface, $crate::InterfaceBuilder::new::<$event>()
                                            $(.action($action))*)
            )*
        }
    }
}