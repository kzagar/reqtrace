# Requirements — reqtrace

> Glossary: see docs/CONTEXT.md. Decisions: see docs/decisions/.

## Goal & Vision

In the AI era, where LLMs perform the majority of code generation, `reqtrace`
ensures that software projects remain coherent, architecturally sound, and
strictly grounded on requirements. It achieves this by maintaining precise,
verifiable traceability links and providing LLMs/agents with exactly the minimal
context needed to implement a new requirement or affect an architecture/code
change, avoiding the need for agents to read through large, unrelated portions
of the codebase.

## 1. Graph Model & Database

<!-- @REQ1@ -->

### Graph Model & Database

`reqtrace` SHALL maintain an in-memory graph of all traceability items and
serialize it into a version-controlled database.

<!-- @REQ1.1@ (FROM: @REQ1@) -->

### Traceability Graph Representation

The system SHALL represent a directed graph of items containing Requirements,
Architectural items, and Tests.

- Priority: MUST
- Rationale: Core data model.
- Recommendation: Use proquint identifiers (e.g., `REQ-lusab`) for better human-readability.
- Acceptance:
  - Scenario: Adding items to the graph
    - GIVEN an empty traceability graph
    - WHEN items of type REQ, ARCH, and TEST are added
    - THEN the graph contains these nodes with correct metadata and link
      relationships

<!-- @REQ1.2@ (FROM: @REQ1@) -->

### Database Storage

The system SHALL persist the traceability graph to `.reqtrace/db.json` in a
pretty-printed, key-sorted, single-item-per-line JSON format.

- Priority: MUST
- Rationale: Ensures readable Git diffs; see docs/decisions/0003-db-schema.md.
- Acceptance:
  - Scenario: Serializing the database
    - GIVEN a populated traceability graph
    - WHEN the database is serialized to disk
    - THEN the file is written to `.reqtrace/db.json`
    - AND the JSON keys are sorted alphabetically
    - AND the indentation is standardized to ensure clean Git diffs

<!-- @REQ1.3@ (FROM: @REQ1@) -->

### Parser and File Scanner

The system SHALL scan source code files (e.g. Rust, Markdown) based on
`.reqtrace/config.toml` patterns to extract item definitions and link
annotations.

- Priority: MUST
- Rationale: Discovers the traceability model from files.
- Acceptance:
  - Scenario: Scanning files for tags
    - GIVEN a project containing source files with tag comments (e.g.,
      `// @ARCH-parser@` or `<!-- @REQ1.1@ -->`)
    - WHEN the scanner is run
    - THEN it finds all tagged items and adds them to the graph with their file
      paths and line numbers

<!-- @REQ1.4@ (FROM: @REQ1.3@) -->

### Python Language Support

The system SHALL support Python (`.py`) files, parsing class, function, and
method declarations to determine their structural names, start lines, and end
lines.

- Priority: MUST
- Rationale: Extends traceability validation to Python codebases.
- Acceptance:
  - Scenario: Scanning Python files
    - GIVEN a Python file containing a class and def method with tag comments
      (e.g., `# @IMP1.1@`)
    - WHEN the scanner is run
    - THEN it resolves the tag to the class/method scope and correctly sets
      the start and end line ranges.

---

## 2. Validation & Checking

<!-- @REQ2@ -->

### Validation & Checking

`reqtrace` SHALL validate the traceability graph and detect gaps or
inconsistencies.

<!-- @REQ2.1@ (FROM: @REQ2@) -->

### Traceability Issue Detection

The system SHALL detect the following validation issues:

1. Untraced requirements (no links from architecture or tests).
2. Orphan architecture items or tests (not derived from any requirement,
   directly or transitively).
3. Broken links (derived-from points to a non-existent item).
4. Cyclic dependencies.
5. Duplicate item IDs.

- Priority: MUST
- Rationale: Detects gaps in verification and implementation.
- Acceptance:
  - Scenario: Identifying orphan architecture items
    - GIVEN a graph with a requirement `@REQ1.1@` and an architectural item
      `@ARCH-helper@` that does not link to any requirement
    - WHEN validation is executed
    - THEN the tool reports `@ARCH-helper@` as an orphan item

<!-- @REQ2.2@ (FROM: @REQ2@) -->

### CLI Validation Validation Output

The CLI validate command SHALL output all validation issues and return a
non-zero exit code if issues are found.

- Priority: MUST
- Rationale: Allows integration into local lint checks and CI/CD.
- Acceptance:
  - Scenario: Validation fail in CI
    - GIVEN a project with duplicate item IDs
    - WHEN `reqtrace validate` is executed
    - THEN the command prints detail about the duplicate IDs to stderr
    - AND exits with exit code 1

