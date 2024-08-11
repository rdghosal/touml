// use rayon::prelude::*;
use anyhow::{self, Result};
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::fs::{self, File};
use std::io::{self, prelude::*, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::Command;
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

fn get_filename_and_code(url: &str) -> Option<(String, String)> {
    match url {
        "/" => Some(("HTTP/1.1 200 OK".into(), "assets/index.html".into())),
        url if url.starts_with("/assets") => {
            let filename = url.split("/").nth(2).unwrap_or_default();
            Some(("HTTP/1.1 200 OK".into(), format!("assets/{filename}")))
        }
        _ => None,
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let mut parts = request_line.split(' ');
    let (status_line, filename) = match parts.next() {
        Some("GET") => get_filename_and_code(parts.next().unwrap_or_default())
            .unwrap_or_else(|| ("HTTP/1.1 404 NOT FOUND".into(), "404.html".into())),
        _ => ("HTTP/1.1 404 NOT FOUND".into(), "404.html".into()),
    };

    dbg!(&filename);
    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
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
        .map(|src| python_to_mermaid(src).map_err(|e| anyhow::anyhow!(e)))
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
            anyhow::bail!("Value to `output` must be an existing directory path.");
        }
    } else {
        // TODO: Separate branch that handles spinning up server (per a command)
        // and print to stdout as the default behavior.
        // println!("{}", header + &diagram);

        // let dir = tempfile::tempdir()?;
        // let file_path = dir.path().join("index.html");
        // let mut file = File::create(&file_path)?;
        // write!(file, "{}", header + &diagram)?;

        // dbg!("{#?}", env::consts::OS);
        // Command::new("open").arg("assets/index.html").output()?;

        let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
        for stream in listener.incoming() {
            let stream = stream.unwrap();

            handle_connection(stream);
        }

        loop {
            match event::read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) => break,
                _ => continue,
            }
        }

        //         dir.close()?;
    }

    Ok(())
}
