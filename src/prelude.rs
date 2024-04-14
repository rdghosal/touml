pub(crate) use anyhow::Result;

#[derive(PartialEq, Eq, Hash)]
pub enum AccessLevel {
    Public,
    Private,
}

