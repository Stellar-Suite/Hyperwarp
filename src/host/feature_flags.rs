// which features have we detected
#[derive(Clone,Copy,Debug)]
pub struct FeatureFlags {
    pub x11_enabled: bool,
    pub gl_enabled: bool,
    pub glfw_enabled: bool,
    pub glx_enabled: bool,
    pub sdl2_enabled: bool,
}

impl FeatureFlags {
    pub fn new() -> Self {
        FeatureFlags {
            x11_enabled: false,
            gl_enabled: false,
            glfw_enabled: false,
            glx_enabled: false,
            sdl2_enabled: false,
        }
    }
}