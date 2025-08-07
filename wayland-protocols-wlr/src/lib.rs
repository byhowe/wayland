#![allow(unreachable_code)]

use wayland_client::protocol::wayland;
use wayland_protocols::*;
use wayland_scanner::generate;

generate!(
    "../upstream/wlr-protocols/unstable/wlr-virtual-pointer-unstable-v1.xml",
    dependencies: [
        crate::wayland::wl_seat,
        crate::wayland::wl_output,
        crate::wayland::wl_pointer,
    ],
);

generate!("../upstream/wlr-protocols/unstable/wlr-input-inhibitor-unstable-v1.xml");

generate!(
    "../upstream/wlr-protocols/unstable/wlr-data-control-unstable-v1.xml",
    dependencies: [
        crate::wayland::wl_seat,
    ],
);

generate!(
    "../upstream/wlr-protocols/unstable/wlr-foreign-toplevel-management-unstable-v1.xml",
    dependencies: [
        crate::wayland::wl_output,
        crate::wayland::wl_surface,
        crate::wayland::wl_seat,
    ],
);

generate!(
    "../upstream/wlr-protocols/unstable/wlr-layer-shell-unstable-v1.xml",
    dependencies: [
        crate::xdg_shell_unstable_v5::xdg_popup,
        crate::wayland::wl_surface,
        crate::wayland::wl_output,
    ],
);

generate!(
    "../upstream/wlr-protocols/unstable/wlr-output-management-unstable-v1.xml",
    dependencies: [
        crate::wayland::wl_output,
    ],
);

generate!(
    "../upstream/wlr-protocols/unstable/wlr-export-dmabuf-unstable-v1.xml",
    dependencies: [
        crate::wayland::wl_output,
    ],
);

generate!(
    "../upstream/wlr-protocols/unstable/wlr-gamma-control-unstable-v1.xml",
    dependencies: [
        crate::wayland::wl_output,
    ],
);

generate!(
    "../upstream/wlr-protocols/unstable/wlr-screencopy-unstable-v1.xml",
    dependencies: [
        crate::wayland::wl_output,
        crate::wayland::wl_buffer,
        crate::wayland::wl_shm,
    ],
);

generate!(
    "../upstream/wlr-protocols/unstable/wlr-output-power-management-unstable-v1.xml",
    dependencies: [
        crate::wayland::wl_output,
    ],
);
