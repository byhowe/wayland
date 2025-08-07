use std::fmt;
use std::hint;
use std::num::NonZero;
use std::os::fd::AsFd;
use std::os::fd::AsRawFd;
use std::os::fd::BorrowedFd;
use std::os::fd::OwnedFd;

use crate::Enum;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnumParseError
{
    pub received: u32,
    pub interface: &'static str,
    pub enu: &'static str,
}

impl fmt::Display for EnumParseError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(
            f,
            "unable to parse wayland enum ({}::{}): received 0x{:08x} ({})",
            self.interface, self.enu, self.received, self.received
        )
    }
}

impl std::error::Error for EnumParseError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Int<T>(pub T)
where
    T: Enum;

impl<T> Int<T>
where
    T: Enum,
{
    #[inline]
    pub const fn new(value: T) -> Self
    {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Uint<T>(pub T)
where
    T: Enum;

impl<T> Uint<T>
where
    T: Enum,
{
    #[inline]
    pub const fn new(value: T) -> Self
    {
        Self(value)
    }
}

/// Wayland fixed-point number (24.8 format)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Fixed(i32);

impl Fixed
{
    /// Create a Fixed from a raw i32 value
    pub const fn from_bits(bits: i32) -> Self
    {
        Fixed(bits)
    }

    /// Get the raw i32 bits
    pub const fn to_bits(self) -> i32
    {
        self.0
    }

    // TODO: check the validity of the following functions

    /// Create a Fixed from a floating point value
    pub const fn from_f64(value: f64) -> Self
    {
        Fixed((value * 256.0) as i32)
    }

    /// Convert to floating point
    pub const fn to_f64(self) -> f64
    {
        self.0 as f64 / 256.0
    }

    /// Create a Fixed from integer and fractional parts
    pub const fn from_parts(integer: i32, fractional: u8) -> Self
    {
        Fixed((integer << 8) | fractional as i32)
    }

    /// Get the integer part
    pub const fn integer_part(self) -> i32
    {
        self.0 >> 8
    }

    /// Get the fractional part (0-255)
    pub const fn fractional_part(self) -> u8
    {
        (self.0 & 0xFF) as u8
    }
}

impl From<f64> for Fixed
{
    fn from(value: f64) -> Self
    {
        Fixed::from_f64(value)
    }
}

impl From<Fixed> for f64
{
    fn from(fixed: Fixed) -> Self
    {
        fixed.to_f64()
    }
}

/// Wayland object ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Object(NonZero<u32>);

impl Object
{
    /// Create a new ObjectId, with validation that it's non-zero
    pub const fn new(id: u32) -> Option<Self>
    {
        match NonZero::new(id) {
            None => None,
            Some(value) => Some(Object(value)),
        }
    }

    /// Create a new ObjectId without validation (for internal use)
    pub const unsafe fn new_unchecked(id: u32) -> Self
    {
        unsafe { Object(NonZero::new_unchecked(id)) }
    }

    /// Get the raw u32 value
    pub const fn get(self) -> u32
    {
        self.0.get()
    }
}

impl TryFrom<u32> for Object
{
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error>
    {
        Self::new(value).ok_or(())
    }
}

impl From<Object> for u32
{
    fn from(value: Object) -> Self
    {
        value.get()
    }
}

/// A new id is a non-nullable object id.
pub type NewId = Object;

#[derive(Debug)]
pub enum Fd<'fd>
{
    Borrowed(BorrowedFd<'fd>),
    Owned(OwnedFd),
}

impl AsFd for Fd<'_>
{
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_>
    {
        match self {
            Self::Borrowed(fd) => fd.as_fd(),
            Self::Owned(fd) => fd.as_fd(),
        }
    }
}

impl AsRawFd for Fd<'_>
{
    #[inline]
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd
    {
        match self {
            Fd::Borrowed(fd) => fd.as_raw_fd(),
            Fd::Owned(fd) => fd.as_raw_fd(),
        }
    }
}

impl<'fd> Fd<'fd>
{
    #[inline]
    pub fn unwrap_borrowed(self) -> Option<BorrowedFd<'fd>>
    {
        match self {
            Fd::Borrowed(fd) => Some(fd),
            Fd::Owned(_) => None,
        }
    }

    #[inline]
    pub fn unwrap_owned(self) -> Option<OwnedFd>
    {
        match self {
            Fd::Borrowed(_) => None,
            Fd::Owned(fd) => Some(fd),
        }
    }

    #[inline]
    pub unsafe fn unwrap_borrowed_unchecked(self) -> BorrowedFd<'fd>
    {
        match self {
            Fd::Borrowed(fd) => fd,
            Fd::Owned(_) => unsafe { hint::unreachable_unchecked() },
        }
    }

    #[inline]
    pub unsafe fn unwrap_owned_unchecked(self) -> OwnedFd
    {
        match self {
            Fd::Borrowed(_) => unsafe { hint::unreachable_unchecked() },
            Fd::Owned(fd) => fd,
        }
    }
}

impl<'fd> From<BorrowedFd<'fd>> for Fd<'fd>
{
    #[inline]
    fn from(value: BorrowedFd<'fd>) -> Self
    {
        Self::Borrowed(value)
    }
}

impl<'fd> From<OwnedFd> for Fd<'fd>
{
    #[inline]
    fn from(value: OwnedFd) -> Self
    {
        Self::Owned(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Header
{
    pub object: Object,
    pub size: u16,
    pub opcode: u16,
}

impl Header
{
    pub const fn new(object: Object, size: u16, opcode: u16) -> Self
    {
        Header {
            object,
            size,
            opcode,
        }
    }
}
