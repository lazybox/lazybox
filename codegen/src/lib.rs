#![recursion_limit = "128"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

#[proc_macro_derive(Prototype)]
pub fn prototype(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = expand_prototype(&ast);
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
