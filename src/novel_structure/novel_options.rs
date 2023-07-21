use std::collections::HashMap;

pub type NovelOptionsMap = HashMap<NovelOptions, bool>;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Hash)]
pub enum NovelOptions {
    LongPreface,
    Unknown,
}

impl NovelOptions {
    pub fn is_options_string(s: impl AsRef<str>) -> bool {
        NovelOptions::Unknown != s.into()
    }

    pub fn default_options() -> HashMap<Self, bool> {
        HashMap::from_iter([(Self::LongPreface, false)])
    }
}

impl<T> From<T> for NovelOptions
where
    T: AsRef<str>,
{
    fn from(value: T) -> Self {
        match value.as_ref().trim() {
            "[LongPreface]" => NovelOptions::LongPreface,
            _ => NovelOptions::Unknown,
        }
    }
}

impl From<NovelOptions> for &str {
    fn from(value: NovelOptions) -> Self {
        match value {
            NovelOptions::LongPreface => "[LongPreface]",
            NovelOptions::Unknown => "UnkownOptions",
        }
    }
}
