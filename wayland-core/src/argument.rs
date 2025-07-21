use crate::Array;
use crate::Fd;
use crate::Fixed;
use crate::Object;
use crate::write_fixed;
use crate::write_int;
use crate::write_new_id;
use crate::write_object;
use crate::write_object_nullable;
use crate::write_string;
use crate::write_string_nullable;
use crate::write_uint;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Argument<'msg>
{
    Int(i32),
    Uint(u32),
    Fixed(Fixed),
    String(&'msg str),
    StringNullable(Option<&'msg str>),
    Object(Object),
    ObjectNullable(Option<Object>),
    NewId(Object),
    Array(Array),
    Fd(Fd),
}

impl Argument<'_>
{
    /// Calculate the size in words (u32) needed for this argument.
    #[inline(always)]
    pub const fn size(self) -> usize
    {
        match self {
            Argument::Int(_) | Argument::Uint(_) | Argument::Fixed(_) => 1,
            Argument::Object(_) | Argument::ObjectNullable(_) | Argument::NewId(_) => 1,
            // val.len() + 1 is the size needed for a null-terminated string.
            // (x + 3) / 4 is the number of words needed to store the x bytes.
            // +1 at the end is for storing the size of the string.
            Argument::String(v) | Argument::StringNullable(Some(v)) => (v.len() + 1 + 3) / 4 + 1,
            // only occupy one word for the length field.
            Argument::StringNullable(None) => 1,
            Argument::Array(_) => unimplemented!(),
            // file descriptors do not occupy any space on the main transport.
            Argument::Fd(_) => 0,
        }
    }

    /// Write this argument to the buffer, returning the remaining buffer.
    #[inline(always)]
    pub fn write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        match self {
            Argument::Int(v) => write_int(buf, *v),
            Argument::Uint(v) => write_uint(buf, *v),
            Argument::Fixed(v) => write_fixed(buf, *v),
            Argument::String(v) => write_string(buf, *v),
            Argument::StringNullable(v) => write_string_nullable(buf, *v),
            Argument::Object(v) => write_object(buf, *v),
            Argument::ObjectNullable(v) => write_object_nullable(buf, *v),
            Argument::NewId(v) => write_new_id(buf, *v),
            Argument::Array(_) => unimplemented!(),
            Argument::Fd(_) => unimplemented!(),
        }
    }
}

pub trait Arguments
{
    fn arguments(&self) -> impl Iterator<Item = Argument<'_>>;
}
