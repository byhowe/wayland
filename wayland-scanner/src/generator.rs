#![allow(dead_code)] // TODO: DO NOT FORGET TO REMOVE

use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::format_ident;
use quote::quote;
use syn::Ident;
use syn::Path;
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

        let interfaces = self
            .0
            .interfaces
            .iter()
            .map(|iface| (InterfaceModule(iface), InterfaceStruct(iface)));

        let interface_modules = interfaces.clone().map(|(iface_module, _)| iface_module);

        let interface_paths_x = interfaces
            .clone()
            .map(|(iface_module, iface_struct)| (iface_module.name(), iface_struct.name()))
            .map(|(iface_module, iface_struct)| quote! { #iface_module::#iface_struct });
        let interface_paths_y = interface_paths_x.clone();

        let meta_name = protocol_name.to_string();

        quote! {
            use super::#protocol_name as __protocol_root;

            #(#interface_modules)*

            #(#[doc(inline)] pub use #interface_paths_x;)*

            pub const META: ::wayland_core::meta::Protocol = ::wayland_core::meta::Protocol {
                name: #meta_name,
                interfaces: &[
                    #(&#interface_paths_y::META,)*
                ],
            };
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
    fn format_name<S: AsRef<str>>(wl_name: S) -> Ident
    {
        format_ident!("{}", wl_name.as_ref())
    }

    fn name(&self) -> Ident
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

#[derive(Debug, Clone)]
pub struct InterfaceStruct<'a>(pub &'a schema::Interface);

impl InterfaceStruct<'_>
{
    fn format_name<S: AsRef<str>>(wl_name: S) -> Ident
    {
        format_ident!("{}", snake_to_camel(wl_name.as_ref()))
    }

    fn resolve_path<S: AsRef<str>>(wl_path: S) -> Path
    {
        let path = format!(
            "__protocol_root::{}::{}",
            InterfaceModule::format_name(wl_path.as_ref()),
            InterfaceStruct::format_name(wl_path.as_ref())
        );
        syn::parse_str(&path).unwrap()
    }

    fn name(&self) -> Ident
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
            #[derive(Debug, Clone)]
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
        let request_args = self.0.args.iter().map(|arg| Arg(arg));

        quote! {
            pub fn #request_name(
                &self,
                #(#request_args,)*
            ) {}
        }
        .to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub struct MessageStruct<'a>(pub &'a schema::Message);

impl MessageStruct<'_>
{
    fn format_name<S: AsRef<str>>(wl_name: S) -> Ident
    {
        format_ident!("{}", snake_to_camel(wl_name.as_ref()))
    }

    fn name(&self) -> Ident
    {
        Self::format_name(&self.0.name)
    }
}

impl ToTokens for MessageStruct<'_>
{
    fn to_tokens(&self, tokens: &mut TokenStream)
    {
        let message_name = self.name();
        let message_args = self.0.args.iter().map(|arg| Arg(arg));
        let message_opcode = self.0.opcode;

        let arg_sizes = self.0.args.iter().map(|arg| ArgSize(arg));
        let no_arg_size = arg_sizes.is_empty().then_some(quote! { 0usize });

        quote! {
            #[derive(Debug, Clone)]
            pub struct #message_name {
                #(pub #message_args,)*
            }

            impl #message_name {
                pub const OPCODE: u16 = #message_opcode;

                /// Calculates how many words (u32) are needed to send this message.
                pub const fn count(&self) -> usize {
                    0usize + #no_arg_size
                    #(#arg_sizes)+*
                }
            }
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
    fn format_name<S: AsRef<str>>(wl_name: S) -> Ident
    {
        format_ident!("{}", snake_to_camel(wl_name.as_ref()))
    }

    fn resolve_path<S: AsRef<str>>(wl_path: S) -> Path
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

    fn name(&self) -> Ident
    {
        Self::format_name(&self.0.name)
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
pub struct Arg<'a>(pub &'a schema::Arg);

impl Arg<'_>
{
    fn name(&self) -> Ident
    {
        format_ident!("{}", self.0.name)
    }

    fn typ(&self) -> Type<'_>
    {
        Type(&self.0.typ)
    }
}

impl ToTokens for Arg<'_>
{
    fn to_tokens(&self, tokens: &mut TokenStream)
    {
        let arg_name = self.name();
        let arg_type = self.typ();

        quote! { #arg_name: #arg_type }.to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub struct ArgSize<'a>(pub &'a schema::Arg);

impl ToTokens for ArgSize<'_>
{
    fn to_tokens(&self, tokens: &mut TokenStream)
    {
        let arg_name = Arg(self.0).name();

        match self.0.typ {
            schema::Type::Int { enu: _ }
            | schema::Type::Uint { enu: _ }
            | schema::Type::Fixed
            | schema::Type::Object {
                interface: _,
                nullable: _,
            }
            | schema::Type::NewId { interface: _ } => quote! { 1usize },
            schema::Type::Fd => quote! { 0usize }, // fd occupies no space on the main transport
            schema::Type::String { nullable: false } => quote! { {
                let len = self.#arg_name.len() + 1; // +1 for the null byte
                1usize + (len + ::core::mem::size_of::<u32>() - 1) / 4
            } }, // TODO: implement
            schema::Type::String { nullable: true } => quote! { {
                1usize + match &self.#arg_name {
                    None => 0usize,
                    Some(arg) => {
                        let len = arg.len() + 1; // +1 for the null byte
                        (len + ::core::mem::size_of::<u32>() - 1) / 4
                    },
                }
            } },
            schema::Type::Array => quote! { { unimplemented!() as usize } }, // TODO: implement
        }
        .to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub struct Description(pub schema::Description);

#[derive(Debug, Clone)]
pub struct Type<'a>(pub &'a schema::Type);

impl ToTokens for Type<'_>
{
    fn to_tokens(&self, tokens: &mut TokenStream)
    {
        match self.0 {
            schema::Type::Int { enu: None } => quote! { i32 },
            schema::Type::Uint { enu: None } => quote! { u32 },
            schema::Type::Int { enu: Some(enu) } | schema::Type::Uint { enu: Some(enu) } => {
                let enum_path = Enum::resolve_path(&enu);
                quote! { #enum_path }
            }
            schema::Type::Fixed => quote! { ::wayland_core::Fixed },
            schema::Type::String { nullable: false } => quote! { String },
            schema::Type::String { nullable: true } => quote! { Option<String> },
            schema::Type::Object {
                interface,
                nullable,
            } => {
                let interface_path = interface
                    .as_ref()
                    .map(InterfaceStruct::resolve_path)
                    .map(|path| quote! { #path })
                    .unwrap_or(quote! { ::wayland_core::Object });
                match nullable {
                    true => quote! { Option<#interface_path> },
                    false => quote! { #interface_path },
                }
            }
            // TODO: find a way to handle new_id
            // schema::Type::NewId { interface } => interface
            //     .as_ref()
            //     .map(InterfaceStruct::resolve_path)
            //     .map(|path| quote! { &mut #path })
            //     .unwrap_or(quote! { &mut ::wayland_core::Object }),
            schema::Type::NewId { interface: _ } => quote! { ::wayland_core::Object },
            schema::Type::Array => quote! { ::wayland_core::Array },
            schema::Type::Fd => quote! { ::wayland_core::Fd },
        }
        .to_tokens(tokens);
    }
}
