//! Testing ground for the work done on the project.

use std::io::ErrorKind;
use std::io::Read;

use wayland_client::Connection;
use wayland_client::protocol::wayland as wl;
use wayland_client::protocol::wayland::WlCallback;
use wayland_client::protocol::wayland::WlDisplay;
use wayland_client::protocol::wayland::WlRegistry;
use wayland_client::protocol::wayland::wl_callback::event::Done;
use wayland_client::protocol::wayland::wl_display::event::Error;
use wayland_client::protocol::wayland::wl_display::request::GetRegistry;
use wayland_client::protocol::wayland::wl_display::request::Sync;
use wayland_client::protocol::wayland::wl_registry::event::Global;
use wayland_core::Header;
use wayland_core::Message;
use wayland_core::Object;
use wayland_core::Wire;
use wayland_core::WireError;
use wayland_core::bytes_mut;
use wayland_core::prepare_buf;

fn debug_buf(buf: &[u32])
{
    buf.iter()
        .enumerate()
        .for_each(|(i, word)| println!("word[{:02}] = {:08x}", i, word));
}

fn read_global(buf: &[u32]) -> Result<Global, WireError>
{
    let (buf, name) = u32::read(buf)?;
    let (buf, interface) = String::read(buf)?;
    let (buf, version) = u32::read(buf)?;
    let _ = buf;

    Ok(Global {
        name,
        interface,
        version,
    })
}

fn read_done(buf: &[u32]) -> Result<Done, WireError>
{
    let (buf, callback_data) = u32::read(buf)?;
    let _ = buf;

    Ok(Done { callback_data })
}

fn read_error(buf: &[u32]) -> Result<Error, WireError>
{
    let (buf, object_id) = Object::read(buf)?;
    let (buf, code) = u32::read(buf)?;
    let (buf, message) = String::read(buf)?;
    let _ = buf;

    Ok(Error {
        object_id,
        code,
        message: message.to_string(),
    })
}

fn main()
{
    let mut conn = Connection::connect().unwrap();
    let mut buf = Vec::new();

    let id_display: WlDisplay = Object::new(1).unwrap().into(); // wl_display (always assigned to 1)
    let id_registry: WlRegistry = Object::new(2).unwrap().into(); // wl_registry
    let id_callback: WlCallback = Object::new(3).unwrap().into(); // wl_callback

    // -- GET REGISTRY --
    conn.send_message(
        id_display,
        &GetRegistry {
            registry: id_registry,
        },
    )
    .unwrap();
    debug_buf(conn.inner_buf());

    // -- SYNC --
    conn.send_message(
        id_display,
        &Sync {
            callback: id_callback,
        },
    )
    .unwrap();
    debug_buf(conn.inner_buf());

    // Read
    loop {
        let mut header = [0u32; 2];
        match conn
            .stream
            .read_exact(bytes_mut(&mut header))
            .map_err(|err| err.kind())
        {
            Err(ErrorKind::UnexpectedEof) => {
                eprintln!("connection severed");
                break;
            }
            _ => {}
        }
        let (_, header) = Header::read(&header).unwrap();

        assert!(header.size % 4 == 0, "message must be 4 bytes aligned");
        assert!(
            header.size >= 8,
            "message size must include the header size"
        );
        prepare_buf(&mut buf, (header.size as usize - 8 + 3) / 4);

        conn.stream.read_exact(bytes_mut(&mut buf)).unwrap();

        match header.opcode {
            wl::wl_registry::event::Global::OPCODE if header.object == id_registry.into() => {
                let evt = read_global(&buf);
                println!("{:?}", evt);
            }
            wl::wl_callback::event::Done::OPCODE if header.object == id_callback.into() => {
                let evt = read_done(&buf);
                println!("{:?}", evt);

                break;
            }
            wl::wl_display::event::Error::OPCODE if header.object == id_display.into() => {
                let evt = read_error(&buf);
                println!("{:?}", evt);
            }
            _ => eprintln!(
                "unexpected object: {}, opcode: {}. ignoring...",
                header.object.get(),
                header.opcode
            ),
        }
    }
}
