#![allow(unreachable_code)]

pub mod ext_image_copy_capture_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    use crate::ext_image_capture_source_v1::ext_image_capture_source_v1;

    generate!("../upstream/wayland-protocols/staging/ext-image-copy-capture/ext-image-copy-capture-v1.xml");
}

pub mod fifo_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/staging/fifo/fifo-v1.xml");
}

pub mod ext_transient_seat_v1
{
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/staging/ext-transient-seat/ext-transient-seat-v1.xml");
}

pub mod alpha_modifier_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/staging/alpha-modifier/alpha-modifier-v1.xml");
}

pub mod ext_session_lock_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/staging/ext-session-lock/ext-session-lock-v1.xml");
}

pub mod drm_lease_v1
{
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/staging/drm-lease/drm-lease-v1.xml");
}

pub mod linux_drm_syncobj_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/staging/linux-drm-syncobj/linux-drm-syncobj-v1.xml");
}

pub mod ext_image_capture_source_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    use crate::ext_foreign_toplevel_list_v1::ext_foreign_toplevel_handle_v1;

    generate!("../upstream/wayland-protocols/staging/ext-image-capture-source/ext-image-capture-source-v1.xml");
}

pub mod ext_workspace_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/staging/ext-workspace/ext-workspace-v1.xml");
}

pub mod color_representation_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/staging/color-representation/color-representation-v1.xml");
}

pub mod xdg_dialog_v1
{
    use wayland_scanner::generate;

    use crate::xdg_shell::xdg_toplevel;

    generate!("../upstream/wayland-protocols/staging/xdg-dialog/xdg-dialog-v1.xml");
}

pub mod color_management_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/staging/color-management/color-management-v1.xml");
}

pub mod xdg_toplevel_tag_v1
{
    use wayland_scanner::generate;

    use crate::xdg_shell::xdg_toplevel;

    generate!("../upstream/wayland-protocols/staging/xdg-toplevel-tag/xdg-toplevel-tag-v1.xml");
}

pub mod single_pixel_buffer_v1
{
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/staging/single-pixel-buffer/single-pixel-buffer-v1.xml");
}

pub mod xdg_activation_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/staging/xdg-activation/xdg-activation-v1.xml");
}

pub mod ext_idle_notify_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/staging/ext-idle-notify/ext-idle-notify-v1.xml");
}

pub mod security_context_v1
{
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/staging/security-context/security-context-v1.xml");
}

pub mod ext_background_effect_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/staging/ext-background-effect/ext-background-effect-v1.xml");
}

pub mod content_type_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/staging/content-type/content-type-v1.xml");
}

pub mod ext_data_control_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/staging/ext-data-control/ext-data-control-v1.xml");
}

pub mod pointer_warp_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/staging/pointer-warp/pointer-warp-v1.xml");
}

pub mod xdg_toplevel_icon_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    use crate::xdg_shell::xdg_toplevel;

    generate!("../upstream/wayland-protocols/staging/xdg-toplevel-icon/xdg-toplevel-icon-v1.xml");
}

pub mod xdg_toplevel_drag_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    use crate::xdg_shell::xdg_toplevel;

    generate!("../upstream/wayland-protocols/staging/xdg-toplevel-drag/xdg-toplevel-drag-v1.xml");
}

pub mod cursor_shape_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    use crate::tablet_v2::zwp_tablet_tool_v2;

    generate!("../upstream/wayland-protocols/staging/cursor-shape/cursor-shape-v1.xml");
}

pub mod ext_foreign_toplevel_list_v1
{
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/staging/ext-foreign-toplevel-list/ext-foreign-toplevel-list-v1.xml");
}

pub mod xwayland_shell_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/staging/xwayland-shell/xwayland-shell-v1.xml");
}

pub mod xdg_system_bell_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/staging/xdg-system-bell/xdg-system-bell-v1.xml");
}

pub mod tearing_control_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/staging/tearing-control/tearing-control-v1.xml");
}

pub mod commit_timing_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/staging/commit-timing/commit-timing-v1.xml");
}

