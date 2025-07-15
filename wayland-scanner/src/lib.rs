use proc_macro2::TokenStream;

#[proc_macro]
pub fn generate(input: proc_macro::TokenStream) -> proc_macro::TokenStream
{
    let input = proc_macro2::TokenStream::from(input);

    let output: proc_macro2::TokenStream = {
        /* transform input */
        TokenStream::new()
    };

    proc_macro::TokenStream::from(output)
}
