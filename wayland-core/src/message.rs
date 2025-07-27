use std::mem::MaybeUninit;

use crate::WireError;

pub trait MessageWire: Sized
{
    // LIFETIME BOUND ANALYSIS: Why `type Output<'a>: 'a` was removed
    //
    // This line used to be:
    // ```rust
    // type Output<'a>: 'a;
    // ```
    //
    // ORIGINAL REASONING:
    // The `'a` bound was added thinking that any type returned by a read function
    // should live at least as long as `'a` (the buffer's lifetime). The logic was:
    // "if the buffer dies, any data borrowed from it should also become invalid."
    //
    // WHY IT WORKED INITIALLY:
    // For simple cases with only `Cow<'a, str>` and `Cow<'a, [u8]>`, this bound was
    // redundant but harmless:
    // - `Cow<'a, str>` already explicitly uses `'a`
    // - The compiler automatically ensures `'a` outlives itself
    // - The bound didn't add extra constraints, just made them explicit
    //
    // THE FILE DESCRIPTOR PROBLEM:
    // When introducing generic `Fd` parameter to support both `OwnedFd` and
    // `BorrowedFd<'fd>`, the bound became problematic:
    //
    // 1. File descriptors are HANDLES (integers), not memory pointers
    // 2. `BorrowedFd<'fd>` lifetime relates to the `OwnedFd` owner, not buffer data
    // 3. These are independent lifetime domains:
    //    - `'a`: How long the message buffer memory lives
    //    - `'fd`: How long the file descriptor handle remains valid
    //
    // THE ACTUAL CONSTRAINT PROBLEM:
    // With `Output<'a>: 'a` and `Fd = BorrowedFd<'fd>`, the compiler required:
    // `BorrowedFd<'fd>: 'a` which means `'fd: 'a`
    //
    // This artificially ties fd handle lifetime to buffer lifetime, preventing
    // valid use cases where:
    // - Buffer is short-lived (temporary parsing)
    // - File descriptor should outlive the buffer (normal usage)
    //
    // CURRENT SOLUTION:
    // Removed the bound since:
    // 1. For `Cow<'a, T>` types, the `'a` constraint is already implicit
    // 2. For `Fd` types, no buffer lifetime constraint should exist
    // 3. The compiler will still enforce memory safety without the explicit bound
    //
    // NOTE: The `Wire` trait still has `Output<'a>: 'a` because it deals with
    // individual field types that might need explicit lifetime constraints.
    // `MessageWire` deals with complete message structures where the relationships
    // are already established through the field types themselves.
    type Output<'a>;

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
