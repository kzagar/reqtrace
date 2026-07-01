use crate::graph::Graph;
use anyhow::Result;
use std::fs::File;
use std::path::Path;

// @IMP1.2@ (FROM: ARC1.2)
pub struct Serializer;

impl Serializer {
    pub fn save(graph: &Graph, path: &Path) -> Result<()> {
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, graph)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Graph> {
        let file = File::open(path)?;
        let graph: Graph = serde_json::from_reader(file)?;
        Ok(graph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::TraceItem;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    // @UT4@ (FROM: REQ1.2)
    #[test]
    fn test_serialization() {
        let mut items = BTreeMap::new();
        items.insert(
            "REQ1".into(),
            TraceItem {
                id: "REQ1".into(),
                item_type: "Requirement".into(),
                requirement_type: Some("Functional".into()),
                title: "Req 1".into(),
                file_path: PathBuf::from("req.md"),
                line_range: crate::scanner::LineRange { start: 1, end: 1 },
                derived_from: vec![],
            },
        );

        let graph = Graph { items };
        let path = Path::new("test_db.json");
        Serializer::save(&graph, path).unwrap();

        let loaded = Serializer::load(path).unwrap();
        assert_eq!(loaded.items.len(), 1);
        assert_eq!(loaded.items["REQ1"].title, "Req 1");

        std::fs::remove_file(path).unwrap();
    }
}
