//! Rust representation of the XML specification as described in <https://gitlab.freedesktop.org/wayland/wayland/-/blob/main/protocol/wayland.dtd>.

#[derive(Debug, Clone)]
pub struct Protocol
{
    pub name: String,
    pub copyright: Option<Copyright>,
    pub description: Option<Description>,
    pub interfaces: Vec<Interface>,
}

#[derive(Debug, Clone)]
pub struct Copyright
{
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct Interface
{
    pub name: String,
    pub version: u32,
    pub frozen: bool,
    pub description: Option<Description>,
    pub requests: Vec<Message>,
    pub events: Vec<Message>,
    pub enums: Vec<Enum>,
}

#[derive(Debug, Clone)]
pub struct Message
{
    pub name: String,
    pub destructor: bool,
    pub since: u32,
    pub deprecated_since: Option<u32>,
    pub description: Option<Description>,
    pub args: Vec<Arg>,
    pub opcode: u16,
}

#[derive(Debug, Clone)]
pub struct Enum
{
    pub name: String,
    pub since: u32,
    pub bitfield: bool,
    pub description: Option<Description>,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone)]
pub struct Entry
{
    pub name: String,
    pub value: String,
    pub summary: Option<String>,
    pub since: u32,
    pub deprecated_since: Option<u32>,
    pub description: Option<Description>,
}

#[derive(Debug, Clone)]
pub struct Arg
{
    pub name: String,
    pub summary: Option<String>,
    pub description: Option<Description>,
    pub typ: Type,
}

#[derive(Debug, Clone)]
pub struct Description
{
    pub summary: String,
    pub content: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Type
{
    Int
    {
        enu: Option<String>,
    },
    Uint
    {
        enu: Option<String>,
    },
    Fixed,
    String
    {
        nullable: bool,
    },
    Object
    {
        interface: Option<String>,
        nullable: bool,
    },
    NewId
    {
        interface: Option<String>,
    },
    Array,
    Fd,
}
