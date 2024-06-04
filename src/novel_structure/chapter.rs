use chinese_number::{ChineseCase, ChineseCountMethod, ChineseVariant, NumberToChinese};
use epub_builder::EpubContent;
use log::debug;
use serde::Serialize;
use tera::Context;

use crate::{
    cli::ConvertOpt, error::AnyError, line_quote_replace, quote_replace, EpubBuilderMut,
    WriteToEpub, TEMPLATE_ENGINE,
};

#[derive(Debug)]
pub(crate) struct Chapter {
    pub id: usize,
    pub part_no: usize,
    pub no: usize,
    pub title: String,
    #[allow(unused)]
    pub raw_title: String,
    pub content: Vec<Line>,
    #[allow(unused)]
    pub start: u64,
    pub end: u64,
}

impl WriteToEpub for Chapter {
    fn write_to_epub<'a>(
        self,
        epub: EpubBuilderMut<'a>,
        options: &mut ConvertOpt,
    ) -> Result<EpubBuilderMut<'a>, AnyError> {
        Into::<SerChapter>::into(self).write_to_epub(epub, options)
    }
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

#[derive(Serialize, Debug)]
pub(crate) enum LineType {
    Line,
    Divider,
}

#[derive(Serialize, Debug)]
pub(crate) struct Line {
    pub(crate) line_type: LineType,
    pub(crate) content: String,
}

#[derive(Serialize)]
pub(crate) struct SerChapter {
    pub global_title: String,
    pub no: usize,
    pub part_no: usize,
    pub title: String,
    pub content: Vec<Line>,
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
                    format!("{:02}/{:04}.xhtml", self.part_no, self.no),
                    self.into_html_string(options)?.as_bytes(),
                )
                .title(title)
                .level(2),
            )?;
        } else {
            epub.add_content(
                EpubContent::new(
                    format!("00/{:04}.xhtml", self.no),
                    self.into_html_string(options)?.as_bytes(),
                )
                .title(title),
            )?;
        }

        Ok(epub)
    }
}

impl SerChapter {
    pub fn into_html_string(mut self, opt: &ConvertOpt) -> Result<String, AnyError> {
        if opt.replace_quote {
            self.content.iter_mut().for_each(line_quote_replace);
            quote_replace(&mut self.title);
        }

        self.content
            .iter_mut()
            .for_each(|s| s.content = autocorrect::format(&s.content));

        Ok(TEMPLATE_ENGINE.render("chapter", &Context::from_serialize(self)?)?)
    }

    pub fn title_string(&self) -> String {
        format!(
            "第{}章 {}",
            (self.no as u128)
                .to_chinese(
                    ChineseVariant::Simple,
                    ChineseCase::Lower,
                    ChineseCountMethod::TenThousand,
                )
                .unwrap(),
            self.title
        )
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use regex::Regex;

    use crate::cli::ConvertOpt;

    use super::{Line, LineType, SerChapter};

    #[test]
    fn into_html_string() -> Result<(), Box<dyn Error>> {
        // construct SerChapter

        let chapter = SerChapter {
            global_title: "第1章".into(),
            no: 1,
            part_no: 1,
            title: "测试".into(),
            content: vec![
                Line {
                    line_type: LineType::Line,
                    content: "测试".into(),
                },
                Line {
                    line_type: LineType::Divider,
                    content: "---".into(),
                },
                Line {
                    line_type: LineType::Line,
                    content: "测试".into(),
                },
            ],
        };

        let res = chapter.into_html_string(&ConvertOpt {
            path: "".into(),
            name: "".into(),
            out_file: "".into(),
            have_section: false,
            part_regex: Regex::new("1").unwrap(),
            chapter_regex: Regex::new("1").unwrap(),
            replace_quote: false,
            long_preface: false,
            divider: vec![],
        })?;

        res.lines().for_each(|l| println!("{}", l));

        Ok(())
    }
}
