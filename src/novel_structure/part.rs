use std::io::{BufRead, Seek, SeekFrom};

use anyhow::Result;
use chinese_number::{ChineseCase, ChineseCountMethod, ChineseVariant, NumberToChinese};
use epub_builder::EpubContent;
use log::debug;
use serde::Serialize;
use tera::Context;

use crate::{
    cli::ConvertOpt,
    error::AnyError,
    novel_structure::chapter::{Line, LineType},
    quote_replace, EpubBuilderMut, WriteToEpub, TEMPLATE_ENGINE,
};

use super::chapter::Chapter;

#[derive(Debug)]
pub(crate) struct Part {
    /// if no is 0, means this part is the only one of novel
    pub no: usize,
    pub title: String,
    #[allow(unused)]
    pub raw_title: String,
    pub chapters: Vec<Chapter>,
    pub preface: Vec<String>,
    pub start: u64,
    pub end: u64,
    pub current_chapter_no: usize,
}

impl WriteToEpub for Part {
    fn write_to_epub<'a>(
        self,
        epub: EpubBuilderMut<'a>,
        options: &mut ConvertOpt,
    ) -> Result<EpubBuilderMut<'a>, AnyError> {
        let (part, content) = self.into_serialized();

        debug!("writing part: {}", &part.title_string());
        part.write_to_epub(epub, options)?;

        for c in content {
            c.write_to_epub(epub, options)?;
        }

        Ok(epub)
    }
}

impl Part {
    pub fn new(no: usize, title: String, raw_title: String, start: u64) -> Self {
        Self {
            no,
            title,
            raw_title,
            chapters: vec![],
            preface: vec![],
            start,
            end: 0,
            current_chapter_no: 1,
        }
    }

    pub fn patch_current_end(&mut self, end: u64) {
        if let Some(last) = self.chapters.last_mut() {
            last.end = end;
        }
    }

    pub fn current_chapter_mut(&mut self) -> &mut Chapter {
        &mut self.chapters[self.current_chapter_no - 1 - 1]
    }

    pub fn scan_chapters<F>(
        &mut self,
        file: &mut F,
        global_chapter_num: &mut usize,
        options: &ConvertOpt,
    ) -> Result<()>
    where
        F: BufRead + Seek,
    {
        let title_regex = &options.chapter_regex;

        if options.have_section {
            debug!("scanning novel chapter of part: {}.", self.title);
        }

        file.seek(SeekFrom::Start(self.start))?;

        let mut preface = vec![];
        let mut chapter_start = false;
        let mut line = String::new();

        while let Ok(len) = file.read_line(&mut line) {
            // quit the loop when read to file end
            if len == 0 {
                break;
            }

            let trimed_line = line.trim();

            if let Some(cap) = title_regex.captures(trimed_line) {
                // search for the chapter title
                chapter_start = true;

                self.patch_current_end(file.stream_position()? - line.as_bytes().len() as u64);

                self.chapters.push(Chapter::new(
                    *global_chapter_num + 1,
                    self.current_chapter_no,
                    self.no,
                    cap[1].to_string(),
                    line.clone(),
                    file.stream_position()?,
                ));

                *global_chapter_num += 1;
                self.current_chapter_no += 1;
            } else if !chapter_start && !trimed_line.is_empty() {
                // if current line is not the chapter content, treat it as the part's preface.
                preface.push(trimed_line.to_string());
            } else if !trimed_line.is_empty() {
                // if current line is the chapter content, push it.

                let line_type = if options.divider.iter().any(|d| is_divider(trimed_line, d)) {
                    LineType::Divider
                } else {
                    LineType::Line
                };

                self.current_chapter_mut().content.push(Line {
                    line_type,
                    content: trimed_line.to_string(),
                })
            }

            // quit the loop if read to the chapter end.
            if file.stream_position()? >= self.end {
                break;
            }

            line.clear();
        }

        self.patch_current_end(file.stream_position()? - line.as_bytes().len() as u64);

        self.preface = preface;

        debug!("found {} chapters.", self.chapters.len());

        Ok(())
    }

    pub fn into_serialized(self) -> (SerPart, Vec<Chapter>) {
        let Self {
            no,
            title,
            preface,
            chapters,
            ..
        } = self;

        (
            SerPart {
                no,
                title,
                preface,
                is_long_preface: false,
            },
            chapters,
        )
    }
}

fn is_divider(trimed_line: &str, d: &str) -> bool {
    d.len() == trimed_line.len() && d == trimed_line
}

#[derive(Serialize)]
pub struct SerPart {
    pub no: usize,
    pub title: String,
    pub preface: Vec<String>,
    pub is_long_preface: bool,
}

impl WriteToEpub for SerPart {
    fn write_to_epub<'a>(
        self,
        epub: EpubBuilderMut<'a>,
        options: &mut ConvertOpt,
    ) -> Result<EpubBuilderMut<'a>, AnyError> {
        let title = self.title_string();

        if options.have_section {
            epub.add_content(
                EpubContent::new(
                    format!("{:02}/intro.xhtml", self.no),
                    self.into_html_string(options)?.as_bytes(),
                )
                .title(title),
            )?;
        }

        Ok(epub)
    }
}

impl SerPart {
    pub fn into_html_string(mut self, opt: &ConvertOpt) -> Result<String> {
        if opt.replace_quote {
            self.preface.iter_mut().for_each(quote_replace);
            quote_replace(&mut self.title);
        }
        Ok(TEMPLATE_ENGINE.render("part", &Context::from_serialize(self)?)?)
    }

    pub fn title_string(&self) -> String {
        format!(
            "第{}卷 {}",
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
