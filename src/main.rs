// use rayon::prelude::*;
use anyhow::{self, Result};
use clap::Parser;
use std::io::Write;
use std::{fs, io, path::PathBuf};
use touml::python_to_mermaid;

static EXTENSIONS: [&str; 1] = ["py"];
static OUTPUT_FILENAME: &str = "output.mmd";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(index(1))]
    path: PathBuf,

    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn get_file_paths(root: PathBuf) -> io::Result<Vec<PathBuf>> {
    let mut result = Vec::<PathBuf>::new();
    if root.is_dir() {
        for entry in root.read_dir()?.flatten() {
            let path = entry.path();
            if path.is_dir() {
                result.append(&mut get_file_paths(path)?);
            } else if let Some(ext) = path.extension() {
                match ext.to_str() {
                    Some(e) if EXTENSIONS.contains(&e) => {
                        result.push(path);
                    }
                    _ => (),
                }
            }
        }
    }
    Ok(result)
}

fn main() -> Result<()> {
    let mut cfg = Cli::parse();
    let paths = get_file_paths(cfg.path)?;

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
        .map(python_to_mermaid)
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("\n\n");

    if let Some(ref mut output) = cfg.output {
        if output.is_dir() && output.exists() {
            output.push(PathBuf::from(OUTPUT_FILENAME));
            let mut file = std::fs::File::create(output)?;
            file.write_all((header + &diagram).as_bytes())?;
        } else {
            anyhow::bail!("Value to `output` must be an existing directory path.");
        }
    } else {
        println!("{}", header + &diagram);
    }

    // println!("Result: {:#?}", ast);
    // dbg!("{#?}", env::consts::OS);
    // Command::new("open").arg("src/index.html").spawn().unwrap();

    Ok(())
}
