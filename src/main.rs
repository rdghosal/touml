use anyhow::Result;
use rayon::prelude::*;
use rustpython_parser::{ast, Parse};

use std::fs;
use std::path::PathBuf;

static EXTENSIONS: [&str; 1] = ["py"];

fn get_file_paths(root: PathBuf) -> Result<Vec<PathBuf>> {
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
    let paths = get_file_paths(PathBuf::from("./tests/inputs/python"))?;
    let ast = paths
        .iter()
        .filter_map(|p| match fs::read_to_string(p) {
            Ok(c) => Some(c),
            Err(_) => {
                eprintln!("Failed to load contents from file {}.", p.to_string_lossy());
                None
            }
        })
        .par_bridge()
        .map(|c| ast::Suite::parse(&c, "path").unwrap())
        .collect::<Vec<_>>();

    println!("Result: {:#?}", ast);
    // dbg!("{#?}", env::consts::OS);
    // Command::new("open").arg("src/index.html").spawn().unwrap();
    Ok(())
}
