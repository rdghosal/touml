use glob::Pattern;
use std::io;
use std::path::PathBuf;
use walkdir::WalkDir;

static EXTENSIONS: [&str; 1] = ["py"];

pub fn get_file_paths(
    root: PathBuf,
    exclude_dirs: &[String],
    exclude_files: &[String],
) -> io::Result<Vec<PathBuf>> {
    let dir_patterns = exclude_dirs
        .iter()
        .map(|d| Pattern::new(d).unwrap())
        .collect::<Vec<_>>();
    let file_patterns = exclude_files
        .iter()
        .map(|f| Pattern::new(f).unwrap())
        .collect::<Vec<_>>();

    let mut paths = Vec::new();
    for entry in WalkDir::new(root) {
        let entry = entry?;
        let path = entry.path();

        if dir_patterns.iter().any(|p| p.matches_path(path)) {
            continue;
        } else if entry.file_type().is_file() {
            if file_patterns.iter().any(|p| p.matches_path(path)) {
                continue;
            }

            if let Some(ext) = path.extension() {
                if EXTENSIONS.contains(&ext.to_str().unwrap()) {
                    paths.push(path.to_path_buf());
                }
            }
        }
    }

    Ok(paths)
}
