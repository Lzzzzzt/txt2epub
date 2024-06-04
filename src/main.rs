use clap::Parser;
use rayon::prelude::*;
use std::time::SystemTime;

use txt2epub::{cli::CLIOptions, error::AnyError, txt2epub};

fn main() -> Result<(), AnyError> {
    let options = CLIOptions::parse().check();
    txt2epub::log::init();

    let start = SystemTime::now();

    log::info!("Covert Start.");

    Into::<Vec<_>>::into(options)
        .into_par_iter()
        .for_each(txt2epub);

    log::info!("Covert Finish. Cost: {:?}", start.elapsed()?);

    Ok(())
}
