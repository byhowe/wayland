use std::num::NonZero;

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

// TODO: implement fd
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Fd(i32);

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
