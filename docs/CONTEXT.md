# Glossary

- **Architectural Diagram** — A UML diagram (e.g., class, component, sequence,
  deployment) embedded within an architectural Markdown document. The diagram
  itself is not tagged; instead, the Markdown section enclosing it is tagged and
  traced.
- **Comment Formatting/Rewriting** — A feature of the `reqtrace CLI` that
  standardizes and updates the descriptive text alongside `Link Declarations` in
  source code comments or manual test documents to match the current
  titles/headings of the referenced items.
- **Comment Tag** — The syntax `[@<item id>@]` or similar, embedded within
  comments in source files (e.g., `// @ARCH-parser@` in Rust or
  `<!-- @REQ-vapik@ -->` in Markdown) to declare an item.
- **Explicit ID Anchor** — An HTML anchor `<a id="<item id>"></a>` placed at the
  end of a Markdown section heading line, providing a stable named target for
  hyperlinking to that item.
- **Item ID** — A unique identifier of the form `<item type><id>` (e.g.,
  `REQ-vapik`, `ARCH-parser`), where `<id>` is a unique identifier within the scope
  of `<item type>`. The `<id>` can be hierarchical (e.g. `1.2.3`), camel-case,
  kebab-case, or an abbreviation.
- **Item Type** — The classification of an item in the graph (e.g., requirement,
  architecture, test). For requirements, the specific type (Functional,
  Performance, Reliability, Usability, Maintainability) is mapped from the ID
  prefix (e.g., `REQ`, `PERF`, `REL`) as configured in `.reqtrace/config.toml`.
- **Link Declaration** — The syntax used in comments to define relationships
  between items, supporting both inline format (e.g.,
  `// @ARCH-parser@ (FROM: @REQ-vapik@)`) and multi-line list format for items with
  many parents (e.g., `// @ARCH-parser@ FROM:` followed by indent-listed IDs
  like `//   REQ-zaruh (title)`).
- **Manual Test** — A test case described in a Markdown file, containing test
  steps and traced back to a requirement.
- **reqtrace CLI** — The command-line interface for executing one-off graph
  updates and validating the graph.
- **reqtrace Configuration File** — A TOML configuration file stored at
  `.reqtrace/config.toml` that defines the paths to scan (source code,
  requirements, architecture, tests) and any patterns/rules for file exclusion
  (see [ADR 0001](decisions/0001-config-format.md)).
- **reqtrace Database** — A pretty-printed JSON file stored in a hidden
  directory (e.g., `.reqtrace/db.json`) that maintains the persistent
  representation of the Traceability Graph, with sorted keys and consistent
  indentation to ensure clean version control diffs (see
  [ADR 0003](decisions/0003-db-schema.md)).
- **reqtrace GitHub Action** — A custom composite GitHub Action defined in the
  `action/` directory that downloads a specific, hard-coded version of the
  pre-compiled `reqtrace` binary (supporting targets specified in
  [ADR 0004](decisions/0004-release-targets.md)) and executes graph validation as a
  CI/CD step.
- **reqtrace Server** — A long-running daemon that monitors files for changes,
  updates the Traceability Graph, and hosts both an MCP server (using HTTP/SSE,
  see [ADR 0002](decisions/0002-mcp-transport.md)) and a web server (for UI visual
  representation).
- **Traceability Graph** — The internal database and queryable graph
  representing links between requirements, architectural elements, and tests.
- **Traceability Issue** — A gap or inconsistency in the traceability graph,
  specifically untraced requirements (no links to architecture/tests) or
  implementation/architectural items not derived from any requirements (orphan
  items).
- **Traceability Link Command (`reqtrace link`)** — A CLI subcommand that
  inserts explicit ID anchors and converts unlinked item references into
  relative Markdown hyperlinks (see
  [ADR 0005](decisions/0005-visible-markdown-traceability-and-linking.md)).
- **Visible Item Definition** — A traceable item definition rendered visibly in
  Markdown (via inline metadata, tables, or attribute lists) rather than hidden
  within an HTML comment (see
  [ADR 0005](decisions/0005-visible-markdown-traceability-and-linking.md)).
