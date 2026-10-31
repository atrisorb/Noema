use serde_json::{Value, json};
use crate::theme::Styler;

pub const AGENT_REGISTRY: &[(&str, &str, &str, &str)] = &[
    ("axon", "Axon", "Core architecture, storage, daemon, IPC, packaging", "claude-haiku-5-5"),
    ("cortex", "Cortex", "Governance, master plan, repo hygiene, cross-block review", "claude-opus-5-5"),
    ("hippocampus", "Hippocampus", "Semantic memory graph, UMAP projection, physics, Dream Mode", "claude-sonnet-5-5"),
    ("retina", "Retina", "Studio UI, session continuity UX, BI widgets, Playwright QA", "claude-sonnet-5-5"),
    ("soma", "Soma", "3D bio-anatomical engine, shaders, camera, body rendering", "claude-sonnet-5-5"),
    ("synapse", "Synapse", "Ecosystem, manifests, MCP supervisor, marketplace, GitHub plugin", "claude-haiku-5-5"),
    ("thalamus", "Thalamus", "Inference adapters, embeddings, clustering, pricing & FinOps", "claude-sonnet-5-5"),
    ("aesthesis", "Aesthesis", "Vitreous design system, brand, SVG/mascots, artifact UX, visual QA", "claude-sonnet-5-5"),
];

pub fn handle(styler: &Styler, as_json: bool) -> Result<Value, String> {
    if as_json {
        let list: Vec<Value> = AGENT_REGISTRY
            .iter()
            .map(|(id, name, domain, model)| {
                json!({
                    "id": id,
                    "name": name,
                    "domain": domain,
                    "model": model,
                    "status": "ready"
                })
            })
            .collect();
        return Ok(json!({ "agents": list }));
    }

    let headers = ["Agent ID", "Name", "Assigned Domain", "Default Model", "Status"];
    let mut rows = Vec::new();

    for (id, name, domain, model) in AGENT_REGISTRY {
        rows.push(vec![
            styler.bold(&styler.artery(id)),
            styler.bold(&styler.ink(name)),
            styler.ink_soft(domain),
            styler.vein(model),
            format!("{} {}", styler.lymph("●"), styler.ink_muted("Ready")),
        ]);
    }

    println!("\n{}", styler.bold(&styler.artery("▶ NOEMA Specialized Agent Roster (8 Agents):")));
    println!("{}", styler.table(&headers, &rows));
    println!("  Total agent workforce: {}\n", styler.lymph(&AGENT_REGISTRY.len().to_string()));

    Ok(json!({ "count": AGENT_REGISTRY.len() }))
}
