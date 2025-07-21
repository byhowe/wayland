use std::borrow::Cow;
use std::ffi::CStr;
use std::num::NonZero;
use std::ptr;
use std::slice;

pub mod meta;

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

#[inline(always)]
#[must_use]
pub fn write_int<'buf>(buf: &'buf mut [u32], value: i32) -> &'buf mut [u32]
{
    buf[0] = value.cast_unsigned();
    &mut buf[1..]
}

#[inline(always)]
#[must_use]
pub fn write_uint<'buf>(buf: &'buf mut [u32], value: u32) -> &'buf mut [u32]
{
    buf[0] = value;
    &mut buf[1..]
}

#[inline(always)]
#[must_use]
pub fn write_fixed<'buf>(buf: &'buf mut [u32], value: Fixed) -> &'buf mut [u32]
{
    buf[0] = value.cast_unsigned();
    &mut buf[1..]
}

#[inline(always)]
#[must_use]
pub fn write_string<'buf>(buf: &'buf mut [u32], value: &str) -> &'buf mut [u32]
{
    let size = value.len() + 1; // +1 for the null byte
    let buf = write_uint(buf, size as u32);
    // PERF: can we use copy_nonoverlapping here?
    // FIXME: we do not do checks on the size of the buffer.
    unsafe { ptr::copy(value.as_ptr(), buf.as_mut_ptr().cast(), value.len()) };
    unsafe { *buf.as_mut_ptr().cast::<u8>().add(size) = 0 };
    let padding = (size + 3) / 4;
    &mut buf[padding..]
}

#[inline(always)]
#[must_use]
pub fn write_string_nullable<'buf>(buf: &'buf mut [u32], value: Option<&str>) -> &'buf mut [u32]
{
    match value {
        Some(value) => write_string(buf, value),
        None => write_uint(buf, 0),
    }
}

#[inline(always)]
#[must_use]
pub fn write_object<'buf, T: Into<Object>>(buf: &'buf mut [u32], value: T) -> &'buf mut [u32]
{
    buf[0] = value.into().get();
    &mut buf[1..]
}

#[inline(always)]
#[must_use]
pub fn write_object_nullable<'buf, T: Into<Object>>(
    buf: &'buf mut [u32],
    value: Option<T>,
) -> &'buf mut [u32]
{
    buf[0] = value.map(|val| val.into().get()).unwrap_or(0);
    &mut buf[1..]
}

#[inline(always)]
#[must_use]
pub fn write_new_id<'buf>(buf: &'buf mut [u32], value: Object) -> &'buf mut [u32]
{
    buf[0] = value.get();
    &mut buf[1..]
}

#[allow(unused_variables)]
#[inline(always)]
#[must_use]
pub fn write_array<'buf>(buf: &'buf mut [u32], value: u32) -> &'buf mut [u32]
{
    unimplemented!()
}

#[inline(always)]
#[must_use]
pub fn write_header<'buf>(buf: &'buf mut [u32], value: Header) -> &'buf mut [u32]
{
    let buf = write_object(buf, value.object);
    let buf = write_uint(buf, ((value.size as u32) << 16) | (value.opcode as u32));
    buf
}

#[inline(always)]
#[must_use]
pub fn read_int<'buf>(buf: &'buf [u32]) -> (&'buf [u32], i32)
{
    (&buf[1..], buf[0].cast_signed())
}

#[inline(always)]
#[must_use]
pub fn read_uint<'buf>(buf: &'buf [u32]) -> (&'buf [u32], u32)
{
    (&buf[1..], buf[0])
}

#[inline(always)]
#[must_use]
pub fn read_fixed<'buf>(buf: &'buf [u32]) -> (&'buf [u32], Fixed)
{
    (&buf[1..], buf[0].cast_signed())
}

#[inline(always)]
#[must_use]
fn read_string_sized<'buf>(buf: &'buf [u32], size: usize) -> (&'buf [u32], Cow<'buf, str>)
{
    // FIXME: check the size of the buf before creating a slice with unknown size.
    // we can't willy-nilly trust the size returned by the compositor.
    let slice = unsafe { std::slice::from_raw_parts(buf.as_ptr().cast(), size) };
    let string = CStr::from_bytes_with_nul(slice).unwrap().to_string_lossy();
    let padding = (size + 3) / 4;
    (&buf[padding..], string)
}

#[inline(always)]
#[must_use]
pub fn read_string<'buf>(buf: &'buf [u32]) -> (&'buf [u32], Cow<'buf, str>)
{
    let (buf, size) = read_uint(buf);
    read_string_sized(buf, size as usize)
}

#[inline(always)]
#[must_use]
pub fn read_string_nullable<'buf>(buf: &'buf [u32]) -> (&'buf [u32], Option<Cow<'buf, str>>)
{
    match read_uint(buf) {
        (buf, 0) => (buf, None),
        (buf, size) => {
            let (buf, string) = read_string_sized(buf, size as usize);
            (buf, Some(string))
        }
    }
}

#[inline(always)]
#[must_use]
pub fn read_object<'buf>(buf: &'buf [u32]) -> (&'buf [u32], Object)
{
    (&buf[1..], Object::new(buf[0]).unwrap())
}

#[inline(always)]
#[must_use]
pub fn read_object_nullable<'buf>(buf: &'buf [u32]) -> (&'buf [u32], Option<Object>)
{
    (&buf[1..], Object::new(buf[0]))
}

#[inline(always)]
#[must_use]
pub fn read_new_id<'buf>(buf: &'buf [u32]) -> (&'buf [u32], Object)
{
    (&buf[1..], Object::new(buf[0]).unwrap())
}

#[allow(unused_variables)]
#[inline(always)]
#[must_use]
pub fn read_array<'buf>(buf: &'buf [u32]) -> (&'buf [u32], Object)
{
    unimplemented!()
}

#[inline(always)]
#[must_use]
pub fn read_header<'buf>(buf: &'buf [u32]) -> (&'buf [u32], Header)
{
    let (buf, object) = read_object(buf);
    let (buf, word) = read_uint(buf);
    let header = Header {
        object,
        size: ((word as u32) >> 16) as u16,
        opcode: (word & 0xFFFF) as u16,
    };
    (buf, header)
}
