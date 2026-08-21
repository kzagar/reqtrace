# Architecture — reqtrace

> Requirements: docs/requirements.md. Decisions: docs/decisions/. Glossary: docs/CONTEXT.md.

## Overview
`reqtrace` is a Rust-based tool that maintains a Traceability Graph by scanning project files for tag annotations. It provides a CLI for validation and updates, a background server with an MCP interface for AI agents, and a web visualization for humans.

## 1. Core Graph & Data Management

<!-- @ARC-gizih@ (FROM: @REQ-gizih@) -->
### Core Graph & Data Management
The foundation of `reqtrace` that manages the in-memory representation and persistence of traceability items.

<!-- @ARC-vapik@ (FROM: @REQ-vapik@) -->
### Traceability Graph Model
An in-memory directed graph where nodes are `TraceItem` objects (Requirements, Architecture, Tests) and edges represent `derived_from` relationships.

<!-- @ARC-sivoh@ (FROM: @REQ-sivoh@) -->
### JSON Database Serializer
Handles reading and writing the `.reqtrace/db.json` file. Ensures stable, sorted, pretty-printed output for clean Git diffs.

<!-- @ARC-zaruh@ (FROM: @REQ-jafaf@) -->
### Configuration Loader
Parses `.reqtrace/config.toml` to determine scan paths, ignore patterns, and ID-to-type mappings.

## 2. Extraction & Transformation

<!-- @ARC-rimad@ (FROM: @REQ-zaruh@, @REQ-rivil@) -->
### Extraction & Transformation
Logic for discovering tags in source files and keeping them in sync.

<!-- @ARC-fuloz@ (FROM: @REQ-zaruh@) -->
### Multi-Language File Scanner
Recursively walks configured paths and uses regex-based parsers to extract tags and metadata (file, line, title) from Rust, Markdown, and other configured file types.

<!-- @ARC-zolag@ (FROM: @REQ-rimad@) -->
### Language Support Registry & Parsers
An isolated plugin/registry architecture for parsing source code files. Each supported language (e.g. Rust, Python) implements a common parser interface to return structured symbol scopes and line ranges.

<!-- @ARC-rulad@ (FROM: @REQ-tusut@) -->
### Comment Formatter
Rewrites source file comments to synchronize referenced item titles with the current state of the graph.

## 3. Analysis & Verification

<!-- @ARC-rivil@ (FROM: @REQ-fuloz@) -->
### Analysis & Verification
Ensures the integrity of the traceability graph.

<!-- @ARC-tusut@ (FROM: @REQ-rulad@) -->
### Graph Validator
Runs a suite of checks: orphan detection, broken link detection, cycle detection, and duplicate ID detection.

## 4. Interfaces & Integration

<!-- @ARC-pisap@ (FROM: @REQ-siris@, @REQ-vugul@, @REQ-puzun@) -->
### Interfaces & Integration
How users and agents interact with `reqtrace`.

<!-- @ARC-siris@ (FROM: @REQ-gamof@, @REQ-zolag@) -->
### CLI Command Dispatcher
The entry point for `validate`, `update`, `server`, `export`, and `gen-id`
commands. Built with `clap`.

<!-- @ARC-vugul@ (FROM: @REQ-kitir@) -->
### Proquint ID Generator
Logic for generating unique, deterministic proquint identifiers. Uses a
16-bit permutation (Feistel network) seeded by the project name hash to ensure
a stable sequence and avoid collisions with existing IDs.

<!-- @ARC-palan@ (FROM: @REQ-palan@) -->
### File Watcher
Uses the `notify` crate to detect file changes and trigger graph re-scans in server mode.

<!-- @ARC-mozum@ (FROM: @REQ-mozum@, @REQ-bofud@) -->
### Visualization Engine
Generates an interactive D3.js or similar graph visualization. Served dynamically by the server or exported as a standalone HTML file.

<!-- @ARC-votar@ (FROM: @REQ-votar@, @REQ-mijom@) -->
### MCP SSE Server
Exposes the Traceability Graph via Model Context Protocol over HTTP/SSE, allowing AI agents to query context and implementation details.

<!-- @ARC-mijom@ (FROM: @REQ-jomag@) -->
### LSP Client Integration
Optional component that connects to a local Language Server to resolve exact source ranges and code snippets for architectural items.

<!-- @ARC-vakih@ (FROM: @REQ-vakih@) -->
### Feature Gating Configuration
Cargo feature flags (`cli`, `server`, `mcp`) that compile modules conditionally, excluding parser/web server/CLI dependencies to minimize compilation time and binary size when those environments are not needed.

## Requirement coverage

| Requirement | Covered by | Notes |
| ----------- | ---------- | ----- |
| @REQ-vapik@    | @ARC-vapik@   |       |
| @REQ-sivoh@    | @ARC-sivoh@   |       |
| @REQ-zaruh@    | @ARC-fuloz@   |       |
| @REQ-rimad@    | @ARC-zolag@   |       |
| @REQ-rulad@    | @ARC-tusut@   |       |
| @REQ-zolag@    | @ARC-siris@   |       |
| @REQ-tusut@    | @ARC-rulad@   |       |
| @REQ-palan@    | @ARC-palan@   |       |
| @REQ-mozum@    | @ARC-mozum@   |       |
| @REQ-votar@    | @ARC-votar@   |       |
| @REQ-mijom@  | @ARC-votar@   |       |
| @REQ-vakih@    | @ARC-vakih@   |       |
| @REQ-gamof@    | @ARC-siris@   |       |
| @REQ-kitir@    | @ARC-vugul@   |       |
| @REQ-jafaf@    | @ARC-zaruh@   |       |
| @REQ-bofud@    | @ARC-mozum@   |       |
| @REQ-kuguj@    | —          | CI/CD concern (deployment) |
| @REQ-zapaj@    | —          | CI/CD concern (action/ dir) |
| @REQ-jomag@    | @ARC-mijom@   |       |
