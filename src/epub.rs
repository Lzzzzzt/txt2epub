use epub_builder::{EpubBuilder, ZipLibrary};

use crate::{error::AnyError, NOVEL_CSS};

pub struct EpubFactory {
    pub builder: EpubBuilder<ZipLibrary>,
}

impl From<EpubFactory> for EpubBuilder<ZipLibrary> {
    fn from(val: EpubFactory) -> Self {
        val.builder
    }
}

impl EpubFactory {
    pub fn with_default_css() -> Result<Self, AnyError> {
        let mut epub = EpubBuilder::new(ZipLibrary::new()?)?;
        epub.stylesheet(NOVEL_CSS.as_bytes())?;
        Ok(Self { builder: epub })
    }

    pub fn default_css(&mut self) -> Result<&mut Self, AnyError> {
        self.builder.stylesheet(NOVEL_CSS.as_bytes())?;
        Ok(self)
    }
}
