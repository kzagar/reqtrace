# 0003. Traceability Graph JSON Database Schema

- Status: accepted
- Date: 2026-06-30

## Context

`reqtrace` maintains a persistent representation of the traceability graph in a
JSON file. We need to define a stable, readable, and version-control-friendly
schema for this database.

The schema must:

- Capture the type of each item (Requirement, Architecture, Test).
- Support specific requirement subtypes (Functional, Performance, etc.).
- Maintain structural locations (file path and line number) for traceback and
  editing.
- Trace dependencies (`derived_from` list of IDs).

## Decision

We chose a flat key-value map of items keyed by their unique Item ID:

```json
{
  "items": {
    "REQ1.1": {
      "id": "REQ1.1",
      "type": "Requirement",
      "requirement_type": "Functional",
      "title": "Email / password login",
      "file_path": "docs/requirements.md",
      "line_number": 33,
      "derived_from": []
    },
    "ARCH-parser": {
      "id": "ARCH-parser",
      "type": "Architecture",
      "title": "File Parser Module",
      "file_path": "src/parser.rs",
      "line_number": 42,
      "derived_from": ["REQ1.3", "REQ3.4"]
    }
  }
}
```

The database will be written with sorted keys, standardized indentation, and one
item per line to produce clean git diffs.

## Consequences

- Flat structure allows fast $O(1)$ lookups by Item ID.
- Easy to serialize/deserialize using Rust's `serde_json` and `serde`.
- Changes to any item are localized in git diffs.
