use anyhow::Result;
use rayon::prelude::*;
use rustpython_parser::{ast, Parse};

use std::fs;
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("Hello world!");
    Ok(())
    // let paths = get_file_paths(PathBuf::from("./tests/inputs/python"))?;
    // let ast = paths
    //     .iter()
    //     .filter_map(|p| match fs::read_to_string(p) {
    //         Ok(c) => Some(c),
    //         Err(_) => {
    //             eprintln!("Failed to load contents from file {}.", p.to_string_lossy());
    //             None
    //         }
    //     })
    //     .par_bridge()
    //     .map(|c| ast::Suite::parse(&c, "path").unwrap())
    //     .collect::<Vec<_>>();
    // println!("Result: {:#?}", ast);
    // // dbg!("{#?}", env::consts::OS);
    // // Command::new("open").arg("src/index.html").spawn().unwrap();
    // Ok(())
}
