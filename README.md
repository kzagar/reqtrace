# reqtrace

Systems and software engineering tooling for establishing traceability to
requirements. `reqtrace` supersedes legacy tooling like `sara` and `shtracer`.

## Vision & Goal

In the AI era, where LLMs perform the majority of code generation, `reqtrace`
ensures that software projects remain coherent, architecturally sound, and
strictly grounded on requirements.

It achieves this by:

1. **Verifiable Traceability**: Maintaining precise, verifiable links between
   requirements, architecture, and tests.
2. **Context Minimization**: Providing LLMs and agentic assistants with exactly
   the upstream and downstream context needed to implement a new requirement or
   affect a change without needing to read large, unrelated portions of the
   codebase.

## Key Features

- **Stateful Daemon**: A background service that monitors file changes and
  maintains a Traceability Graph in memory.
- **Traceability Database**: A key-sorted, pretty-printed JSON file
  (`.reqtrace/db.json`) optimized for meaningful Git diffs.
- **Model Context Protocol (MCP) Server**: Native HTTP/SSE MCP server endpoints
  allowing agents to query the graph and retrieve context with sub-millisecond
  latency.
- **Interactive Graph Visualization**: Aesthetically pleasing, navigable visual
  representation of the graph (web UI and standalone static HTML export).
- **Auto-Formatting Comments**: Standardizes and keeps descriptive comment
  headers in sync with requirement titles.
- **LSP Integration**: Queries the Language Server Protocol (if available) to
  locate source code definitions for architectural and test items.
- **CI/CD Integration**: Custom composite GitHub Action for automated graph
  validation.
