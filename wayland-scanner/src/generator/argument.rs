use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::format_ident;
use quote::quote;
use syn::Ident;
use wayland_xml::schema;

use crate::generator::*;

#[derive(Debug, Clone)]
pub struct ArgField<'a>
{
    pub arg: &'a schema::Arg,
    pub ctx: TypeContext,
}

impl ArgField<'_>
{
    pub fn name(&self) -> Ident
    {
        format_ident!("{}", self.arg.name)
    }

    pub fn typ(&self) -> Type<'_>
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
