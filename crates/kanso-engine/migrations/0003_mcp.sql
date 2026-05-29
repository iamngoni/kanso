-- MCP access control: approved clients and their granted capabilities.
-- The engine owns the permission model; the MCP server consults it per call.

CREATE TABLE mcp_clients (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    trusted    INTEGER NOT NULL DEFAULT 0, -- 1 = bypass capability checks
    created_at INTEGER NOT NULL
);

CREATE TABLE mcp_permissions (
    client_id  TEXT NOT NULL REFERENCES mcp_clients (id),
    capability TEXT NOT NULL, -- read | write | delete | run_skill
    PRIMARY KEY (client_id, capability)
);
