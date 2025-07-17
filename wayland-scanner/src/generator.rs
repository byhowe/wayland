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

        quote! {
            pub mod #protocol_name {
                #(#interfaces)*
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

        quote! {
            pub mod #interface_name {
                #(#enums)*
            }
        }
        .to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub struct Message(pub schema::Message);

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
            },
            false => quote! {
                #[repr(u32)]
                #[derive(Copy, Clone, Debug, PartialEq, Eq)]
                #[non_exhaustive]
                pub enum #enum_name {
                    #(#entries,)*
                }
            },
        }
        .to_tokens(tokens);

        match self.0.bitfield {
            true => quote! {
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
