use rustpython_parser::{ast, Parse};
use std::error::Error;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::{fs, thread};

const FILE_EXTS: [&str; 1] = ["py"];

fn set_file_paths(file_paths: &mut Vec<PathBuf>, curr_path: PathBuf) -> Result<(), Box<dyn Error>> {
    if curr_path.is_dir() {
        for entry in curr_path.read_dir()? {
            if let Ok(e) = entry {
                let path = e.path();
                if path.is_dir() {
                    set_file_paths(file_paths, path.clone())?;
                }
                if let Some(ext) = path.extension() {
                    match ext.to_str() {
                        Some(e) => {
                            if FILE_EXTS.contains(&e) {
                                file_paths.push(path);
                            }
                        }
                        None => (),
                    }
                }
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut paths = Vec::new();
    set_file_paths(&mut paths, PathBuf::from("./tests/inputs/python"))?;
    let ast_nodes = Arc::new(Mutex::new(Vec::<ast::Stmt>::new()));
    let mut handles = Vec::new();
    for path in paths {
        let nodes = Arc::clone(&ast_nodes);
        let handle = thread::spawn(move || -> Result<(), std::io::Error> {
            let mut n = nodes.lock().unwrap();
            let contents = fs::read_to_string(&path)?;
            n.append(&mut ast::Suite::parse(&contents, path.to_str().unwrap()).unwrap());
            Ok(())
        });
        handles.push(handle);
    }
    for handle in handles {
        let _ = handle.join().unwrap();
    }
    println!("Result: {:#?}", ast_nodes.lock().unwrap());
    Ok(())
}
