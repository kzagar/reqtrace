# 0001. Configuration File Format

- Status: accepted
- Date: 2026-06-30

## Context

`reqtrace` needs a configuration file (stored in the `.reqtrace/` hidden
directory) to define source code paths, requirements paths, ignore rules, and
other repository-specific settings. We need to choose a serialization format
that is easy for developers to read and edit, and easy to parse in Rust.

We considered:

- **JSON**: Simple, but does not support comments, which are critical for
  explaining configuration parameters.
- **YAML**: Flexible and readable, but sensitive to indentation/whitespace
  errors which can be frustrating to debug.
- **TOML**: The standard configuration format in the Rust ecosystem (used by
  Cargo). It is easy to write, supports comments, has minimal syntax overhead,
  and is supported by robust crates.

## Decision

We chose **TOML** (specifically `.reqtrace/config.toml`) as the format for the
project configuration file.

## Consequences

- Developers can configure the tool using a familiar format with inline
  comments.
- We will use the `toml` crate in Rust to deserialize the configuration.
- Any configuration change requires updating this file in version control.

## Alternatives considered

- YAML (rejected due to white space complexity and lack of native first-class
  Rust compiler tools representation).
- JSON (rejected due to lack of comment support).
