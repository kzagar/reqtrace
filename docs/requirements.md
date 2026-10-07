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

<!-- @REQ-gizih@ -->

### Graph Model & Database

`reqtrace` SHALL maintain an in-memory graph of all traceability items and
serialize it into a version-controlled database.

<!-- @REQ-vapik@ (FROM: @REQ-gizih@) -->

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

<!-- @REQ-sivoh@ (FROM: @REQ-gizih@) -->

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

<!-- @REQ-zaruh@ (FROM: @REQ-gizih@) -->

### Parser and File Scanner

The system SHALL scan source code files (e.g. Rust, Markdown) based on
`.reqtrace/config.toml` patterns to extract item definitions and link
annotations.

- Priority: MUST
- Rationale: Discovers the traceability model from files.
- Acceptance:
  - Scenario: Scanning files for tags
    - GIVEN a project containing source files with tag comments (e.g.,
      `// @ARCH-parser@` or `<!-- @REQ-vapik@ -->`)
    - WHEN the scanner is run
    - THEN it finds all tagged items and adds them to the graph with their file
      paths and line numbers

<!-- @REQ-rimad@ (FROM: @REQ-zaruh@) -->

### Python Language Support

The system SHALL support Python (`.py`) files, parsing class, function, and
method declarations to determine their structural names, start lines, and end
lines.

- Priority: MUST
- Rationale: Extends traceability validation to Python codebases.
- Acceptance:
  - Scenario: Scanning Python files
    - GIVEN a Python file containing a class and def method with tag comments
      (e.g., `# @IMP-gizih@`)
    - WHEN the scanner is run
    - THEN it resolves the tag to the class/method scope and correctly sets
      the start and end line ranges.

<!-- @REQ-nikag@ (FROM: @REQ-zaruh@) -->

### Visible Markdown Traceability Items

The system SHALL parse visible item definitions in Markdown files across
multiple formats (compact inline/blockquote metadata, Markdown tables, and
attribute bullet lists with single or sub-bullet parents), extracting the item
ID, upstream parent links, and binding the item title and start line to the
nearest preceding heading within the section.

- Priority: MUST
- Rationale: Allows displaying requirement and architecture IDs visibly in rendered documentation while preserving traceability extraction; see [ADR 0005](decisions/0005-visible-markdown-traceability-and-linking.md).
- Acceptance:
  - Scenario: Inline metadata line
    - GIVEN a Markdown file with heading `### Document Splitting` followed by `**ID**: REQ-lusab | **FROM**: REQ-babad`
    - WHEN the scanner is run
    - THEN an item with ID `REQ-lusab` is discovered with title `Document Splitting`, parent `REQ-babad`, and start line set to the heading
  - Scenario: Table metadata format
    - GIVEN a Markdown file with heading `### Document Splitting` followed by a table with columns `| ID | FROM |` and row `| REQ-lusab | REQ-babad |`
    - WHEN the scanner is run
    - THEN an item with ID `REQ-lusab` is discovered with title `Document Splitting` and parent `REQ-babad`
  - Scenario: Attribute list with sub-bullets
    - GIVEN a Markdown file with heading `### Document Splitting` followed by `- **ID**: REQ-lusab` and `- **From**:` with sub-bullets `- REQ-babad` and `- REQ-dalap`
    - WHEN the scanner is run
    - THEN an item with ID `REQ-lusab` is discovered with parents `REQ-babad` and `REQ-dalap`

<!-- @REQ-tokuk@ (FROM: @REQ-nikag@) -->

### Bare and Hyperlinked ID Parsing

The system SHALL accept bare IDs without `@` delimiters in recognized explicit
metadata fields (`ID:`, `From:`, and table columns), and SHALL extract target
IDs from inside Markdown hyperlinks.

