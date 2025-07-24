//! Wire format serialization for Wayland protocol types.
//!
//! This module implements the `Wire` trait for Wayland protocol primitive
//! types, handling conversion between Rust types and the binary wire format
//! used in Wayland messages.
//!
//! # Wire Format
//!
//! - All values are serialized as 32-bit words in host byte order
//! - Strings and arrays are prefixed with their length in bytes
//! - Data is padded to 4-byte boundaries
//! - File descriptors are transmitted via control messages (not in the data
//!   buffer)

use std::borrow::Cow;
use std::ffi::CStr;
use std::ffi::FromBytesWithNulError;
use std::fmt;
use std::slice;
use std::str::Utf8Error;

use crate::Enum;
use crate::EnumParseError;
use crate::Fixed;
use crate::Header;
use crate::Int;
use crate::Object;
use crate::Uint;
use crate::pad;

/// Errors that can happen during deserialization from the wire format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireError
{
    /// Not enough bytes available to read the complete type.
    ///
    /// This occurs when trying to read a value that extends beyond the
    /// available buffer space.
    UnexpectedEof
    {
        /// Number of 32-bit words needed to complete the read.
        needed: usize,
        /// Number of 32-bit words actually available in the buffer.
        available: usize,
    },
    /// Improperly serialized type.
    ///
    /// Some errors include:
    /// - Object ID is zero when it shouldn't be
    /// - String is not properly null-terminated
    /// - Invalid UTF-8 in a string that requires valid UTF-8
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

impl From<EnumParseError> for WireError
{
    fn from(_value: EnumParseError) -> Self
    {
        WireError::Malformed
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
///
/// The Wayland protocol uses a binary wire format where all data is aligned to
/// 32-bit word boundaries. This trait provides serialization and
/// deserialization for protocol types.
///
/// # Wire Format Rules
///
/// - All data is stored as 32-bit words (`u32`)
/// - Strings and byte arrays are length-prefixed
/// - All data is padded to 4-byte boundaries
/// - Object IDs must be non-zero
///
/// # Safety
///
/// Implementations must ensure that `wire_write()` uses exactly `wire_size()`
/// words, and that `wire_read()` advances the buffer by the same amount.
pub trait Wire: Sized
{
    /// Write this value to the buffer, returning the remaining buffer slice.
    ///
    /// # Panics
    ///
    /// Panics if the buffer is too small. Callers should ensure the buffer has
    /// enough space by using `wire_size()` or similar methods.
    #[must_use]
    fn wire_write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32];

    /// Read a value from the buffer, returning the value and the remaining
    /// buffer.
    ///
    /// Returns an error if:
    /// - The buffer does not contain enough data
    /// - The data is malformed (e.g., string is not null-terminated)
    /// - The data is semantically invalid (e.g., zero object ID)
    #[must_use]
    fn wire_read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>;

    /// Returns the size in words (number of `u32`) this value will occupy when
    /// written to the socket. Importantly, a file descriptor has zero size
    /// since its value is transmitted through control message.
    #[must_use]
    fn wire_size(&self) -> usize;
}

impl Wire for i32
{
    fn wire_write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        buf[0] = self.cast_unsigned();
        &mut buf[1..]
    }

    fn wire_read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>
    {
        buf.get(0)
            .map(|v| (&buf[1..], v.cast_signed()))
            .ok_or(WireError::UnexpectedEof {
                needed: 1,
                available: buf.len(),
            })
    }

    #[inline]
    fn wire_size(&self) -> usize
    {
        1
    }
}

impl<T> Wire for Int<T>
where
    T: Enum + TryFrom<u32, Error = EnumParseError>,
{
    fn wire_write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        self.0.int().wire_write(buf)
    }

    fn wire_read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>
    {
        let (buf, value) = i32::wire_read(buf)?;
        let enu = T::try_from(value.cast_unsigned())?;
        Ok((buf, Int(enu)))
    }

    fn wire_size(&self) -> usize
    {
        self.0.int().wire_size()
    }
}

