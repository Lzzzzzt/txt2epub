use clap::Parser;
use colored::Colorize;

use txt2epub::{cli::CLIOptions, error::AnyError, txt2epub};

fn main() -> Result<(), AnyError> {
    let options = CLIOptions::parse();
    txt2epub::log::init();

    for opt in Into::<Vec<_>>::into(options) {
        if let Err(err) = txt2epub(&opt) {
            log::error!("Failed to convert {}. Due to: ", opt.path.display());
            log::error!("{}\n", err.to_string().on_red());
        }
    }

    Ok(())
}
