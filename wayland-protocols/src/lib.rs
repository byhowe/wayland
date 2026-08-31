#![allow(unreachable_code)]
// #![cfg_attr(rustfmt, rustfmt_skip)]

use wayland_client::protocol::wayland;
use wayland_scanner::generate;

generate!(
    "../upstream/wayland-protocols/staging/ext-image-copy-capture/ext-image-copy-capture-v1.xml",
    dependencies: [

        crate::ext_image_capture_source_v1::ext_image_capture_source_v1,
        crate::wayland::wl_buffer,
        crate::wayland::wl_output,
        crate::wayland::wl_pointer,
        crate::wayland::wl_shm
    ,
    ],
);

generate!(
    "../upstream/wayland-protocols/staging/fifo/fifo-v1.xml",
    dependencies: [
        crate::wayland::wl_surface,
    ],
);

generate!("../upstream/wayland-protocols/staging/ext-transient-seat/ext-transient-seat-v1.xml");

generate!(
    "../upstream/wayland-protocols/staging/alpha-modifier/alpha-modifier-v1.xml",
    dependencies: [
        crate::wayland::wl_surface,
    ],
);

generate!(
    "../upstream/wayland-protocols/staging/ext-session-lock/ext-session-lock-v1.xml",
    dependencies: [
        crate::wayland::wl_surface,
        crate::wayland::wl_output,
    ],
);

generate!("../upstream/wayland-protocols/staging/drm-lease/drm-lease-v1.xml");

generate!(
    "../upstream/wayland-protocols/staging/linux-drm-syncobj/linux-drm-syncobj-v1.xml",
    dependencies: [
        crate::wayland::wl_surface,
    ],
);

generate!(
    "../upstream/wayland-protocols/staging/ext-image-capture-source/ext-image-capture-source-v1.xml",
    dependencies: [
        crate::ext_foreign_toplevel_list_v1::ext_foreign_toplevel_handle_v1,
        crate::wayland::wl_output,
    ],
);

generate!(
    "../upstream/wayland-protocols/staging/ext-workspace/ext-workspace-v1.xml",
    dependencies: [
        crate::wayland::wl_output,
    ],
);

generate!(
    "../upstream/wayland-protocols/staging/color-representation/color-representation-v1.xml",
    dependencies: [
        crate::wayland::wl_surface,
    ],
);

generate!(
    "../upstream/wayland-protocols/staging/xdg-dialog/xdg-dialog-v1.xml",
    dependencies: [
        crate::xdg_shell::xdg_toplevel,
    ],
);

generate!(
    "../upstream/wayland-protocols/staging/color-management/color-management-v1.xml",
    dependencies: [
        crate::wayland::wl_surface,
        crate::wayland::wl_output,
    ],
);

generate!(
    "../upstream/wayland-protocols/staging/xdg-toplevel-tag/xdg-toplevel-tag-v1.xml",
    dependencies: [
        crate::xdg_shell::xdg_toplevel,
    ],
);

generate!(
    "../upstream/wayland-protocols/staging/single-pixel-buffer/single-pixel-buffer-v1.xml",
    dependencies: [
        crate::wayland::wl_buffer,
    ],
);

generate!(
    "../upstream/wayland-protocols/staging/xdg-activation/xdg-activation-v1.xml",
    dependencies: [
        crate::wayland::wl_seat,
        crate::wayland::wl_surface,
    ],
);

generate!(
    "../upstream/wayland-protocols/staging/ext-idle-notify/ext-idle-notify-v1.xml",
    dependencies: [
        crate::wayland::wl_seat,
    ],
);

generate!("../upstream/wayland-protocols/staging/security-context/security-context-v1.xml");

generate!(
    "../upstream/wayland-protocols/staging/ext-background-effect/ext-background-effect-v1.xml",
    dependencies: [
        crate::wayland::wl_surface,
        crate::wayland::wl_region,
    ],
);

generate!(
    "../upstream/wayland-protocols/staging/content-type/content-type-v1.xml",
    dependencies: [
        crate::wayland::wl_surface,
    ],
);

generate!(
    "../upstream/wayland-protocols/staging/ext-data-control/ext-data-control-v1.xml",
    dependencies: [
        crate::wayland::wl_seat,
    ],
);

generate!(
    "../upstream/wayland-protocols/staging/pointer-warp/pointer-warp-v1.xml",
    dependencies: [
        crate::wayland::wl_pointer,
        crate::wayland::wl_surface,
    ],
);

