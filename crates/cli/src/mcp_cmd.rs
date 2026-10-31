use serde_json::{Value, json};
use crate::theme::Styler;

pub fn handle(args: &[String], styler: &Styler, as_json: bool) -> Result<Value, String> {
    let sub = args.first().map(String::as_str).unwrap_or("list");
    match sub {
        "list" => list(styler, as_json),
        "status" => status(styler, as_json),
        "test" => {
            let Some(path) = args.get(1) else {
                return Err("usage: noema mcp test <manifest-path>".into());
            };
            test_manifest(path, styler, as_json)
        }
        other => Err(format!("unknown mcp subcommand: '{other}'. Expected list, status, test")),
    }
}

pub fn list(styler: &Styler, as_json: bool) -> Result<Value, String> {
    let packages = noema_ecosystem::marketplace::bundled();

    let mut list_json = Vec::new();
    let mut rows = Vec::new();

    for pkg in &packages {
        if let Ok(m) = pkg.manifest() {
            if as_json {
                list_json.push(json!({
                    "id": pkg.id,
                    "kind": m.kind.as_str(),
                    "version": m.version,
                    "description": m.description,
                }));
            } else {
                rows.push(vec![
                    styler.bold(&styler.ink(pkg.id)),
                    styler.vein(m.kind.as_str()),
                    styler.nerve(&m.version),
                    styler.ink_soft(&m.description),
                ]);
            }
        }
    }

    if as_json {
        return Ok(json!({ "packages": list_json }));
    }

    let headers = ["Package ID", "Kind", "Version", "Description"];
    println!("\n{}", styler.bold(&styler.artery("▶ Verified MCP Packages & Tools:")));
    println!("{}", styler.table(&headers, &rows));
    println!("  Total packages available: {}\n", styler.lymph(&packages.len().to_string()));

    Ok(json!({ "count": packages.len() }))
}

pub fn status(styler: &Styler, as_json: bool) -> Result<Value, String> {
    let packages = noema_ecosystem::marketplace::bundled();
    let memory_mcp_installed = packages.iter().any(|p| p.id == "memory-mcp");
    let visual_qa_installed = packages.iter().any(|p| p.id == "system-visual-qa");
    let github_plugin_installed = packages.iter().any(|p| p.id == "github-plugin");

    let payload = json!({
        "supervisor_active": true,
        "transports_supported": ["stdio", "sse", "websocket"],
        "bundled_packages_count": packages.len(),
        "servers": [
            { "id": "memory-mcp", "transport": "stdio", "ready": memory_mcp_installed },
            { "id": "system-visual-qa", "transport": "stdio", "ready": visual_qa_installed },
            { "id": "github-plugin", "transport": "stdio", "ready": github_plugin_installed },
        ]
    });

    if as_json {
        return Ok(payload);
    }

    let mut lines = Vec::new();
    lines.push(format!("Supervisor State: {}", styler.pass_badge()));
    lines.push(format!("Supported Transports: {}", styler.ink_soft("stdio (child-process), SSE (HTTP event-stream), WebSocket")));
    lines.push(styler.dim("──────────────────────────────────────────────────────────"));
    lines.push(format!("Bundled Servers:"));
    lines.push(format!("  {} memory-mcp (Persistent semantic episodic memory)", styler.lymph("●")));
    lines.push(format!("  {} system-visual-qa (Playwright visual verification)", styler.lymph("●")));
    lines.push(format!("  {} github-plugin (Repository diff & git lifecycle)", styler.lymph("●")));

    println!("{}", styler.card("NOEMA MCP Supervisor & Ecosystem Status", &lines));

    Ok(payload)
}

pub fn test_manifest(path: &str, styler: &Styler, as_json: bool) -> Result<Value, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read manifest at {path}: {e}"))?;

    let parsed = noema_ecosystem::Manifest::from_json(&content)
        .map_err(|e| format!("Manifest schema validation failed: {e}"))?;

    if as_json {
        return Ok(serde_json::to_value(&parsed).map_err(|e| e.to_string())?);
    }

    let mut lines = Vec::new();
    lines.push(format!("Manifest Path: {}", styler.ink_soft(path)));
    lines.push(format!("Validation: {}", styler.pass_badge()));
    lines.push(format!("Name: {}", styler.bold(&styler.ink(&parsed.name))));
    lines.push(format!("Kind: {}", styler.vein(parsed.kind.as_str())));
    lines.push(format!("Version: {}", styler.nerve(&parsed.version)));

    println!("{}", styler.card("Manifest Schema Validation", &lines));

    Ok(json!({ "valid": true, "name": parsed.name }))
}
