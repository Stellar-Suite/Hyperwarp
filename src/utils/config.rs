use std::{env, str::FromStr};

#[derive(Debug, Clone)]
pub struct Config {
    pub enable_x11: bool,
    pub enable_gl: bool,
    pub enable_glfw: bool,
    pub enable_glx: bool,
    pub enable_sdl2: bool,
    pub debug_mode: bool,
    // windowing
    pub window_width_override: Option<u32>,
    pub window_height_override: Option<u32>,
    pub window_zero_origin: bool,
    // connection
    pub connection_type: String,
    pub connection_timeout: Option<u32>,
}

fn get<T: FromStr>(key: &str, default: T) -> T {
    match env::var(key) {
        Ok(val) => val.parse::<T>().unwrap_or(default),
        Err(_) => default,
    }
}

fn try_get<T: FromStr>(key: &str) -> Option<T> {
    match env::var(key) {
        Ok(val) => match val.parse::<T>() {
            Ok(val) => Some(val),
            Err(_) => None,
        },
        Err(_) => None,
    }
}

fn intify(key: &str, default: i32) -> i32 {
    match env::var(key) {
        Ok(val) => val.parse::<i32>().unwrap_or(default),
        Err(_) => default,
    }
}

fn u_intify(key: &str, default: u32) -> u32 {
    match env::var(key) {
        Ok(val) => val.parse::<u32>().unwrap_or(default),
        Err(_) => default,
    }
}

fn booleanify(key: &str, default: bool) -> bool {
    match env::var(key) {
        Ok(value) => match value.as_str() {
            "true" => true,
            "false" => false,
            "1" => true,
            "0" => false,
            "yes" => true,
            "no" => false,
            _ => default,
        },
        Err(_) => default,
    }
}

impl Config {
    pub fn from_env() -> Config {
        Config {
            enable_x11: booleanify("ENABLE_X11", true),
            enable_gl: booleanify("ENABLE_GL", true),
            enable_glx: booleanify("ENABLE_GLX", true),
            enable_glfw: booleanify("ENABLE_GLFW", true),
            enable_sdl2: booleanify("ENABLE_SDL2", true),
            debug_mode: booleanify("DEBUG_HW", false),
            window_width_override: try_get::<u32>("WINDOW_WIDTH"),
            window_height_override: try_get::<u32>("WINDOW_HEIGHT"),
            window_zero_origin: booleanify("WINDOW_ZERO_ORIGIN", false),
            connection_timeout: None,
            connection_type: try_get::<String>("CONNECTION_TYPE").unwrap_or("null".to_owned()),
        }
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            enable_x11: true,
            enable_gl: true,
            enable_glx: true,
            enable_glfw: true,
            enable_sdl2: true,
            debug_mode: false,
            window_width_override: None,
            window_height_override: None,
            window_zero_origin: false,
            connection_type: "null".to_owned(),
            connection_timeout: None,
        }
    }
}
