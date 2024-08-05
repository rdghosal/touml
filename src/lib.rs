pub(crate) mod prelude;

mod _ast;
mod errors;
mod mermaid;
mod python;

use mermaid::MermaidMappable;
use prelude::*;

pub fn python_to_mermaid(src: String) -> Result<Option<String>, Box<dyn std::error::Error>> {
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
