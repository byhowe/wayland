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
pub struct Enum<'a>(pub &'a schema::Enum);

impl Enum<'_>
{
    pub fn format_name<S: AsRef<str>>(wl_name: S) -> Ident
    {
        format_ident!("{}", snake_to_camel(wl_name.as_ref()))
    }

    pub fn resolve_path<S: AsRef<str>>(wl_path: S) -> Path
    {
        let mut parts = wl_path.as_ref().split('.').rev();
        let enum_name = parts.next().map(Self::format_name).unwrap();
        let path = match parts.next() {
            Some(iface) => {
                let iface_name = InterfaceModule::format_name(iface);
                format!("__protocol_root::{}::{}", iface_name, enum_name)
            }
            None => format!("__interface_root::{}", enum_name),
        };
        syn::parse_str(&path).unwrap()
    }

    pub fn name(&self) -> Ident
    {
        Self::format_name(&self.0.name)
    }

    pub fn entries(&self) -> impl Iterator<Item = Variant<'_>>
    {
        self.0.entries.iter().map(|entry| Variant(entry))
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

                    #[inline(always)]
                    fn try_from(value: u32) -> ::core::result::Result<Self, Self::Error> {
                        #enum_name::from_bits(value).ok_or(())
                    }
                }

                impl ::core::convert::From<#enum_name> for u32 {
                    #[inline(always)]
                    fn from(value: #enum_name) -> Self {
                        value.bits()
                    }
                }

                impl #enum_name {
                    #[inline(always)]
                    pub const fn value_unsigned(self) -> u32 {
                        self.bits()
                    }

                    #[inline(always)]
                    pub const fn value_signed(self) -> i32 {
                        self.bits().cast_signed()
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

                        #[inline(always)]
                        fn try_from(value: u32) -> ::core::result::Result<Self, Self::Error> {
                            match value {
                                #(#try_from_arms,)*
                                _ => Err(()),
                            }
                        }
                    }

                    impl ::core::convert::From<#enum_name> for u32 {
                        #[inline(always)]
                        fn from(value: #enum_name) -> Self {
                            value as Self
                        }
                    }

                    impl #enum_name {
                        #[inline(always)]
                        pub const fn value_unsigned(self) -> u32 {
                            self as u32
                        }

                        #[inline(always)]
                        pub const fn value_signed(self) -> i32 {
                            (self as u32).cast_signed()
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
pub struct Variant<'a>(pub &'a schema::Entry);

impl Variant<'_>
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
