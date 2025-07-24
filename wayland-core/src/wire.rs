use std::ffi::CStr;
use std::ffi::FromBytesWithNulError;
use std::fmt;
use std::slice;
use std::str::Utf8Error;

use crate::Fixed;
use crate::Header;
use crate::Object;
use crate::pad;

/// Errors that can happen during deserialization from the wire format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireError
{
    /// Not enough bytes available to read the complete type.
    UnexpectedEof
    {
        /// Number of words (`u32`) needed.
        needed: usize,
        /// Number of words (`u32`) available.
        available: usize,
    },
    /// Improperly serialized type.
    ///
    /// Some errors include:
    /// - String is not properly null-terminated.
    /// - Object ID is zero when it shouldn't be.
    Malformed,
}

impl fmt::Display for WireError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self {
            WireError::UnexpectedEof { needed, available } => write!(
                f,
                "unexpected end of buffer: needed {} words ({} bytes), got {} words ({} bytes)",
                needed,
                needed * 4,
                available,
                available * 4
            ),
            WireError::Malformed => write!(f, "malformed value read from the buffer"),
        }
    }
}

impl From<FromBytesWithNulError> for WireError
{
    fn from(_value: FromBytesWithNulError) -> Self
    {
        WireError::Malformed
    }
}

impl From<Utf8Error> for WireError
{
    fn from(_value: Utf8Error) -> Self
    {
        WireError::Malformed
    }
}

impl std::error::Error for WireError {}

/// Trait for types that can be serialized to and from Wayland wire format.
pub trait Wire: Sized
{
    /// Write this value to the buffer, returning the remaining buffer slice.
    ///
    /// # Panics
    ///
    /// Panics if the buffer is too small. Callers should ensure the buffer has
    /// enough space by using `size()` or similar methods.
    #[must_use]
    fn write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32];

    /// Read a value from the buffer, returning the value and the remaining
    /// buffer.
    ///
    /// Returns an error if:
    /// - The buffer does not contain enough data
    /// - The data is malformed (e.g., string is not null-terminated)
    /// - The data is semantically invalid (e.g., zero object ID)
    #[must_use]
    fn read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>;

    /// Returns the size in words (number of `u32`) this value will occupy when
    /// written to the socket. Importantly, a file descriptor has zero size
    /// since its value is transmitted through control message.
    #[must_use]
    fn size(&self) -> usize;
}

impl Wire for i32
{
    #[inline(always)]
    fn write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        buf[0] = self.cast_unsigned();
        &mut buf[1..]
    }

    #[inline(always)]
    fn read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>
    {
        buf.get(0)
            .map(|v| (&buf[1..], v.cast_signed()))
            .ok_or(WireError::UnexpectedEof {
                needed: 1,
                available: buf.len(),
            })
    }

    #[inline(always)]
    fn size(&self) -> usize
    {
        1
    }
}

impl Wire for u32
{
    #[inline(always)]
    fn write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        buf[0] = *self;
        &mut buf[1..]
    }

    #[inline(always)]
    fn read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>
    {
        buf.get(0)
            .map(|v| (&buf[1..], *v))
            .ok_or(WireError::UnexpectedEof {
                needed: 1,
                available: buf.len(),
            })
    }

    #[inline(always)]
    fn size(&self) -> usize
    {
        1
    }
}

impl Wire for &CStr
{
    fn write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        write_sized_data(self, buf)
    }

    fn read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>
    {
        let (buf, bytes) = <&[u8] as Wire>::read(buf)?;

        // NOTE: This only returns an error if the string is not properly
        // null-terminated. Otherwise, it does not check for UTF-8 errors. The
        // wayland docs specify that the strings are UTF-8 encoded. Should we
        // throw an error here if the &CStr is not encoded properly? Maybe that
        // should be enforced by &str, which it is currently.
        //
        // TODO: Make sure to document handling of UTF-8 when using &CStr.
        let value = CStr::from_bytes_with_nul(bytes)?;
        Ok((buf, value))
    }

    fn size(&self) -> usize
    {
        1 + pad(self.to_bytes_with_nul().len())
    }
}

impl Wire for &str
{
    fn write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        write_sized_data(self, buf)
    }

    fn read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>
    {
        let (buf, value) = <&CStr as Wire>::read(buf)?;
        // NOTE: `to_str` throws an error if the &CStr is not proper UTF-8.
        Ok((buf, value.to_str()?))
    }

    fn size(&self) -> usize
    {
        1 + pad(self.len() + 1)
    }
}

// FIX: Do we need this impl? I think it is good to have.
impl Wire for String
{
    #[inline(always)]
    fn write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        self.as_str().write(buf)
    }

    #[inline(always)]
    fn read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>
    {
        let (buf, value) = <&CStr as Wire>::read(buf)?;
        Ok((buf, value.to_str()?.to_string()))
    }

    #[inline(always)]
    fn size(&self) -> usize
    {
        self.as_str().size()
    }
}

