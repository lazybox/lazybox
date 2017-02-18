#![recursion_limit = "128"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use std::collections::HashSet;
use proc_macro::TokenStream;

#[proc_macro_derive(Prototype, attributes(batch))]
pub fn prototype(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = expand_prototype(&ast);
    gen.parse().unwrap()
}

#[proc_macro_derive(StateAccess, attributes(name, read, write))]
pub fn model(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = expand_state_access(&ast);
    gen.parse().unwrap()
}

fn expand_prototype(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    let fields = match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref fields)) => fields,
        syn::Body::Struct(_) |
        syn::Body::Enum(_) => panic!("can only be used with regular structs")
    };

    let batch_name = find_ident_attribute("batch", ast.attrs.iter());

    let field_names: &Vec<_> = &fields.iter().map(|f| &f.ident).collect();
    let component_types: &Vec<_> = &fields.iter().map(|f| &f.ty).collect();

    let attaches = field_names.iter().fold(quote! {}, |tokens, f| {
        quote! { #tokens self.#f.attach(accessor, prototype.#f); }
    });

    quote! {
        impl ::lazybox::core::spawn::Prototype for #name {
            fn spawn_later_with<'a, Cx: ::lazybox::core::Context>(self, spawn: ::lazybox::core::SpawnRequest<'a, Cx>) {
                spawn #(.set::<#component_types>(self.#field_names))* ;
            }
        }

        impl #name {
            fn batch<'a, Cx: ::lazybox::core::Context>(commit: ::lazybox::core::state::Commit<'a, Cx>)
                                   -> #batch_name<'a, Cx> {
                #batch_name {
                    #(#field_names: commit.update_queue::<#component_types>(),)*
                    commit: commit,
                }
            }
        }

        pub struct #batch_name<'a, Cx: 'a + ::lazybox::core::Context> {
            commit: ::lazybox::core::state::Commit<'a, Cx>,
            #(#field_names: &'a ::lazybox::core::state::update_queue::UpdateQueue<#component_types>,)*
        }

        impl<'a, Cx: 'a + ::lazybox::core::Context> #batch_name<'a, Cx> {
            fn spawn_later(&self, prototype: #name) {
                let entity = self.commit.spawn().entity();
                let accessor = unsafe {
                    ::lazybox::core::Accessor::new_unchecked(entity.id())
                };
                #attaches
            }
        }
    }
}

fn expand_state_access(ast: &syn::DeriveInput) -> quote::Tokens {
    let fields = match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref fields)) => fields,
        _ => panic!("expected regular struct")
    };

    let name = find_ident_attribute("name", ast.attrs.iter());

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
            #ident: ::lazybox::core::StorageReadGuard<'a, <<#component as ::lazybox::core::Component>::Module as ::lazybox::core::HasComponent<#component>>::Storage>,
        });

    let guards = write_idents.iter().zip(write_types.iter())
        .fold(read_guards, |tokens, (&ident, &component)| quote! {
            #tokens
            #ident: ::lazybox::core::StorageWriteGuard<'a, <<#component as ::lazybox::core::Component>::Module as ::lazybox::core::HasComponent<#component>>::Storage>,
        });

    let read_idents = &read_idents[..];
    let read_types = &read_types[..];
    let write_idents = &write_idents[..];
    let write_types = &write_types[..];
    quote! {
        pub struct #name<'a> {
            #guards
        }

        impl<'a, Cx: ::lazybox::core::Context> ::lazybox::core::processor::StateAccess<'a, Cx> for #name<'a> {
            fn from_state(state: &'a ::lazybox::core::State<Cx>) -> Self {
                #name {
                    #(#read_idents: state.read::<#read_types>(),)*
                    #(#write_idents: state.write::<#write_types>(),)*
                }
            }

            fn reads() -> Vec<::lazybox::core::ComponentType> {
                vec![#(::lazybox::core::ComponentType::of::<#read_types>()),*]
            }

            fn writes() -> Vec<::lazybox::core::ComponentType> {
                vec![#(::lazybox::core::ComponentType::of::<#write_types>()),*]
            }
        }
    }
}

fn find_ident_attribute<'a, A>(name: &str, mut attrs: A) -> &'a syn::Ident
    where A: Iterator<Item=&'a syn::Attribute>
{
    if let Some(ref a) = attrs.find(|&a| a.name() == name) {
        if let syn::MetaItem::List(_, ref items) = a.value {
            if items.len() == 1 {
                if let syn::NestedMetaItem::MetaItem(
                    syn::MetaItem::Word(ref ident)
                ) = items[0] {
                    return ident;
                }
            }
        }
    };

    panic!("malformed '{}' attribute", name);
}
