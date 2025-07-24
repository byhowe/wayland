use crate::Argument;
use crate::Arguments;

pub trait Message
{
    const OPCODE: u16;

    /// Calculates how many words (u32) are needed to send this message, not
    /// including the header.
    fn size(&self) -> usize;

    // Write the message to the buffer, not including the header.
    fn write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32];
}

pub trait Opcode
{
    const OPCODE: u16;
}

impl<T> Message for T
where
    T: Opcode + Arguments,
{
    const OPCODE: u16 = <T as Opcode>::OPCODE;

    #[inline]
    fn size(&self) -> usize
    {
        self.arguments().map(Argument::size).sum::<usize>()
    }

    #[inline]
    fn write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        self.arguments().fold(buf, |buf, arg| arg.write(buf))
    }
}
