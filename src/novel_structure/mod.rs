use colored::Colorize;
use epub_builder::EpubContent;
use log::warn;
use serde::{Deserialize, Serialize};
use tera::Context;

use crate::{error::AnyError, get_cover_image, EpubBuilderMut, WriteToEpub, TEMPLATE_ENGINE};

pub mod chapter;
pub mod novel;
pub mod novel_options;
pub mod part;

#[derive(Deserialize, Debug, Default)]
pub struct Metadata {
    #[serde(alias = "书名")]
    #[serde(default)]
    book_name: String,
    #[serde(alias = "作者")]
    #[serde(default)]
    author: String,
    #[serde(alias = "封面")]
    #[serde(default)]
    cover: Option<String>,
    #[serde(alias = "简介")]
    #[serde(default)]
    description: Vec<String>,
}

impl WriteToEpub for Metadata {
    fn write_to_epub(self, epub: EpubBuilderMut) -> anyhow::Result<EpubBuilderMut, AnyError> {
        if self.cover.is_some() {
            match get_cover_image(self.cover.as_ref().unwrap()) {
                Ok(cover) => {
                    epub.add_cover_image("cover.jpg", &cover[..], "image/jpeg")?;
                }
                Err(e) => {
                    warn!("Failed to add cover image. Due to: ");
                    warn!("{}", e.to_string().on_yellow());
                    warn!("Skip adding cover image.");
                }
            }
        }

        Into::<SerMetaData>::into(self).write_to_epub(epub)?;

        Ok(epub)
    }
}

impl From<Metadata> for SerMetaData {
    fn from(value: Metadata) -> Self {
        let Metadata {
            book_name,
            author,
            description,
            ..
        } = value;

        Self {
            book_name,
            author,
            description,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct SerMetaData {
    #[serde(rename = "title")]
    book_name: String,
    author: String,
    description: Vec<String>,
}

impl SerMetaData {
    pub fn into_html_string(self) -> anyhow::Result<String> {
        Ok(TEMPLATE_ENGINE.render("intro", &Context::from_serialize(self)?)?)
    }
}

impl WriteToEpub for SerMetaData {
    fn write_to_epub(self, epub: EpubBuilderMut) -> anyhow::Result<EpubBuilderMut, AnyError> {
        epub.metadata("author", &self.author)?
            .metadata("title", &self.book_name)?
            .metadata("lang", "zh-CN")?
            .metadata("toc_name", "目录")?
            .metadata("description", self.description.join("\n"))?;

        epub.add_content(
            EpubContent::new("intro.html", self.into_html_string()?.as_bytes()).title("简介"),
        )?;

        Ok(epub)
    }
}
