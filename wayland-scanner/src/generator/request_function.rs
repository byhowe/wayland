use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::format_ident;
use quote::quote;
use syn::Ident;
use wayland_xml::schema;

use crate::generator::*;

#[derive(Debug, Clone)]
pub struct RequestFunction<'a>(pub &'a schema::Message);

impl RequestFunction<'_>
{
    pub fn name(&self) -> Ident
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
