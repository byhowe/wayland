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
//!
//! # Type Categories
//!
//! Wire types are divided into two categories:
//! - **Primitive types**: Fixed-size types that occupy exactly one 32-bit word
//!   (integers, fixed-point numbers, object IDs)
//! - **Dynamic types**: Variable-size types with length prefixes (strings, byte
//!   arrays)

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
pub trait Wire
{
    /// The type returned when reading from the wire format.
    ///
    /// For primitive types, this is typically `Self`.
    /// For dynamic types, this is usually a borrowed reference like `&[u8]` or
    /// `&str`.
    type Output<'a>: 'a;

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
    fn wire_read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self::Output<'buf>), WireError>;

    /// Returns the size in words (number of `u32`) this value will occupy when
    /// written to the socket. Importantly, a file descriptor has zero size
    /// since its value is transmitted through control message.
    #[must_use]
    fn wire_size(&self) -> usize;
}

/// Trait for Wayland types that occupy exactly one word in the wire format.
///
/// Primitive types include integers, fixed-point numbers, and object IDs.
/// They are stored directly as 32-bit values without any length prefix.
pub trait WirePrimitive: Copy
{
    /// Convert this value to its wire format representation.
    fn value(self) -> u32;

    /// Parse a value from its wire format representation.
    ///
    /// Returns `None` if the wire value is invalid for this type
    /// (e.g., zero object ID, invalid enum variant).
    fn parse(value: u32) -> Option<Self>;
}

/// Trait for dynamically sized Wayland types such as arrays and strings.
///
/// Dynamic types are stored with a length prefix followed by the data,
/// padded to 4-byte boundaries.
///
/// # Wire Format
///
/// ```text
/// [length: u32][data: bytes...][padding to 4-byte boundary]
/// ```
pub trait WireDynamic
{
    /// The type returned when reading from the wire format.
    type Output<'a>: 'a;

    /// Get the raw byte data for this value.
    fn data(&self) -> &[u8];

    /// Get the size of the data in bytes.
    ///
    /// For most types this is just `self.data().len()`, but strings
    /// need to account for null termination.
    #[inline]
    fn dynamic_size(&self) -> usize
    {
        self.data().len()
    }

    /// Write the data to the provided byte buffer.
    ///
    /// The buffer is guaranteed to be exactly `dynamic_size()` bytes long.
    /// For strings, the null termination must also be handled here. But, it is
    /// not necessary since the caller sets the last word of the allocated
    /// buffer to zero.
    #[inline]
    fn dynamic_write<'buf>(&self, buf: &'buf mut [u8])
    {
        buf.copy_from_slice(self.data());
    }

    /// Parse data from the provided byte buffer.
    ///
    /// The buffer contains exactly the number of bytes specified in the
    /// length prefix, without padding.
    fn dynamic_read<'buf>(buf: &'buf [u8]) -> Result<Self::Output<'buf>, WireError>;
}

impl WirePrimitive for i32
{
    #[inline]
    fn value(self) -> u32
    {
        self.cast_unsigned()
    }

    #[inline]
    fn parse(value: u32) -> Option<Self>
    {
        Some(value.cast_signed())
    }
}

impl<T> WirePrimitive for Int<T>
where
    T: Enum + TryFrom<u32, Error = EnumParseError>,
{
    #[inline]
    fn value(self) -> u32
    {
        self.0.int().cast_unsigned()
    }

    #[inline]
    fn parse(value: u32) -> Option<Self>
    {
        let enu = T::try_from(value).ok()?;
        Some(Int(enu))
    }
}

impl WirePrimitive for u32
{
    #[inline]
    fn value(self) -> u32
    {
        self
    }

    #[inline]
    fn parse(value: u32) -> Option<Self>
    {
        Some(value)
    }
}

impl<T> WirePrimitive for Uint<T>
where
    T: Enum + TryFrom<u32, Error = EnumParseError>,
{
    #[inline]
    fn value(self) -> u32
    {
        self.0.uint()
    }

    #[inline]
    fn parse(value: u32) -> Option<Self>
    {
        let enu = T::try_from(value).ok()?;
        Some(Uint(enu))
    }
}

impl WirePrimitive for Fixed
{
    #[inline]
    fn value(self) -> u32
    {
        self.to_bits().cast_unsigned()
    }

    #[inline]
    fn parse(value: u32) -> Option<Self>
    {
        Some(Fixed::from_bits(value.cast_signed()))
    }
}

impl WirePrimitive for Object
{
    #[inline]
    fn value(self) -> u32
    {
        self.get()
    }

    #[inline]
    fn parse(value: u32) -> Option<Self>
    {
        Self::new(value)
    }
}

