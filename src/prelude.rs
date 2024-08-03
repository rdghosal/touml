#[cfg(windows)]
pub static EOL: &str = "\r\n";

#[cfg(not(windows))]
pub static EOL: &str = "\n";

pub trait Accessible {
    fn is_public(&self) -> bool;
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Field {
    pub name: String,
    pub dtype: Option<String>,
    pub default: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Method {
    pub name: String,
    pub args: Vec<Field>,
    pub returns: Option<String>,
}

impl Accessible for Field {
    fn is_public(&self) -> bool {
        !self.name.starts_with('_')
    }
}

impl Accessible for Method {
    fn is_public(&self) -> bool {
        !self.name.starts_with('_')
    }
}
