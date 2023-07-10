use epub_builder::EpubContent;
use serde::{Deserialize, Serialize};
use tera::Context;

use crate::{WriteToEpub, TEMPLATE_ENGINE};

pub mod chapter;
pub mod novel;
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
    fn write_to_epub(
        self,
        epub: &mut epub_builder::EpubBuilder<epub_builder::ZipLibrary>,
    ) -> anyhow::Result<
        &mut epub_builder::EpubBuilder<epub_builder::ZipLibrary>,
        Box<dyn std::error::Error>,
    > {
        epub.metadata("author", &self.author)?
            .metadata("title", &self.book_name)?
            .metadata("lang", "zh")?
            .metadata("toc_name", "目录")?
            .metadata("description", self.description.join("\n"))?;

        epub.add_content(
            EpubContent::new("intro.html", self.into_html_string()?.as_bytes()).title("简介"),
        )?;

        Ok(epub)
    }
}