impl<T> Wire for Uint<T>
where
    T: Enum + TryFrom<u32, Error = EnumParseError>,
{
    fn wire_write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        self.0.uint().wire_write(buf)
    }

    fn wire_read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>
    {
        let (buf, value) = u32::wire_read(buf)?;
        let enu = T::try_from(value)?;
        Ok((buf, Uint(enu)))
    }

    fn wire_size(&self) -> usize
    {
        self.0.uint().wire_size()
    }
}

impl Wire for u32
{
    fn wire_write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        buf[0] = *self;
        &mut buf[1..]
    }

    fn wire_read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>
    {
        buf.get(0)
            .map(|v| (&buf[1..], *v))
            .ok_or(WireError::UnexpectedEof {
                needed: 1,
                available: buf.len(),
            })
    }

    #[inline]
    fn wire_size(&self) -> usize
    {
        1
    }
}

impl Wire for &CStr
{
    fn wire_write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        write_sized_data(self, buf)
    }

    fn wire_read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>
    {
        let (buf, bytes) = <&[u8] as Wire>::wire_read(buf)?;

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

    #[inline]
    fn wire_size(&self) -> usize
    {
        1 + pad(self.to_bytes_with_nul().len())
    }
}

impl Wire for &str
{
    fn wire_write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        write_sized_data(self, buf)
    }

    fn wire_read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>
    {
        let (buf, value) = <&CStr as Wire>::wire_read(buf)?;
        // NOTE: `to_str` throws an error if the &CStr is not proper UTF-8.
        Ok((buf, value.to_str()?))
    }

    #[inline]
    fn wire_size(&self) -> usize
    {
        1 + pad(self.len() + 1)
    }
}

// FIX: Do we need this impl? I think it is good to have.
impl Wire for String
{
    fn wire_write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        self.as_str().wire_write(buf)
    }

    fn wire_read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>
    {
        let (buf, value) = <&CStr as Wire>::wire_read(buf)?;
        Ok((buf, value.to_str()?.to_string()))
    }

    #[inline]
    fn wire_size(&self) -> usize
    {
        self.as_str().wire_size()
    }
}

impl Wire for Cow<'_, str>
{
    fn wire_write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        self.as_ref().wire_write(buf)
    }

    fn wire_read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>
    {
        let (buf, value) = <&str as Wire>::wire_read(buf)?;
        Ok((buf, Cow::Borrowed(value)))
    }

    fn wire_size(&self) -> usize
    {
        self.as_ref().wire_size()
    }
}

impl Wire for Fixed
{
    fn wire_write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        i32::wire_write(&self.to_bits(), buf)
    }

    fn wire_read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>
    {
        i32::wire_read(buf).map(|(buf, value)| (buf, Fixed::from_bits(value)))
    }

    #[inline]
    fn wire_size(&self) -> usize
    {
        1
    }
}

impl Wire for Object
{
    fn wire_write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        self.get().wire_write(buf)
    }

    fn wire_read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>
    {
        let (buf, id) = u32::wire_read(buf)?;
        Self::new(id)
            .map(|value| (buf, value))
            .ok_or(WireError::Malformed)
    }

    #[inline]
    fn wire_size(&self) -> usize
    {
        1
    }
}

impl Wire for Vec<u8>
{
    fn wire_write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        self.as_slice().wire_write(buf)
    }

    fn wire_read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>
    {
        let (buf, value) = <&[u8] as Wire>::wire_read(buf)?;
        Ok((buf, value.to_vec()))
    }

    #[inline]
    fn wire_size(&self) -> usize
    {
        self.as_slice().wire_size()
    }
}

impl Wire for &[u8]
{
    fn wire_write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        write_sized_data(self, buf)
    }

    fn wire_read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>
    {
        let (buf, size) = u32::wire_read(buf)?;
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

    #[inline]
    fn wire_size(&self) -> usize
    {
        1 + pad(self.len())
    }
}