---

## 3. Comment Formatting & Updating

<!-- @REQ3@ -->

### Comment Formatting & Updating

`reqtrace` SHALL format and update reference comments in project source files.

<!-- @REQ3.1@ (FROM: @REQ3@) -->

### Standardizing Reference Comments

The system SHALL format link comments in source files to include standard titles
of the derived-from items.

- Priority: SHOULD
- Rationale: Keeps inline comments in sync with requirement changes
  automatically.
- Acceptance:
  - Scenario: Updating multi-line comments
    - GIVEN a source file with `// @ARCH-parser@ FROM: REQ1.3`
    - WHEN the update CLI command is executed
    - THEN the comment is rewritten to
      `// @ARCH-parser@ FROM:\n//   REQ1.3 (input file name is specified for parser)`
    - AND the requirement title is appended in parentheses

---

## 4. Server Mode

<!-- @REQ4@ -->

### Server Mode

`reqtrace` SHALL support a long-running daemon server that monitors files,
serves a Web UI, and exposes an MCP server.

<!-- @REQ4.1@ (FROM: @REQ4@) -->

### Automatic Reloading

The server SHALL monitor project files for modifications and rebuild the
traceability graph dynamically upon any change.

- Priority: MUST
- Rationale: Keeps the in-memory graph up-to-date during interactive
  development.
- Acceptance:
  - Scenario: Reloading on file change
    - GIVEN the reqtrace server is running
    - WHEN a source file's comment is edited and saved
    - THEN the server detects the change
    - AND updates the in-memory traceability graph automatically

<!-- @REQ4.2@ (FROM: @REQ4@) -->

### Web UI & Interactive Graph Visualization

The server SHALL host a web interface serving an interactive page that
visualizes the traceability graph and lists validation issues. The dynamic part
of the visualization (the graph database JSON) SHALL be dynamically loaded from
the server, sharing the same HTML/CSS/JS frontend as the static export.

- Priority: MUST
- Rationale: Visual mapping and validation of requirements to implementation.
- Acceptance:
  - Scenario: Navigation on graph UI
    - GIVEN the web UI is loaded with a graph containing requirements,
      architecture, and tests
    - WHEN a node is clicked
    - THEN the node is centered in the viewport
    - AND its derived and deriving items are shown
    - AND nodes are color-coded and shaped according to their type (Requirement,
      Architecture, or Test)

<!-- @REQ4.3@ (FROM: @REQ4@) -->

### MCP Server

The server SHALL expose a Model Context Protocol (MCP) server over HTTP/SSE on
the same port to allow agents to query the graph.

- Priority: MUST
- Rationale: Enables agents to query requirements/architecture details; see
  docs/decisions/0002-mcp-transport.md.
- Acceptance:
  - Scenario: Querying graph via MCP tool
    - GIVEN the reqtrace server is running and the MCP SSE endpoint is active
    - WHEN an MCP client connects and invokes a query tool
    - THEN the server returns the requested node metadata from the warmed
      in-memory graph

<!-- @REQ4.3.1@ (FROM: @REQ4.3@) -->

### MCP Tool for Context Retrieval

The MCP server SHALL expose a tool that, given a list of item IDs, retrieves
those items, their upstream and downstream related items, and their
corresponding source code implementations resolved via the Language Server.

- Priority: MUST
- Rationale: Allows AI agents to gather full implementation context and
  traceability links for specific requirements or code components.
- Related: @REQ7.1@
- Acceptance:
  - Scenario: Retrieve context for a requirement ID
    - GIVEN the MCP server is active and LSP is available
    - WHEN the tool is called with `["REQ1.1"]`
    - THEN it returns the metadata for `REQ1.1`
    - AND it returns its linked architecture and test items
    - AND it returns the source code snippets implementing those
      architectural/test items resolved via the Language Server

<!-- @REQ4.4@ (FROM: @REQ4@) -->

### Feature Gating

The system SHALL support conditional compilation of major features—CLI,
Server Mode, and MCP Server—using Cargo feature flags.

- Priority: MUST
- Rationale: Minimizes binary size and dependency footprint for specialized
  environments (e.g. CI runner vs active daemon).
- Acceptance:
  - Scenario: Compiling without server features
    - WHEN reqtrace is built with `--no-default-features --features cli`
    - THEN the compiler builds the binary without axum or notify dependencies
    - AND the resulting binary is smaller in size than the full-featured binary

---

## 5. CLI Operations

<!-- @REQ5@ -->

### CLI Operations

