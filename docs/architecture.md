# Architecture — reqtrace

> Requirements: docs/requirements.md. Decisions: docs/decisions/. Glossary: docs/CONTEXT.md.

## Overview
`reqtrace` is a Rust-based tool that maintains a Traceability Graph by scanning project files for tag annotations. It provides a CLI for validation and updates, a background server with an MCP interface for AI agents, and a web visualization for humans.

## 1. Core Graph & Data Management

<!-- @ARC1@ (FROM: @REQ1@) -->
### Core Graph & Data Management
The foundation of `reqtrace` that manages the in-memory representation and persistence of traceability items.

<!-- @ARC1.1@ (FROM: @REQ1.1@) -->
### Traceability Graph Model
An in-memory directed graph where nodes are `TraceItem` objects (Requirements, Architecture, Tests) and edges represent `derived_from` relationships.

<!-- @ARC1.2@ (FROM: @REQ1.2@) -->
### JSON Database Serializer
Handles reading and writing the `.reqtrace/db.json` file. Ensures stable, sorted, pretty-printed output for clean Git diffs.

<!-- @ARC1.3@ (FROM: @REQ5.2@) -->
### Configuration Loader
Parses `.reqtrace/config.toml` to determine scan paths, ignore patterns, and ID-to-type mappings.

## 2. Extraction & Transformation

<!-- @ARC2@ (FROM: @REQ1.3@, @REQ3@) -->
### Extraction & Transformation
Logic for discovering tags in source files and keeping them in sync.

<!-- @ARC2.1@ (FROM: @REQ1.3@) -->
### Multi-Language File Scanner
Recursively walks configured paths and uses regex-based parsers to extract tags and metadata (file, line, title) from Rust, Markdown, and other configured file types.

<!-- @ARC2.2@ (FROM: @REQ3.1@) -->
### Comment Formatter
Rewrites source file comments to synchronize referenced item titles with the current state of the graph.

## 3. Analysis & Verification

<!-- @ARC3@ (FROM: @REQ2@) -->
### Analysis & Verification
Ensures the integrity of the traceability graph.

<!-- @ARC3.1@ (FROM: @REQ2.1@) -->
### Graph Validator
Runs a suite of checks: orphan detection, broken link detection, cycle detection, and duplicate ID detection.

## 4. Interfaces & Integration

<!-- @ARC4@ (FROM: @REQ4@, @REQ5@, @REQ7@) -->
### Interfaces & Integration
How users and agents interact with `reqtrace`.

<!-- @ARC4.1@ (FROM: @REQ5.1@, @REQ2.2@) -->
### CLI Command Dispatcher
The entry point for `validate`, `update`, `server`, and `export` commands. Built with `clap`.

<!-- @ARC4.2@ (FROM: @REQ4.1@) -->
### File Watcher
Uses the `notify` crate to detect file changes and trigger graph re-scans in server mode.

<!-- @ARC4.3@ (FROM: @REQ4.2@, @REQ5.3@) -->
### Visualization Engine
Generates an interactive D3.js or similar graph visualization. Served dynamically by the server or exported as a standalone HTML file.

<!-- @ARC4.4@ (FROM: @REQ4.3@, @REQ4.3.1@) -->
### MCP SSE Server
Exposes the Traceability Graph via Model Context Protocol over HTTP/SSE, allowing AI agents to query context and implementation details.

<!-- @ARC4.5@ (FROM: @REQ7.1@) -->
### LSP Client Integration
Optional component that connects to a local Language Server to resolve exact source ranges and code snippets for architectural items.

## Requirement coverage

| Requirement | Covered by | Notes |
| ----------- | ---------- | ----- |
| @REQ1.1@    | @ARC1.1@   |       |
| @REQ1.2@    | @ARC1.2@   |       |
| @REQ1.3@    | @ARC2.1@   |       |
| @REQ2.1@    | @ARC3.1@   |       |
| @REQ2.2@    | @ARC4.1@   |       |
| @REQ3.1@    | @ARC2.2@   |       |
| @REQ4.1@    | @ARC4.2@   |       |
| @REQ4.2@    | @ARC4.3@   |       |
| @REQ4.3@    | @ARC4.4@   |       |
| @REQ4.3.1@  | @ARC4.4@   |       |
| @REQ5.1@    | @ARC4.1@   |       |
| @REQ5.2@    | @ARC1.3@   |       |
| @REQ5.3@    | @ARC4.3@   |       |
| @REQ6.1@    | —          | CI/CD concern (deployment) |
| @REQ6.2@    | —          | CI/CD concern (action/ dir) |
| @REQ7.1@    | @ARC4.5@   |       |
