use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::quote;
use syn::Lifetime;
use wayland_xml::schema;

use crate::generator::*;

#[derive(Debug, Clone)]
pub struct Type<'a>
{
    pub typ: &'a schema::Type,
    pub ctx: TypeContext,
}

impl Type<'_>
{
    pub fn string(&self) -> TokenStream
    {
        let lt = self.lifetime();
        match self.ctx {
            TypeContext::Struct { .. } => quote! { ::std::borrow::Cow<#lt, str> },
            TypeContext::Function => quote! { &str },
        }
    }

    pub fn array(&self) -> TokenStream
    {
        let lt = self.lifetime();
        match self.ctx {
            TypeContext::Struct { .. } => quote! { ::std::borrow::Cow<#lt, [u8]> },
            TypeContext::Function => quote! { &[u8] },
        }
    }

    pub fn fd(&self) -> TokenStream
    {
        let lt = self.lifetime();
        match self.ctx {
            TypeContext::Struct { request: true, .. } => quote! { ::std::os::fd::BorrowedFd<#lt> },
            TypeContext::Struct { .. } => quote! { ::std::os::fd::OwnedFd },
            TypeContext::Function => quote! { ::std::os::fd::BorrowedFd<'_> },
        }
    }

    pub fn lifetime(&self) -> Option<Lifetime>
    {
        let lifetime = match self.typ {
            schema::Type::String { .. } => match self.ctx {
                TypeContext::Struct { lifetime, .. } => Some(lifetime),
                _ => None,
            },
            schema::Type::Array => match self.ctx {
                TypeContext::Struct { lifetime, .. } => Some(lifetime),
                _ => None,
            },
            schema::Type::Fd => match self.ctx {
                TypeContext::Struct { request, lifetime } if request => Some(lifetime),
                _ => None,
            },
            _ => None,
        };
        lifetime.map(|lifetime| syn::parse_str(lifetime).unwrap())
    }
}

impl ToTokens for Type<'_>
{
    fn to_tokens(&self, tokens: &mut TokenStream)
    {
        let string = self.string();
        let array = self.array();
        let fd = self.fd();

        match self.typ {
            schema::Type::Int { enu: None } => quote! { i32 },
            schema::Type::Uint { enu: None } => quote! { u32 },
            t @ (schema::Type::Int { enu: Some(enu) } | schema::Type::Uint { enu: Some(enu) }) => {
                let enum_path = Enum::resolve_path(&enu);
                match t {
                    schema::Type::Int { .. } => quote! { ::wayland_core::Int<#enum_path> },
                    schema::Type::Uint { .. } => quote! { ::wayland_core::Uint<#enum_path> },
                    _ => unreachable!(),
                }
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
            schema::Type::Fd => quote! { #fd },
        }
        .to_tokens(tokens);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TypeContext
{
    Struct
    {
        // false = event, true = request
        request: bool,
        lifetime: &'static str,
    },
    Function,
}
