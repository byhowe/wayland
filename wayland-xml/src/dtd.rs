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
    pub version: String, // FIX: parse as int
    pub description: Option<Description>,
    pub requests: Vec<Request>,
    pub events: Vec<Event>,
    pub enums: Vec<Enum>,
}

#[derive(Debug, Clone)]
pub struct Request
{
    pub name: String,
    pub kind: Option<String>,  // type
    pub since: Option<String>, // FIX: parse as int
    pub deprecated_since: Option<String>,
    pub description: Option<Description>,
    pub args: Vec<Arg>,
}

#[derive(Debug, Clone)]
pub struct Event
{
    pub name: String,
    pub kind: Option<String>,             // type
    pub since: Option<String>,            // FIX: parse as int
    pub deprecated_since: Option<String>, // FIX: parse as int
    pub description: Option<Description>,
    pub args: Vec<Arg>,
}

#[derive(Debug, Clone)]
pub struct Enum
{
    pub name: String,
    pub since: Option<String>,    // FIX: parse as int
    pub bitfield: Option<String>, // FIX: parse as bool
    pub description: Option<Description>,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone)]
pub struct Entry
{
    pub name: String,
    pub value: String,
    pub summary: Option<String>,
    pub since: Option<String>,            // FIX: parse as int
    pub deprecated_since: Option<String>, // FIX: parse as int
    pub description: Option<Description>,
}

#[derive(Debug, Clone)]
pub struct Arg
{
    pub name: String,
    pub kind: String, // type
    pub summary: Option<String>,
    pub interface: Option<String>,
    pub allow_null: Option<String>,
    pub interface_enum: Option<String>, // enum
    pub description: Option<Description>,
}

#[derive(Debug, Clone)]
pub struct Description
{
    pub summary: String,
    pub content: Option<String>,
}
