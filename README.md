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

## Cargo Features & Binary Size

`reqtrace` supports conditional compilation using Cargo feature flags to reduce dependencies and binary size. The following measurements are for the release Windows build (`x86_64-pc-windows-msvc` built with `cargo build --release`):

| Feature Set | Enabled Features | Binary Size (Windows 64-bit Release) | Description |
| ----------- | ---------------- | ------------------------------------ | ----------- |
| **None** (Stub) | None | 1.50 MB | Pure stub binary. Excludes CLI, Server, MCP, and all language support. |
| **CLI Only (Rust + Python)** | `cli`, `rust`, `python` | 5.76 MB | Standard CLI commands (`validate`, `update`, `export`) with Rust & Python AST parsing. |
| **CLI + Server (Rust + Python)** | `cli`, `rust`, `python`, `server` | 5.73 MB | Adds the background server and file watcher (`server` subcommand). |
| **All Features** (Default) | `cli`, `rust`, `python`, `server`, `mcp` | 5.79 MB | Full capabilities including Model Context Protocol (MCP) server endpoints. |

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
