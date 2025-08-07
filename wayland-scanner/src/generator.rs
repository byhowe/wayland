use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use wayland_xml::schema;

use crate::util::snake_to_camel;
use crate::wayland::Arg;
use crate::wayland::Entry;
use crate::wayland::Enumeration;
use crate::wayland::Interface;
use crate::wayland::Message;
use crate::wayland::Protocol;

impl<'a> Protocol<'a>
{
    pub fn module_ident(&self) -> syn::Ident
    {
        format_ident!("{}", self.name())
    }

    pub fn generate_module(&self, dependencies: &[syn::Path]) -> TokenStream
    {
        let protocol_module_ident = self.module_ident();
        let protocol_interface_modules = self.interfaces().map(|it| it.generate_module());

        quote! {
            pub mod #protocol_module_ident {
                #(use #dependencies;)*

                #(#protocol_interface_modules)*

                // TODO: Optionally export the interface structs
                // #(#[doc(inline)] pub use #interface_paths;)*
            }
        }
    }
}

impl<'a> Interface<'a>
{
    pub fn format_module_ident(wl_name: &str) -> syn::Ident
    {
        format_ident!("{}", wl_name)
    }

    pub fn format_struct_ident(wl_name: &str) -> syn::Ident
    {
        format_ident!("{}", snake_to_camel(wl_name))
    }

    pub fn module_ident(&self) -> syn::Ident
    {
        Self::format_module_ident(self.name())
    }

    pub fn struct_ident(&self) -> syn::Ident
    {
        Self::format_struct_ident(self.name())
    }

    pub fn resolve_enumeration_struct_path(&self, wl_path: &str) -> syn::Path
    {
        let mut parts = wl_path.split('.').rev();
        let enum_struct_ident = parts.next().map(Self::format_struct_ident).unwrap();
        let interface_module_ident = parts
            .next()
            .map(Interface::format_module_ident)
            .unwrap_or(self.module_ident());
        syn::parse_quote! { #interface_module_ident::#enum_struct_ident }
    }

    pub fn resolve_interface_struct_path(&self, wl_path: &str) -> syn::Path
    {
        let interface_module_ident = Interface::format_module_ident(wl_path);
        let interface_struct_ident = Interface::format_struct_ident(wl_path);
        syn::parse_quote! { #interface_module_ident::#interface_struct_ident }
    }

    pub fn generate_module(&self) -> TokenStream
    {
        let interface_module_ident = self.module_ident();
        let interface_struct = self.generate_struct();
        let interface_enumerations = self.enums().map(|it| it.generate_struct());
        let interface_requests = self.requests().map(|it| it.generate_message_struct());
        let interface_events = self.events().map(|it| it.generate_message_struct());

        quote! {
            pub mod #interface_module_ident {
                #interface_struct

                #(#interface_enumerations)*

                pub mod request {
                    #(#interface_requests)*
                }

                pub mod event {
                    #(#interface_events)*
                }
            }
        }
    }

    pub fn generate_struct(&self) -> TokenStream
    {
        let interface_struct_ident = self.struct_ident();

        quote! {
            #[repr(transparent)]
            #[derive(Copy, Clone, Debug, PartialEq, Eq)]
            pub struct #interface_struct_ident {
                object: ::wayland_core::Object,
            }

            impl ::core::convert::From<#interface_struct_ident> for ::wayland_core::Object {
                fn from(value: #interface_struct_ident) -> Self {
                    value.object()
                }
            }

            impl ::core::convert::From<::wayland_core::Object> for #interface_struct_ident {
                fn from(value: ::wayland_core::Object) -> Self {
                    Self { object: value }
                }
            }

            impl ::wayland_core::Wire for #interface_struct_ident {
                type Output<'buf> = #interface_struct_ident;

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

            impl #interface_struct_ident {
                #[inline(always)]
                pub const fn object(self) -> ::wayland_core::Object {
                    self.object
                }

                pub const fn new(object: ::wayland_core::Object) -> Self {
                    Self { object }
                }

                // TODO: optionally add #(#request_functions)*
            }
        }
    }
}

impl<'a> Enumeration<'a>
{
    pub fn format_struct_ident(wl_name: &str) -> syn::Ident
    {
        format_ident!("{}", snake_to_camel(wl_name))
    }

    pub fn struct_ident(&self) -> syn::Ident
    {
        Self::format_struct_ident(self.name())
    }

