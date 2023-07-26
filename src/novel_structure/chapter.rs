use anyhow::Result;
use chinese_number::{ChineseCase, ChineseCountMethod, ChineseVariant, NumberToChinese};
use epub_builder::EpubContent;
use log::debug;
use serde::Serialize;
use tera::Context;

use crate::{cli::ConvertOpt, error::AnyError, EpubBuilderMut, WriteToEpub, TEMPLATE_ENGINE};

#[derive(Debug)]
pub struct Chapter {
    pub id: usize,
    pub part_no: usize,
    pub no: usize,
    pub title: String,
    pub raw_title: String,
    pub content: Vec<String>,
    pub start: u64,
    pub end: u64,
}

impl From<Chapter> for SerChapter {
    fn from(value: Chapter) -> Self {
        let Chapter {
            id,
            no,
            title,
            content,
            part_no,
            ..
        } = value;

        Self {
            global_title: format!("第{}章 {}", id, title),
            part_no,
            no_string: (value.no as u128)
                .to_chinese(
                    ChineseVariant::Simple,
                    ChineseCase::Lower,
                    ChineseCountMethod::TenThousand,
                )
                .unwrap(),
            no,
            title,
            content,
        }
    }
}

impl Chapter {
    pub fn new(
        id: usize,
        no: usize,
        part_no: usize,
        title: String,
        raw_title: String,
        start: u64,
    ) -> Self {
        Self {
            id,
            no,
            part_no,
            title,
            raw_title,
            content: vec![],
            start,
            end: 0,
        }
    }
}

#[derive(Serialize)]
pub struct SerChapter {
    pub global_title: String,
    #[serde(skip)]
    pub no: usize,
    #[serde(rename = "no")]
    pub no_string: String,
    #[serde(skip)]
    pub part_no: usize,
    pub title: String,
    pub content: Vec<String>,
}

impl WriteToEpub for SerChapter {
    fn write_to_epub<'a>(
        self,
        epub: EpubBuilderMut<'a>,
        options: &mut ConvertOpt,
    ) -> Result<EpubBuilderMut<'a>, AnyError> {
        let title = self.title_string();

        debug!("writing chapter: {}", &title);

        if options.have_section {
            epub.add_content(
                EpubContent::new(
                    format!("P{:02}C{:04}.html", self.part_no, self.no),
                    self.into_html_string()?.as_bytes(),
                )
                .title(title)
                .level(2),
            )?;
        } else {
            epub.add_content(
                EpubContent::new(
                    format!("C{:04}.html", self.no),
                    self.into_html_string()?.as_bytes(),
                )
                .title(title),
            )?;
        }

        Ok(epub)
    }
}

impl SerChapter {
    pub fn into_html_string(self) -> Result<String> {
        Ok(TEMPLATE_ENGINE.render("chapter", &Context::from_serialize(self)?)?)
    }

    pub fn title_string(&self) -> String {
        format!("第{}章 {}", self.no_string, self.title)
    }
}
