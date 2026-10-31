//! Manifest schemas and validation for MCP servers, skills and plugins.
//!
//! Claude Code compatible: a skill is a `SKILL.md` with YAML frontmatter plus
//! an optional `noema.json`. Every error names the offending field.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Mcp,
    Skill,
    Plugin,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mcp => "mcp",
            Self::Skill => "skill",
            Self::Plugin => "plugin",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Transport {
    Stdio,
    Sse,
    #[serde(rename = "websocket")]
    WebSocket,
}

impl Transport {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stdio => "stdio",
            Self::Sse => "sse",
            Self::WebSocket => "websocket",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpSpec {
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillSpec {
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default, rename = "allowed-tools", alias = "allowed_tools")]
    pub allowed_tools: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginSpec {
    #[serde(default)]
    pub homepage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub kind: Kind,
    pub description: String,
    #[serde(default)]
    pub entry: Option<String>,
    #[serde(default)]
    pub transport: Option<Transport>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub env: Vec<String>,
    #[serde(default)]
    pub mcp: Option<McpSpec>,
    #[serde(default)]
    pub skill: Option<SkillSpec>,
    #[serde(default)]
    pub plugin: Option<PluginSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self { path: path.into(), message: message.into() }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

impl std::error::Error for ValidationError {}

impl Manifest {
    pub fn from_json(src: &str) -> Result<Self, ValidationError> {
        let manifest: Self = serde_json::from_str(src)
            .map_err(|e| ValidationError::new("manifest", e.to_string()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".into())
    }

    /// First field-scoped error, if any.
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.validate_all().into_iter().next().map_or(Ok(()), Err)
    }

    pub fn validate_all(&self) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        let mut push = |path: &str, message: &str| {
            errors.push(ValidationError::new(path, message));
        };

        let name = self.name.trim();
        if name.is_empty() {
            push("name", "is required");
        } else if name.len() > 64
            || !name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        {
            push("name", "must be 1-64 chars of [A-Za-z0-9._-]");
        }

        let version = self.version.trim();
        if version.is_empty() {
            push("version", "is required");
        } else if !is_semver(version) {
            push("version", "must be a stable semver x.y.z");
        }

        if self.description.trim().is_empty() {
            push("description", "is required");
        }

        match self.kind {
            Kind::Mcp => self.validate_mcp(&mut errors),
            Kind::Skill => {}
            Kind::Plugin => {
                if self.entry.as_deref().unwrap_or("").trim().is_empty() {
                    push("entry", "plugin manifest requires an entry point");
                }
            }
        }

        errors
    }

    fn validate_mcp(&self, errors: &mut Vec<ValidationError>) {
        let Some(transport) = self.transport else {
            errors.push(ValidationError::new("transport", "mcp manifest requires stdio, sse or websocket"));
            return;
        };
        let Some(spec) = self.mcp.as_ref() else {
            errors.push(ValidationError::new("mcp", "mcp manifest requires an mcp section"));
            return;
        };
        match transport {
            Transport::Stdio => {
                if spec.command.as_deref().unwrap_or("").trim().is_empty() {
                    errors.push(ValidationError::new("mcp.command", "stdio transport requires a command"));
                }
            }
            Transport::Sse | Transport::WebSocket => {
                let url = spec.url.as_deref().unwrap_or("");
                let ok = match transport {
                    Transport::Sse => url.starts_with("http://") || url.starts_with("https://"),
                    _ => ["ws://", "wss://", "http://", "https://"].iter().any(|p| url.starts_with(p)),
                };
                if !ok {
                    errors.push(ValidationError::new(
                        "mcp.url",
                        format!("{} transport requires an http(s){} url", transport.as_str(), if transport == Transport::Sse { "" } else { "/ws(s)" }),
                    ));
                }
            }
        }
    }
}

fn is_semver(v: &str) -> bool {
    let parts: Vec<&str> = v.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

/// JSON Schema for one extension kind. The Studio and the CLI both publish this.
pub fn json_schema(kind: Kind) -> Value {
    let mut required = vec!["name", "version", "kind", "description"];
    let mut properties = json!({
        "name": {"type": "string", "pattern": "^[A-Za-z0-9._-]{1,64}$"},
        "version": {"type": "string", "pattern": "^[0-9]+\\.[0-9]+\\.[0-9]+$"},
        "kind": {"const": kind.as_str()},
        "description": {"type": "string", "minLength": 1},
        "entry": {"type": "string"},
        "permissions": {"type": "array", "items": {"type": "string"}},
        "env": {"type": "array", "items": {"type": "string"}}
    });

    match kind {
        Kind::Mcp => {
            required.extend(["transport", "mcp"]);
            properties["transport"] = json!({"enum": ["stdio", "sse", "websocket"]});
            properties["mcp"] = json!({
                "type": "object",
                "properties": {
                    "command": {"type": "string"},
                    "args": {"type": "array", "items": {"type": "string"}},
                    "env": {"type": "object", "additionalProperties": {"type": "string"}},
                    "cwd": {"type": "string"},
                    "url": {"type": "string", "format": "uri"},
                    "headers": {"type": "object", "additionalProperties": {"type": "string"}}
                }
            });
        }
        Kind::Skill => {
            properties["skill"] = json!({
                "type": "object",
                "properties": {
                    "license": {"type": "string"},
                    "allowed-tools": {"type": "array", "items": {"type": "string"}}
                }
            });
        }
        Kind::Plugin => {
            required.push("entry");
            properties["plugin"] = json!({"type": "object", "properties": {"homepage": {"type": "string"}}});
        }
    }

    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("https://noema.dev/schemas/{}.json", kind.as_str()),
        "title": format!("NOEMA {} manifest", kind.as_str()),
        "type": "object",
        "additionalProperties": false,
        "required": required,
        "properties": properties
    })
}

/// Parsed `SKILL.md`: Claude Code frontmatter plus the instruction body.
#[derive(Debug, Clone)]
pub struct SkillDoc {
    pub manifest: Manifest,
    pub instructions: String,
}

pub fn parse_skill_md(src: &str) -> Result<SkillDoc, ValidationError> {
    let normalized = src.replace("\r\n", "\n");
    let rest = normalized
        .strip_prefix("---\n")
        .ok_or_else(|| ValidationError::new("SKILL.md", "must start with a --- frontmatter block"))?;
    let (front, body) = rest
        .split_once("\n---")
        .ok_or_else(|| ValidationError::new("SKILL.md", "frontmatter block is not closed"))?;

    let mut name = None;
    let mut description = None;
    let mut version = None;
    let mut license = None;
    let mut tools: Vec<String> = Vec::new();
    let mut list_key: Option<String> = None;

    for (index, line) in front.lines().enumerate() {
        let line = line.trim_end();
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        if let Some(item) = line.trim_start().strip_prefix("- ") {
            let key = list_key.as_deref().unwrap_or("");
            if key == "allowed-tools" || key == "allowed_tools" {
                tools.push(unquote(item));
            }
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            return Err(ValidationError::new(
                format!("SKILL.md:frontmatter[{}]", index + 2),
                "expected key: value",
            ));
        };
        let key = key.trim();
        let value = unquote(value.trim());
        list_key = None;
        match key {
            "name" => name = Some(value),
            "description" => description = Some(value),
            "version" => version = Some(value),
            "license" => license = Some(value),
            "allowed-tools" | "allowed_tools" => {
                if value.starts_with('[') {
                    tools = value
                        .trim_matches(['[', ']'])
                        .split(',')
                        .map(|s| unquote(s.trim()))
                        .filter(|s| !s.is_empty())
                        .collect();
                } else {
                    list_key = Some(key.to_string());
                }
            }
            _ => list_key = None,
        }
    }

    let name = name.ok_or_else(|| ValidationError::new("SKILL.md:frontmatter.name", "is required"))?;
    let description = description
        .ok_or_else(|| ValidationError::new("SKILL.md:frontmatter.description", "is required"))?;

    let manifest = Manifest {
        name,
        version: version.unwrap_or_else(|| "0.1.0".into()),
        kind: Kind::Skill,
        description,
        entry: Some("SKILL.md".into()),
        transport: None,
        permissions: Vec::new(),
        env: Vec::new(),
        mcp: None,
        skill: Some(SkillSpec { license, allowed_tools: tools }),
        plugin: None,
    };
    manifest.validate()?;
    Ok(SkillDoc { manifest, instructions: body.trim_start_matches('\n').to_string() })
}

fn unquote(v: &str) -> String {
    let v = v.trim();
    if v.len() >= 2 && ((v.starts_with('"') && v.ends_with('"')) || (v.starts_with('\'') && v.ends_with('\''))) {
        v[1..v.len() - 1].to_string()
    } else {
        v.to_string()
    }
}