    pub fn generate_struct(&self) -> TokenStream
    {
        let enum_struct_ident = self.struct_ident();

        let enum_struct = match self.bitfield() {
            false => self.generate_enum_struct(),
            true => self.generate_bitflags_struct(),
        };

        quote! {
            #enum_struct

            impl ::wayland_core::Enum for #enum_struct_ident {
                fn int(self) -> i32 {
                    self.value_signed()
                }

                fn uint(self) -> u32 {
                    self.value_unsigned()
                }
            }
        }
    }

    pub fn generate_bitflags_struct(&self) -> TokenStream
    {
        let enum_struct_ident = self.struct_ident();
        let enum_wl_name = self.name();
        let interface_wl_name = self.interface().name();

        let entries = self.generate_struct_entries();

        quote! {
            ::bitflags::bitflags! {
                #[derive(Debug, Clone, Copy, PartialEq, Eq)]
                pub struct #enum_struct_ident: u32 {
                    #(const #entries;)*
                }
            }

            impl ::core::convert::TryFrom<u32> for #enum_struct_ident {
                type Error = ::wayland_core::EnumParseError;

                #[inline]
                fn try_from(value: u32) -> ::core::result::Result<Self, Self::Error> {
                    #enum_struct_ident::from_bits(value).ok_or(::wayland_core::EnumParseError {
                        received: value,
                        interface: #interface_wl_name,
                        enu: #enum_wl_name,
                    })
                }
            }

            impl ::core::convert::From<#enum_struct_ident> for u32 {
                #[inline]
                fn from(value: #enum_struct_ident) -> u32 {
                    value.bits()
                }
            }

            impl #enum_struct_ident {
                #[inline]
                pub const fn value_unsigned(self) -> u32 {
                    self.bits()
                }

                #[inline]
                pub const fn value_signed(self) -> i32 {
                    self.bits().cast_signed()
                }
            }
        }
    }

    pub fn generate_enum_struct(&self) -> TokenStream
    {
        let enum_struct_ident = self.struct_ident();
        let enum_wl_name = self.name();
        let interface_wl_name = self.interface().name();

        let entries = self.generate_struct_entries();

        let try_from_arms = self.entries().map(|it| {
            let entry_ident = it.variant_ident();
            let entry_value = it.variant_value();

            quote! { #entry_value => Ok(#enum_struct_ident::#entry_ident) }
        });

        quote! {
            #[repr(u32)]
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            #[non_exhaustive]
            pub enum #enum_struct_ident {
                #(#entries,)*
            }

            impl ::core::convert::TryFrom<u32> for #enum_struct_ident {
                type Error = ::wayland_core::EnumParseError;

                #[inline(always)]
                fn try_from(value: u32) -> ::core::result::Result<Self, Self::Error> {
                    match value {
                        #(#try_from_arms,)*
                        _ => Err(::wayland_core::EnumParseError {
                            received: value,
                            interface: #interface_wl_name,
                            enu: #enum_wl_name,
                        }),
                    }
                }
            }

            impl ::core::convert::From<#enum_struct_ident> for u32 {
                #[inline(always)]
                fn from(value: #enum_struct_ident) -> Self {
                    value as u32
                }
            }

            impl #enum_struct_ident {
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

    pub fn generate_struct_entries(&self) -> impl Iterator<Item = TokenStream>
    {
        self.entries().map(|it| {
            let entry_ident = it.variant_ident();
            let entry_value = it.variant_value();

            quote! { #entry_ident = #entry_value }
        })
    }
}

impl<'a> Entry<'a>
{
    pub fn variant_ident(&self) -> syn::Ident
    {
        // NOTE: Parsing as Ident using syn allows us to check if the string contains a
        // rust keyword so that we can add a prefix to it.
        let variant_ident = snake_to_camel(&self.name());
        if variant_ident.chars().next().unwrap().is_numeric()
            || syn::parse_str::<syn::Ident>(&variant_ident).is_err()
        {
            format_ident!("_{}", variant_ident)
        } else {
            format_ident!("{}", variant_ident)
        }
    }

    pub fn variant_value(&self) -> TokenStream
    {
        self.value().parse::<TokenStream>().unwrap()
    }
}

impl<'a> Message<'a>
{
    pub fn struct_ident(&self) -> syn::Ident
    {
        format_ident!("{}", snake_to_camel(self.name()))
    }

