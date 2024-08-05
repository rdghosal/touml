pub(crate) mod prelude;

mod _ast;
mod errors;
mod mermaid;
mod python;

use mermaid::MermaidMappable;
use prelude::*;

pub fn python_to_mermaid(src: String) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let mut result = String::new();

    python::PyClassInfo::from_source(&src)?
        .map(|c| c.map(|c| c.to_mermaid().print()))
        .try_for_each(|m| {
            result.push_str(&format!("{}{EOL}{EOL}", m?));
            Ok::<(), crate::errors::ParseError>(())
        })?;

    if result.is_empty() {
        Ok(None)
    } else {
        Ok(Some(result.trim_end().to_string()))
    }
}
