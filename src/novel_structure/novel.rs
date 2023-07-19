use std::{
    error::Error,
    io::{BufRead, Seek},
};

use anyhow::Result;
use log::{debug, info, warn};
use regex::Regex;

use crate::{
    get_cover_image,
    novel_structure::{chapter::SerChapter, SerMetaData},
    EpubBuilderMut, WriteToEpub, HAVE_SECTIONS,
};

use super::{part::Part, Metadata};

#[derive(Debug, Default)]
pub struct Novel {
    pub parts: Vec<Part>,
    pub metadata: Option<Metadata>,
    current_part_no: usize,
}

impl WriteToEpub for Novel {
    fn write_to_epub(self, epub: EpubBuilderMut) -> Result<EpubBuilderMut, Box<dyn Error>> {
        debug!("writing metadata.");

        match self.metadata {
            Some(metadata) if metadata.cover.is_some() => {
                match get_cover_image(metadata.cover.as_ref().unwrap()) {
                    Ok(cover) => {
                        epub.add_cover_image("cover.jpg", &cover[..], "image/jpeg")?;
                    }
                    Err(e) => {
                        warn!("Failed to add cover image. Due to: ");
                        warn!("{}", e);
                        warn!("Skip adding cover image.");
                    }
                }

                Into::<SerMetaData>::into(metadata).write_to_epub(epub)?;
            }
            Some(metadata) => {
                Into::<SerMetaData>::into(metadata).write_to_epub(epub)?;
            }
            None => (),
        }

        let mut chapter_count = 0;

        for part in self.parts {
            let (part, content) = part.into_serialized();
            debug!("writing part: {}", &part.title_string());
            part.write_to_epub(epub)?;

            for c in content {
                Into::<SerChapter>::into(c).write_to_epub(epub)?;
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

    pub(crate) fn scan_metadata<F>(&mut self, file: &mut F) -> Result<()>
    where
        F: BufRead + Seek,
    {
        file.rewind()?;

        debug!("scanning novel metadata.");

        let mut line = String::new();
        let mut metadata_string = String::new();
        let part_regex = Regex::new(r"^第.+[部|卷|章] (.*)$")?;

        while let Ok(len) = file.read_line(&mut line) {
            if len == 0 || part_regex.captures(line.trim()).is_some() {
                break;
            }

            metadata_string += &line;

            line.clear();
        }

        self.metadata = Some(serde_yaml::from_str(&metadata_string)?);

        debug!("{:#?}", self.metadata);

        Ok(())
    }

    pub(crate) fn scan_parts<F>(&mut self, file: &mut F) -> Result<()>
    where
        F: BufRead + Seek,
    {
        debug!("scanning novel parts.");

        self.check_part_range(file)?;

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
            unsafe { HAVE_SECTIONS = false };
            self.make_whole_chapter_as_a_part(file)?;
        }

        file.rewind()?;

        let mut global_chapter_number = 0;

        for part in self.parts.iter_mut() {
            part.scan_chapters(file, &mut global_chapter_number)?;
        }

        Ok(())
    }

    fn check_part_range<F>(&mut self, file: &mut F) -> Result<()>
    where
        F: BufRead + Seek,
    {
        file.rewind()?;

        let mut line = String::new();

        let part_regex = Regex::new(r"^第.+[部|卷] (.*)$")?;

        while let Ok(len) = file.read_line(&mut line) {
            if len == 0 {
                break;
            }

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

    fn make_whole_chapter_as_a_part<F>(&mut self, file: &mut F) -> Result<()>
    where
        F: Seek + BufRead,
    {
        file.rewind()?;

        let mut line = String::new();

        let chapter_regex = Regex::new(r"^第.+[章] (.*)$")?;

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
}
