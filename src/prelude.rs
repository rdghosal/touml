pub(crate) use anyhow::Result;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Accessibility {
    Public,
    Private,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Field {
    pub name: String,
    pub pytype: Option<String>,
    pub default: Option<String>,
    pub access: Accessibility,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Method {
    pub name: String,
    pub access: Accessibility,
    pub args: Vec<String>,
    pub returns: Option<String>,
}
