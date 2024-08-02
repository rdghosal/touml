pub(crate) mod prelude;

mod _ast;
mod compiler;
mod errors;
mod parser;

pub fn python_to_mermaid(path: &'static str) -> Result<(), Box<dyn std::error::Error>> {
    todo!()
}
