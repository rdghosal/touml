pub(crate) mod parsers;
pub(crate) mod generators;
pub(crate) mod prelude;

use std::collections::HashSet;
use std::path::PathBuf;
use anyhow::Result;

pub fn get_file_paths(root: PathBuf, target_exts: &HashSet<&'static str>) -> Result<Vec<PathBuf>> {
    let mut result = Vec::<PathBuf>::new();
    if root.is_dir() {
        for entry in root.read_dir()? {
            if let Ok(e) = entry {
                let path = e.path();
                if path.is_dir() {
                    result.append(&mut get_file_paths(path, target_exts)?);
                } else if let Some(ext) = path.extension() {
                    match ext.to_str() {
                        Some(e) if target_exts.contains(e) => {
                            result.push(path);
                        }
                        _ => (),
                    }
                }
            }
        }
    }
    Ok(result)
}

pub fn python_to_mermaid(path: &'static str) -> Result<()> {
    todo!()
}
