use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::format_ident;
use quote::quote;
use syn::Ident;
use syn::Path;
use wayland_xml::schema;

use crate::generator::*;
use crate::util::snake_to_camel;

#[derive(Debug, Clone)]
pub struct InterfaceStruct<'a>(pub &'a schema::Interface);

impl InterfaceStruct<'_>
{
    pub fn format_name<S: AsRef<str>>(wl_name: S) -> Ident
    {
        format_ident!("{}", snake_to_camel(wl_name.as_ref()))
    }

    pub fn resolve_path<S: AsRef<str>>(wl_path: S) -> Path
    {
        let path = format!(
            "__protocol_root::{}::{}",
            InterfaceModule::format_name(wl_path.as_ref()),
            InterfaceStruct::format_name(wl_path.as_ref()),
        );
        syn::parse_str(&path).unwrap()
    }

    pub fn name(&self) -> Ident
    {
        Self::format_name(&self.0.name)
    }
}

impl ToTokens for InterfaceStruct<'_>
{
    fn to_tokens(&self, tokens: &mut TokenStream)
    {
        let struct_name = self.name();
        let module_name = InterfaceModule(self.0).name();

        let request_functions = self.0.requests.iter().map(|req| RequestFunction(req));

        let meta_enums = self.0.enums.iter().map(|enu| Enum(enu)).map(|enu| {
            let enum_name = enu.name();
            quote! { #enum_name::META }
        });

        let meta_name = module_name.to_string();
        let meta_version = self.0.version;

        // TODO: Continue implementing the request functions
        quote! {
            #[repr(transparent)]
            #[derive(Copy, Clone, Debug, PartialEq, Eq)]
            pub struct #struct_name {
                object: ::wayland_core::Object,
            }

            impl ::core::convert::From<#struct_name> for ::wayland_core::Object {
                fn from(value: #struct_name) -> Self {
                    value.object()
                }
            }

            impl ::core::convert::From<::wayland_core::Object> for #struct_name {
                fn from(value: ::wayland_core::Object) -> Self {
                    Self { object: value }
                }
            }

            impl ::wayland_core::Wire for #struct_name {
                type Output<'a> = #struct_name;

                fn wire_write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32] {
                    self.object().wire_write(buf)
                }

                fn wire_read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self::Output<'buf>), ::wayland_core::WireError> {
                    let (buf, object) = ::wayland_core::Object::wire_read(buf)?;
                    Ok((buf, Self::new(object)))
                }

                fn wire_size(&self) -> usize {
                    self.object().wire_size()
                }
            }

            impl #struct_name {
                #[inline(always)]
                pub const fn object(self) -> ::wayland_core::Object {
                    self.object
                }

                pub const fn new(object: ::wayland_core::Object) -> Self {
                    Self { object }
                }

                #(#request_functions)*

                pub const META: ::wayland_core::meta::Interface = ::wayland_core::meta::Interface {
                    name: #meta_name,
                    version: #meta_version,
                    requests: &[], // TODO: Make sure to fill these
                    events: &[],
                    enums: &[
                        #(&#meta_enums,)*
                    ],
                };
            }
        }
        .to_tokens(tokens);
    }
}