generate!(
    "../upstream/wayland-protocols/staging/xdg-toplevel-icon/xdg-toplevel-icon-v1.xml",
    dependencies: [
        crate::wayland::wl_buffer,
        crate::xdg_shell::xdg_toplevel,
    ],
);

generate!(
    "../upstream/wayland-protocols/staging/xdg-toplevel-drag/xdg-toplevel-drag-v1.xml",
    dependencies: [
        crate::xdg_shell::xdg_toplevel,
        crate::wayland::wl_data_source,
    ],
);

generate!(
    "../upstream/wayland-protocols/staging/cursor-shape/cursor-shape-v1.xml",
    dependencies: [
        crate::tablet_v2::zwp_tablet_tool_v2,
        crate::wayland::wl_pointer,
    ],
);

generate!(
    "../upstream/wayland-protocols/staging/ext-foreign-toplevel-list/ext-foreign-toplevel-list-v1.xml",
);

generate!(
    "../upstream/wayland-protocols/staging/xwayland-shell/xwayland-shell-v1.xml",
    dependencies: [
        crate::wayland::wl_surface,
    ],
);

generate!(
    "../upstream/wayland-protocols/staging/xdg-system-bell/xdg-system-bell-v1.xml",
    dependencies: [
        crate::wayland::wl_surface,
    ],
);

generate!(
    "../upstream/wayland-protocols/staging/tearing-control/tearing-control-v1.xml",
    dependencies: [
        crate::wayland::wl_surface,
    ],
);

generate!(
    "../upstream/wayland-protocols/staging/commit-timing/commit-timing-v1.xml",
    dependencies: [
        crate::wayland::wl_surface,
    ],
);

generate!(
    "../upstream/wayland-protocols/staging/fractional-scale/fractional-scale-v1.xml",
    dependencies: [
        crate::wayland::wl_surface,
    ],
);

generate!(
    "../upstream/wayland-protocols/experimental/xx-text-input/xx-text-input-v3.xml",
    dependencies: [
        crate::wayland::wl_surface,
        crate::wayland::wl_seat,
    ],
);

generate!(
    "../upstream/wayland-protocols/experimental/xx-input-method/xx-input-method-v2.xml",
    dependencies: [
        crate::wayland::wl_seat,
        crate::wayland::wl_surface,
        crate::xx_text_input_unstable_v3::xx_text_input_v3,
    ],
);

generate!(
    "../upstream/wayland-protocols/experimental/xx-session-management/xx-session-management-v1.xml",
    dependencies: [
        crate::xdg_shell::xdg_toplevel,
    ],
);

generate!(
    "../upstream/wayland-protocols/unstable/keyboard-shortcuts-inhibit/keyboard-shortcuts-inhibit-unstable-v1.xml",
    dependencies: [
        crate::wayland::wl_seat,
        crate::wayland::wl_surface,
    ],
);

generate!(
    "../upstream/wayland-protocols/unstable/pointer-constraints/pointer-constraints-unstable-v1.xml",
    dependencies: [
        crate::wayland::wl_surface,
        crate::wayland::wl_pointer,
        crate::wayland::wl_region,
    ],
);

generate!(
    "../upstream/wayland-protocols/unstable/primary-selection/primary-selection-unstable-v1.xml",
    dependencies: [
        crate::wayland::wl_seat,
    ],
);

generate!(
    "../upstream/wayland-protocols/unstable/xdg-foreign/xdg-foreign-unstable-v1.xml",
    dependencies: [
        crate::wayland::wl_surface,
    ],
);

generate!(
    "../upstream/wayland-protocols/unstable/xdg-foreign/xdg-foreign-unstable-v2.xml",
    dependencies: [
        crate::wayland::wl_surface,
    ],
);

generate!(
    "../upstream/wayland-protocols/unstable/xwayland-keyboard-grab/xwayland-keyboard-grab-unstable-v1.xml",
    dependencies: [
        crate::wayland::wl_seat,
        crate::wayland::wl_surface,
    ],
);

generate!(
    "../upstream/wayland-protocols/unstable/idle-inhibit/idle-inhibit-unstable-v1.xml",
    dependencies: [
        crate::wayland::wl_surface,
    ],
);

generate!(
    "../upstream/wayland-protocols/unstable/input-method/input-method-unstable-v1.xml",
    dependencies: [
        crate::wayland::wl_output,
        crate::wayland::wl_surface,
        crate::wayland::wl_keyboard,
    ],
);