    pub fn generate_message_struct(&self) -> TokenStream
    {
        let struct_ident = self.struct_ident();
        let struct_generics_a = self.generate_lifetime_generics(syn::parse_quote! { 'a });
        let struct_generics_buf = self.generate_lifetime_generics(syn::parse_quote! { 'buf });

        let fields = self.args().map(|it| {
            let field_ident = it.field_ident();
            let field_type = it.field_type(syn::parse_quote! { 'a });

            quote! { #field_ident: #field_type }
        });

        let message_opcode = self.opcode();
        let wire_arguments = self.args().filter(|it| !it.is_fd()).collect::<Vec<_>>();
        let wire_field_idents = wire_arguments
            .iter()
            .map(|it| it.field_ident())
            .collect::<Vec<_>>();
        let wire_field_types = wire_arguments
            .iter()
            .map(|it| it.field_type(syn::parse_quote! { 'a }))
            .collect::<Vec<_>>();

        quote! {
            #[derive(Debug)]
            pub struct #struct_ident #struct_generics_a {
                #(pub #fields,)*
            }

            impl #struct_generics_a ::wayland_core::MessageOpcode for #struct_ident #struct_generics_a {
                const OPCODE: u16 = #message_opcode;
            }

            impl #struct_generics_a ::wayland_core::MessageWire for #struct_ident #struct_generics_a {
                type Output<'buf> = #struct_ident #struct_generics_buf;

                fn message_write(&self, buf: &mut [u32]) {
                    use ::wayland_core::Wire;

                    #(let buf = self.#wire_field_idents.wire_write(buf);)*
                    _ = buf;
                }

                fn message_read_into<'buf>(
                    buf: &'buf[u32],
                    msg: &mut ::std::mem::MaybeUninit<Self::Output<'buf>>
                ) -> Result<(), ::wayland_core::WireError> {
                    use ::wayland_core::Wire;

                    let __ptr = msg.as_mut_ptr();

                    #(let (buf, #wire_field_idents) = <#wire_field_types as Wire>::wire_read(buf)?;)*
                    #(unsafe { (&raw mut ((*__ptr).#wire_field_idents)).write(#wire_field_idents) };)*

                    _ = buf;

                    Ok(())
                }

                fn message_size(&self) -> usize {
                    use ::wayland_core::Wire;
                    0 #(+ self.#wire_field_idents.wire_size())*
                }
            }
        }
    }

    pub fn generate_lifetime_generics(
        &self,
        buf_lifetime: syn::GenericParam,
    ) -> Option<syn::Generics>
    {
        let buf_generic: Option<syn::GenericParam> = self
            .args()
            .find(|it| it.has_lifetime_generic_buf())
            .map(|_| buf_lifetime);
        let fd_generic: Option<syn::GenericParam> = self
            .args()
            .find(|it| it.has_lifetime_generic_fd())
            .map(|_| syn::parse_quote! { 'fd });
        match (buf_generic, fd_generic) {
            (None, None) => None,
            (None, Some(fd)) => Some(syn::parse_quote! { <#fd> }),
            (Some(buf), None) => Some(syn::parse_quote! { <#buf> }),
            (Some(buf), Some(fd)) => Some(syn::parse_quote! { <#buf, #fd> }),
        }
    }
}

impl<'a> Arg<'a>
{
    pub fn field_ident(&self) -> syn::Ident
    {
        format_ident!("{}", self.name())
    }

    pub fn field_type(&self, buf_lifetime: syn::GenericParam) -> TokenStream
    {
        match self.typ() {
            schema::Type::Int { enu: None } => quote! { i32 },
            schema::Type::Uint { enu: None } => quote! { u32 },
            t @ (schema::Type::Int { enu: Some(enu) } | schema::Type::Uint { enu: Some(enu) }) => {
                let int_type = match t {
                    schema::Type::Int { .. } => quote! { ::wayland_core::Int },
                    schema::Type::Uint { .. } => quote! { ::wayland_core::Uint },
                    _ => unreachable!(),
                };
                let struct_path = self
                    .interface()
                    .resolve_enumeration_struct_path(enu.as_str());
                quote! { #int_type <super::super::#struct_path> }
            }
            schema::Type::Fixed => quote! { ::wayland_core::Fixed },
            schema::Type::String { nullable: false } => {
                quote! { ::std::borrow::Cow<#buf_lifetime, str> }
            }
            schema::Type::String { nullable: true } => {
                quote! { Option<::std::borrow::Cow<#buf_lifetime, str>> }
            }
            t @ (schema::Type::Object { interface, .. } | schema::Type::NewId { interface }) => {
                let default_type = match t {
                    schema::Type::Object { .. } => syn::parse_quote! { ::wayland_core::Object },
                    schema::Type::NewId { .. } => syn::parse_quote! { ::wayland_core::NewId },
                    _ => unreachable!(),
                };
                let nullable = match t {
                    schema::Type::Object { nullable, .. } => *nullable,
                    _ => false,
                };
                let struct_path: syn::Path = interface
                    .as_ref()
                    .map(|it| self.interface().resolve_interface_struct_path(it.as_str()))
                    .map(|it| syn::parse_quote! { super::super::#it })
                    .unwrap_or(default_type);
                match nullable {
                    false => quote! { #struct_path },
                    true => quote! { Option<#struct_path> },
                }
            }
            schema::Type::Array => quote! { ::std::borrow::Cow<#buf_lifetime, [u8]> },
            schema::Type::Fd => quote! { ::wayland_core::Fd<'fd> },
        }
    }

    pub fn has_lifetime_generic_buf(&self) -> bool
    {
        match self.typ() {
            schema::Type::String { .. } | schema::Type::Array => true,
            _ => false,
        }
    }

    pub fn has_lifetime_generic_fd(&self) -> bool
    {
        match self.typ() {
            schema::Type::Fd => true,
            _ => false,
        }
    }

    pub fn is_fd(&self) -> bool
    {
        match self.typ() {
            schema::Type::Fd => true,
            _ => false,
        }
    }
}
