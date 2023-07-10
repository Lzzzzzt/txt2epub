use std::io::{BufRead, Seek};

use anyhow::Result;
use log::debug;

use crate::{novel_structure::novel::Novel, WriteToEpub};

pub fn parse_txt<F>(file: &mut F) -> Result<impl WriteToEpub>
where
    F: BufRead + Seek,
{
    let mut novel = Novel::new();

    debug!("parsing txt.");

    novel.scan_metadata(file)?;
    novel.scan_parts(file)?;

    Ok(novel)
}
