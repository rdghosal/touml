use crate::parsers::*;
use crate::prelude::*;

use std::collections::HashSet;

pub struct MermaidClass {
    name: String,
    parents: HashSet<String>,
    methods: HashSet<Method>,
    fields: HashSet<Field>,
}

pub trait MermaidMappable {
    fn as_mermaid(self) -> MermaidClass;
}

impl MermaidMappable for PyClass {
    fn as_mermaid(self) -> MermaidClass {
        return MermaidClass {
            name: self.name,
            parents: self.parents,
            methods: self.methods,
            fields: self.fields,
        };
    }
}

fn make_class_diagram(nodes: Vec<&dyn MermaidMappable>) -> String {
    todo!()
}
