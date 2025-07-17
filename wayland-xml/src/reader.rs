use std::io::BufRead;
use std::str::FromStr;

use quick_xml::Reader;
use quick_xml::escape::resolve_xml_entity;
use quick_xml::events::Event;
use quick_xml::events::attributes::Attribute;

use crate::schema::Arg;
use crate::schema::Copyright;
use crate::schema::Description;
use crate::schema::Entry;
use crate::schema::Enum;
use crate::schema::Interface;
use crate::schema::Message;
use crate::schema::Protocol;
use crate::schema::Type;

impl Protocol
{
    pub fn from_reader<R: BufRead>(reader: &mut Reader<R>, root: &Event<'_>) -> Self
    {
        let mut name = None;

        let inner = match root {
            Event::Start(data) => data,
            _ => panic!("unexpected event: {:?}", root),
        };

        for attr in inner.attributes().map(Result::unwrap) {
            let Attribute { key: k, value: v } = attr;
            match k.0 {
                b"name" => name = Some(str::from_utf8(&v).unwrap().to_string()),
                _ => panic!("unexpected attribute: {:?}", Attribute { key: k, value: v }),
            }
        }

        let mut copyright = None;
        let mut description = None;
        let mut interfaces = Vec::new();

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf).unwrap() {
                ref root @ Event::Start(ref data) if data.name().as_ref() == b"copyright" => {
                    assert!(copyright.is_none(), "unexpected event: {:?}", root);
                    copyright = Some(Copyright::from_reader(reader, root));
                }
                ref root @ Event::Start(ref data) if data.name().as_ref() == b"description" => {
                    assert!(description.is_none(), "unexpected event: {:?}", root);
                    description = Some(Description::from_reader(reader, root));
                }
                ref root @ Event::Start(ref data) if data.name().as_ref() == b"interface" => {
                    interfaces.push(Interface::from_reader(reader, root));
                }
                Event::End(ref data) if data.name().as_ref() == b"protocol" => break,
                Event::Comment(_) => {}
                event => panic!("unexpected event: {:?}", event),
            }
            buf.clear();
        }

        assert!(
            !interfaces.is_empty(),
            "at least one interface must be present"
        );

        Self {
            name: name.unwrap(),
            copyright,
            description,
            interfaces,
        }
    }
}

impl Copyright
{
    pub fn from_reader<R: BufRead>(reader: &mut Reader<R>, root: &Event<'_>) -> Self
    {
        let inner = match root {
            Event::Start(data) => data,
            _ => panic!("unexpected event: {:?}", root),
        };

        // for consistency
        for attr in inner.attributes().map(Result::unwrap) {
            let Attribute { key: k, value: v } = attr;
            match k.0 {
                _ => panic!("unexpected attribute: {:?}", Attribute { key: k, value: v }),
            }
        }

        let mut content = None;

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf).unwrap() {
                Event::Text(ref data) => content
                    .get_or_insert(String::new())
                    .push_str(str::from_utf8(&data).unwrap()),
                Event::GeneralRef(ref data) => content
                    .get_or_insert(String::new())
                    .push_str(resolve_xml_entity(str::from_utf8(&data).unwrap()).unwrap()),
                Event::End(ref data) if data.name().as_ref() == b"copyright" => break,
                event => panic!("unexpected event: {:?}", event),
            }
            buf.clear();
        }

        Self {
            content: content.unwrap(),
        }
    }
}

impl Interface
{
    pub fn from_reader<R: BufRead>(reader: &mut Reader<R>, root: &Event<'_>) -> Self
    {
        let mut name = None;
        let mut version = None;

        let inner = match root {
            Event::Start(data) => data,
            _ => panic!("unexpected event: {:?}", root),
        };

        for attr in inner.attributes().map(Result::unwrap) {
            let Attribute { key: k, value: v } = attr;
            match k.0 {
                b"name" => name = Some(str::from_utf8(&v).unwrap().to_string()),
                b"version" => version = Some(str::from_utf8(&v).unwrap().parse().unwrap()),
                _ => panic!("unexpected attribute: {:?}", Attribute { key: k, value: v }),
            }
        }

        let mut description = None;
        let mut requests = Vec::new();
        let mut events = Vec::new();
        let mut enums = Vec::new();

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf).unwrap() {
                ref root @ Event::Start(ref data) if data.name().as_ref() == b"description" => {
                    assert!(description.is_none(), "unexpected event: {:?}", root);
                    description = Some(Description::from_reader(reader, root));
                }
                ref root @ Event::Start(ref data) if data.name().as_ref() == b"request" => {
                    requests.push(Message::from_reader(reader, root));
                }
                ref root @ Event::Start(ref data) if data.name().as_ref() == b"event" => {
                    events.push(Message::from_reader(reader, root));
                }
                ref root @ Event::Start(ref data) if data.name().as_ref() == b"enum" => {
                    enums.push(Enum::from_reader(reader, root));
                }
                Event::End(ref data) if data.name().as_ref() == b"interface" => break,
                Event::Comment(_) => {}
                event => panic!("unexpected event: {:?}", event),
            }
            buf.clear();
        }

