use std::env;
use std::ffi::OsString;
use std::io;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use thiserror::Error;

#[rustfmt::skip]
pub mod protocol
{
    pub mod wayland {
        use wayland_scanner::generate;
        generate!("./wayland/protocol/wayland.xml");
    }
}

pub struct Connection
{
    pub stream: UnixStream,
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

        Ok(Self { stream })
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
