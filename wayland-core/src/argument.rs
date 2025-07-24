use crate::Fd;
use crate::Fixed;
use crate::NewId;
use crate::Object;
use crate::Wire;

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
    NewId(NewId),
    Array(&'msg [u8]),
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
            Argument::Int(v) => v.write(buf),
            Argument::Uint(v) => v.write(buf),
            Argument::Fixed(v) => v.write(buf),
            Argument::String(v) => v.write(buf),
            Argument::StringNullable(v) => v.write(buf),
            Argument::Object(v) => v.write(buf),
            Argument::ObjectNullable(v) => v.write(buf),
            Argument::NewId(v) => v.write(buf),
            Argument::Array(v) => v.write(buf),
            // The Wire trait is not implemented by Fd, since it is not sent through the main
            // transport.
            Argument::Fd(_) => buf,
        }
    }
}

pub trait Arguments
{
    fn arguments(&self) -> impl Iterator<Item = Argument<'_>>;
}
