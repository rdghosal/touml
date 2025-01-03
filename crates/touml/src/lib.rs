pub(crate) mod prelude;

mod _ast;
pub mod errors;
mod mermaid;
mod python;

use mermaid::MermaidAdapter;
use prelude::*;

pub fn python_to_mermaid(
    src: String,
    exclude_names: &[String],
    exclude_bases: &[String],
) -> Result<Option<String>, String> {
    let exclude_patterns = exclude_names
        .iter()
        .map(|n| glob::Pattern::new(n).unwrap())
        .collect::<Vec<_>>();
    let exclude_parents = exclude_bases
        .iter()
        .map(|n| glob::Pattern::new(n).unwrap())
        .collect::<Vec<_>>();
    let result = python::PyClassInfo::from_source(&src)
        .map_err(|e| e.to_string())?
        .filter_map(|c| {
            if let Ok(c) = c {
                if exclude_parents
                    .iter()
                    .any(|p| p.matches(&c.name) || c.parents.iter().any(|pp| p.matches(pp)))
                    || exclude_patterns.iter().any(|p| p.matches(&c.name))
                {
                    None
                } else {
                    Some(c.to_mermaid().print())
                }
            } else {
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
