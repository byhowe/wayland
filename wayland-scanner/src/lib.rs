#![feature(trim_prefix_suffix)]
#![feature(exact_size_is_empty)]

use std::env;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use quote::quote;
use syn::LitStr;
use syn::parse_macro_input;

use crate::generator::Protocol;

mod generator;
mod util;

#[proc_macro]
pub fn generate(input: proc_macro::TokenStream) -> proc_macro::TokenStream
{
    let mut path: PathBuf = env::var_os("CARGO_MANIFEST_DIR").unwrap().into();
    let relative: PathBuf = parse_macro_input!(input as LitStr).value().into();
    path.push(&relative);

    let reader = File::open(&path).unwrap();
    let binding = wayland_xml::from_reader(BufReader::new(reader));
    assert!(
        binding.len() == 1,
        "there must be only one protocol per xml file"
    );
    let protocol = Protocol(&binding[0]);

    let output = quote! { #protocol };

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
