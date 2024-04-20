pub(crate) use anyhow::Result;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum AccessLevel {
    Public,
    Private,
}

