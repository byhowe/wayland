use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::str::FromStr;

use wayland_xml::from_reader;

fn main()
{
    let args: Vec<String> = env::args().collect();
    let path = if let Some(path) = args.iter().nth(1) {
        PathBuf::from_str(path).unwrap()
    } else {
        let mut path = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
        path.push("../wayland-scanner/wayland/protocol/wayland.xml");
        path
    };
    let file = File::open(&path).unwrap();
    let protocols = from_reader(BufReader::new(file));
    println!("{:#?}", protocols);
}
