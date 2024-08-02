pub(crate) mod prelude;

mod _ast;
mod mermaid;
mod errors;
mod python;

pub fn python_to_mermaid(path: &'static str) -> Result<(), Box<dyn std::error::Error>> {
    todo!()
}
