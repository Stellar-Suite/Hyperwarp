use std::env;

pub struct Config{
    pub enable_x11: bool,
    pub enable_gl: bool,
    pub debug_mode: bool,
}

fn booleanify(key: &str, default: bool) -> bool{
    match env::var(key) {
        Ok(value) => {
            match value.as_str() {
                "true" => true,
                "false" => false,
                "1" => true,
                "0" => false,
                "yes" => true,
                "no" => false,
                _ => default,
            }
        }
        Err(_) => {
            default
        }
    }
}

impl Config {
    pub fn from_env() -> Config {
        Config {
            enable_x11: booleanify("ENABLE_X11", true),
            enable_gl: booleanify("ENABLE_X11", true),
            debug_mode: booleanify("DEBUG_HW", false),
        }
    }
}

impl Default for Config{
    fn default() -> Config {
        Config {
            enable_x11: true,
            enable_gl: true,
            debug_mode: false,
        }
    }
}