- **Proquint IDs**: Recommends and supports [proquint](https://arxiv.org/html/0901.4016)
  identifiers for stable, pronounceable, and human-friendly traceability links.

## Repository Layout

- `docs/` — Specifications and design records.
  - [CONTEXT.md](docs/CONTEXT.md) — Glossary and domain terminology.
  - [requirements.md](docs/requirements.md) — System requirements with
    Gherkin-style scenarios.
  - [decisions/](docs/decisions/) — Architecture Decision Records (ADRs).
- `.reqtrace/` — Traceability configuration and database.
- `action/` — Custom composite GitHub Action.

- `src/` — Rust source code.

## Installation

Install the CLI (with all default features enabled) to `~/.cargo/bin/reqtrace`:

```bash
cargo install --path .
```

## Configuration (`.reqtrace/config.toml`)

`reqtrace` looks for `.reqtrace/config.toml` in the current working directory:

```toml
[paths]
scan = ["src", "docs", "tests"]
ignore = ["target", ".git", "node_modules", "dist"]
db = ".reqtrace/db.json"

[[types]]
prefix = "REQ-"
item_type = "Requirement"
requirement_type = "Functional"

[[types]]
prefix = "ARC-"
item_type = "Architecture"

[[types]]
prefix = "IMP-"
item_type = "Implementation"

[[types]]
prefix = "UT-"
item_type = "Test"

[[types]]
prefix = "IT-"
item_type = "Test"
```

- **Strict TOML schema**: `[paths]` requires `scan` and `db` (`ignore` is optional). Each `[[types]]` entry requires `prefix` and `item_type` (plus optional `requirement_type`). Unrecognized or differently named keys (such as `name` or `path`) fail deserialization.
- **Fallback behavior**: If `.reqtrace/config.toml` is missing **or fails to parse**, `reqtrace` prints `Warning: .reqtrace/config.toml not found, using defaults` and falls back to `scan = ["src", "docs"]`, `ignore = ["target", ".git"]`, `db = ".reqtrace/db.json"`, and only `REQ-` (`Requirement`) and `ARC-` (`Architecture`) types (leaving `tests/` unscanned and `IMP-`/`UT-`/`IT-` tags typed as `Unknown`).
- **Ignore patterns**: Single-component names (e.g., `"target"`) and directory globs (`"target/**"`, `"**/__pycache__/**"`) are expanded to match both relative paths and directory/file basenames.

## Tag Syntax & Supported Languages

Traceability tags use the form `@PREFIX-id@` (matching `@[a-zA-Z0-9][a-zA-Z0-9.-]*@`) and declare upstream links with `(FROM: @PARENT1@, @PARENT2@)` (trailing `@` on parent IDs is optional; multiple parents are comma-separated).

- **Code files with symbol-scoped AST/regex parsers**:
  - Rust (`.rs`): `//` or `/*` comments
  - Python (`.py`): `#` comments
  - TypeScript / JavaScript (`.ts`, `.tsx`, `.js`, `.jsx`, `.mts`, `.cts`, `.mjs`, `.cjs`): `//` or `/*` comments
  - Tags are matched to the enclosing or nearest following function/class/test symbol to compute exact `line_range` and `title`.
- **Markdown and other non-parsed files**:
  - Place `<!-- @PREFIX-id@ -->` or `<!-- @PREFIX-id@ (FROM: @PARENT@) -->` immediately **above** the section heading (`### Title`). The scanner takes the next non-empty line (stripping leading `#`, `*`, `-`) as the item's `title` and extends `line_range` until the next `<!--`, `//`, or `/*` tag definition. (Lines starting directly with `@PREFIX-id@` or `#` containing `@PREFIX-id@` are also detected as definitions.)
  - Visible metadata-only blocks (`- **ID**: REQ-...` without `<!-- @REQ-...@ -->`) are not yet supported in v0.1.6.
- **Raw string guard**: Lines between `r#"` and `"#` are skipped while scanning so embedded test fixtures or templates are not indexed as tags.

## CLI Commands

- `reqtrace update` — Scans `paths.scan` and writes a key-sorted JSON graph to `paths.db` (creating parent directories if needed).
- `reqtrace validate` — Checks the live scanned graph for `BrokenLink`, `DuplicateId`, `Orphan` (non-`Requirement` item with empty `derived_from`), and `UntestedRequirement` (no direct or transitive `Test` child). Exits `1` if any issue is found.
- `reqtrace list [--type <TYPE>] [--search <TEXT>] [--gaps] [--json]` — Reads `.reqtrace/db.json` if present (otherwise scans on the fly). `--type` accepts `req`/`requirement`, `arc`/`arch`/`architecture`, `imp`/`impl`/`implementation`, and `ut`/`it`/`test`. `--gaps` lists `Requirement` items missing a **direct** `Architecture` or `Test` child.
- `reqtrace show <ID>... [--upstream] [--downstream] [--no-snippets] [--full] [--max-lines <N>] [--json]` — Reads `.reqtrace/db.json` if present. By default shows immediate parents and children; pass `--upstream` and/or `--downstream` for transitive traversal. Displays full sections for `Requirement`/`Architecture` items and caps `Implementation`/`Test` snippets at `--max-lines` (`25` by default) unless `--full` is passed.
- `reqtrace gen-id [-p|--prefix <PREFIX>] [-c|-n|--count <N>]` — Scans `paths.scan` for existing IDs and outputs collision-free 16-bit proquints. Deterministic sequence seed is derived from the project name (`Cargo.toml` `package.name` → `pyproject.toml` `project.name` → current directory name).
- `reqtrace export -o|--output <FILE>` — Generates a standalone HTML graph visualization.
- `reqtrace server` — Starts the file watcher, web UI, and HTTP/SSE MCP server (requires `server` feature).

## Cargo Features & Binary Size

Default features enabled in `Cargo.toml`: `cli`, `server`, `mcp` (implies `server`), `rust`, `python`, `typescript`.

`reqtrace` supports conditional compilation using Cargo feature flags to reduce dependencies and binary size. The following measurements are for the release Windows build (`x86_64-pc-windows-msvc` built with `cargo build --release`):

| Feature Set | Enabled Features | Binary Size (Windows 64-bit Release) | Description |
| ----------- | ---------------- | ------------------------------------ | ----------- |
| **None** (Stub) | None | 1.50 MB | Pure stub binary. Excludes CLI, Server, MCP, and all language support. |
| **CLI Only (Rust + Python)** | `cli`, `rust`, `python` | 5.76 MB | Standard CLI commands (`validate`, `update`, `export`) with Rust & Python AST parsing. |
| **CLI + Server (Rust + Python)** | `cli`, `rust`, `python`, `server` | 5.73 MB | Adds the background server and file watcher (`server` subcommand). |
| **All Features** (Measured Baseline) | `cli`, `rust`, `python`, `server`, `mcp` | 5.79 MB | Capabilities including Model Context Protocol (MCP) server endpoints (prior to `typescript` parser). |

### Generating IDs

To maintain human-friendly and unique identifiers, `reqtrace` recommends using
proquints. You can generate the next available 16-bit proquint ID for a given
prefix using the `gen-id` command:

```bash
reqtrace gen-id --prefix REQ-
# Output: REQ-lusab
```

The generation algorithm is deterministic and based on the project name,
ensuring a stable sequence of IDs while avoiding collisions with identifiers
already in use.

To compile a custom feature set:

```bash
# Build CLI + language support (no server/mcp)
cargo build --release --no-default-features --features "cli rust python"

# Build CLI + Server without MCP
cargo build --release --no-default-features --features "cli rust python server"
```
