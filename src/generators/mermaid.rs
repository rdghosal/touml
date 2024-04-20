use crate::prelude::*;

use std::collections::HashSet;

#[derive(PartialEq, Eq, Hash)]
struct MermaidMethod {
    name: String,
    access: AccessLevel,
    args: Vec<String>,
    returns: Option<String>,
}

struct MermaidField(String, AccessLevel);

pub(crate) struct MermaidClass {
    name: String,
    parents: HashSet<String>,
    methods: HashSet<MermaidMethod>,
    fields: HashSet<MermaidField>,
}

pub(crate) trait MerimaidMapper {
    fn to_mermaid(self) -> MermaidClass;
}
