#![feature(path_file_prefix)]

use std::{
    error::Error,
    fs::File,
    io::{BufReader, Cursor},
    time::SystemTime,
};

use ::log::{debug, info};
use anyhow::Result;
use cli::ConvertOpt;
use epub_builder::{EpubBuilder, ZipLibrary};
use image::ImageOutputFormat;
use lazy_static::lazy_static;
use tera::Tera;

use crate::{epub::EpubFactory, parse::parse_txt};

pub mod cli;
pub mod epub;
pub mod error;
pub mod log;
pub mod novel_structure;
pub mod parse;

static NOVEL_PART_TEMPLATE: &str = include_str!("templates/part.html");
static NOVEL_CHAPTER_TEMPLATE: &str = include_str!("templates/chapter.html");
static NOVEL_INTRO_TEMPLATE: &str = include_str!("templates/intro.html");
static NOVEL_NO_PART_TEMPLATE: &str = include_str!("templates/no_part.html");

pub static NOVEL_CSS: &str = include_str!("templates/stylesheet.css");
pub static mut HAVE_SECTIONS: bool = true;

lazy_static! {
    pub static ref TEMPLATE_ENGINE: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_template("part", NOVEL_PART_TEMPLATE).unwrap();
        tera.add_raw_template("chapter", NOVEL_CHAPTER_TEMPLATE)
            .unwrap();
        tera.add_raw_template("intro", NOVEL_INTRO_TEMPLATE)
            .unwrap();
        tera.add_raw_template("no_part", NOVEL_NO_PART_TEMPLATE)
            .unwrap();
        tera
    };
}

pub type EpubBuilderMut<'a> = &'a mut EpubBuilder<ZipLibrary>;

pub trait WriteToEpub {
    fn write_to_epub(self, epub: EpubBuilderMut) -> Result<EpubBuilderMut, Box<dyn Error>>;
}

pub fn get_cover_image(url: &str) -> Result<Vec<u8>> {
    debug!("fetching cover image.");

    let response = reqwest::blocking::get(url)?;
    let mut image = vec![];

    image::load_from_memory(&response.bytes()?)?
        .write_to(&mut Cursor::new(&mut image), ImageOutputFormat::Jpeg(100))?;

    Ok(image)
}

pub fn txt2epub(opt: &ConvertOpt) -> Result<(), Box<dyn Error>> {
    info!("converting {}.", opt.path.display());

    let start = SystemTime::now();

    parse_txt(&mut BufReader::new(File::open(&opt.path)?))?
        .write_to_epub(&mut EpubFactory::with_default_css()?.into())?
        .generate(File::create(&opt.out_file)?)?;

    info!("saving file to {}", opt.out_file.display());
    info!("finish converting {}.", opt.path.display());
    info!("cost {}s.\n", start.elapsed()?.as_secs_f32());

    Ok(())
}
