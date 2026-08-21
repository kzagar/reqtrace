use anyhow::Result;

#[cfg(feature = "python")]
pub mod python;
#[cfg(feature = "rust")]
pub mod rust;

// @IMP-rimad@ (FROM: @ARC-zolag@)
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedSymbol {
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
}

pub trait LanguageParser: Send + Sync {
    fn parse(&self, content: &str) -> Result<Vec<ParsedSymbol>>;
}

pub fn get_parser(extension: &str) -> Option<Box<dyn LanguageParser>> {
    match extension {
        #[cfg(feature = "rust")]
        "rs" => Some(Box::new(rust::RustParser)),
        #[cfg(feature = "python")]
        "py" => Some(Box::new(python::PythonParser)),
        _ => None,
    }
}
