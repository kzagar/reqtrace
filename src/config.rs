use serde::Deserialize;
use std::path::PathBuf;
use anyhow::Result;

// @ARC1.3@ (FROM: @REQ5.2@)
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub paths: Paths,
    pub types: Vec<TypeMapping>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Paths {
    pub scan: Vec<PathBuf>,
    pub ignore: Option<Vec<String>>,
    pub db: PathBuf,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TypeMapping {
    pub prefix: String,
    pub item_type: String,
    pub requirement_type: Option<String>,
}

impl Config {
    pub fn from_toml(content: &str) -> Result<Self> {
        Ok(toml::from_str(content)?)
    }

    pub fn load_default() -> Result<Self> {
        let content = std::fs::read_to_string(".reqtrace/config.toml")?;
        Self::from_toml(&content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // @UT1@ (FROM: @REQ5.2@)
    #[test]
    fn test_parse_valid_toml() {
        let toml = r#"
            [paths]
            scan = ["src", "docs"]
            ignore = ["target"]
            db = ".reqtrace/db.json"

            [[types]]
            prefix = "REQ"
            item_type = "Requirement"
            requirement_type = "Functional"

            [[types]]
            prefix = "ARCH"
            item_type = "Architecture"
        "#;

        let config = Config::from_toml(toml).unwrap();
        assert_eq!(config.paths.scan.len(), 2);
        assert_eq!(config.paths.db, PathBuf::from(".reqtrace/db.json"));
        assert_eq!(config.types.len(), 2);
        assert_eq!(config.types[0].prefix, "REQ");
        assert_eq!(config.types[1].item_type, "Architecture");
    }
}
