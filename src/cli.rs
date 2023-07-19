use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
/// Convert TXT file to Epub
pub struct CLIOptions {
    #[clap(index = 1)]
    /// The Files those need to be convert into epub
    pub files: Vec<PathBuf>,
    #[clap(short, long)]
    /// Output directory
    pub out_dir: Option<PathBuf>,
}

impl From<CLIOptions> for Vec<ConvertOpt> {
    fn from(value: CLIOptions) -> Self {
        let CLIOptions { files, out_dir } = value;

        files
            .into_iter()
            .map(|p| {
                let name = p.file_prefix().unwrap().to_string_lossy().to_string();
                let out_file = out_dir
                    .clone()
                    .unwrap_or(p.parent().unwrap().to_path_buf())
                    .join(format!("{}.epub", name));

                ConvertOpt::new(p, name, out_file)
            })
            .collect()
    }
}

pub struct ConvertOpt {
    pub path: PathBuf,
    pub name: String,
    pub out_file: PathBuf,
}

impl ConvertOpt {
    pub fn new(path: PathBuf, name: String, out_file: PathBuf) -> Self {
        Self {
            path,
            name,
            out_file,
        }
    }
}
