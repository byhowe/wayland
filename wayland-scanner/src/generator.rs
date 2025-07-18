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
    pub fn name(&self) -> Ident
    {
        format_ident!("{}", self.0.name)
    }

    pub fn interfaces(&self) -> impl Iterator<Item = Interface<'_>>
    {
        self.0
            .interfaces
            .iter()
            .map(|interface| Interface(interface))
    }
}

impl ToTokens for Protocol<'_>
{
    fn to_tokens(&self, tokens: &mut TokenStream)
    {
        let protocol_name = self.name();

        let interfaces = self.interfaces().map(|interface| quote! { #interface });

        let meta_interfaces = self.interfaces().map(|interface| {
            let interface_name = interface.name();
            quote! { #interface_name::META }
        });

        let meta_name = protocol_name.to_string();

        quote! {
            pub mod #protocol_name {
                #(#interfaces)*

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
pub struct Interface<'a>(pub &'a schema::Interface);

impl Interface<'_>
{
    fn name(&self) -> Ident
    {
        format_ident!("{}", self.0.name.trim_prefix("wl_"))
    }

    fn enums(&self) -> impl Iterator<Item = Enum<'_>>
    {
        self.0.enums.iter().map(|enu| Enum(enu))
    }
}

impl ToTokens for Interface<'_>
{
    fn to_tokens(&self, tokens: &mut TokenStream)
    {
        let interface_name = self.name();

        let enums = self.enums().map(|enu| quote! { #enu });

        let meta_enums = self.enums().map(|enu| {
            let enum_name = enu.name();
            quote! { #enum_name::META }
        });

        let meta_name = interface_name.to_string();
        let meta_version = self.0.version;

        quote! {
            pub mod #interface_name {
                #(#enums)*

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

#[derive(Debug, Clone)]
pub struct Message<'a>(pub &'a schema::Message);

impl Message<'_> {}

impl ToTokens for Message<'_>
{
    fn to_tokens(&self, tokens: &mut TokenStream) {}
}

#[derive(Debug, Clone)]
pub struct Enum<'a>(pub &'a schema::Enum);

impl Enum<'_>
{
    pub fn name(&self) -> Ident
    {
        format_ident!("{}", snake_to_camel(&self.0.name))
    }

    pub fn entries(&self) -> impl Iterator<Item = Entry<'_>>
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
    pub fn name(&self) -> Ident
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

    pub fn value(&self) -> TokenStream
    {
        self.0.value.parse::<TokenStream>().unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct Arg(pub schema::Arg);

#[derive(Debug, Clone)]
pub struct Description(pub schema::Description);
