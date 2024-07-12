use std::collections::HashMap;

use lazy_static::lazy_static;

use super::sdl2_dynapi::DYNAPI_FUNCS;

lazy_static! {
    pub static ref DYNAPI_FUNCS_INDEX: HashMap<String, usize> = {
        let mut map = HashMap::new();
        for (i, func) in DYNAPI_FUNCS.iter().enumerate() {
            map.insert(func.to_string(), i);
        }
        map
    };
}