# 0002. MCP Server Transport Protocol

- Status: accepted
- Date: 2026-06-30

## Context

`reqtrace` provides a Model Context Protocol (MCP) server so that AI agents can
query the traceability graph. The graph is maintained dynamically in memory by a
long-running stateful `reqtrace` server that monitors file changes. We need to
choose the transport protocol for the MCP server.

We considered:

- **stdio**: The standard transport for local MCP. However, running a new stdio
  process means the process either has to parse all project files from scratch
  on startup (poor performance) or act as a thin client proxy communicating with
  the background daemon (complex IPC).
- **HTTP with Server-Sent Events (SSE)**: The native network transport for MCP.
  The background daemon is already running a web server. Exposing the MCP
  endpoints via SSE on the same HTTP port leverages the warmed in-memory graph
  directly, offering optimal performance and simplicity.

## Decision

We chose **HTTP with Server-Sent Events (SSE)** as the transport for the MCP
server. The MCP server will be hosted on the same dynamically allocated port as
the `reqtrace` Web UI.

## Consequences

- The daemon will serve both the Web UI and the MCP SSE endpoints.
- Agents will connect to the MCP server via an HTTP URL (e.g.,
  `http://127.0.0.1:<port>/mcp`).
- We avoid parsing overhead on MCP connection start, as the daemon already
  maintains the parsed graph in-memory.

## Alternatives considered

- stdio (rejected due to startup latency or proxy IPC complexity).
