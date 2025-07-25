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
        let message_generics_a = self.generics();
        let message_elided_generic = message_generics_a.clone().map(|_| quote! { <'_> });
        // FIX: temporarily disable fd.
        let message_fields = self.0.args.iter().filter_map(|arg| match arg.typ {
            schema::Type::Fd => None,
            _ => Some(ArgField {
                arg,
                ctx: TypeContext::Struct { lifetime: "'a" },
            }),
        });
        let message_opcode = self.0.opcode;

        let wire_arguments = message_fields
            .clone()
            .filter_map(|field| {
                let arg_name = field.name();
                match field.arg.typ {
                    schema::Type::Fd => None,
                    _ => Some(quote! { #arg_name }),
                }
            })
            .collect::<Vec<_>>();
        let wire_argument_types = message_fields
            .clone()
            .filter_map(|field| match &field.arg.typ {
                schema::Type::Fd => None,
                t => {
                    let type_path = Type {
                        typ: t,
                        ctx: TypeContext::Struct { lifetime: "'_" },
                    };
                    Some(quote! { <#type_path as ::wayland_core::Wire> })
                }
            });

        quote! {
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct #message_name #message_generics_a {
                #(pub #message_fields,)*
            }

            impl #message_generics_a ::wayland_core::Opcode for #message_name #message_generics_a {
                const OPCODE: u16 = #message_opcode;
            }

            impl ::wayland_core::Wire for #message_name #message_elided_generic {
                type Output<'a> = #message_name #message_generics_a;

                fn wire_write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32] {
                    #(let buf = self.#wire_arguments.wire_write(buf);)*
                    buf
                }

                fn wire_read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self::Output<'buf>), ::wayland_core::WireError> {
                    #(let (buf, #wire_arguments) = #wire_argument_types::wire_read(buf)?;)*
                    Ok((buf, #message_name {
                        #(#wire_arguments,)*
                    }))
                }

                fn wire_size(&self) -> usize {
                    0 #(+ self.#wire_arguments.wire_size())*
                }
            }
        }
        .to_tokens(tokens);
    }
}
