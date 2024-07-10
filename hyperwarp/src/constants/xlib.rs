use crate::hooks::xlib::Window;

// https://tronche.com/gui/x/xlib/window/configure.html#XWindowChanges
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct XWindowChanges {
    pub x: libc::c_int,
    pub y: libc::c_int,
    pub width: libc::c_int,
    pub height: libc::c_int,
    pub border_width: libc::c_int,
    pub sibling: Window, // TODO: this is antoher struct
    pub stack_mode: libc::c_ulong,
}