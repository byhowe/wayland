use std::env;
use std::ffi::OsString;
use std::io;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use thiserror::Error;
use wayland_core::Header;
use wayland_core::MessageOpcode;
use wayland_core::MessageWire;
use wayland_core::Object;
use wayland_core::Wire;
use wayland_core::bytes;
use wayland_core::prepare_buf;

#[rustfmt::skip]
pub mod protocol
{
    #![allow(unreachable_code)]

    wayland_scanner::generate!("../upstream/wayland/protocol/wayland.xml");
}

pub struct Connection
{
    pub stream: UnixStream,
    buf: Vec<u32>,
}

impl Connection
{
    pub fn connect() -> Result<Self, ConnectError>
    {
        let socket_name: PathBuf = env::var_os("WAYLAND_DISPLAY")
            .unwrap_or(OsString::from("wayland-0"))
            .into();

        let socket_path = if socket_name.is_absolute() {
            socket_name
        } else {
            let mut xdg_path = env::var_os("XDG_RUNTIME_DIR")
                .map(Into::<PathBuf>::into)
                .filter(|dir| dir.is_absolute())
                .ok_or(ConnectError::XdgInvalid)?;
            xdg_path.push(socket_name);
            xdg_path
        };

        // The CLOEXEC flag is set on supported platforms.
        let stream = UnixStream::connect(socket_path)?;

        Ok(Self {
            stream,
            buf: Vec::new(),
        })
    }

    pub fn send_message<T, O: Into<Object<T>>, M: MessageWire + MessageOpcode>(
        &mut self,
        object: O,
        msg: &M,
    ) -> io::Result<()>
    {
        let size = 2 + msg.message_size();
        let header = Header::new(object.into().plain(), size as u16 * 4, M::OPCODE);

        prepare_buf(&mut self.buf, size);
        let buf = header.wire_write(&mut self.buf);
        msg.message_write(buf);

        self.stream.write_all(bytes(&self.buf))?;

        Ok(())
    }

    pub fn inner_buf<'conn>(&'conn self) -> &'conn [u32]
    {
        &self.buf
    }
}

#[derive(Error, Debug)]
pub enum ConnectError
{
    #[error("failed to connect to the wayland socket")]
    Io(#[from] io::Error),
    #[error("invalid XDG_RUNTIME_DIR path")]
    XdgInvalid,
}
