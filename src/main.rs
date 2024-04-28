use anyhow::Result;
use rayon::prelude::*;
use rustpython_parser::{ast, Parse};

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use touml::get_file_paths;

fn main() -> Result<()> {
    println!("Hello world!");
    let mut exts = HashSet::new();
    exts.insert("py");
    let paths = get_file_paths(PathBuf::from("./tests/inputs/python"), &exts)?;
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
