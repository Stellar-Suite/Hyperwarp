use libc::c_void;

use crate::constants::Library;

#[derive(Debug)]
pub struct Window {
    pub id: usize,
    pub lib: Library,
}

impl Window {
    pub fn as_ptr(&self) -> *const c_void{
        self.id as *const c_void
    }
}