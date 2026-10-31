-- NOEMA schema v1. Timestamps are unix-ms, ids are TEXT uuids.
CREATE TABLE projects (
  id TEXT PRIMARY KEY, name TEXT NOT NULL, root_path TEXT NOT NULL,
  created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL);

CREATE TABLE sessions (
  id TEXT PRIMARY KEY, project_id TEXT REFERENCES projects(id) ON DELETE SET NULL,
  agent_id TEXT NOT NULL, model_id TEXT NOT NULL, cwd TEXT NOT NULL,
  repo TEXT, branch TEXT, widget_layout TEXT,
  created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL);
CREATE INDEX idx_sessions_updated ON sessions(updated_at);

CREATE TABLE messages (
  id TEXT PRIMARY KEY, session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  role TEXT NOT NULL, content TEXT NOT NULL, reasoning TEXT,
  input_tokens INTEGER NOT NULL DEFAULT 0, output_tokens INTEGER NOT NULL DEFAULT 0,
  reasoning_tokens INTEGER NOT NULL DEFAULT 0, cache_read INTEGER NOT NULL DEFAULT 0,
  cache_write INTEGER NOT NULL DEFAULT 0, cost_usd REAL NOT NULL DEFAULT 0,
  model_id TEXT, created_at INTEGER NOT NULL);
CREATE INDEX idx_messages_session ON messages(session_id, created_at);

CREATE TABLE providers (
  id TEXT PRIMARY KEY, name TEXT NOT NULL, standard TEXT NOT NULL,
  base_url TEXT NOT NULL, key_ref TEXT, headers TEXT, created_at INTEGER NOT NULL);

CREATE TABLE models (
  id TEXT PRIMARY KEY, provider_id TEXT REFERENCES providers(id) ON DELETE CASCADE,
  name TEXT NOT NULL, capability TEXT NOT NULL,
  context_window INTEGER, params TEXT, created_at INTEGER NOT NULL);
CREATE INDEX idx_models_capability ON models(capability);

CREATE TABLE rate_cards (
  id TEXT PRIMARY KEY, model_id TEXT NOT NULL REFERENCES models(id) ON DELETE CASCADE,
  currency TEXT NOT NULL DEFAULT 'USD',
  input REAL NOT NULL DEFAULT 0, output REAL NOT NULL DEFAULT 0,
  reasoning REAL NOT NULL DEFAULT 0, cache_read REAL NOT NULL DEFAULT 0,
  cache_write REAL NOT NULL DEFAULT 0, per_call REAL NOT NULL DEFAULT 0,
  local_hardware_usd REAL NOT NULL DEFAULT 0, local_lifespan_hours REAL NOT NULL DEFAULT 0,
  local_watts REAL NOT NULL DEFAULT 0, kwh_price REAL NOT NULL DEFAULT 0);

CREATE TABLE rate_windows (
  id TEXT PRIMARY KEY, rate_card_id TEXT NOT NULL REFERENCES rate_cards(id) ON DELETE CASCADE,
  timezone TEXT NOT NULL, weekday_mask INTEGER NOT NULL,
  start_minute INTEGER NOT NULL, end_minute INTEGER NOT NULL, multiplier REAL NOT NULL);

CREATE TABLE usage_events (
  id TEXT PRIMARY KEY, session_id TEXT, agent_id TEXT, model_id TEXT NOT NULL,
  kind TEXT NOT NULL, tokens INTEGER NOT NULL, cost_usd REAL NOT NULL,
  peak INTEGER NOT NULL DEFAULT 0, created_at INTEGER NOT NULL);
CREATE INDEX idx_usage_time ON usage_events(created_at);

CREATE TABLE thoughts (
  id TEXT PRIMARY KEY, session_id TEXT, agent_id TEXT, title TEXT NOT NULL,
  body TEXT NOT NULL, concepts TEXT, cluster_id TEXT,
  embedding BLOB, importance REAL NOT NULL DEFAULT 0.5,
  pos_x REAL, pos_y REAL, pos_z REAL, created_at INTEGER NOT NULL);
CREATE INDEX idx_thoughts_session ON thoughts(session_id);

CREATE TABLE thought_edges (
  id TEXT PRIMARY KEY, from_id TEXT NOT NULL REFERENCES thoughts(id) ON DELETE CASCADE,
  to_id TEXT NOT NULL REFERENCES thoughts(id) ON DELETE CASCADE,
  weight REAL NOT NULL, relation TEXT, UNIQUE(from_id, to_id));

CREATE TABLE dreams (
  id TEXT PRIMARY KEY, session_id TEXT, started_at INTEGER NOT NULL,
  finished_at INTEGER, summary TEXT, insights TEXT,
  input_thoughts TEXT, output_thoughts TEXT);

CREATE TABLE agents (
  id TEXT PRIMARY KEY, name TEXT NOT NULL, role TEXT, model_id TEXT,
  workdirs TEXT, skills TEXT, plugins TEXT, mcp_servers TEXT,
  greeting TEXT NOT NULL, created_at INTEGER NOT NULL);

CREATE TABLE widgets (
  id TEXT PRIMARY KEY, name TEXT NOT NULL, chart TEXT NOT NULL,
  spec TEXT NOT NULL, x INTEGER NOT NULL, y INTEGER NOT NULL,
  w INTEGER NOT NULL, h INTEGER NOT NULL, pinned INTEGER NOT NULL DEFAULT 0);

CREATE TABLE mcp_servers (
  id TEXT PRIMARY KEY, name TEXT NOT NULL, transport TEXT NOT NULL,
  command TEXT, args TEXT, env TEXT, cwd TEXT, url TEXT, headers TEXT,
  auth TEXT, timeout_ms INTEGER, retry TEXT, enabled INTEGER NOT NULL DEFAULT 1);

CREATE TABLE skills (
  id TEXT PRIMARY KEY, name TEXT NOT NULL, version TEXT NOT NULL,
  path TEXT NOT NULL, manifest TEXT NOT NULL, enabled INTEGER NOT NULL DEFAULT 1);

CREATE TABLE plugins (
  id TEXT PRIMARY KEY, name TEXT NOT NULL, version TEXT NOT NULL,
  path TEXT NOT NULL, manifest TEXT NOT NULL, enabled INTEGER NOT NULL DEFAULT 1);

CREATE TABLE repos (
  id TEXT PRIMARY KEY, full_name TEXT NOT NULL, branch TEXT,
  last_diff_stat TEXT, last_synced_at INTEGER, pinned INTEGER NOT NULL DEFAULT 0);

CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL, scope TEXT NOT NULL);
