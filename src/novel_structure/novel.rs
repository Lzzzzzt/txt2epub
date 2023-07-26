use std::io::{BufRead, Seek};

use anyhow::Result;
use log::{debug, info};

use crate::{
    cli::ConvertOpt, error::AnyError, novel_structure::chapter::SerChapter, EpubBuilderMut,
    WriteToEpub,
};

use super::{novel_options::NovelOptions, part::Part, Metadata};

#[derive(Debug, Default)]
pub struct Novel {
    pub parts: Vec<Part>,
    pub metadata: Option<Metadata>,
    current_part_no: usize,
}

impl WriteToEpub for Novel {
    fn write_to_epub<'a>(
        self,
        epub: EpubBuilderMut<'a>,
        options: &mut ConvertOpt,
    ) -> Result<EpubBuilderMut<'a>, AnyError> {
        debug!("writing metadata.");

        if let Some(metadata) = self.metadata {
            metadata.write_to_epub(epub, options)?;
        }

        let mut chapter_count = 0;

        for part in self.parts {
            let (part, content) = part.into_serialized();
            debug!("writing part: {}", &part.title_string());
            part.write_to_epub(epub, options)?;

            for c in content {
                Into::<SerChapter>::into(c).write_to_epub(epub, options)?;
                chapter_count += 1;
            }
        }

        debug!("total {} chapters.", chapter_count);

        Ok(epub)
    }
}

impl Novel {
    pub fn new() -> Self {
        Self {
            parts: vec![],
            metadata: None,
            current_part_no: 1,
        }
    }

    pub(crate) fn scan_metadata<F>(&mut self, file: &mut F, options: &ConvertOpt) -> Result<()>
    where
        F: BufRead + Seek,
    {
        file.rewind()?;

        debug!("scanning novel metadata.");

        let mut line = String::new();
        let mut metadata_string = String::new();
        let part_regex = &options.part_regex;
        let chapter_regex = &options.chapter_regex;

        while let Ok(len) = file.read_line(&mut line) {
            if len == 0 || part_regex.is_match(line.trim()) || chapter_regex.is_match(line.trim()) {
                break;
            }

            metadata_string += &line;

            line.clear();
        }

        self.metadata = Some(serde_yaml::from_str(&metadata_string)?);

        debug!("{:#?}", self.metadata);

        Ok(())
    }

    pub(crate) fn scan_parts<F>(&mut self, file: &mut F, options: &mut ConvertOpt) -> Result<()>
    where
        F: BufRead + Seek,
    {
        debug!("scanning novel parts.");

        self.check_part_range(file, options)?;

        debug!("found {} parts", self.parts.len());
        debug!(
            "{:#?}",
            self.parts
                .iter()
                .map(|p| p.title.clone())
                .collect::<Vec<_>>()
        );

        if self.parts.is_empty() {
            info!("No part has been found.");
            info!("Treat whole novel as a part.");
            options.have_section = false;
            self.make_whole_chapter_as_a_part(file, options)?;
        }

        file.rewind()?;

        let mut global_chapter_number = 0;

        for part in self.parts.iter_mut() {
            part.scan_chapters(file, &mut global_chapter_number, options)?;
        }

        Ok(())
    }

    fn check_part_range<F>(&mut self, file: &mut F, options: &mut ConvertOpt) -> Result<()>
    where
        F: BufRead + Seek,
    {
        file.rewind()?;

        let mut line = String::new();

        // let part_regex = Regex::new(r"^第.+[部|卷] (.*)$")?;
        // let part_regex = Regex::new(&options.part_regex)?;
        let part_regex = &options.part_regex;

        while let Ok(len) = file.read_line(&mut line) {
            if len == 0 {
                break;
            }

            if NovelOptions::is_options_string(&line) {
                self.patch_options(line.as_str().into())
            }

            // if line.starts_with("[LongPreface]") {
            //     if let Some(part) = self.parts.last_mut() {
            //         part.is_long_preface = true;
            //     }
            // }

            if let Some(cap) = part_regex.captures(line.trim()) {
                if let Some(part) = self.parts.last_mut() {
                    part.end = file.stream_position()? - line.as_bytes().len() as u64;
                }

                self.parts.push(Part::new(
                    self.current_part_no,
                    cap[1].to_string(),
                    line.clone(),
                    file.stream_position()?,
                ));

                self.current_part_no += 1;
            }

            line.clear();
        }

        if let Some(part) = self.parts.last_mut() {
            part.end = file.stream_position()?;
        }

        Ok(())
    }

    fn make_whole_chapter_as_a_part<F>(&mut self, file: &mut F, options: &ConvertOpt) -> Result<()>
    where
        F: Seek + BufRead,
    {
        file.rewind()?;

        let mut line = String::new();

        // let chapter_regex = Regex::new(&options.chapter_regex)?;
        let chapter_regex = &options.chapter_regex;

        while let Ok(len) = file.read_line(&mut line) {
            if len == 0 {
                break;
            }

            if chapter_regex.is_match(line.trim()) {
                self.parts.push(Part::new(
                    0,
                    "".into(),
                    "".into(),
                    file.stream_position()? - line.as_bytes().len() as u64,
                ));

                break;
            }

            line.clear();
        }

        self.parts[0].end = file.seek(std::io::SeekFrom::End(0))?;
        Ok(())
    }

    fn patch_options(&mut self, options: NovelOptions) {
        if let Some(part) = self.parts.last_mut() {
            part.options.entry(options).and_modify(|v| *v = true);
        }
    }
}
