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

## Repository Layout

- `docs/` — Specifications and design records.
  - [CONTEXT.md](docs/CONTEXT.md) — Glossary and domain terminology.
  - [requirements.md](docs/requirements.md) — System requirements with
    Gherkin-style scenarios.
  - [decisions/](docs/decisions/) — Architecture Decision Records (ADRs).
- `.reqtrace/` — Traceability configuration and database.
- `action/` — Custom composite GitHub Action.

- `src/` — Rust source code.
