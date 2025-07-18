#![allow(dead_code)] // TODO: DO NOT FORGET TO REMOVE

use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::format_ident;
use quote::quote;
use syn::Ident;
use wayland_xml::schema;

use crate::util::snake_to_camel;

#[derive(Debug, Clone)]
pub struct Protocol<'a>(pub &'a schema::Protocol);

impl Protocol<'_>
{
    fn name(&self) -> Ident
    {
        format_ident!("{}", self.0.name)
    }
}

impl ToTokens for Protocol<'_>
{
    fn to_tokens(&self, tokens: &mut TokenStream)
    {
        let protocol_name = self.name();

        let interface_modules = self.0.interfaces.iter().map(|iface| InterfaceModule(iface));
        let interface_structs = self.0.interfaces.iter().map(|iface| InterfaceStruct(iface));

        let meta_interfaces = interface_structs.clone().map(|interface| {
            let interface_name = interface.name();
            quote! { #interface_name::META }
        });

        let meta_name = protocol_name.to_string();

        quote! {
            pub mod #protocol_name {
                #(#interface_structs)*

                pub mod interfaces {
                    #(#interface_modules)*
                }

                pub const META: ::wayland_core::meta::Protocol = ::wayland_core::meta::Protocol {
                    name: #meta_name,
                    interfaces: &[
                        #(&#meta_interfaces,)*
                    ],
                };
            }
        }
        .to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub struct Copyright(pub schema::Copyright);

#[derive(Debug, Clone)]
pub struct InterfaceModule<'a>(pub &'a schema::Interface);

impl InterfaceModule<'_>
{
    fn name(&self) -> Ident
    {
        format_ident!("{}", self.0.name.trim_prefix("wl_"))
    }
}

impl ToTokens for InterfaceModule<'_>
{
    fn to_tokens(&self, tokens: &mut TokenStream)
    {
        let interface_name = self.name();

        let enums = self.0.enums.iter().map(|enu| Enum(enu));

        quote! {
            pub mod #interface_name {
                #(#enums)*
            }
        }
        .to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub struct InterfaceStruct<'a>(pub &'a schema::Interface);

impl InterfaceStruct<'_>
{
    fn name(&self) -> Ident
    {
        format_ident!("{}", snake_to_camel(self.0.name.trim_prefix("wl_")))
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
            quote! { interfaces::#module_name::#enum_name::META }
        });

        let meta_name = module_name.to_string();
        let meta_version = self.0.version;

        // TODO: Continue implementing the request functions
        quote! {
            pub struct #struct_name { }

            impl #struct_name {
                pub const META: ::wayland_core::meta::Interface = ::wayland_core::meta::Interface {
                    name: #meta_name,
                    version: #meta_version,
                    requests: &[], // TODO: Make sure to fill these
                    events: &[],
                    enums: &[
                        #(&#meta_enums,)*
                    ],
                };

                #(#request_functions)*
            }
        }
        .to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub struct RequestFunction<'a>(pub &'a schema::Message);

impl RequestFunction<'_>
{
    fn name(&self) -> Ident
    {
        if syn::parse_str::<Ident>(&self.0.name).is_err() {
            format_ident!("{}_", self.0.name)
        } else {
            format_ident!("{}", self.0.name)
        }
    }
}

impl ToTokens for RequestFunction<'_>
{
    fn to_tokens(&self, tokens: &mut TokenStream)
    {
        let request_name = self.name();

        quote! {
            pub fn #request_name(&self) {}
        }
        .to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub struct Event<'a>(pub &'a schema::Message);

#[derive(Debug, Clone)]
pub struct Enum<'a>(pub &'a schema::Enum);

impl Enum<'_>
{
    fn name(&self) -> Ident
    {
        format_ident!("{}", snake_to_camel(&self.0.name))
    }

    fn entries(&self) -> impl Iterator<Item = Entry<'_>>
    {
        self.0.entries.iter().map(|entry| Entry(entry))
    }
}

impl ToTokens for Enum<'_>
{
    fn to_tokens(&self, tokens: &mut TokenStream)
    {
        let enum_name = self.name();

        let entries = self.entries().map(|entry| {
            let name = entry.name();
            let value = entry.value();

            quote! { #name = #value }
        });

        match self.0.bitfield {
            true => quote! {
                bitflags::bitflags! {
                    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
                    pub struct #enum_name: u32 {
                        #(const #entries;)*
                    }
                }

                impl ::core::convert::TryFrom<u32> for #enum_name {
                    type Error = ();

                    fn try_from(value: u32) -> ::core::result::Result<Self, Self::Error> {
                        #enum_name::from_bits(value).ok_or(())
                    }
                }

                impl ::core::convert::From<#enum_name> for u32 {
                    fn from(value: #enum_name) -> Self {
                        value.bits()
                    }
                }
            },
            false => {
                let try_from_arms = self.entries().map(|entry| {
                    let name = entry.name();
                    let value = entry.value();

                    quote! { #value => Ok(#enum_name::#name) }
                });

                quote! {
                    #[repr(u32)]
                    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
                    #[non_exhaustive]
                    pub enum #enum_name {
                        #(#entries,)*
                    }

                    impl ::core::convert::TryFrom<u32> for #enum_name {
                        type Error = ();

                        fn try_from(value: u32) -> ::core::result::Result<Self, Self::Error> {
                            match value {
                                #(#try_from_arms,)*
                                _ => Err(()),
                            }
                        }
                    }

                    impl ::core::convert::From<#enum_name> for u32 {
                        fn from(value: #enum_name) -> Self {
                            value as Self
                        }
                    }
                }
            }
        }
        .to_tokens(tokens);

        let meta_entries = self.entries().map(|entry| {
            let entry_name = entry.name().to_string();
            let entry_value = entry.value().to_string();
            let entry_since = entry.0.since;
            let deprecated_since = match entry.0.deprecated_since {
                Some(v) => quote! { Some(#v) },
                None => quote! { None },
            };

            quote! {
                ::wayland_core::meta::Entry {
                    name: #entry_name,
                    value: #entry_value,
                    since: #entry_since,
                    deprecated_since: #deprecated_since,
                }
            }
        });

        let meta_name = enum_name.to_string();
        let meta_since = self.0.since;
        let meta_bitfield = self.0.bitfield;

        quote! {
            impl #enum_name {
                pub const META: ::wayland_core::meta::Enum = ::wayland_core::meta::Enum {
                    name: #meta_name,
                    since: #meta_since,
                    bitfield: #meta_bitfield,
                    entries: &[
                        #(#meta_entries,)*
                    ],
                };
            }
        }
        .to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub struct Entry<'a>(pub &'a schema::Entry);

impl Entry<'_>
{
    fn name(&self) -> Ident
    {
        // NOTE: Parsing as Ident using syn allows us to check if the string contains a
        // rust keyword so that we can add a prefix to it.
        let variant_name = snake_to_camel(&self.0.name);
        if variant_name.chars().next().unwrap().is_numeric()
            || syn::parse_str::<Ident>(&variant_name).is_err()
        {
            format_ident!("_{}", variant_name)
        } else {
            format_ident!("{}", variant_name)
        }
    }

    fn value(&self) -> TokenStream
    {
        self.0.value.parse::<TokenStream>().unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct Arg(pub schema::Arg);

#[derive(Debug, Clone)]
pub struct Description(pub schema::Description);