        assert!(
            requests.len() + events.len() + enums.len() > 0,
            "at least one item must be present"
        );

        Self {
            name: name.unwrap(),
            version: version.unwrap(),
            description,
            requests,
            events,
            enums,
        }
    }
}

impl Message
{
    pub fn from_reader<R: BufRead>(reader: &mut Reader<R>, root: &Event<'_>) -> Self
    {
        let mut name = None;
        let mut destructor = false;
        let mut since = 1;
        let mut deprecated_since = None;

        let inner = match root {
            Event::Start(data) => data,
            _ => panic!("unexpected event: {:?}", root),
        };

        for attr in inner.attributes().map(Result::unwrap) {
            let Attribute { key: k, value: v } = attr;
            match k.0 {
                b"name" => name = Some(str::from_utf8(&v).unwrap().to_string()),
                b"type" => {
                    destructor = match v.as_ref() {
                        b"destructor" => true,
                        _ => panic!("unexpected attribute: {:?}", Attribute { key: k, value: v }),
                    }
                }
                b"since" => since = str::from_utf8(&v).unwrap().parse().unwrap(),
                b"deprecated-since" => {
                    deprecated_since = Some(str::from_utf8(&v).unwrap().parse().unwrap())
                }
                _ => panic!("unexpected attribute: {:?}", Attribute { key: k, value: v }),
            }
        }

        let mut description = None;
        let mut args = Vec::new();

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf).unwrap() {
                ref root @ Event::Start(ref data) if data.name().as_ref() == b"description" => {
                    assert!(description.is_none(), "unexpected event: {:?}", root);
                    description = Some(Description::from_reader(reader, root));
                }
                ref root @ Event::Start(ref data) if data.name().as_ref() == b"arg" => {
                    args.push(Arg::from_reader(reader, root));
                }
                Event::End(ref data) if data.name() == inner.name() => break,
                Event::Comment(_) => {}
                event => panic!("unexpected event: {:?}", event),
            }
            buf.clear();
        }

        Self {
            name: name.unwrap(),
            destructor,
            since,
            deprecated_since,
            description,
            args,
        }
    }
}

impl Enum
{
    pub fn from_reader<R: BufRead>(reader: &mut Reader<R>, root: &Event<'_>) -> Self
    {
        let mut name = None;
        let mut since = 1;
        let mut bitfield = false;

        let inner = match root {
            Event::Start(data) => data,
            _ => panic!("unexpected event: {:?}", root),
        };

        for attr in inner.attributes().map(Result::unwrap) {
            let Attribute { key: k, value: v } = attr;
            match k.0 {
                b"name" => name = Some(str::from_utf8(&v).unwrap().to_string()),
                b"since" => since = str::from_utf8(&v).unwrap().parse().unwrap(),
                b"bitfield" if v.as_ref() == b"true" => bitfield = true,
                _ => panic!("unexpected attribute: {:?}", Attribute { key: k, value: v }),
            }
        }

        let mut description = None;
        let mut entries = Vec::new();

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf).unwrap() {
                ref root @ Event::Start(ref data) if data.name().as_ref() == b"description" => {
                    assert!(description.is_none(), "unexpected event: {:?}", root);
                    description = Some(Description::from_reader(reader, root));
                }
                ref root @ Event::Start(ref data) if data.name().as_ref() == b"entry" => {
                    entries.push(Entry::from_reader(reader, root));
                }
                Event::End(ref data) if data.name().as_ref() == b"enum" => break,
                Event::Comment(_) => {}
                event => panic!("unexpected event: {:?}", event),
            }
            buf.clear();
        }

        Self {
            name: name.unwrap(),
            since,
            bitfield,
            description,
            entries,
        }
    }
}

