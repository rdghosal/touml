pub(crate) mod prelude;

mod _ast;
mod errors;
mod mermaid;
mod python;

use mermaid::MermaidMappable;
use prelude::*;

pub fn python_to_mermaid(src: String) -> Result<String, Box<dyn std::error::Error>> {
    let classes = python::PyClassInfo::from_source(&src)?;
    let mapped = classes
        .map(|c| c.map(|c| c.to_mermaid().print()))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(mapped.join(&format!("{EOL}{EOL}")))
}
