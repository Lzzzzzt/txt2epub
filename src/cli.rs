use std::path::PathBuf;

use clap::Parser;
use regex::Regex;

#[derive(Debug, Parser)]
/// Convert TXT file to Epub
pub struct CLIOptions {
    #[clap(index = 1)]
    /// The Files those need to be convert into epub
    pub files: Vec<String>,

    #[clap(short, long)]
    /// Output directory
    pub out_dir: Option<PathBuf>,

    #[clap(value_parser = parse_regex, short, long)]
    /// The regex to match part title, at least one capture group needed.
    pub part_regex: Option<Regex>,

    #[clap(value_parser = parse_regex, short, long)]
    /// The regex to match chapter title, at least one capture group needed.
    pub chapter_regex: Option<Regex>,

    #[clap(long)]
    /// global replace “ -> 「, ” -> 」, ‘ -> 『, ’ -> 』.
    pub replace_quote: bool,

    #[clap(long)]
    /// if the novel's preface is too long, then enable this
    pub long_preface: bool,

    #[clap(long)]
    /// the string that treated to be a divider.
    pub divider: Vec<String>,
}

impl CLIOptions {
    pub fn check(self) -> Self {
        if self.files.is_empty() {
            eprintln!("should provide one file at least.");
            std::process::exit(1);
        }

        self
    }
}

fn parse_regex(s: &str) -> Result<Regex, &'static str> {
    let regex = Regex::new(s).map_err(|_| "Invalid regex")?;

    if regex.captures_len() > 1 {
        Ok(regex)
    } else {
        Err("The regex must have one capture group")
    }
}

impl From<CLIOptions> for Vec<ConvertOpt> {
    fn from(value: CLIOptions) -> Self {
        let CLIOptions {
            files,
            out_dir,
            part_regex,
            chapter_regex,
            replace_quote,
            long_preface,
            divider,
        } = value;

        let part_regex = part_regex.unwrap_or_else(|| Regex::new("^第.+[部|卷] (.*)$").unwrap());
        let chapter_regex = chapter_regex.unwrap_or_else(|| Regex::new("^第.+[章] (.*)$").unwrap());

        files
            .into_iter()
            .filter_map(|p| glob::glob(&p).ok())
            .flat_map(|p| p.collect::<Vec<_>>())
            .filter_map(|p| p.ok())
            .map(|path| {
                let name = path.file_prefix().unwrap().to_string_lossy().to_string();
                let out_file = out_dir
                    .clone()
                    .unwrap_or_else(|| path.parent().unwrap().to_path_buf())
                    .join(format!("{}.epub", name));

                ConvertOpt {
                    path,
                    name,
                    out_file,
                    have_section: true,
                    part_regex: part_regex.clone(),
                    chapter_regex: chapter_regex.clone(),
                    replace_quote,
                    long_preface,
                    divider: divider.clone(),
                }
            })
            .collect()
    }
}

pub struct ConvertOpt {
    pub path: PathBuf,
    pub name: String,
    pub out_file: PathBuf,
    pub have_section: bool,
    pub part_regex: Regex,
    pub chapter_regex: Regex,
    pub replace_quote: bool,
    pub long_preface: bool,
    pub divider: Vec<String>,
}
