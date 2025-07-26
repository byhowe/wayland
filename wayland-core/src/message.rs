use std::mem::MaybeUninit;

use crate::WireError;

pub trait MessageWire: Sized
{
    type Output<'a>: 'a;

    fn message_write(&self, buf: &mut [u32]);

    #[must_use]
    fn message_read_into<'buf>(
        buf: &'buf [u32],
        msg: &mut MaybeUninit<Self::Output<'buf>>,
    ) -> Result<(), WireError>;

    #[must_use]
    fn message_size(&self) -> usize;

    #[inline]
    #[must_use]
    fn message_read<'buf>(buf: &'buf [u32]) -> Result<MaybeUninit<Self::Output<'buf>>, WireError>
    {
        let mut msg = MaybeUninit::uninit();
        Self::message_read_into(buf, &mut msg)?;
        Ok(msg)
    }
}

pub trait MessageOpcode
{
    const OPCODE: u16;
}
