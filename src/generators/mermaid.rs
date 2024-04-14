use crate::prelude::*;
use crate::parsers::*;

use std::collections::HashSet;

#[derive(PartialEq, Eq, Hash)]
struct MermaidMethod {
    name: String,
    access: AccessLevel,
    args: Vec<String>,
    returns: Option<String>,
}

struct MermaidField(String, AccessLevel);

struct MermaidClass {
    name: String,
    parents: HashSet<String>,
    methods: HashSet<MermaidMethod>,
    fields: HashSet<MermaidField>,
}

impl MermaidClass {
    fn from_python(cls: PyClass) -> Self {
        todo!()
    }
}
