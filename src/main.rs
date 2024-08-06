// use rayon::prelude::*;
use clap::Parser;
use std::io::Write;
use std::{fs, io, path::PathBuf};
use touml::python_to_mermaid;

static EXTENSIONS: [&str; 1] = ["py"];

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(index(1))]
    path: String,

    #[arg(short, long)]
    output: Option<String>,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Cli::parse();
    let root = cfg.path;
    let paths = get_file_paths(PathBuf::from(root))?;

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
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("\n\n");

    if let Some(output) = cfg.output {
        let mut path = PathBuf::from(output);
        if path.is_dir() {
            path.push(PathBuf::from("output.mmd"));
            let mut file = std::fs::File::create(path)?;
            file.write_all((header + &diagram).as_bytes())?;
        } else {
            return Err(Box::new(touml::errors::CliError));
        }
    } else {
        println!("{}", header + &diagram);
    }

    // println!("Result: {:#?}", ast);
    // dbg!("{#?}", env::consts::OS);
    // Command::new("open").arg("src/index.html").spawn().unwrap();

    Ok(())
}
