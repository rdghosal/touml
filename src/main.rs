use rustpython_parser::{ast, Parse};
use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::{fs, thread};

const FILE_EXTS: [&str; 1] = ["py"];

#[derive(PartialEq, Eq, Hash)]
enum AccessLevel {
    Public,
    Private,
}

#[derive(PartialEq, Eq, Hash)]
struct MermaidMethod {
    name: String,
    access: AccessLevel,
    args: Vec<String>,
    returns: Option<String>,
}

struct MermaidField(String, AccessLevel);

struct MermaidClass {
    name: String,
    access: AccessLevel,
    parents: HashSet<Box<MermaidClass>>, // Maybe create an Rc on the parent struct?
    methods: HashSet<MermaidMethod>,
    fields: HashSet<MermaidField>,
}

trait MermaidMapper {
    // TODO fix type
    fn to_mermaid(self) -> Result<(), Box<dyn Error>>;
}

impl MermaidMapper for ast::StmtClassDef {
    fn to_mermaid(self) -> Result<(), Box<dyn Error>> {
        let name = self.name.to_string();
        let mut parents = HashSet::new();
        let mut methods = HashSet::new();
        // let mut fields = HashSet::new();
        for parent in self.bases {
            match parent {
                ast::Expr::Name(name) => {
                    parents.insert(name.id.to_string());
                }
                ast::Expr::Attribute(attr) => {
                    if let ast::Expr::Name(name) = attr.value.as_ref() {
                        parents.insert(name.id.to_string());
                    } else {
                        // TODO: return error here instead of printing
                        eprintln!("Failed to extract base class name from attribute.");
                    }
                }
                _ => todo!(),
            }
        }
        for attr in self.body {
            match attr {
                ast::Stmt::FunctionDef(func) => {
                    let name = func.name.to_string();
                    let mut args = Vec::new();
                    args.extend(func.args.posonlyargs);
                    args.extend(func.args.kwonlyargs);
                    args.extend(func.args.args);
                    let args = args.iter().map(|a| a.def.arg.to_string()).collect();
                    let returns = if func.returns.is_some() {
                        match *func.returns.unwrap() {
                            ast::Expr::Constant(c) => {
                                Some(String::new())
                                // todo!("Match constant as ast::Expr::Constant")
                                // constant.value
                            }
                            ast::Expr::Attribute(a) => Some(a.attr.to_string()),
                            ast::Expr::Name(n) => Some(n.id.to_string()),
                            _ => todo!(),
                        }
                    } else {
                        None
                    };

                    let access = if name.starts_with("_") {
                        AccessLevel::Private
                    } else {
                        AccessLevel::Public
                    };

                    methods.insert(MermaidMethod {
                        name,
                        access,
                        args,
                        returns,
                    });
                }
                _ => todo!(),
            }
        }
        Ok(())
    }
}

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
    // dbg!("{#?}", env::consts::OS);
    // Command::new("open").arg("src/index.html").spawn().unwrap();
    Ok(())
}
