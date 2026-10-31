export type LayerName = 'skin' | 'skeleton' | 'nerves' | 'vessels' | 'organs' | 'agents';

export interface UsageEvent { kind: string; tokens: number; cost_usd: number; at: number }
export interface ModelUsage { model: string; tokens: number; cost_usd: number }

export interface Agent {
  id: string;
  name: string;
  role: string;
  model: string;
  accent: string;
  greeting: string;
  tabs: {
    activity: { events: UsageEvent[] };
    tokens: { total: number; by_model: ModelUsage[] };
    costs: { total_usd: number; by_model: ModelUsage[] };
    ecosystem: { skills: string[]; plugins: string[]; mcp: string[] };
    workspace: { cwd: string | null; repo: string | null; branch: string | null; last_seen: number };
  };
}

export interface Anatomy {
  version: string;
  anima: { identity: string; system_prompt: string };
  agents: Agent[];
}
