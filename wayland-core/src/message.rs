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
