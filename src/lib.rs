pub(crate) mod prelude;

mod _ast;
pub mod errors;
mod mermaid;
mod python;

use anyhow::Result;
use mermaid::MermaidMappable;
use prelude::*;

pub fn python_to_mermaid(src: String) -> Result<Option<String>> {
    let result = python::PyClassInfo::from_source(&src)?
        .map(|c| c.map(|c| c.to_mermaid().print()))
        .collect::<Result<Vec<_>, _>>()?
        .join(&format!("{EOL}{EOL}"));

    if result.is_empty() {
        Ok(None)
    } else {
        Ok(Some(result))
    }
}