impl Entry
{
    pub fn from_reader<R: BufRead>(reader: &mut Reader<R>, root: &Event<'_>) -> Self
    {
        let mut name = None;
        let mut value = None;
        let mut summary = None;
        let mut since = 1;
        let mut deprecated_since = None;

        let inner = match root {
            Event::Start(data) => data,
            _ => panic!("unexpected event: {:?}", root),
        };

        for attr in inner.attributes().map(Result::unwrap) {
            let Attribute { key: k, value: v } = attr;
            match k.0 {
                b"name" => name = Some(str::from_utf8(&v).unwrap().to_string()),
                b"value" => value = Some(str::from_utf8(&v).unwrap().to_string()),
                b"summary" => summary = Some(str::from_utf8(&v).unwrap().to_string()),
                b"since" => since = str::from_utf8(&v).unwrap().parse().unwrap(),
                b"deprecated-since" => {
                    deprecated_since = Some(str::from_utf8(&v).unwrap().parse().unwrap());
                }
                _ => panic!("unexpected attribute: {:?}", Attribute { key: k, value: v }),
            }
        }

        let mut description = None;

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf).unwrap() {
                ref root @ Event::Start(ref data) if data.name().as_ref() == b"description" => {
                    assert!(description.is_none(), "unexpected event: {:?}", root);
                    description = Some(Description::from_reader(reader, root))
                }
                Event::End(ref data) if data.name().as_ref() == b"entry" => break,
                event => panic!("unexpected event: {:?}", event),
            }
            buf.clear();
        }

        Self {
            name: name.unwrap(),
            value: value.unwrap(),
            summary,
            since,
            deprecated_since,
            description,
        }
    }
}

impl Arg
{
    pub fn from_reader<R: BufRead>(reader: &mut Reader<R>, root: &Event<'_>) -> Self
    {
        let mut name = None;
        let mut kind = None;
        let mut summary = None;
        let mut interface = None;
        let mut allow_null = false;
        let mut interface_enum = None;

        let inner = match root {
            Event::Start(data) => data,
            _ => panic!("unexpected event: {:?}", root),
        };

        for attr in inner.attributes().map(Result::unwrap) {
            let Attribute { key: k, value: v } = attr;
            match k.0 {
                b"name" => name = Some(str::from_utf8(&v).unwrap().to_string()),
                b"type" => kind = Some(str::from_utf8(&v).unwrap().parse().unwrap()),
                b"summary" => summary = Some(str::from_utf8(&v).unwrap().to_string()),
                b"interface" => interface = Some(str::from_utf8(&v).unwrap().to_string()),
                b"allow-null" => {
                    allow_null = match v.as_ref() {
                        b"true" => true,
                        _ => panic!("unexpected attribute: {:?}", Attribute { key: k, value: v }),
                    }
                }
                b"enum" => interface_enum = Some(str::from_utf8(&v).unwrap().to_string()),
                _ => panic!("unexpected attribute: {:?}", Attribute { key: k, value: v }),
            }
        }

        let mut description = None;

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf).unwrap() {
                ref root @ Event::Start(ref data) if data.name().as_ref() == b"description" => {
                    assert!(description.is_none(), "unexpected event: {:?}", root);
                    description = Some(Description::from_reader(reader, root))
                }
                Event::End(ref data) if data.name().as_ref() == b"arg" => break,
                event => panic!("unexpected event: {:?}", event),
            }
            buf.clear();
        }

        Self {
            name: name.unwrap(),
            kind: kind.unwrap(),
            summary,
            interface,
            allow_null,
            interface_enum,
            description,
        }
    }
}

impl Description
{
    pub fn from_reader<R: BufRead>(reader: &mut Reader<R>, root: &Event<'_>) -> Self
    {
        let mut summary = None;

        let inner = match root {
            Event::Start(data) => data,
            _ => panic!("unexpected event: {:?}", root),
        };

        for attr in inner.attributes().map(Result::unwrap) {
            let Attribute { key: k, value: v } = attr;
            match k.0 {
                b"summary" => summary = Some(str::from_utf8(&v).unwrap().to_string()),
                _ => panic!("unexpected attribute: {:?}", Attribute { key: k, value: v }),
            }
        }

        let mut content = None;

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf).unwrap() {
                Event::Text(ref data) => {
                    content
                        .get_or_insert(String::new())
                        .push_str(str::from_utf8(&data).unwrap());
                }
                Event::End(ref data) if data.name().as_ref() == b"description" => break,
                Event::Comment(_) => {}
                event => panic!("unexpected event: {:?}", event),
            }
            buf.clear();
        }

        Self {
            summary: summary.unwrap(),
            content,
        }
    }
}

impl FromStr for Type
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        match s {
            "int" => Ok(Type::Int),
            "uint" => Ok(Type::Uint),
            "fixed" => Ok(Type::Fixed),
            "string" => Ok(Type::String),
            "object" => Ok(Type::Object),
            "new_id" => Ok(Type::NewId),
            "array" => Ok(Type::Array),
            "fd" => Ok(Type::Fd),
            _ => Err(format!("unknown type: {}", s)),
        }
    }
}
