use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::format_ident;
use quote::quote;
use syn::Ident;
use wayland_xml::schema;

use crate::generator::*;

#[derive(Debug, Clone)]
pub struct Protocol<'a>(pub &'a schema::Protocol);

impl Protocol<'_>
{
    pub fn name(&self) -> Ident
    {
        format_ident!("{}", self.0.name)
    }
}

impl ToTokens for Protocol<'_>
{
    fn to_tokens(&self, tokens: &mut TokenStream)
    {
        let protocol_name = self.name();

        let interfaces = self
            .0
            .interfaces
            .iter()
            .map(|iface| (InterfaceModule(iface), InterfaceStruct(iface)));

        let interface_modules = interfaces.clone().map(|(iface_module, _)| iface_module);

        let interface_paths = interfaces
            .clone()
            .map(|(iface_module, iface_struct)| (iface_module.name(), iface_struct.name()))
            .map(|(iface_module, iface_struct)| quote! { #iface_module::#iface_struct })
            .collect::<Vec<_>>();

        let meta_name = protocol_name.to_string();

        quote! {
            use super::#protocol_name as __protocol_root;

            #(#interface_modules)*

            #(#[doc(inline)] pub use #interface_paths;)*

            pub const META: ::wayland_core::meta::Protocol = ::wayland_core::meta::Protocol {
                name: #meta_name,
                interfaces: &[
                    #(&#interface_paths::META,)*
                ],
            };
        }
        .to_tokens(tokens);
    }
}
