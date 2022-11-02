use std::{env, str::FromStr};

pub struct Config{
    pub enable_x11: bool,
    pub enable_gl: bool,
    pub debug_mode: bool,
    // windowing
    pub window_width_override: Option<u32>,
    pub window_height_override: Option<u32>,
}

fn get<T: FromStr>(key: &str, default: T) -> T{
    match env::var(key){
        Ok(val) => val.parse::<T>().unwrap_or(default),
        Err(_) => default,
    }
}

fn try_get<T: FromStr>(key: &str) -> Option<T>{
    match env::var(key){
        Ok(val) => {
            match val.parse::<T>(){
                Ok(val) => Some(val),
                Err(_) => None,
            }
        },
        Err(_) => None
    }
}

fn intify(key: &str, default: i32) -> i32{
    match env::var(key){
        Ok(val) => val.parse::<i32>().unwrap_or(default),
        Err(_) => default,
    }
}

fn u_intify(key: &str, default: u32) -> u32{
    match env::var(key){
        Ok(val) => val.parse::<u32>().unwrap_or(default),
        Err(_) => default,
    }
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
            enable_gl: booleanify("ENABLE_GL", true),
            debug_mode: booleanify("DEBUG_HW", false),
            window_width_override: try_get::<u32>("WINDOW_WIDTH"),
            window_height_override: try_get::<u32>("WINDOW_HEIGHT"),
        }
    }
}

impl Default for Config{
    fn default() -> Config {
        Config {
            enable_x11: true,
            enable_gl: true,
            debug_mode: false,
            window_width_override: None,
            window_height_override: None,
        }
    }
}