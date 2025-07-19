//! Testing ground for the work done on the project.

use std::alloc::Layout;
use std::alloc::alloc;
use std::ffi::CStr;
use std::io::Read;
use std::io::Write;
use std::mem::MaybeUninit;
use std::mem::{self};

use wayland_client::Connection;
use wayland_client::protocol::wayland as wl;

fn main()
{
    let mut conn = Connection::connect().unwrap();

    let id_display: u32 = 1; // wl_display (always assigned to 1)
    let id_registry: u32 = 2; // wl_registry
    let id_callback: u32 = 3; // wl_callback

    // -- GET REGISTRY --
    let size: u16 = 12;
    let opcode: u16 = wl::display::request::GetRegistry::OPCODE; // wl_display::get_registry

    let req = wl::display::request::GetRegistry {
        registry: id_registry,
    };

    let layout = Layout::from_size_align(size as usize, align_of::<u32>()).unwrap();
    let mut buf = unsafe { Vec::from_raw_parts(alloc(layout), size as usize, size as usize) };

    unsafe {
        *buf.as_mut_ptr().add(0).cast() = id_display;
        *buf.as_mut_ptr().add(4).cast() = ((size as u32) << 16) | (opcode as u32);
        *buf.as_mut_ptr().add(8).cast() = req.registry;
    }

    conn.stream.write_all(&mut buf).unwrap();

    // -- SYNC --
    let size: u16 = 12;
    let opcode: u16 = wl::display::request::Sync::OPCODE;

    let req = wl::display::request::Sync {
        callback: id_callback,
    };

    let layout = Layout::from_size_align(size as usize, align_of::<u32>()).unwrap();
    let mut buf = unsafe { Vec::from_raw_parts(alloc(layout), size as usize, size as usize) };

    unsafe {
        *buf.as_mut_ptr().add(0).cast() = id_display;
        *buf.as_mut_ptr().add(4).cast() = ((size as u32) << 16) | (opcode as u32);
        *buf.as_mut_ptr().add(8).cast() = req.callback;
    }

    conn.stream.write_all(&mut buf).unwrap();

    // Read
    loop {
        let mut header: [u8; 2 * size_of::<u32>()] =
            unsafe { mem::transmute([const { MaybeUninit::<u32>::uninit() }; 2]) };
        conn.stream.read_exact(&mut header).unwrap();

        let oid = unsafe { *header.as_ptr().add(0).cast::<u32>() };
        let size = unsafe { *header.as_ptr().add(4).cast::<u32>() >> 16 } as usize - header.len();
        let opcode = unsafe { *header.as_ptr().add(4).cast::<u32>() & 0xFFFF } as u16;

        assert!(Layout::from_size_align(u16::MAX as usize, align_of::<u32>()).is_ok());
        let layout = unsafe { Layout::from_size_align_unchecked(size, align_of::<u32>()) };
        let mut buf = unsafe { Vec::from_raw_parts(alloc(layout), size, size) };

        conn.stream.read_exact(&mut buf).unwrap();

        match (oid, opcode) {
            (_, wl::registry::event::Global::OPCODE) if oid == id_registry => {
                let name: u32 = unsafe { *buf.as_ptr().add(0).cast() };
                let string_size: u32 = unsafe { *buf.as_ptr().add(4).cast() };
                assert_eq!(
                    unsafe { *buf.as_ptr().add(8).add(string_size as usize - 1) },
                    0
                );
                let iface = unsafe { CStr::from_ptr(buf.as_ptr().add(8).cast()) };
                let padding =
                    (string_size as usize + align_of::<u32>() - 1) & !(align_of::<u32>() - 1);
                let version: u32 = unsafe { *buf.as_ptr().add(8).add(padding).cast() };

                let evt = wl::registry::event::Global {
                    name,
                    interface: iface.to_owned().into_string().unwrap(),
                    version,
                };

                println!("{:?}", evt);
            }
            (_, wl::callback::event::Done::OPCODE) if oid == id_callback => {
                let callback_data: u32 = unsafe { *buf.as_ptr().add(0).cast() };

                let evt = wl::callback::event::Done { callback_data };

                println!("{:?}", evt);

                break;
            }
            _ => panic!("unexpected oid: {oid}, opcode: {opcode}"),
        }
    }
}
