#!/usr/bin/env node
// NOEMA memory MCP: read-only semantic introspection over noema.db.
// JSON-RPC over stdio. Nothing but protocol frames reaches stdout.

import { createInterface } from 'node:readline';

const TOOLS = [
  {
    name: 'memory.search',
    description: 'Search thoughts by title or body, ranked by importance.',
    inputSchema: {
      type: 'object',
      properties: {
        query: { type: 'string' },
        limit: { type: 'integer', minimum: 1, maximum: 50 },
      },
      required: ['query'],
    },
  },
  {
    name: 'memory.thought',
    description: 'Fetch one thought with its edges and cluster.',
    inputSchema: {
      type: 'object',
      properties: { id: { type: 'string' } },
      required: ['id'],
    },
  },
  {
    name: 'memory.stats',
    description: 'Counts for thoughts, edges, dreams and clusters.',
    inputSchema: { type: 'object', properties: {} },
  },
];

let db;
let schema = new Set();

async function connect() {
  const path = process.env.NOEMA_DB;
  if (!path) return;
  try {
    const { DatabaseSync } = await import('node:sqlite');
    db = new DatabaseSync(path, { readOnly: true });
    db.prepare('SELECT 1').get();
    for (const row of db.prepare("SELECT name FROM sqlite_master WHERE type = 'table'").all()) {
      schema.add(row.name);
    }
  } catch {
    db = undefined;
  }
}

function reply(id, result) {
  process.stdout.write(JSON.stringify({ jsonrpc: '2.0', id, result }) + '\n');
}

function fail(id, code, message) {
  process.stdout.write(JSON.stringify({ jsonrpc: '2.0', id, error: { code, message } }) + '\n');
}

function toolResult(id, payload) {
  reply(id, { content: [{ type: 'text', text: JSON.stringify(payload) }] });
}

function callTool(name, args) {
  if (name === 'memory.search') {
    if (!db) return { results: [], note: 'NOEMA_DB not readable' };
    const limit = Math.min(Math.max(Number(args.limit) || 10, 1), 50);
    const needle = '%' + String(args.query ?? '') + '%';
    if (!schema.has('thoughts')) return { results: [] };
    const rows = db
      .prepare(
        'SELECT id, title, body, cluster_id, importance, created_at FROM thoughts ' +
          'WHERE title LIKE ? OR body LIKE ? ORDER BY importance DESC LIMIT ?',
      )
      .all(needle, needle, limit);
    return { results: rows };
  }
  if (name === 'memory.thought') {
    if (!db) return { thought: null, edges: [], note: 'NOEMA_DB not readable' };
    if (!schema.has('thoughts')) return { thought: null, edges: [] };
    const thought = db.prepare('SELECT * FROM thoughts WHERE id = ?').get(String(args.id ?? ''));
    const edges = schema.has('thought_edges')
      ? db.prepare('SELECT * FROM thought_edges WHERE from_id = ? OR to_id = ?').all(args.id, args.id)
      : [];
    return { thought: thought ?? null, edges };
  }
  if (name === 'memory.stats') {
    if (!db) return { thoughts: 0, edges: 0, dreams: 0, clusters: 0, note: 'NOEMA_DB not readable' };
    const count = (table) =>
      schema.has(table) ? db.prepare('SELECT COUNT(*) AS n FROM ' + table).get().n : 0;
    return {
      thoughts: count('thoughts'),
      edges: count('thought_edges'),
      dreams: count('dreams'),
      clusters: schema.has('thoughts')
        ? db.prepare('SELECT COUNT(DISTINCT cluster_id) AS n FROM thoughts WHERE cluster_id IS NOT NULL').get().n
        : 0,
    };
  }
  throw new Error('unknown tool: ' + name);
}

async function handle(message) {
  const { id, method, params } = message;
  if (method === 'initialize') {
    reply(id, {
      protocolVersion: params?.protocolVersion ?? '2025-06-18',
      capabilities: { tools: {} },
      serverInfo: { name: 'noema-memory', version: '1.0.0' },
    });
    return;
  }
  if (method === 'tools/list') {
    reply(id, { tools: TOOLS });
    return;
  }
  if (method === 'tools/call') {
    try {
      toolResult(id, callTool(params?.name, params?.arguments ?? {}));
    } catch (error) {
      fail(id, -32602, error.message);
    }
    return;
  }
  if (id !== undefined) fail(id, -32601, 'method not found: ' + method);
}

await connect();
const input = createInterface({ input: process.stdin, crlfDelay: Infinity });
for await (const line of input) {
  if (!line.trim()) continue;
  let message;
  try {
    message = JSON.parse(line);
  } catch {
    fail(null, -32700, 'parse error');
    continue;
  }
  await handle(message);
}
