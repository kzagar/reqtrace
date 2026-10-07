# 0005. Visible Markdown Traceability Items and Explicit Hyperlinking

- Status: accepted
- Date: 2026-10-07

## Context

In `docs/requirements.md` and `docs/architecture.md`, traceability items historically relied solely on HTML comment tags (e.g., `<!-- @REQ-pudup@ (FROM: @REQ-gilun@) -->`) placed immediately before section headings. While this prevented interference with raw prose, it hid requirement and architecture identifiers from rendered documentation previews in GitHub, web documentation sites, and markdown preview tools.

Users need requirement and architecture IDs to be visibly displayed alongside section titles and attributes. Additionally, authors want to navigate between linked requirements and architectural items easily via clickable hyperlinks.

We considered:
1. **Comment tags only**: Retains hidden metadata, but requires readers to inspect raw Markdown source to discover item IDs.
2. **Single mandatory visible format**: Mandating only tables or only attribute lists. Too restrictive for varying document styles and workflows.
3. **Multi-format visible metadata with explicit HTML heading anchors**: Allowing multiple visible representations (compact inline/blockquote, tables, and attribute lists with nested parent bullets), supporting bare IDs in recognized fields, and providing an explicit heading anchor convention (`<a id="..."></a>`) automated by a dedicated `reqtrace link` subcommand.

## Decision

We chose **Multi-format visible metadata with explicit HTML heading anchors** and the addition of a `reqtrace link` CLI command.

Specifically:
- Visible item definitions are recognized in Markdown files across three formats:
  - Compact inline/blockquote metadata (`**ID**: REQ-xxx | **FROM**: REQ-yyy` or `> **ID**: ...`).
  - Markdown tables with `ID` and `FROM` columns (`| ID | FROM |`).
  - Attribute bullet lists (`- **ID**: REQ-xxx` followed by `- **From**: REQ-yyy` or nested sub-bullets).
- Metadata blocks can appear anywhere within the section under their heading before the next heading of equivalent or higher level.
- Item titles and start lines bind to the section heading.
- Bare IDs without `@` delimiters are permitted in recognized explicit metadata fields (`ID:`, `From:`, and table columns), and IDs wrapped in Markdown links (`[REQ-xxx](...)`) are cleanly extracted.
- Explicit case-sensitive HTML anchors `<a id="..."></a>` are placed at the end of section heading lines.
- A new CLI subcommand, `reqtrace link`, automates inserting missing explicit anchors and rewriting plain item references in Markdown files into relative Markdown links, with a `--check` flag for CI validation.
- Legacy HTML comment tags (`<!-- @REQ-xxx@ -->`) remain fully supported for backward compatibility.

## Consequences

- Documentation renders item IDs and relationships clearly in GitHub and web viewers.
- Authors can choose between concise inline metadata, structured tables, or bullet lists based on project preference.
- Heading anchors remain stable even if the human-readable heading title is edited.
- `reqtrace link --check` can be integrated into CI/CD pipelines to ensure all references are linked without broken references.

## Alternatives considered

- **Heading slug anchors** (e.g. `[REQ-xxx](#some-heading-title)`): Rejected because links break whenever a heading title is rephrased or renamed.
- **Strict delimiter requirement only** (disallowing bare `REQ-xxx` in tables/lists): Rejected because requiring `@REQ-xxx@` in human-facing rendered tables and lists degrades visual cleanliness.
