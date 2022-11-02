use lazy_static::lazy_static;
use crate::utils::config::Config;

pub struct ApplicationHost {
    pub config: Config,
}

impl ApplicationHost {
    pub fn new() -> Self {
        ApplicationHost {
            config: Config::from_env(),
        }
    }
}

lazy_static! {
    pub static ref HOST: ApplicationHost = ApplicationHost::new();
}