generate!(
    "../upstream/wayland-protocols/unstable/pointer-gestures/pointer-gestures-unstable-v1.xml",
    dependencies: [
        crate::wayland::wl_pointer,
        crate::wayland::wl_surface,
    ],
);

generate!(
    "../upstream/wayland-protocols/unstable/xdg-shell/xdg-shell-unstable-v5.xml",
    dependencies: [
        crate::wayland::wl_output,
        crate::wayland::wl_seat,
        crate::wayland::wl_surface,
    ],
);

generate!(
    "../upstream/wayland-protocols/unstable/xdg-shell/xdg-shell-unstable-v6.xml",
    dependencies: [
        crate::wayland::wl_seat,
        crate::wayland::wl_output,
        crate::wayland::wl_surface,
    ],
);

generate!(
    "../upstream/wayland-protocols/unstable/relative-pointer/relative-pointer-unstable-v1.xml",
    dependencies: [
        crate::wayland::wl_pointer,
    ],
);

generate!(
    "../upstream/wayland-protocols/unstable/text-input/text-input-unstable-v1.xml",
    dependencies: [
        crate::wayland::wl_surface,
        crate::wayland::wl_seat,
    ],
);

generate!(
    "../upstream/wayland-protocols/unstable/text-input/text-input-unstable-v3.xml",
    dependencies: [
        crate::wayland::wl_surface,
        crate::wayland::wl_seat,
    ],
);

generate!(
    "../upstream/wayland-protocols/unstable/xdg-decoration/xdg-decoration-unstable-v1.xml",
    dependencies: [
        crate::xdg_shell::xdg_toplevel,
    ],
);

generate!(
    "../upstream/wayland-protocols/unstable/tablet/tablet-unstable-v1.xml",
    dependencies: [
        crate::wayland::wl_surface,
        crate::wayland::wl_seat,
    ],
);

generate!(
    "../upstream/wayland-protocols/unstable/tablet/tablet-unstable-v2.xml",
    dependencies: [

        crate::wayland::wl_seat,
        crate::wayland::wl_surface,
]
);

generate!(
    "../upstream/wayland-protocols/unstable/fullscreen-shell/fullscreen-shell-unstable-v1.xml",
    dependencies: [
        crate::wayland::wl_output,
        crate::wayland::wl_surface,
    ],
);

generate!(
    "../upstream/wayland-protocols/unstable/linux-explicit-synchronization/linux-explicit-synchronization-unstable-v1.xml",
    dependencies: [
        crate::wayland::wl_surface,
    ],
);

generate!(
    "../upstream/wayland-protocols/unstable/linux-dmabuf/linux-dmabuf-unstable-v1.xml",
    dependencies: [
        crate::wayland::wl_buffer,
        crate::wayland::wl_surface,
    ],
);

generate!(
    "../upstream/wayland-protocols/unstable/xdg-output/xdg-output-unstable-v1.xml",
    dependencies: [
        crate::wayland::wl_output,
    ],
);

generate!(
    "../upstream/wayland-protocols/unstable/input-timestamps/input-timestamps-unstable-v1.xml",
    dependencies: [
        crate::wayland::wl_touch,
        crate::wayland::wl_keyboard,
        crate::wayland::wl_pointer,
    ],
);

generate!(
    "../upstream/wayland-protocols/stable/viewporter/viewporter.xml",
    dependencies: [
        crate::wayland::wl_surface,
    ],
);

generate!(
    "../upstream/wayland-protocols/stable/xdg-shell/xdg-shell.xml",
    dependencies: [
        crate::wayland::wl_surface,
        crate::wayland::wl_output,
        crate::wayland::wl_seat,
    ],
);

generate!(
    "../upstream/wayland-protocols/stable/presentation-time/presentation-time.xml",
    dependencies: [
        crate::wayland::wl_output,
        crate::wayland::wl_surface,
    ],
);

generate!(
    "../upstream/wayland-protocols/stable/tablet/tablet-v2.xml",
    dependencies: [

        crate::wayland::wl_surface,
        crate::wayland::wl_seat,
]
);

generate!(
    "../upstream/wayland-protocols/stable/linux-dmabuf/linux-dmabuf-v1.xml",
    dependencies: [

        crate::wayland::wl_surface,
        crate::wayland::wl_buffer,
]
);
