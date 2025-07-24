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
        let request_args = self.0.args.iter().map(|arg| ArgField {
            arg,
            ctx: TypeContext::Function,
        });

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

    fn generics(&self) -> Option<TokenStream>
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
        let message_generics = self.generics();
        let message_args = self.0.args.iter().map(|arg| ArgField {
            arg,
            ctx: TypeContext::Struct,
        });
        let message_opcode = self.0.opcode;

        quote! {
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct #message_name #message_generics {
                #(pub #message_args,)*
            }

            impl #message_generics ::wayland_core::Opcode for #message_name # message_generics {
                const OPCODE: u16 = #message_opcode;
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
pub struct ArgField<'a>
{
    pub arg: &'a schema::Arg,
    pub ctx: TypeContext,
}

impl ArgField<'_>
{
    fn name(&self) -> Ident
    {
        format_ident!("{}", self.arg.name)
    }

    fn typ(&self) -> Type<'_>
    {
        Type {
            typ: &self.arg.typ,
            ctx: self.ctx,
        }
    }
}

impl ToTokens for ArgField<'_>
{
    fn to_tokens(&self, tokens: &mut TokenStream)
    {
        let arg_name = self.name();
        let arg_type = self.typ();

        quote! { #arg_name: #arg_type }.to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub struct Description(pub schema::Description);

#[derive(Debug, Clone, Copy)]
pub enum TypeContext
{
    Struct,
    Function,
}

#[derive(Debug, Clone)]
pub struct Type<'a>
{
    pub typ: &'a schema::Type,
    pub ctx: TypeContext,
}

impl Type<'_>
{
    fn string(&self) -> TokenStream
    {
        match self.ctx {
            TypeContext::Struct => quote! { ::std::borrow::Cow<'a, str> },
            TypeContext::Function => quote! { &str },
        }
    }

    fn array(&self) -> TokenStream
    {
        match self.ctx {
            TypeContext::Struct => quote! { ::std::borrow::Cow<'a, [u8]> },
            TypeContext::Function => quote! { &[u8] },
        }
    }
}

impl ToTokens for Type<'_>
{
    fn to_tokens(&self, tokens: &mut TokenStream)
    {
        let string = self.string();
        let array = self.array();

        match self.typ {
            schema::Type::Int { enu: None } => quote! { i32 },
            schema::Type::Uint { enu: None } => quote! { u32 },
            schema::Type::Int { enu: Some(enu) } | schema::Type::Uint { enu: Some(enu) } => {
                let enum_path = Enum::resolve_path(&enu);
                quote! { #enum_path }
            }
            schema::Type::Fixed => quote! { ::wayland_core::Fixed },
            schema::Type::String { nullable: false } => quote! { #string },
            schema::Type::String { nullable: true } => quote! { Option<#string> },
            t @ (schema::Type::Object { interface, .. } | schema::Type::NewId { interface }) => {
                let nullable = match t {
                    schema::Type::Object { nullable, .. } => *nullable,
                    schema::Type::NewId { .. } => false,
                    _ => unreachable!(),
                };
                let arg_type = interface
                    .as_ref()
                    .map(InterfaceStruct::resolve_path)
                    .map(|path| quote! { #path })
                    .unwrap_or(quote! { ::wayland_core::Object });
                match nullable {
                    false => quote! { #arg_type },
                    true => quote! { Option<#arg_type> },
                }
            }
            schema::Type::Array => quote! { #array },
            schema::Type::Fd => quote! { ::wayland_core::Fd },
        }
        .to_tokens(tokens);
    }
}