`reqtrace` SHALL provide a command-line interface for running the server,
updating comment headers, and validating the graph.

<!-- @REQ5.1@ (FROM: @REQ5@) -->

### CLI Commands

The CLI tool SHALL expose the following subcommands: `server` (start the
server), `update` (sync database and rewrite comments), `validate` (lint the
graph), `export` (generate standalone visualization), and `gen-id` (generate
proquint identifiers).

- Priority: MUST
- Rationale: Primary developer interaction.
- Acceptance:
  - Scenario: Help menu
    - WHEN `reqtrace --help` is executed
    - THEN it lists `server`, `update`, `validate`, `export`, and `gen-id` as
      available subcommands

<!-- @REQ5.4@ (FROM: @REQ5.1@) -->

### ID Generation

The system SHALL provide a command to generate a new, unique proquint
identifier based on a deterministic sequence seeded by the project name.

- Priority: SHOULD
- Rationale: Simplifies creating human-friendly, non-colliding IDs.
- Acceptance:
  - Scenario: Generating a unique ID
    - GIVEN a project with existing IDs `REQ-lusab` and `REQ-babad`
    - WHEN `reqtrace gen-id --prefix REQ-` is executed
    - THEN it outputs the next available ID in the proquint sequence
    - AND the outputted ID does not conflict with any existing IDs

<!-- @REQ5.2@ (FROM: @REQ5@) -->

### Configuration File Parsing

The system SHALL parse `.reqtrace/config.toml` to load project configuration
details.

- Priority: MUST
- Rationale: Customizes path settings; see docs/decisions/0001-config-format.md.
- Acceptance:
  - Scenario: Reading invalid configuration
    - GIVEN an invalid or missing config file
    - WHEN the CLI is invoked
    - THEN it exits with a descriptive error message indicating configuration
      loading failed

<!-- @REQ5.3@ (FROM: @REQ5@) -->

### Static HTML Graph Export

The CLI export command SHALL generate a standalone, static HTML/CSS/JS file
containing the visualization page with the current graph database JSON embedded
directly into it.

- Priority: MUST
- Rationale: Enables sharing and viewing the graph without requiring a
  background daemon.
- Acceptance:
  - Scenario: Generating static export
    - GIVEN a project with a traceability graph
    - WHEN `reqtrace export --output report.html` is executed
    - THEN a single static HTML file is generated at `report.html`
    - AND opening the HTML file in a browser loads the interactive graph
      navigation without requiring a running server

---

## 6. Deployment & CI/CD

<!-- @REQ6@ -->

### Deployment & CI/CD

`reqtrace` SHALL support multi-platform pre-compiled releases and a GitHub
Action.

<!-- @REQ6.1@ (FROM: @REQ6@) -->

### Pre-compiled Releases

The tool SHALL build and publish pre-compiled executable releases for Linux
64-bit AMD and Windows 64-bit AMD.

- Priority: MUST
- Rationale: Ease of installation; see docs/decisions/0004-release-targets.md.
- Acceptance:
  - Scenario: Execution on target platform
    - GIVEN the pre-compiled binary for Windows 64-bit
    - WHEN run in a standard 64-bit Windows environment
    - THEN it starts without dynamic linking failures

<!-- @REQ6.2@ (FROM: @REQ6@) -->

### Custom GitHub Action

The system SHALL provide a composite GitHub Action in the `action/` directory
that downloads a hard-coded release version of the `reqtrace` binary and runs
validation.

- Priority: MUST
- Rationale: Integrates validation into GitHub workflow runs.
- Acceptance:
  - Scenario: Running in GitHub Action step
    - GIVEN a GitHub workflow step referencing `action/`
    - WHEN the workflow runs
    - THEN the Action downloads the correct release binary for the runner OS
    - AND executes validation against the repository

---

## 7. Language Server Integration

<!-- @REQ7@ -->

### Language Server Integration

`reqtrace` SHALL integrate with Language Servers to resolve source code
locations and details.

<!-- @REQ7.1@ (FROM: @REQ7@) -->

### LSP Source Location Resolution

The system SHALL query the local Language Server (if available) to locate source
code definitions or retrieve implementation context of architectural or test
items.

- Priority: SHOULD
- Rationale: Delegates source symbol lookup to a dedicated language server for
  accurate line ranges and content.
- Acceptance:
  - Scenario: Resolve function definition via LSP
    - GIVEN a running language server supporting Rust
    - WHEN `reqtrace` queries for symbol `@ARCH-parser@` representing a function
      in parser.rs
    - THEN it resolves the exact line range and contents of the implementation
      via LSP