- Priority: MUST
- Rationale: Enables clean rendered typography and Markdown cross-linking without breaking ID parsing; see [ADR 0005](decisions/0005-visible-markdown-traceability-and-linking.md).
- Acceptance:
  - Scenario: Parsing bare IDs in metadata fields
    - GIVEN an attribute list `- **ID**: REQ-lusab` and `- **From**: REQ-babad` without `@` wrappers
    - WHEN the scanner is run
    - THEN `REQ-lusab` is parsed as the item ID and `REQ-babad` as the derived-from parent
  - Scenario: Extracting IDs from Markdown hyperlinks
    - GIVEN an attribute list `- **ID**: [REQ-lusab](#document-splitting)` and `- **From**: [REQ-babad](requirements.md#REQ-babad)`
    - WHEN the scanner is run
    - THEN `REQ-lusab` is extracted as the item ID and `REQ-babad` as the parent ID

---

## 2. Validation & Checking

<!-- @REQ-fuloz@ -->

### Validation & Checking

`reqtrace` SHALL validate the traceability graph and detect gaps or
inconsistencies.

<!-- @REQ-rulad@ (FROM: @REQ-fuloz@) -->

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
    - GIVEN a graph with a requirement `@REQ-vapik@` and an architectural item
      `@ARCH-helper@` that does not link to any requirement
    - WHEN validation is executed
    - THEN the tool reports `@ARCH-helper@` as an orphan item

<!-- @REQ-zolag@ (FROM: @REQ-fuloz@) -->

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

<!-- @REQ-rivil@ -->

### Comment Formatting & Updating

`reqtrace` SHALL format and update reference comments in project source files.

<!-- @REQ-tusut@ (FROM: @REQ-rivil@) -->

### Standardizing Reference Comments

The system SHALL format link comments in source files to include standard titles
of the derived-from items.

- Priority: SHOULD
- Rationale: Keeps inline comments in sync with requirement changes
  automatically.
- Acceptance:
  - Scenario: Updating multi-line comments
    - GIVEN a source file with `// @ARCH-parser@ FROM: REQ-zaruh`
    - WHEN the update CLI command is executed
    - THEN the comment is rewritten to
      `// @ARCH-parser@ FROM:\n//   REQ-zaruh (input file name is specified for parser)`
    - AND the requirement title is appended in parentheses

---

## 4. Server Mode

<!-- @REQ-siris@ -->

### Server Mode

`reqtrace` SHALL support a long-running daemon server that monitors files,
serves a Web UI, and exposes an MCP server.

<!-- @REQ-palan@ (FROM: @REQ-siris@) -->

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

<!-- @REQ-mozum@ (FROM: @REQ-siris@) -->

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

<!-- @REQ-votar@ (FROM: @REQ-siris@) -->

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

<!-- @REQ-mijom@ (FROM: @REQ-votar@) -->

### MCP Tool for Context Retrieval

The MCP server SHALL expose a tool that, given a list of item IDs, retrieves
those items, their upstream and downstream related items, and their
corresponding source code implementations resolved via the Language Server.

- Priority: MUST
- Rationale: Allows AI agents to gather full implementation context and
  traceability links for specific requirements or code components.
- Related: @REQ-jomag@
- Acceptance:
  - Scenario: Retrieve context for a requirement ID
    - GIVEN the MCP server is active and LSP is available
    - WHEN the tool is called with `["REQ-vapik"]`
    - THEN it returns the metadata for `REQ-vapik`
    - AND it returns its linked architecture and test items
    - AND it returns the source code snippets implementing those
      architectural/test items resolved via the Language Server

<!-- @REQ-vakih@ (FROM: @REQ-siris@) -->

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

<!-- @REQ-vugul@ -->

### CLI Operations

`reqtrace` SHALL provide a command-line interface for running the server,
updating comment headers, and validating the graph.

<!-- @REQ-gamof@ (FROM: @REQ-vugul@) -->

### CLI Commands

The CLI tool SHALL expose the following subcommands: `server` (start the
server), `update` (sync database and rewrite comments), `validate` (lint the
graph), `export` (generate standalone visualization), `gen-id` (generate
proquint identifiers), and `link` (insert explicit anchors and link item references).

- Priority: MUST
- Rationale: Primary developer interaction.
- Acceptance:
  - Scenario: Help menu
    - WHEN `reqtrace --help` is executed
    - THEN it lists `server`, `update`, `validate`, `export`, `gen-id`, and `link` as
      available subcommands

<!-- @REQ-rudar@ (FROM: @REQ-gamof@) -->

### Traceability Link Command

