use libc::c_void;

#[derive(Debug)]
pub struct Window {
    pub id: usize,
    pub is_SDL2: bool,
}

impl Window {
    pub fn as_ptr(&self) -> *const c_void{
        self.id as *const c_void
    }
}