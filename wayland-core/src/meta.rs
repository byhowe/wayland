#[derive(Debug, Clone)]
pub struct Protocol
{
    pub name: &'static str,
    pub interfaces: &'static [&'static Interface],
}

#[derive(Debug, Clone)]
pub struct Interface
{
    pub name: &'static str,
    pub version: u32,
    pub requests: &'static [&'static Message],
    pub events: &'static [&'static Message],
    pub enums: &'static [&'static Enum],
}

#[derive(Debug, Clone)]
pub struct Message
{
    pub name: &'static str,
    pub destructor: bool,
    pub since: u32,
    pub deprecated_since: Option<u32>,
    pub args: &'static [Arg],
}

#[derive(Debug, Clone)]
pub struct Enum
{
    pub name: &'static str,
    pub since: u32,
    pub bitfield: bool,
    pub entries: &'static [Entry],
}

#[derive(Debug, Clone)]
pub struct Entry
{
    pub name: &'static str,
    pub value: &'static str,
    pub since: u32,
    pub deprecated_since: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Arg
{
    pub name: &'static str,
    pub typ: Type,
    pub interface: Option<&'static Interface>,
    pub allow_null: bool,
    pub enu: Option<&'static Enum>,
}

#[derive(Debug, Clone)]
pub enum Type
{
    Int,
    Uint,
    Fixed,
    String,
    Object,
    NewId,
    Array,
    Fd,
}
