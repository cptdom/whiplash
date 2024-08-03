
use env_logger::Builder;
use std::env;

pub fn init_logger() {
    let mut builder = Builder::from_default_env();

    if env::var("RUST_LOG").is_err() {
        builder.filter_level(log::LevelFilter::Info);
    }

    builder.init();

}