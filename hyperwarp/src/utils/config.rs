use std::{env, net::SocketAddr, str::FromStr};

use super::utils::generate_random_id;

#[derive(Debug, Clone)]
pub struct Config {
    pub enable_x11: bool,
    pub enable_gl: bool,
    pub enable_glfw: bool,
    pub enable_glx: bool,
    pub enable_sdl2: bool,
    pub debug_mode: bool,
    pub tracing_mode: bool,
    pub capture_mode: bool,
    // windowing
    pub window_width_override: Option<u32>,
    pub window_height_override: Option<u32>,
    pub window_zero_origin: bool,
    // connection
    pub connection_type: String,
    pub connection_timeout: Option<u32>,
    // session and user
    pub session_id: String,
    pub user_id: String,
    // unix socket transport
    pub unix_socket_path: Option<String>,
    pub bind_addr: Option<SocketAddr>, 
    pub bind_type: Option<String>,
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
        let sid = get("HW_SESSION_ID", generate_random_id());
        let uid = get("HW_USER_ID", generate_random_id());
        let socket_path = get("HW_SOCKET_PATH", format!("/tmp/hw-{}.sock", sid));
        Config {
            enable_x11: booleanify("ENABLE_X11", true),
            enable_gl: booleanify("ENABLE_GL", true),
            enable_glx: booleanify("ENABLE_GLX", true),
            enable_glfw: booleanify("ENABLE_GLFW", true),
            enable_sdl2: booleanify("ENABLE_SDL2", true),
            debug_mode: booleanify("DEBUG_HW", false),
            tracing_mode: booleanify("TRACING_HW", false),
            window_width_override: try_get::<u32>("WINDOW_WIDTH"),
            window_height_override: try_get::<u32>("WINDOW_HEIGHT"),
            window_zero_origin: booleanify("WINDOW_ZERO_ORIGIN", false),
            connection_timeout: None,
            connection_type: try_get::<String>("CONNECTION_TYPE").unwrap_or("null".to_owned()),
            session_id: sid,
            user_id: uid,
            unix_socket_path: Some(socket_path),
            capture_mode: booleanify("CAPTURE_MODE", false),
            bind_addr: try_get::<SocketAddr>("SOCKET_ADDR"),
            bind_type: try_get::<String>("SOCKET_TYPE"),
        }
    }
}

impl Default for Config {
    fn default() -> Config {
        let sid = generate_random_id();
        let uid = generate_random_id();
        let socket_path = format!("/tmp/hw-{}.sock", sid);
        Config {
            enable_x11: true,
            enable_gl: true,
            enable_glx: true,
            enable_glfw: true,
            enable_sdl2: true,
            debug_mode: false,
            tracing_mode: false,
            window_width_override: None,
            window_height_override: None,
            window_zero_origin: false,
            connection_type: "null".to_owned(),
            connection_timeout: None,
            session_id: sid,
            user_id: uid,
            unix_socket_path: Some(socket_path),
            capture_mode: false,
            bind_addr: None,
            bind_type: None,
        }
    }
}
