#![feature(trim_prefix_suffix)]
#![feature(exact_size_is_empty)]

use std::env;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use syn::Token;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse_macro_input;

//use crate::generator::Protocol;
use crate::wayland::Protocol;

mod generator;
mod util;
mod wayland;

// This helps rust-analyzer understand the macro syntax
#[cfg(false)] // This ensures it's never compiled
macro_rules! generate {
    ($path:literal) => {};
    ($path:literal, dependencies: [$($dep:path),* $(,)?]) => {};
    ($path:literal, $($field:ident: $value:expr),* $(,)?) => {};
}

#[derive(Debug)]
struct GenerateMacroInput
{
    pub path: syn::LitStr,
    pub dependencies: Vec<syn::Path>,
}

impl Parse for GenerateMacroInput
{
    fn parse(input: ParseStream) -> syn::Result<Self>
    {
        // Parse the mandatory path
        let path: syn::LitStr = input.parse()?;

        let mut dependencies = Vec::new();

        // Parse optional named fields
        while input.peek(Token![,]) && !input.is_empty() {
            input.parse::<Token![,]>()?;

            if input.is_empty() {
                break; // trailing comma
            }

            let field_name: syn::Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match field_name.to_string().as_str() {
                "dependencies" => {
                    let deps_array: syn::ExprArray = input.parse()?;
                    dependencies.extend(deps_array.elems.into_iter().map(|it| match it {
                        syn::Expr::Path(path) => path.path,
                        _ => panic!("expected a path"),
                    }));
                }
                _ => {
                    // Skip unknown fields for forward compatibility
                    let _: syn::Expr = input.parse()?;
                }
            }
        }

        Ok(GenerateMacroInput { path, dependencies })
    }
}

#[proc_macro]
pub fn generate(input: proc_macro::TokenStream) -> proc_macro::TokenStream
{
    let macro_input = syn::parse_macro_input!(input as GenerateMacroInput);

    let mut path: PathBuf = env::var_os("CARGO_MANIFEST_DIR").unwrap().into();
    let relative: PathBuf = macro_input.path.value().into();
    path.push(&relative);

    let reader = File::open(&path).unwrap();
    let binding = wayland_xml::from_reader(BufReader::new(reader));
    assert!(
        binding.len() == 1,
        "there must be only one protocol per xml file"
    );

    let protocol = Protocol { it: &binding[0] };
    let output = protocol.generate_module(&macro_input.dependencies);

    // Dump generated code to a file in target/wayland-gen
    let out_dir = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("target"));

    let file_stem = path.file_stem().unwrap().to_string_lossy();
    let out_path = out_dir.join("wayland-gen").join(format!("{file_stem}.rs"));

    fs::create_dir_all(out_path.parent().unwrap()).unwrap();
    fs::write(&out_path, output.to_string()).expect("Failed to write generated code");

    // eprintln!("Generated code written to: {}", out_path.display());

    output.into()
}
