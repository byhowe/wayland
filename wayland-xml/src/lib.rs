use std::io::BufRead;

use quick_xml::Reader;
use quick_xml::events::Event;

pub mod dtd;
pub mod reader;

pub fn from_reader<R: BufRead>(reader: R) -> Vec<dtd::Protocol>
{
    let mut reader = Reader::from_reader(reader);
    reader.config_mut().trim_text(true);
    reader.config_mut().enable_all_checks(true);

    let mut protocols = Vec::new();

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf).unwrap() {
            Event::Decl(_) => {}
            ref root @ Event::Start(ref data) if data.name().as_ref() == b"protocol" => {
                protocols.push(dtd::Protocol::from_reader(&mut reader, root));
            }
            Event::Eof => break,
            event => panic!("unexpected event: {:?}", event),
        }
        buf.clear();
    }

    protocols
}
