-- desktop-only, never synced: what outside agents did through the local MCP server.
-- undo holds what is needed to reverse a write, or null for reads and failures
CREATE TABLE mcp_calls (
    id      INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    at      INTEGER NOT NULL,
    tool    TEXT NOT NULL,
    args    TEXT NOT NULL,
    ok      INTEGER NOT NULL,
    summary TEXT NOT NULL,
    undo    TEXT,
    undone  INTEGER NOT NULL DEFAULT 0
);
