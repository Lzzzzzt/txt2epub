#![feature(path_file_prefix)]

use std::{
    fs::File,
    io::{BufReader, Cursor},
    time::SystemTime,
};

use ::log::{debug, info};
use chinese_number::{ChineseCase, ChineseCountMethod, ChineseVariant, NumberToChinese};
use colored::Colorize;
use epub_builder::{EpubBuilder, ZipLibrary};
use image::ImageOutputFormat;
use lazy_static::lazy_static;
use novel_structure::chapter::Line;
use tera::{Tera, Value};

use cli::ConvertOpt;
use error::AnyError;

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

pub static NOVEL_CSS: &str = include_str!("templates/stylesheet.css");

lazy_static! {
    pub static ref TEMPLATE_ENGINE: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_template("part", NOVEL_PART_TEMPLATE).unwrap();
        tera.add_raw_template("chapter", NOVEL_CHAPTER_TEMPLATE)
            .unwrap();
        tera.add_raw_template("intro", NOVEL_INTRO_TEMPLATE)
            .unwrap();

        tera.register_filter("to_chinese_string", |value: &Value, _: &_| {
            if let Some(no) = value.as_u64() {
                return Ok(Value::String(
                    no.to_chinese(
                        ChineseVariant::Simple,
                        ChineseCase::Lower,
                        ChineseCountMethod::TenThousand,
                    )
                    .unwrap(),
                ));
            }

            Ok(value.clone())
        });

        tera.register_filter("to_tradition_chinese_string", |value: &Value, _: &_| {
            if let Some(no) = value.as_u64() {
                return Ok(Value::String(
                    no.to_chinese(
                        ChineseVariant::Traditional,
                        ChineseCase::Lower,
                        ChineseCountMethod::TenThousand,
                    )
                    .unwrap(),
                ));
            }

            Ok(value.clone())
        });

        tera
    };
}

pub type EpubBuilderMut<'a> = &'a mut EpubBuilder<ZipLibrary>;

pub(crate) trait WriteToEpub {
    fn write_to_epub<'a>(
        self,
        epub: EpubBuilderMut<'a>,
        options: &mut ConvertOpt,
    ) -> Result<EpubBuilderMut<'a>, AnyError>;
}

pub(crate) fn get_cover_image(url: &str) -> Result<Vec<u8>, AnyError> {
    debug!("fetching cover image.");

    let response = reqwest::blocking::get(url)?;
    let mut image = vec![];

    image::load_from_memory(&response.bytes()?)?
        .write_to(&mut Cursor::new(&mut image), ImageOutputFormat::Jpeg(100))?;

    debug!("successfully fetched cover image.");
    debug!("size: {:.3}KB", image.len() as f64 / 1024.0);

    Ok(image)
}

pub fn txt2epub(mut opt: ConvertOpt) {
    if let Err(err) = txt2epub_inner(&mut opt) {
        ::log::error!("Failed to convert {}. Due to: ", opt.path.display());
        ::log::error!("{}\n", err.to_string().on_red());
    }
}

fn txt2epub_inner(opt: &mut ConvertOpt) -> Result<(), AnyError> {
    info!("converting `{}`.", opt.path.display());

    let start = SystemTime::now();

    let mut epub = EpubFactory::with_default_css()?.into();

    parse_txt(&mut BufReader::new(File::open(&opt.path)?), opt)?
        .write_to_epub(&mut epub, opt)?
        .generate(File::create(&opt.out_file)?)?;

    info!("saving file to {}", opt.out_file.display());
    info!("finish converting {}.", opt.path.display());
    info!("cost {}s.\n", start.elapsed()?.as_secs_f32());

    Ok(())
}

pub(crate) fn quote_replace(s: &mut String) {
    lazy_static! {
        static ref QUOTE_REGEX: regex::Regex = regex::Regex::new(r#"(“|”|‘|’)"#).unwrap();
    }

    *s = QUOTE_REGEX
        .replace_all(s, |cap: &regex::Captures| match &cap[0] {
            "“" => "「",
            "”" => "」",
            "‘" => "『",
            "’" => "』",
            _ => unreachable!(),
        })
        .to_string()
}

pub(crate) fn line_quote_replace(s: &mut Line) {
    lazy_static! {
        static ref QUOTE_REGEX: regex::Regex = regex::Regex::new(r#"(“|”|‘|’)"#).unwrap();
    }

    s.content = QUOTE_REGEX
        .replace_all(&s.content, |cap: &regex::Captures| match &cap[0] {
            "“" => "「",
            "”" => "」",
            "‘" => "『",
            "’" => "』",
            _ => unreachable!(),
        })
        .to_string()
}