impl Wire for Fixed
{
    #[inline(always)]
    fn write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        i32::write(&self.to_bits(), buf)
    }

    #[inline(always)]
    fn read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>
    {
        i32::read(buf).map(|(buf, value)| (buf, Fixed::from_bits(value)))
    }

    #[inline(always)]
    fn size(&self) -> usize
    {
        1
    }
}

impl Wire for Object
{
    #[inline(always)]
    fn write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        self.get().write(buf)
    }

    #[inline(always)]
    fn read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>
    {
        let (buf, id) = u32::read(buf)?;
        Self::new(id)
            .map(|value| (buf, value))
            .ok_or(WireError::Malformed)
    }

    #[inline(always)]
    fn size(&self) -> usize
    {
        1
    }
}

impl Wire for Vec<u8>
{
    #[inline(always)]
    fn write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        self.as_slice().write(buf)
    }

    #[inline(always)]
    fn read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>
    {
        let (buf, value) = <&[u8] as Wire>::read(buf)?;
        Ok((buf, value.to_vec()))
    }

    #[inline(always)]
    fn size(&self) -> usize
    {
        self.as_slice().size()
    }
}

impl Wire for &[u8]
{
    #[inline(always)]
    fn write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        write_sized_data(self, buf)
    }

    #[inline(always)]
    fn read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>
    {
        let (buf, size) = u32::read(buf)?;
        let words = pad(size as usize);
        if buf.len() < words {
            return Err(WireError::UnexpectedEof {
                needed: words,
                available: buf.len(),
            });
        }
        let value = unsafe { slice::from_raw_parts(buf.as_ptr().cast(), size as usize) };
        Ok((&buf[words..], value))
    }

    #[inline(always)]
    fn size(&self) -> usize
    {
        1 + pad(self.len())
    }
}

impl Wire for Header
{
    #[inline(always)]
    fn write<'buf>(&self, mut buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        let word = ((self.size as u32) << 16) | (self.opcode as u32);
        buf = self.object.write(buf);
        word.write(buf)
    }

    #[inline(always)]
    fn read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>
    {
        let (buf, object) = Object::read(buf)?;
        let (buf, word) = u32::read(buf)?;
        let header = Header {
            object,
            size: ((word as u32) >> 16) as u16,
            opcode: (word & 0xFFFF) as u16,
        };
        Ok((buf, header))
    }

    #[inline(always)]
    fn size(&self) -> usize
    {
        2
    }
}

impl<T> Wire for Option<T>
where
    T: Wire,
{
    #[inline(always)]
    fn write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        match self {
            None => Wire::write(&0u32, buf),
            Some(value) => Wire::write(value, buf),
        }
    }

    #[inline(always)]
    fn read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>
    {
        let (_, value) = u32::read(buf)?;
        match value {
            0 => Ok((&buf[1..], None)),
            _ => T::read(buf).map(|(buf, value)| (buf, Some(value))),
        }
    }

    #[inline(always)]
    fn size(&self) -> usize
    {
        match self {
            None => 1,
            Some(value) => Wire::size(value),
        }
    }
}

/// Helper trait to help with sized types such as strings and arrays.
trait SizedData
{
    /// Length required by the object in bytes, without the size field.
    fn data_len(&self) -> usize;

    /// Write the contents of `self` into `dest`.
    ///
    /// The length of `dest` must be the same as the length requrned by
    /// `data_len`.
    fn write_data(&self, dest: &mut [u8]);
}

impl SizedData for &[u8]
{
    #[inline(always)]
    fn data_len(&self) -> usize
    {
        self.len()
    }

    #[inline(always)]
    fn write_data(&self, dest: &mut [u8])
    {
        dest.copy_from_slice(self);
    }
}

impl SizedData for &CStr
{
    #[inline(always)]
    fn data_len(&self) -> usize
    {
        self.to_bytes_with_nul().len()
    }

    #[inline(always)]
    fn write_data(&self, dest: &mut [u8])
    {
        dest.copy_from_slice(self.to_bytes_with_nul());
    }
}

impl SizedData for &str
{
    #[inline(always)]
    fn data_len(&self) -> usize
    {
        self.len() + 1
    }

    #[inline(always)]
    fn write_data(&self, dest: &mut [u8])
    {
        dest[..self.len()].copy_from_slice(self.as_bytes());
        // NOTE: Setting the last byte to zero explicitly is not necessary.
        // `write_sized_data` function already sets the last word to
        // zero. dest[self.len()] = 0;
    }
}

#[inline(always)]
fn write_sized_data<'buf>(data: &impl SizedData, mut buf: &'buf mut [u32]) -> &'buf mut [u32]
{
    buf = (data.data_len() as u32).write(buf);
    let words = pad(data.data_len());
    let dest = unsafe { slice::from_raw_parts_mut(buf.as_mut_ptr().cast(), data.data_len()) };
    // When we write the bytes provided by `data` into `buf`, there may be untouched
    // bytes at the end of `buf` since the contents of `buf` are padded to 4
    // bytes. We set the last word to zero before writing bytes so that we do
    // not leak any data from the memory. Also, string type has
    // to be null-terminated.
    buf[words] = 0;
    data.write_data(dest);
    &mut buf[words..]
}
