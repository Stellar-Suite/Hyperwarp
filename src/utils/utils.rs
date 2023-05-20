
use rand::prelude::*;
use rand::distributions::{Alphanumeric, DistString};

pub fn generate_random_id() -> String {
    let mut rng = thread_rng();
    let id: String = rng.sample_iter(&Alphanumeric).take(10).map(char::from).collect();
    return id;
}