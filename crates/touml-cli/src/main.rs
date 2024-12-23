mod utils;

use anyhow::{self, Result};
use clap::Parser;
use rayon::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

static OUTPUT_FILENAME: &str = "out.mmd";

/// A tool to generate mermaid class diagrams from Python source code.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(index(1))]
    path: PathBuf,

    /// Path to write the output to. If not provided, the output will be written to stdout.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Globs to exclude directories from the search, e.g. `**/my_secret_dir/*`.
    #[arg(long, value_delimiter = ' ', num_args = 1..)]
    exclude_dirs: Vec<String>,

    /// Globs to exclude files from the search, e.g. `**/my_secret_file.*`.
    #[arg(long, value_delimiter = ' ')]
    exclude_files: Vec<String>,

    /// Globs to exclude classes from the search, e.g. `*Secret*`.
    #[arg(long, value_delimiter = ' ')]
    exclude_classes: Vec<String>,
}

fn main() -> Result<()> {
    let mut cfg = Cli::parse();
    let paths = utils::get_file_paths(cfg.path, &cfg.exclude_dirs, &cfg.exclude_files)?;

    let header = String::from("classDiagram\n\n");
    let diagram = paths
        .iter()
        .filter_map(|p| match fs::read_to_string(p) {
            Ok(c) => Some(c),
            Err(_) => {
                eprintln!("Failed to load contents from file {}.", p.to_string_lossy());
                None
            }
        })
        .par_bridge()
        .map(|src| {
            touml::python_to_mermaid(src, &cfg.exclude_classes).map_err(|e| anyhow::anyhow!(e))
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("\n\n");

    if let Some(ref mut output) = cfg.output {
        if output.is_dir() && output.exists() {
            output.push(PathBuf::from(OUTPUT_FILENAME));
            let mut file = File::create(output)?;
            file.write_all((header + &diagram).as_bytes())?;
        } else {
            anyhow::bail!("--output (-o) must be an existing directory path.");
        }
    } else {
        std::io::stdout().write_all((header + &diagram).as_bytes())?;
    }

    Ok(())
}
