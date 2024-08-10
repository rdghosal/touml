pub(crate) mod prelude;

mod _ast;
pub mod errors;
mod mermaid;
mod python;

use mermaid::MermaidMappable;
use prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn python_to_mermaid(src: String) -> Result<Option<String>, String> {
    let result = python::PyClassInfo::from_source(&src)
        .map_err(|e| e.to_string())?
        .map(|c| c.map(|c| c.to_mermaid().print()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
        .join(&format!("{EOL}{EOL}"));

    if result.is_empty() {
        Ok(None)
    } else {
        Ok(Some(result))
    }
}
