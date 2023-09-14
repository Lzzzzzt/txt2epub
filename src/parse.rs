use std::io::{BufRead, Seek};

use anyhow::Result;
use log::debug;

use crate::{cli::ConvertOpt, novel_structure::novel::Novel, WriteToEpub};

pub(crate) fn parse_txt<F>(file: &mut F, options: &mut ConvertOpt) -> Result<impl WriteToEpub>
where
    F: BufRead + Seek,
{
    let mut novel = Novel::new();

    debug!("parsing txt.");

    novel.scan_metadata(file, options)?;
    novel.scan_parts(file, options)?;

    Ok(novel)
}