pub mod fractional_scale_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/staging/fractional-scale/fractional-scale-v1.xml");
}

pub mod input_method_experimental_v2
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    use crate::text_input_unstable_v3::zwp_text_input_v3;

    generate!("../upstream/wayland-protocols/experimental/xx-input-method/xx-input-method-v2.xml");
}

pub mod xx_session_management_v1
{
    use wayland_scanner::generate;

    use crate::xdg_shell::xdg_toplevel;

    generate!("../upstream/wayland-protocols/experimental/xx-session-management/xx-session-management-v1.xml");
}

pub mod keyboard_shortcuts_inhibit_unstable_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/unstable/keyboard-shortcuts-inhibit/keyboard-shortcuts-inhibit-unstable-v1.xml");
}

pub mod pointer_constraints_unstable_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/unstable/pointer-constraints/pointer-constraints-unstable-v1.xml");
}

pub mod wp_primary_selection_unstable_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/unstable/primary-selection/primary-selection-unstable-v1.xml");
}

pub mod xdg_foreign_unstable_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/unstable/xdg-foreign/xdg-foreign-unstable-v1.xml");
}

pub mod xdg_foreign_unstable_v2
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/unstable/xdg-foreign/xdg-foreign-unstable-v2.xml");
}

pub mod xwayland_keyboard_grab_unstable_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/unstable/xwayland-keyboard-grab/xwayland-keyboard-grab-unstable-v1.xml");
}

pub mod idle_inhibit_unstable_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/unstable/idle-inhibit/idle-inhibit-unstable-v1.xml");
}

pub mod input_method_unstable_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/unstable/input-method/input-method-unstable-v1.xml");
}

pub mod pointer_gestures_unstable_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/unstable/pointer-gestures/pointer-gestures-unstable-v1.xml");
}

pub mod xdg_shell_unstable_v5
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/unstable/xdg-shell/xdg-shell-unstable-v5.xml");
}

pub mod xdg_shell_unstable_v6
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/unstable/xdg-shell/xdg-shell-unstable-v6.xml");
}

pub mod relative_pointer_unstable_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/unstable/relative-pointer/relative-pointer-unstable-v1.xml");
}

pub mod text_input_unstable_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/unstable/text-input/text-input-unstable-v1.xml");
}

pub mod text_input_unstable_v3
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/unstable/text-input/text-input-unstable-v3.xml");
}

pub mod xdg_decoration_unstable_v1
{
    use wayland_scanner::generate;

    use crate::xdg_shell::xdg_toplevel;

    generate!("../upstream/wayland-protocols/unstable/xdg-decoration/xdg-decoration-unstable-v1.xml");
}

pub mod tablet_unstable_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/unstable/tablet/tablet-unstable-v1.xml");
}

pub mod tablet_unstable_v2
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/unstable/tablet/tablet-unstable-v2.xml");
}

pub mod fullscreen_shell_unstable_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/unstable/fullscreen-shell/fullscreen-shell-unstable-v1.xml");
}

pub mod zwp_linux_explicit_synchronization_unstable_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/unstable/linux-explicit-synchronization/linux-explicit-synchronization-unstable-v1.xml");
}

pub mod linux_dmabuf_unstable_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/unstable/linux-dmabuf/linux-dmabuf-unstable-v1.xml");
}

pub mod xdg_output_unstable_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/unstable/xdg-output/xdg-output-unstable-v1.xml");
}

pub mod input_timestamps_unstable_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/unstable/input-timestamps/input-timestamps-unstable-v1.xml");
}

pub mod viewporter
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/stable/viewporter/viewporter.xml");
}

pub mod xdg_shell
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/stable/xdg-shell/xdg-shell.xml");
}

pub mod presentation_time
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/stable/presentation-time/presentation-time.xml");
}

pub mod tablet_v2
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/stable/tablet/tablet-v2.xml");
}

pub mod linux_dmabuf_v1
{
    use wayland_client::protocol::wayland::*;
    use wayland_scanner::generate;

    generate!("../upstream/wayland-protocols/stable/linux-dmabuf/linux-dmabuf-v1.xml");
}
