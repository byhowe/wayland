use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::quote;
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
        match self.ctx {
            TypeContext::Struct => quote! { ::std::borrow::Cow<'a, str> },
            TypeContext::Function => quote! { &str },
        }
    }

    pub fn array(&self) -> TokenStream
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

#[derive(Debug, Clone, Copy)]
pub enum TypeContext
{
    Struct,
    Function,
}
