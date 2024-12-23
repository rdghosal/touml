use std::io;
use std::path::PathBuf;

static EXTENSIONS: [&str; 1] = ["py"];

pub fn get_file_paths(root: PathBuf) -> io::Result<Vec<PathBuf>> {
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
