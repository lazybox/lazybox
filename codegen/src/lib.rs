#![recursion_limit = "128"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use std::collections::HashSet;
use proc_macro::TokenStream;

#[proc_macro_derive(Prototype)]
pub fn prototype(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = expand_prototype(&ast);
    gen.parse().unwrap()
}

#[proc_macro_derive(Model)]
pub fn model(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = expand_model(&ast);
    gen.parse().unwrap()
}

fn expand_prototype(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    let fields = match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref fields)) => fields,
        syn::Body::Struct(_) |
        syn::Body::Enum(_) => panic!("can only be used with regular structs")
    };

    let batch_name: syn::Ident = match ast.attrs.iter().find(|&a| a.name() == "batch") {
        None => panic!("expected 'batch' attribute"),
        Some(ref a) => match a.value {
            syn::MetaItem::NameValue(_, syn::Lit::Str(ref s, _)) => (s as &str).into(),
            _ => panic!("malformed 'batch' attribute")
        }
    };

    let field_names: &Vec<_> = &fields.iter().map(|f| &f.ident).collect();
    let component_types: &Vec<_> = &fields.iter().map(|f| &f.ty).collect();

    let attaches = field_names.iter().fold(quote! {}, |tokens, f| {
        quote! { #tokens self.#f.attach(accessor, prototype.#f); }
    });

    quote! {
        impl ::lazybox::ecs::spawn::Prototype for #name {
            fn spawn_later_with<'a, Cx: Send>(self, spawn: ::lazybox::ecs::SpawnRequest<'a, Cx>) {
                spawn #(.set::<#component_types>(self.#field_names))* ;
            }
        }

        impl #name {
            fn batch<'a, Cx: Send>(commit: ::lazybox::ecs::state::Commit<'a, Cx>)
                                   -> #batch_name<'a, Cx> {
                #batch_name {
                    #(#field_names: commit.update_queue::<#component_types>(),)*
                    commit: commit,
                }
            }
        }

        pub struct #batch_name<'a, Cx: 'a + Send> {
            commit: ::lazybox::ecs::state::Commit<'a, Cx>,
            #(#field_names: &'a ::lazybox::ecs::state::update_queue::UpdateQueue<#component_types>,)*
        }

        impl<'a, Cx: 'a + Send> #batch_name<'a, Cx> {
            fn spawn_later(&self, prototype: #name) {
                let entity = self.commit.spawn_later().entity();
                let accessor = unsafe {
                    ::lazybox::ecs::entity::Accessor::new_unchecked(entity.id())
                };
                #attaches
            }
        }
    }
}

fn expand_model(ast: &syn::DeriveInput) -> quote::Tokens {
    let fields = match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref fields)) => fields,
        _ => panic!("expected regular struct")
    };

    let name: syn::Ident = match ast.attrs.iter().find(|&a| a.name() == "name") {
        None => panic!("expected 'name' attribute"),
        Some(ref a) => match a.value {
            syn::MetaItem::NameValue(_, syn::Lit::Str(ref s, _)) => (s as &str).into(),
            _ => panic!("malformed 'name' attribute")
        }
    };

    let mut components = HashSet::new();
    let mut read_idents = Vec::new();
    let mut read_types = Vec::new();
    let mut write_idents = Vec::new();
    let mut write_types = Vec::new();

    for f in fields {
        if !components.insert(&f.ty) {
            panic!("cannot refer to the same component multiple times");
        }
        if f.attrs.len() != 1 {
            panic!("expected exactly one field attribute");
        }
        match f.attrs[0].name() {
            "read" => {
                read_idents.push(&f.ident);
                read_types.push(&f.ty);
            },
            "write" => {
                write_idents.push(&f.ident);
                write_types.push(&f.ty);
            }
            _ => panic!("expected 'read' or 'write' field attribute")
        }
    }

    let read_guards = read_idents.iter().zip(read_types.iter())
        .fold(quote! {}, |tokens, (&ident, &component)| quote! {
            #tokens
            #ident: ::lazybox::ecs::module::StorageReadGuard<'a, <<#component as ::lazybox::ecs::module::Component>::Module as ::lazybox::ecs::module::HasComponent<#component>>::Storage>,
        });

    let guards = write_idents.iter().zip(write_types.iter())
        .fold(read_guards, |tokens, (&ident, &component)| quote! {
            #tokens
            #ident: ::lazybox::ecs::module::StorageWriteGuard<'a, <<#component as ::lazybox::ecs::module::Component>::Module as ::lazybox::ecs::module::HasComponent<#component>>::Storage>,
        });

    let read_idents = &read_idents[..];
    let read_types = &read_types[..];
    let write_idents = &write_idents[..];
    let write_types = &write_types[..];
    quote! {
        pub struct #name<'a> {
            #guards
        }

        impl<'a, Cx: ::lazybox::ecs::Context> ::lazybox::ecs::processor::Model<'a, Cx> for #name<'a> {
            fn from_state(state: &'a ::lazybox::ecs::state::State<Cx>) -> Self {
                #name {
                    #(#read_idents: state.read::<#read_types>(),)*
                    #(#write_idents: state.write::<#write_types>(),)*
                }
            }

            fn reads() -> Vec<::lazybox::ecs::module::ComponentType> {
                vec![#(::lazybox::ecs::module::ComponentType::of::<#read_types>()),*]
            }

            fn writes() -> Vec<::lazybox::ecs::module::ComponentType> {
                vec![#(::lazybox::ecs::module::ComponentType::of::<#write_types>()),*]
            }
        }
    }
}
