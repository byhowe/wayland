use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::str::FromStr;

use wayland_xml::from_reader;

fn main()
{
    let mut path = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
    path.push("../wayland-scanner/wayland/protocol/wayland.xml");
    let file = File::open(&path).unwrap();
    let protocols = from_reader(BufReader::new(file));
    println!("{:#?}", protocols);
}
