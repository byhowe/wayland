use std::slice;

pub mod meta;

mod argument;
mod primitive;
mod wire;

pub use argument::Argument;
pub use primitive::*;
pub use wire::*;

#[inline(always)]
pub fn prepare_buf(buf: &mut Vec<u32>, count: usize)
{
    if let Some(additional) = count.checked_sub(buf.len()) {
        buf.reserve(additional);
    }
    unsafe { buf.set_len(count) };
}

#[inline(always)]
pub fn bytes<'buf>(buf: &'buf [u32]) -> &'buf [u8]
{
    unsafe { slice::from_raw_parts(buf.as_ptr().cast(), buf.len() * 4) }
}

#[inline(always)]
pub fn bytes_mut<'buf>(buf: &'buf mut [u32]) -> &'buf mut [u8]
{
    unsafe { slice::from_raw_parts_mut(buf.as_mut_ptr().cast(), buf.len() * 4) }
}
