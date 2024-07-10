use libc::c_void;

use crate::constants::Library;

#[derive(Debug)]
pub struct Window {
    pub id: usize,
    pub lib: Library,
    pub width: u32,
    pub height: u32,
    pub position: Option<(i32, i32)>,
}

impl Window {
    pub fn as_ptr(&self) -> *const c_void{
        self.id as *const c_void
    }
}

impl Window {
    pub fn new(id: usize, lib: Library) -> Self {
        Window {
            id,
            lib,
            width: 0,
            height: 0,
            position: None,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn position(&mut self, x: i32, y: i32) {
        self.position = Some((x, y));
    }
}

