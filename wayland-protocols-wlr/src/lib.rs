#![allow(unreachable_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]

pub mod wlr_virtual_pointer_unstable_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wlr-protocols/unstable/wlr-virtual-pointer-unstable-v1.xml");
}

pub mod wlr_input_inhibit_unstable_v1
{
    use wayland_scanner::generate;

    generate!("../upstream/wlr-protocols/unstable/wlr-input-inhibitor-unstable-v1.xml");
}

pub mod wlr_data_control_unstable_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wlr-protocols/unstable/wlr-data-control-unstable-v1.xml");
}

pub mod wlr_foreign_toplevel_management_unstable_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wlr-protocols/unstable/wlr-foreign-toplevel-management-unstable-v1.xml");
}

pub mod wlr_layer_shell_unstable_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_protocols::xdg_shell::xdg_popup;
    use wayland_scanner::generate;

    generate!("../upstream/wlr-protocols/unstable/wlr-layer-shell-unstable-v1.xml");
}

pub mod wlr_output_management_unstable_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wlr-protocols/unstable/wlr-output-management-unstable-v1.xml");
}

pub mod wlr_export_dmabuf_unstable_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wlr-protocols/unstable/wlr-export-dmabuf-unstable-v1.xml");
}

pub mod wlr_gamma_control_unstable_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wlr-protocols/unstable/wlr-gamma-control-unstable-v1.xml");
}

pub mod wlr_screencopy_unstable_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wlr-protocols/unstable/wlr-screencopy-unstable-v1.xml");
}

pub mod wlr_output_power_management_unstable_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wlr-protocols/unstable/wlr-output-power-management-unstable-v1.xml");
}