impl<T> helper::WireHelper<helper::WirePrimitiveMarker> for T
where
    T: WirePrimitive + 'static,
{
    type Output<'a> = T;

    #[inline]
    fn wire_write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        buf[0] = self.value();
        &mut buf[1..]
    }

    #[inline]
    fn wire_read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self), WireError>
    {
        let bits = buf.get(0).ok_or(WireError::UnexpectedEof {
            needed: 1,
            available: buf.len(),
        })?;
        let value = Self::parse(*bits).ok_or(WireError::Malformed)?;
        Ok((&buf[1..], value))
    }

    #[inline]
    fn wire_size(&self) -> usize
    {
        1
    }
}

impl WireDynamic for [u8]
{
    type Output<'a> = &'a [u8];

    #[inline]
    fn data(&self) -> &[u8]
    {
        self
    }

    #[inline]
    fn dynamic_read<'buf>(buf: &'buf [u8]) -> Result<Self::Output<'buf>, WireError>
    {
        Ok(buf)
    }
}

impl WireDynamic for CStr
{
    type Output<'a> = &'a CStr;

    #[inline]
    fn data(&self) -> &[u8]
    {
        self.to_bytes_with_nul()
    }

    #[inline]
    fn dynamic_read<'buf>(buf: &'buf [u8]) -> Result<Self::Output<'buf>, WireError>
    {
        Ok(CStr::from_bytes_with_nul(buf)?)
    }
}

impl WireDynamic for str
{
    type Output<'a> = &'a str;

    #[inline]
    fn data(&self) -> &[u8]
    {
        // NOTE: The bytes returned by string is not null terminated. Hence, we must
        // handle writing this type manually. See `<str as
        // WireDynamic>::dynamic_write` for details.
        self.as_bytes()
    }

    #[inline]
    fn dynamic_size(&self) -> usize
    {
        // +1 for the null byte
        self.data().len() + 1
    }

    #[inline]
    fn dynamic_write<'buf>(&self, buf: &'buf mut [u8])
    {
        // NOTE: The caller must guarantee that the length of `buf` is exactly the size
        // provided by `<str as WireDynamic>::dynamic_size`. It includes the null byte
        // in this case. We must put the null byte at the end of `buf`. However, we do
        // not have to. In this crate, the last word of the buffer allocated for an
        // array-like type is automatically set to zero.
        buf[..self.len()].copy_from_slice(self.data());
        // buf[self.len()] should be set to 0, but it's set by the caller
    }

    #[inline]
    fn dynamic_read<'buf>(buf: &'buf [u8]) -> Result<Self::Output<'buf>, WireError>
    {
        let value = <CStr as WireDynamic>::dynamic_read(buf)?;
        Ok(value.to_str()?)
    }
}

impl<T> WireDynamic for &T
where
    T: WireDynamic,
{
    type Output<'a> = T::Output<'a>;

    #[inline]
    fn data(&self) -> &[u8]
    {
        <T as WireDynamic>::data(self)
    }

    #[inline]
    fn dynamic_size(&self) -> usize
    {
        <T as WireDynamic>::dynamic_size(self)
    }

    #[inline]
    fn dynamic_write<'buf>(&self, buf: &'buf mut [u8])
    {
        <T as WireDynamic>::dynamic_write(self, buf)
    }

    #[inline]
    fn dynamic_read<'buf>(buf: &'buf [u8]) -> Result<Self::Output<'buf>, WireError>
    {
        <T as WireDynamic>::dynamic_read(buf)
    }
}

impl<T> helper::WireHelper<helper::WireDynamicMarker> for T
where
    T: WireDynamic + ?Sized,
{
    type Output<'a> = T::Output<'a>;

    fn wire_write<'buf>(&self, mut buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        buf = Wire::wire_write(&(self.dynamic_size() as u32), buf);
        let words = pad(self.dynamic_size());
        // Let Rust handle the bound checks.
        let dest = &mut buf[..words];
        let dest =
            unsafe { slice::from_raw_parts_mut(dest.as_mut_ptr().cast(), self.dynamic_size()) };
        // NOTE: When we write the bytes provided by `self` into `buf`, there may be
        // untouched bytes at the end of `buf` since the contents of `buf` are padded to
        // 4 bytes. We set the last word to zero before writing bytes so that we do not
        // leak any data from the memory. Also, string type has to be null-terminated.
        buf[words - 1] = 0;
        self.dynamic_write(dest);
        &mut buf[words..]
    }

    fn wire_read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self::Output<'buf>), WireError>
    {
        let (buf, size) = <u32 as Wire>::wire_read(buf)?;
        let words = pad(size as usize);
        if buf.len() < words {
            return Err(WireError::UnexpectedEof {
                needed: words,
                available: buf.len(),
            });
        }
        let data = unsafe { slice::from_raw_parts(buf.as_ptr().cast(), size as usize) };
        let value = T::dynamic_read(data)?;
        Ok((&buf[words..], value))
    }

    #[inline]
    fn wire_size(&self) -> usize
    {
        1 + pad(self.dynamic_size())
    }
}

// FIX: Do we need this impl? I think it is good to have.
impl Wire for String
{
    type Output<'a> = String;

