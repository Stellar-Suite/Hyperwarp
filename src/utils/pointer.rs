use libc::c_void;

pub struct Pointer(pub *const c_void);

impl Pointer {
    pub fn as_func(&self) -> *const c_void {
        self.0
    }
}

unsafe impl Send for Pointer {}
unsafe impl Sync for Pointer {}

pub struct MutPointer(pub *mut c_void);

unsafe impl Send for MutPointer {}
unsafe impl Sync for MutPointer {}