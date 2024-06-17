// https://stackoverflow.com/a/36848555

use std::env;
use std::path::Path;
use std::ffi::OsStr;

pub fn prog() -> Option<String> {
    env::args().next()
        .as_ref()
        .map(Path::new)
        .and_then(Path::file_name)
        .and_then(OsStr::to_str)
        .map(String::from)
}