    fn wire_write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        self.as_str().wire_write(buf)
    }

    fn wire_read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self::Output<'buf>), WireError>
    {
        <str as Wire>::wire_read(buf).map(|(buf, value)| (buf, value.to_string()))
    }

    #[inline]
    fn wire_size(&self) -> usize
    {
        self.as_str().wire_size()
    }
}

impl Wire for Cow<'_, str>
{
    type Output<'a> = Cow<'a, str>;

    fn wire_write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        self.as_ref().wire_write(buf)
    }

    fn wire_read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self::Output<'buf>), WireError>
    {
        <str as Wire>::wire_read(buf).map(|(buf, value)| (buf, Cow::Borrowed(value)))
    }

    #[inline]
    fn wire_size(&self) -> usize
    {
        self.as_ref().wire_size()
    }
}

impl Wire for Vec<u8>
{
    type Output<'a> = Vec<u8>;

    fn wire_write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        self.as_slice().wire_write(buf)
    }

    fn wire_read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self::Output<'buf>), WireError>
    {
        let (buf, value) = <[u8] as Wire>::wire_read(buf)?;
        Ok((buf, value.to_vec()))
    }

    #[inline]
    fn wire_size(&self) -> usize
    {
        self.as_slice().wire_size()
    }
}

impl Wire for Cow<'_, [u8]>
{
    type Output<'a> = Cow<'a, [u8]>;

    fn wire_write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        self.as_ref().wire_write(buf)
    }

    fn wire_read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self::Output<'buf>), WireError>
    {
        let (buf, value) = <[u8] as Wire>::wire_read(buf)?;
        Ok((buf, Cow::Borrowed(value)))
    }

    fn wire_size(&self) -> usize
    {
        self.as_ref().wire_size()
    }
}

impl Wire for Header
{
    type Output<'a> = Header;

    fn wire_write<'buf>(&self, mut buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        // Pack size and opcode into a single 32-bit word: [size:16][opcode:16]
        let word = ((self.size as u32) << 16) | (self.opcode as u32);
        buf = self.object.wire_write(buf);
        word.wire_write(buf)
    }

    fn wire_read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self::Output<'buf>), WireError>
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
    type Output<'a> = Option<T::Output<'a>>;

    fn wire_write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
    {
        match self {
            None => Wire::wire_write(&0u32, buf),
            Some(value) => Wire::wire_write(value, buf),
        }
    }

    fn wire_read<'buf>(buf: &'buf [u32]) -> Result<(&'buf [u32], Self::Output<'buf>), WireError>
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

pub trait WireAncillary: Sized {}

mod helper
{
    pub trait WireHelper<M>
    {
        type Output<'a>: 'a;

        #[must_use]
        fn wire_write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32];

        #[must_use]
        fn wire_read<'buf>(
            buf: &'buf [u32],
        ) -> Result<(&'buf [u32], Self::Output<'buf>), super::WireError>;

        #[must_use]
        fn wire_size(&self) -> usize;
    }

    impl<T> super::Wire for T
    where
        T: Marker + WireHelper<T::Marker> + ?Sized,
    {
        type Output<'a> = T::Output<'a>;

        fn wire_write<'buf>(&self, buf: &'buf mut [u32]) -> &'buf mut [u32]
        {
            <T as WireHelper<T::Marker>>::wire_write(self, buf)
        }

        fn wire_read<'buf>(
            buf: &'buf [u32],
        ) -> Result<(&'buf [u32], Self::Output<'buf>), super::WireError>
        {
            <T as WireHelper<T::Marker>>::wire_read(buf)
        }

        fn wire_size(&self) -> usize
        {
            <T as WireHelper<T::Marker>>::wire_size(self)
        }
    }

    pub trait Marker
    {
        type Marker;
    }

    pub enum WirePrimitiveMarker {}

    pub enum WireDynamicMarker {}

    #[rustfmt::skip] impl Marker for i32 { type Marker = WirePrimitiveMarker; }
    #[rustfmt::skip] impl Marker for u32 { type Marker = WirePrimitiveMarker; }
    #[rustfmt::skip] impl<T> Marker for crate::Int<T> where T: crate::Enum { type Marker = WirePrimitiveMarker; }
    #[rustfmt::skip] impl<T> Marker for crate::Uint<T> where T: crate::Enum { type Marker = WirePrimitiveMarker; }
    #[rustfmt::skip] impl Marker for crate::Fixed { type Marker = WirePrimitiveMarker; }
    #[rustfmt::skip] impl Marker for crate::Object { type Marker = WirePrimitiveMarker; }

    #[rustfmt::skip] impl Marker for [u8] { type Marker = WireDynamicMarker; }
    #[rustfmt::skip] impl Marker for std::ffi::CStr { type Marker = WireDynamicMarker; }
    #[rustfmt::skip] impl Marker for str { type Marker = WireDynamicMarker; }
    #[rustfmt::skip] impl<T> Marker for &T where T: Marker { type Marker = T::Marker; }
}
