//! Testing ground for the work done on the project.

use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;

use wayland_client::Connection;
use wayland_client::protocol::wayland as wl;
use wayland_client::protocol::wayland::wl_callback::event::Done;
use wayland_client::protocol::wayland::wl_display::event::Error;
use wayland_client::protocol::wayland::wl_display::request::GetRegistry;
use wayland_client::protocol::wayland::wl_display::request::Sync;
use wayland_client::protocol::wayland::wl_registry::event::Global;
use wayland_core::Header;
use wayland_core::bytes;
use wayland_core::bytes_mut;
use wayland_core::prepare_buf;
use wayland_core::read_header;
use wayland_core::read_string;
use wayland_core::read_uint;
use wayland_core::write_header;
use wayland_core::write_uint;

fn debug_buf(buf: &[u32])
{
    buf.iter()
        .enumerate()
        .for_each(|(i, word)| println!("word[{:02}] = {:08x}", i, word));
}

fn write_get_registry<'buf>(buf: &'buf mut Vec<u32>, object: u32, msg: &GetRegistry)
{
    let count = msg.count() + 2; // +2 for the header size
    prepare_buf(buf, count);
    let opcode = GetRegistry::OPCODE;
    let buf = write_header(buf, Header::new(object, count as u16 * 4, opcode));
    let buf = write_uint(buf, msg.registry);
}

fn write_sync<'buf>(buf: &'buf mut Vec<u32>, object: u32, msg: &Sync)
{
    let count = msg.count() + 2;
    prepare_buf(buf, count);
    let opcode = Sync::OPCODE;
    let buf = write_header(buf, Header::new(object, count as u16 * 4, opcode));
    let buf = write_uint(buf, msg.callback);
}

fn read_global(buf: &[u32]) -> Global
{
    let (buf, name) = read_uint(buf);
    let (buf, interface) = read_string(buf);
    let (buf, version) = read_uint(buf);
    let _ = buf;

    Global {
        name,
        interface: interface.to_string(),
        version,
    }
}

fn read_done(buf: &[u32]) -> Done
{
    let (buf, callback_data) = read_uint(buf);
    let _ = buf;

    Done { callback_data }
}

fn read_error(buf: &[u32]) -> Error
{
    let (buf, object_id) = read_uint(buf);
    let (buf, code) = read_uint(buf);
    let (buf, message) = read_string(buf);
    let _ = buf;

    Error {
        object_id,
        code,
        message: message.to_string(),
    }
}

fn main()
{
    let mut conn = Connection::connect().unwrap();
    let mut buf = Vec::new();

    let id_display: u32 = 1; // wl_display (always assigned to 1)
    let id_registry: u32 = 2; // wl_registry
    let id_callback: u32 = 3; // wl_callback

    // -- GET REGISTRY --
    let req = GetRegistry {
        registry: id_registry,
    };
    write_get_registry(&mut buf, id_display, &req);
    conn.stream.write_all(bytes(&buf)).unwrap();
    debug_buf(&buf);

    // -- SYNC --
    let req = Sync {
        callback: id_callback,
    };
    write_sync(&mut buf, id_display, &req);
    conn.stream.write_all(bytes(&buf)).unwrap();
    debug_buf(&buf);

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
        let (_, header) = read_header(&header);

        assert!(header.size % 4 == 0, "message must be 4 bytes aligned");
        assert!(
            header.size >= 8,
            "message size must include the header size"
        );
        prepare_buf(&mut buf, (header.size as usize - 8 + 3) / 4);

        conn.stream.read_exact(bytes_mut(&mut buf)).unwrap();

        match header.opcode {
            wl::wl_registry::event::Global::OPCODE if header.object == id_registry => {
                let evt = read_global(&buf);
                println!("{:?}", evt);
            }
            wl::wl_callback::event::Done::OPCODE if header.object == id_callback => {
                let evt = read_done(&buf);
                println!("{:?}", evt);

                break;
            }
            wl::wl_display::event::Error::OPCODE if header.object == id_display => {
                let evt = read_error(&buf);
                println!("{:?}", evt);
            }
            _ => eprintln!(
                "unexpected object: {}, opcode: {}. ignoring...",
                header.object, header.opcode
            ),
        }
    }
}
