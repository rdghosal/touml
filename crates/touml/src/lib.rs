pub(crate) mod prelude;

mod _ast;
pub mod errors;
mod mermaid;
mod python;

use mermaid::MermaidAdapter;
use prelude::*;

pub fn python_to_mermaid(src: String, exclude_names: &[String]) -> Result<Option<String>, String> {
    let exclude_patterns = exclude_names
        .iter()
        .map(|n| glob::Pattern::new(n).unwrap())
        .collect::<Vec<_>>();
    let result = python::PyClassInfo::from_source(&src)
        .map_err(|e| e.to_string())?
        .filter_map(|c| match c {
            Ok(c) => {
                if exclude_patterns.iter().any(|p| p.matches(&c.name)) {
                    None
                } else {
                    Some(c.to_mermaid().print())
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                None
            }
        })
        .collect::<Vec<_>>()
        .join(&format!("{EOL}{EOL}"));

    if result.is_empty() {
        Ok(None)
    } else {
        Ok(Some(result))
    }
}