The CLI tool SHALL provide a `link` subcommand that scans Markdown files,
inserts missing explicit HTML anchors `<a id="..."></a>` at the end of section
heading lines for all defined items, and rewrites unlinked item references into
relative Markdown links.

- Priority: MUST
- Rationale: Automates cross-document and intra-document navigation between related requirements and architectural components; see [ADR 0005](decisions/0005-visible-markdown-traceability-and-linking.md).
- Acceptance:
  - Scenario: Inserting explicit heading anchors
    - GIVEN a Markdown file with heading `### Document Splitting` defining `REQ-lusab` without an HTML anchor
    - WHEN `reqtrace link` is executed
    - THEN the heading is updated to `### Document Splitting <a id="REQ-lusab"></a>`
  - Scenario: Rewriting unlinked references to relative Markdown links
    - GIVEN a Markdown file containing `From: REQ-lusab` where `REQ-lusab` is defined in `docs/requirements.md`
    - WHEN `reqtrace link` is executed
    - THEN the reference is replaced with `From: [REQ-lusab](requirements.md#REQ-lusab)`
  - Scenario: Preserving existing links
    - GIVEN a Markdown file containing an already hyperlinked reference `[REQ-lusab](requirements.md#REQ-lusab)`
    - WHEN `reqtrace link` is executed
    - THEN the reference is not duplicated or double-linked

<!-- @REQ-toloz@ (FROM: @REQ-rudar@) -->

### Traceability Link Verification

The `link` CLI command SHALL support a `--check` flag that validates whether
all explicit heading anchors and item reference hyperlinks in Markdown files
are up to date, exiting with code 0 if all links and anchors are present and
correct, or exiting with a non-zero code if any anchors or hyperlinks need
updating.

- Priority: MUST
- Rationale: Allows CI pipelines to enforce that documentation hyperlinks and anchors remain synchronized without mutating files in-place; see [ADR 0005](decisions/0005-visible-markdown-traceability-and-linking.md).
- Acceptance:
  - Scenario: Check mode succeeds when all anchors and links are present
    - GIVEN a repository where all headings have explicit anchors and all references are hyperlinked
    - WHEN `reqtrace link --check` is executed
    - THEN the command exits with code 0 without modifying any files
  - Scenario: Check mode fails when anchors or links are missing
    - GIVEN a repository containing an unlinked reference `From: REQ-lusab`
    - WHEN `reqtrace link --check` is executed
    - THEN the command exits with a non-zero exit code and outputs the list of files requiring updates

<!-- @REQ-kitir@ (FROM: @REQ-gamof@) -->

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

<!-- @REQ-litip@ (FROM: @REQ-kitir@) -->

### Batch ID Generation

The `gen-id` command SHALL support generating multiple unique IDs in a single
invocation via `-c`, `-n`, or `--count` flags. All IDs generated in the batch SHALL be unique
within the batch and unique across the project.

- Priority: SHOULD
- Rationale: Allows users and agents to obtain a batch of non-colliding IDs
  upfront without requiring interleaved database updates between each ID.
- Acceptance:
  - Scenario: Batch generation of unique IDs
    - GIVEN a project with existing IDs
    - WHEN `reqtrace gen-id --prefix REQ- --count 5` or `reqtrace gen-id --prefix REQ- -n 5` is executed
    - THEN it outputs 5 distinct IDs, one per line
    - AND none of the generated IDs collide with existing project IDs or with
      each other

<!-- @REQ-jafaf@ (FROM: @REQ-vugul@) -->

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

<!-- @REQ-bofud@ (FROM: @REQ-vugul@) -->

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

<!-- @REQ-komop@ -->

### Deployment & CI/CD

`reqtrace` SHALL support multi-platform pre-compiled releases and a GitHub
Action.

<!-- @REQ-kuguj@ (FROM: @REQ-komop@) -->

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

<!-- @REQ-zapaj@ (FROM: @REQ-komop@) -->

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

<!-- @REQ-puzun@ -->

### Language Server Integration

`reqtrace` SHALL integrate with Language Servers to resolve source code
locations and details.

<!-- @REQ-jomag@ (FROM: @REQ-puzun@) -->

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
