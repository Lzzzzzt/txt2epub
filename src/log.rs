use std::{env, str::FromStr};

use log::LevelFilter;

pub fn init() {
    let level = env::var("TXT2EPUB_LOG").unwrap_or("INFO".into());

    let level = LevelFilter::from_str(&level).unwrap_or_else(|e| {
        eprintln!("Log Level Error: {}", e);
        eprintln!("Fallback to `INFO`\n");
        LevelFilter::Info
    });

    pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Info)
        .filter_module("txt2epub", level)
        .init();
}
