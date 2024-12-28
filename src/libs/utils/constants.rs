use lazy_static::lazy_static;
use std::env;

lazy_static! {
    pub static ref SECRET: String = set_secret();
}

fn set_secret() -> String {
    dotenv::dotenv().ok();
    env::var("SECRET").unwrap()
}