impl Wire for Cow<'_, [u8]>
{
    fn wire_write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        self.as_ref().wire_write(buf)
    }

    fn wire_read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>
    {
        let (buf, value) = <&[u8] as Wire>::wire_read(buf)?;
        Ok((buf, Cow::Borrowed(value)))
    }

    fn wire_size(&self) -> usize
    {
        self.as_ref().wire_size()
    }
}

impl Wire for Header
{
    fn wire_write<'buf>(&self, mut buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        // Pack size and opcode into a single 32-bit word: [size:16][opcode:16]
        let word = ((self.size as u32) << 16) | (self.opcode as u32);
        buf = self.object.wire_write(buf);
        word.wire_write(buf)
    }

    fn wire_read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>
    {
        let (buf, object) = Object::wire_read(buf)?;
        let (buf, word) = u32::wire_read(buf)?;
        // Unpack the size and opcode from the combined word
        let header = Header {
            object,
            size: ((word as u32) >> 16) as u16, // extract upper 16 bits
            opcode: (word & 0xFFFF) as u16,     // extract lower 16 bits
        };
        Ok((buf, header))
    }

    #[inline]
    fn wire_size(&self) -> usize
    {
        2
    }
}

impl<T> Wire for Option<T>
where
    T: Wire,
{
    fn wire_write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        match self {
            None => Wire::wire_write(&0u32, buf),
            Some(value) => Wire::wire_write(value, buf),
        }
    }

    fn wire_read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>
    {
        let (_, value) = u32::wire_read(buf)?;
        match value {
            0 => Ok((&buf[1..], None)),
            _ => T::wire_read(buf).map(|(buf, value)| (buf, Some(value))),
        }
    }

    #[inline]
    fn wire_size(&self) -> usize
    {
        match self {
            None => 1,
            Some(value) => Wire::wire_size(value),
        }
    }
}

/// Helper trait for types that can be written as sized data (length + content).
trait SizedData
{
    /// Length required by the object in bytes, without the size field.
    ///
    /// For strings, this includes the null terminator.
    /// For byte arrays, this is just the array length.
    fn data_len(&self) -> usize;

    /// Write the contents of `self` into `dest`.
    ///
    /// The length of `dest` must be the same as the length requrned by
    /// `data_len`.
    fn write_data(&self, dest: &mut [u8]);
}

impl SizedData for &[u8]
{
    #[inline]
    fn data_len(&self) -> usize
    {
        self.len()
    }

    #[inline]
    fn write_data(&self, dest: &mut [u8])
    {
        dest.copy_from_slice(self);
    }
}

impl SizedData for &CStr
{
    #[inline]
    fn data_len(&self) -> usize
    {
        self.to_bytes_with_nul().len()
    }

    #[inline]
    fn write_data(&self, dest: &mut [u8])
    {
        dest.copy_from_slice(self.to_bytes_with_nul());
    }
}

impl SizedData for &str
{
    #[inline]
    fn data_len(&self) -> usize
    {
        self.len() + 1
    }

    #[inline]
    fn write_data(&self, dest: &mut [u8])
    {
        dest[..self.len()].copy_from_slice(self.as_bytes());
        // NOTE: Setting the last byte to zero explicitly is not necessary.
        // `write_sized_data` function already sets the last word to
        // zero. dest[self.len()] = 0;
    }
}

#[inline]
fn write_sized_data<'buf>(data: &impl SizedData, mut buf: &'buf mut [u32]) -> &'buf mut [u32]
{
    buf = (data.data_len() as u32).wire_write(buf);
    let words = pad(data.data_len());
    // Let Rust handle the bound checks.
    let dest = &mut buf[..words];
    let dest = unsafe { slice::from_raw_parts_mut(dest.as_mut_ptr().cast(), data.data_len()) };
    // When we write the bytes provided by `data` into `buf`, there may be untouched
    // bytes at the end of `buf` since the contents of `buf` are padded to 4
    // bytes. We set the last word to zero before writing bytes so that we do
    // not leak any data from the memory. Also, string type has
    // to be null-terminated.
    buf[words - 1] = 0;
    data.write_data(dest);
    &mut buf[words..]
}
