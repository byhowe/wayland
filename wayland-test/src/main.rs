//! Testing ground for the work done on the project.

use std::alloc::Layout;
use std::alloc::alloc;
use std::ffi::CStr;
use std::io::Read;
use std::io::Write;
use std::mem;
use std::mem::MaybeUninit;

use wayland_client::Connection;
use wayland_client::protocol::wayland as wl;
use wayland_client::protocol::wayland::wl_display::request::GetRegistry;
use wayland_client::protocol::wayland::wl_display::request::Sync;

fn main()
{
    let mut conn = Connection::connect().unwrap();

    let id_display: u32 = 1; // wl_display (always assigned to 1)
    let id_registry: u32 = 2; // wl_registry
    let id_callback: u32 = 3; // wl_callback

    // -- GET REGISTRY --
    let opcode: u16 = GetRegistry::OPCODE; // wl_display::get_registry

    let req = GetRegistry {
        registry: id_registry,
    };

    let size = req.size() + 8;
    println!("get_registry size: {size}");

    let layout = Layout::from_size_align(size, align_of::<u32>()).unwrap();
    let mut buf = unsafe { Vec::from_raw_parts(alloc(layout), size, size) };

    unsafe {
        *buf.as_mut_ptr().add(0).cast() = id_display;
        *buf.as_mut_ptr().add(4).cast() = ((size as u32) << 16) | (opcode as u32);
        *buf.as_mut_ptr().add(8).cast() = req.registry;
    }

    conn.stream.write_all(&mut buf).unwrap();

    // -- SYNC --
    let opcode: u16 = Sync::OPCODE;

    let req = Sync {
        callback: id_callback,
    };

    let size = req.size() + 8;
    println!("sync size: {size}");

    let layout = Layout::from_size_align(size, align_of::<u32>()).unwrap();
    let mut buf = unsafe { Vec::from_raw_parts(alloc(layout), size, size) };

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

        match opcode {
            wl::wl_registry::event::Global::OPCODE if oid == id_registry => {
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

                let evt = wl::wl_registry::event::Global {
                    name,
                    interface: iface.to_owned().into_string().unwrap(),
                    version,
                };

                println!("{:?}", evt);
            }
            wl::wl_callback::event::Done::OPCODE if oid == id_callback => {
                let callback_data: u32 = unsafe { *buf.as_ptr().add(0).cast() };

                let evt = wl::wl_callback::event::Done { callback_data };

                println!("{:?}", evt);

                break;
            }
            _ => eprintln!("unexpected oid: {oid}, opcode: {opcode}. ignoring..."),
        }
    }
}
