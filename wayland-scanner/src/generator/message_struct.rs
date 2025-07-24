use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::format_ident;
use quote::quote;
use syn::Ident;
use wayland_xml::schema;

use crate::generator::*;
use crate::util::snake_to_camel;

#[derive(Debug, Clone)]
pub struct MessageStruct<'a>(pub &'a schema::Message);

impl MessageStruct<'_>
{
    pub fn format_name<S: AsRef<str>>(wl_name: S) -> Ident
    {
        format_ident!("{}", snake_to_camel(wl_name.as_ref()))
    }

    pub fn name(&self) -> Ident
    {
        Self::format_name(&self.0.name)
    }

    pub fn generics(&self) -> Option<TokenStream>
    {
        self.0
            .args
            .iter()
            .find(|arg| match arg.typ {
                schema::Type::String { .. } | schema::Type::Array => true,
                _ => false,
            })
            .is_some()
            .then_some(quote! { <'a> })
    }
}

impl ToTokens for MessageStruct<'_>
{
    fn to_tokens(&self, tokens: &mut TokenStream)
    {
        let message_name = self.name();
        let message_generics = self.generics();
        let message_args = self.0.args.iter().map(|arg| ArgField {
            arg,
            ctx: TypeContext::Struct,
        });
        let message_opcode = self.0.opcode;

        quote! {
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct #message_name #message_generics {
                #(pub #message_args,)*
            }

            impl #message_generics ::wayland_core::Opcode for #message_name #message_generics {
                const OPCODE: u16 = #message_opcode;
            }
        }
        .to_tokens(tokens);
    }
}
