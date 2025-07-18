use std::env;
use std::fs::File;
use std::io::BufReader;
use std::panic;
use std::path::PathBuf;
use std::str::FromStr;

use quick_xml::Reader;
use quick_xml::events::Event;
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
    let result = panic::catch_unwind(|| {
        let protocols = from_reader(BufReader::new(file));
        //println!("{:#?}", protocols);
    });
    if result.is_err() {
        println!("{:?}", path);
    }

    // let mut reader = Reader::from_reader(BufReader::new(file));
    // reader.config_mut().trim_text(true);
    // let mut buf = Vec::new();
    // loop {
    //     let event = reader.read_event_into(&mut buf).unwrap();
    //     println!("{:?}", event);
    //     if event == Event::Eof {
    //         break;
    //     }
    // }
}
