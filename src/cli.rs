use std::path::PathBuf;

use clap::Parser;
use regex::Regex;

#[derive(Debug, Parser)]
/// Convert TXT file to Epub
pub struct CLIOptions {
    #[clap(index = 1)]
    /// The Files those need to be convert into epub
    pub files: Vec<PathBuf>,
    #[clap(short, long)]
    /// Output directory
    pub out_dir: Option<PathBuf>,
    #[clap(value_parser = parse_regex, short, long)]
    /// The regex to match part title, at least one capture group needed.
    pub part_regex: Option<Regex>,
    #[clap(value_parser = parse_regex, short, long)]
    /// The regex to match chapter title, at least one capture group needed.
    pub chapter_regex: Option<Regex>,
}

fn parse_regex(s: &str) -> Result<Regex, String> {
    let regex = Regex::new(s).map_err(|_| "Invalid regex")?;

    if regex.captures_len() > 1 {
        Ok(regex)
    } else {
        Err("The regex must have one capture group".into())
    }
}

impl From<CLIOptions> for Vec<ConvertOpt> {
    fn from(value: CLIOptions) -> Self {
        let CLIOptions {
            files,
            out_dir,
            part_regex,
            chapter_regex,
        } = value;

        let part_regex = part_regex.unwrap_or(Regex::new("^第.+[部|卷] (.*)$").unwrap());
        let chapter_regex = chapter_regex.unwrap_or(Regex::new("^第.+[章] (.*)$").unwrap());

        files
            .into_iter()
            .map(|p| {
                let name = p.file_prefix().unwrap().to_string_lossy().to_string();
                let out_file = out_dir
                    .clone()
                    .unwrap_or(p.parent().unwrap().to_path_buf())
                    .join(format!("{}.epub", name));

                ConvertOpt {
                    path: p,
                    name,
                    out_file,
                    have_section: true,
                    part_regex: part_regex.clone(),
                    chapter_regex: chapter_regex.clone(),
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
}
