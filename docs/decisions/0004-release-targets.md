# 0004. Pre-compiled Binary Release Targets

- Status: accepted
- Date: 2026-06-30

## Context

`reqtrace` needs to be compiled and released for developers running on various
operating systems. We need to identify standard target architectures to support
64-bit AMD/Intel PCs.

## Decision

We chose the following standard Rust target triples to compile and release
`reqtrace` binaries:

1. Linux (64-bit AMD/Intel): `x86_64-unknown-linux-gnu`
2. Windows (64-bit AMD/Intel): `x86_64-pc-windows-msvc`

## Consequences

- The release workflow (e.g. GitHub Actions) will cross-compile binaries for
  these target triples.
- Binary releases will be published under these names (e.g.
  `reqtrace-x86_64-unknown-linux-gnu` and
  `reqtrace-x86_64-pc-windows-msvc.exe`).
- The custom GitHub Action will detect the host platform and download the
  corresponding pre-compiled binary.
