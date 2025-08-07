#![allow(dead_code)]

use wayland_xml::schema;

#[derive(Debug, Clone, Copy)]
pub enum MessageType
{
    Request,
    Event,
}

#[derive(Debug, Clone, Copy)]
pub struct Protocol<'a>
{
    pub it: &'a schema::Protocol,
}

impl<'a> Protocol<'a>
{
    pub fn name(&self) -> &'a str
    {
        &self.it.name
    }

    pub fn copyright(&self) -> Option<Copyright<'a>>
    {
        self.it
            .copyright
            .as_ref()
            .map(|it| Copyright { it, parent: *self })
    }

    // pub fn description(&self)

    pub fn interfaces(&self) -> impl Iterator<Item = Interface<'a>> + Clone
    {
        self.it
            .interfaces
            .iter()
            .map(|it| Interface { it, parent: *self })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Copyright<'a>
{
    pub it: &'a schema::Copyright,
    pub parent: Protocol<'a>,
}

impl<'a> Copyright<'a>
{
    pub fn protocol(&self) -> Protocol<'a>
    {
        self.parent
    }

    pub fn content(&self) -> &'a str
    {
        &self.it.content
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Interface<'a>
{
    pub it: &'a schema::Interface,
    pub parent: Protocol<'a>,
}

impl<'a> Interface<'a>
{
    pub fn protocol(&self) -> Protocol<'a>
    {
        self.parent
    }

    pub fn name(&self) -> &'a str
    {
        &self.it.name
    }

    pub fn version(&self) -> u32
    {
        self.it.version
    }

    // pub fn description(&self)

    pub fn requests(&self) -> impl Iterator<Item = Message<'a>> + Clone
    {
        self.it.requests.iter().map(|it| Message {
            it,
            parent: *self,
            ctx: MessageType::Request,
        })
    }

    pub fn events(&self) -> impl Iterator<Item = Message<'a>> + Clone
    {
        self.it.events.iter().map(|it| Message {
            it,
            parent: *self,
            ctx: MessageType::Event,
        })
    }

    pub fn enums(&self) -> impl Iterator<Item = Enumeration<'a>> + Clone
    {
        self.it.enums.iter().map(|it| Enumeration { it, parent: *self })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Message<'a>
{
    pub it: &'a schema::Message,
    pub parent: Interface<'a>,
    pub ctx: MessageType,
}

impl<'a> Message<'a>
{
    pub fn protocol(&self) -> Protocol<'a>
    {
        self.parent.parent
    }

    pub fn interface(&self) -> Interface<'a>
    {
        self.parent
    }

    pub fn name(&self) -> &'a str
    {
        &self.it.name
    }

    pub fn destructor(&self) -> bool
    {
        self.it.destructor
    }

    pub fn since(&self) -> u32
    {
        self.it.since
    }

    pub fn deprecated_since(&self) -> Option<u32>
    {
        self.it.deprecated_since
    }

    //pub fn description(&self) -> Option<Description> {}

    pub fn args(&self) -> impl Iterator<Item = Arg<'a>> + Clone
    {
        self.it.args.iter().map(|it| Arg { it, parent: *self })
    }

    pub fn opcode(&self) -> u16
    {
        self.it.opcode
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Enumeration<'a>
{
    pub it: &'a schema::Enum,
    pub parent: Interface<'a>,
}

impl<'a> Enumeration<'a>
{
    pub fn protocol(&self) -> Protocol<'a>
    {
        self.parent.parent
    }

    pub fn interface(&self) -> Interface<'a>
    {
        self.parent
    }

    pub fn name(&self) -> &'a str
    {
        &self.it.name
    }

    pub fn since(&self) -> u32
    {
        self.it.since
    }

    pub fn bitfield(&self) -> bool
    {
        self.it.bitfield
    }

    // pub fn description(&self) -> Option<Description> {}

    pub fn entries(&self) -> impl Iterator<Item = Entry<'a>> + Clone
    {
        self.it.entries.iter().map(|it| Entry { it, parent: *self })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Entry<'a>
{
    pub it: &'a schema::Entry,
    pub parent: Enumeration<'a>,
}

impl<'a> Entry<'a>
{
    pub fn protocol(&self) -> Protocol<'a>
    {
        self.parent.parent.parent
    }

    pub fn interface(&self) -> Interface<'a>
    {
        self.parent.parent
    }

    pub fn enu(&self) -> Enumeration<'a>
    {
        self.parent
    }

    pub fn name(&self) -> &'a str
    {
        &self.it.name
    }

    pub fn value(&self) -> &'a str
    {
        &self.it.value
    }

    pub fn summary(&self) -> Option<&'a str>
    {
        self.it.summary.as_ref().map(String::as_str)
    }

    pub fn since(&self) -> u32
    {
        self.it.since
    }

    pub fn deprecated_since(&self) -> Option<u32>
    {
        self.it.deprecated_since
    }

    // pub fn description(&self) -> Option<Description> {}
}

#[derive(Debug, Clone, Copy)]
pub struct Arg<'a>
{
    pub it: &'a schema::Arg,
    pub parent: Message<'a>,
}

impl<'a> Arg<'a>
{
    pub fn protocol(&self) -> Protocol<'a>
    {
        self.parent.parent.parent
    }

    pub fn interface(&self) -> Interface<'a>
    {
        self.parent.parent
    }

    pub fn message(&self) -> Message<'a>
    {
        self.parent
    }

    pub fn name(&self) -> &'a str
    {
        &self.it.name
    }

    pub fn summary(&self) -> Option<&'a str>
    {
        self.it.summary.as_ref().map(String::as_str)
    }

    // pub fn description(&self) -> Option<Description> {}

    pub fn typ(&self) -> &'a schema::Type
    {
        &self.it.typ
    }
}

// #[derive(Debug, Clone, Copy)]
// pub struct Description<'a>
// {
//     pub it: &'a schema::Description,
//     pub parent: (),
// }

// #[derive(Debug, Clone, Copy)]
// pub enum Type
// {
//     Int
//     {
//         enu: Option<String>,
//     },
//     Uint
//     {
//         enu: Option<String>,
//     },
//     Fixed,
//     String
//     {
//         nullable: bool,
//     },
//     Object
//     {
//         interface: Option<String>,
//         nullable: bool,
//     },
//     NewId
//     {
//         interface: Option<String>,
//     },
//     Array,
//     Fd,
// }
