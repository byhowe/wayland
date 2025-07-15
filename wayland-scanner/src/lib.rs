use std::env;
use std::path::PathBuf;

use proc_macro2::TokenStream;
use syn::LitStr;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse_macro_input;

// Path to the .xml file.
struct SpecPath(Option<LitStr>);

impl Parse for SpecPath
{
    fn parse(input: ParseStream) -> syn::Result<Self>
    {
        if input.is_empty() {
            Ok(SpecPath(None))
        } else {
            let path: LitStr = input.parse()?;
            Ok(SpecPath(Some(path)))
        }
    }
}

impl Into<PathBuf> for SpecPath
{
    fn into(self) -> PathBuf
    {
        match self.0 {
            Some(input) => {
                let base: PathBuf = input.value().into();
                if base.is_file() {
                    base
                } else {
                    let mut cargo_dir: PathBuf = env::var_os("CARGO_MANIFEST_DIR")
                        .map(Into::<PathBuf>::into)
                        .unwrap();
                    cargo_dir.push(base);
                    cargo_dir
                }
            }
            None => {
                let mut cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                cargo_dir.push("wayland/protocol/wayland.xml");
                cargo_dir
            }
        }
    }
}

#[proc_macro]
pub fn generate(input: proc_macro::TokenStream) -> proc_macro::TokenStream
{
    let path: PathBuf = parse_macro_input!(input as SpecPath).into();

    panic!("{:?}", path);

    let output: proc_macro2::TokenStream = {
        /* transform input */
        TokenStream::new()
    };

    proc_macro::TokenStream::from(output)
}
