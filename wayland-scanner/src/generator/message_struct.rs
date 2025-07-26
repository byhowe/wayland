use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::format_ident;
use quote::quote;
use syn::Ident;
use wayland_xml::schema;

use crate::generator::*;
use crate::util::snake_to_camel;

#[derive(Debug, Clone)]
pub struct MessageStruct<'a>
{
    pub msg: &'a schema::Message,
    pub request: bool,
}

impl MessageStruct<'_>
{
    pub fn format_name<S: AsRef<str>>(wl_name: S) -> Ident
    {
        format_ident!("{}", snake_to_camel(wl_name.as_ref()))
    }

    pub fn name(&self) -> Ident
    {
        Self::format_name(&self.msg.name)
    }

    pub fn generics(&self) -> Option<TokenStream>
    {
        self.msg
            .args
            .iter()
            .find_map(|arg| {
                Type {
                    typ: &arg.typ,
                    ctx: TypeContext::Struct {
                        request: self.request,
                        lifetime: "'a",
                    },
                }
                .lifetime()
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
        let message_fields = self.msg.args.iter().map(|arg| ArgField {
            arg,
            ctx: TypeContext::Struct {
                request: self.request,
                lifetime: "'a",
            },
        });
        let message_opcode = self.msg.opcode;

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
                        ctx: TypeContext::Struct {
                            request: self.request,
                            lifetime: "'_",
                        },
                    };
                    Some(quote! { <#type_path as ::wayland_core::Wire> })
                }
            });

        let derives = &[
            Some(quote! { Debug }),
            self.request.then_some(quote! { Clone }),
        ];

        quote! {
            #[derive(#(#derives),*)]
            pub struct #message_name #message_generics_a {
                #(pub #message_fields,)*
            }

            impl #message_generics_a ::wayland_core::MessageOpcode for #message_name #message_generics_a {
                const OPCODE: u16 = #message_opcode;
            }

            impl ::wayland_core::MessageWire for #message_name #message_elided_generic {
                type Output<'a> = #message_name #message_generics_a;

                fn message_write(&self, buf: &mut [u32]) {
                    use ::wayland_core::Wire;
                    #(let buf = self.#wire_arguments.wire_write(buf);)*
                    _ = buf;
                }

                fn message_read_into<'buf>(
                    buf: &'buf[u32],
                    msg: &mut ::std::mem::MaybeUninit<Self::Output<'buf>>
                ) -> Result<(), ::wayland_core::WireError> {
                    let __ptr = msg.as_mut_ptr();

                    #(let (buf, #wire_arguments) = #wire_argument_types::wire_read(buf)?;)*
                    #(unsafe { (&raw mut ((*__ptr).#wire_arguments)).write(#wire_arguments) };)*

                    _ = buf;

                    Ok(())
                }

                fn message_size(&self) -> usize {
                    use ::wayland_core::Wire;
                    0 #(+ self.#wire_arguments.wire_size())*
                }
            }
        }
        .to_tokens(tokens);
    }
}
