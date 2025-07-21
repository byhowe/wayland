use std::num::NonZero;

pub type Fixed = i32;

pub type Object = NonZero<u32>;

pub type Array = ();

pub type Fd = i32;

#[derive(Debug, Clone, Copy)]
pub struct Header
{
    pub object: Object,
    pub size: u16,
    pub opcode: u16,
}

impl Header
{
    #[inline(always)]
    pub fn new<O: Into<Object>>(object: O, size: u16, opcode: u16) -> Self
    {
        Header {
            object: object.into(),
            size,
            opcode,
        }
    }
}
