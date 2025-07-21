use std::env;
use std::fs::File;
use std::io::BufReader;
use std::panic;
use std::path::PathBuf;

use wayland_xml::from_reader;

fn main()
{
    let args: Vec<String> = env::args().collect();
    let Some(path) = args.into_iter().nth(1) else {
        panic!("expected path to the xml file");
    };
    let path = PathBuf::from(path);
    let file = File::open(&path).unwrap();
    let result = panic::catch_unwind(|| {
        let protocols = from_reader(BufReader::new(file));
        println!("{:#?}", protocols);
    });
    if result.is_err() {
        println!("{:?}", path);
    }
}
