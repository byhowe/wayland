use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::format_ident;
use quote::quote;
use syn::Ident;
use wayland_xml::schema;

use crate::generator::*;

#[derive(Debug, Clone)]
pub struct InterfaceModule<'a>(pub &'a schema::Interface);

impl InterfaceModule<'_>
{
    pub fn format_name<S: AsRef<str>>(wl_name: S) -> Ident
    {
        format_ident!("{}", wl_name.as_ref())
    }

    pub fn name(&self) -> Ident
    {
        Self::format_name(&self.0.name)
    }
}

impl ToTokens for InterfaceModule<'_>
{
    fn to_tokens(&self, tokens: &mut TokenStream)
    {
        let interface_name = self.name();

        let interface_struct = InterfaceStruct(self.0);

        let requests = self.0.requests.iter().map(|msg| MessageStruct(msg));
        let events = self.0.events.iter().map(|msg| MessageStruct(msg));
        let enums = self.0.enums.iter().map(|enu| Enum(enu));

        quote! {
            pub mod #interface_name {
                use super::__protocol_root;
                // refers to the current protocol intercace
                use super::#interface_name as __interface_root;

                #(#enums)*

                pub mod request {
                    use super::__protocol_root;
                    use super::__interface_root;

                    #(#requests)*
                }

                pub mod event {
                    use super::__protocol_root;
                    use super::__interface_root;

                    #(#events)*
                }

                #interface_struct
            }
        }
        .to_tokens(tokens);
    }
}
