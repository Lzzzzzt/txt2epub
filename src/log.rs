use log::LevelFilter;

pub fn init() {
    pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Info)
        .filter_module("txt2epub", LevelFilter::Info)
        .init();
}
