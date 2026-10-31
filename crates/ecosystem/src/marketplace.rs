//! Bundled marketplace: memory MCP, system visual-QA skill and the
//! proprietary GitHub plugin. Installs are validated before they touch disk.

use crate::manifest::{Kind, Manifest, ValidationError, parse_skill_md};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum MarketError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Validation(ValidationError),
    NotFound(String),
}

impl fmt::Display for MarketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io: {e}"),
            Self::Json(e) => write!(f, "json: {e}"),
            Self::Validation(e) => write!(f, "invalid package: {e}"),
            Self::NotFound(id) => write!(f, "package '{id}' is not in the marketplace"),
        }
    }
}

impl std::error::Error for MarketError {}

impl From<std::io::Error> for MarketError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for MarketError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<ValidationError> for MarketError {
    fn from(e: ValidationError) -> Self {
        Self::Validation(e)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Installed {
    pub name: String,
    pub version: String,
    pub kind: Kind,
    pub path: String,
}

pub struct BundledPackage {
    pub id: &'static str,
    pub files: &'static [(&'static str, &'static str)],
}

impl BundledPackage {
    pub fn manifest(&self) -> Result<Manifest, MarketError> {
        let (path, source) = self.files[0];
        if path == "SKILL.md" {
            Ok(parse_skill_md(source)?.manifest)
        } else {
            Ok(Manifest::from_json(source)?)
        }
    }
}

/// Packages shipped inside the binary. Free, MIT-compatible, no network at install.
pub fn bundled() -> Vec<BundledPackage> {
    vec![
        BundledPackage {
            id: "memory-mcp",
            files: &[
                ("noema.json", include_str!("../../../marketplace/memory-mcp/noema.json")),
                ("server.mjs", include_str!("../../../marketplace/memory-mcp/server.mjs")),
            ],
        },
        BundledPackage {
            id: "system-visual-qa",
            files: &[
                ("SKILL.md", include_str!("../../../marketplace/system-visual-qa/SKILL.md")),
                (
                    "scripts/visual-qa.mjs",
                    include_str!("../../../marketplace/system-visual-qa/scripts/visual-qa.mjs"),
                ),
            ],
        },
        BundledPackage {
            id: "github-plugin",
            files: &[(
                "noema.json",
                include_str!("../../../marketplace/github-plugin/noema.json"),
            )],
        },
    ]
}

pub fn find(id: &str) -> Option<BundledPackage> {
    bundled().into_iter().find(|p| p.id == id)
}

pub struct Marketplace {
    pub root: PathBuf,
}

impl Marketplace {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn installed(&self) -> Result<Vec<Installed>, MarketError> {
        let path = self.root.join("installed.json");
        if !path.exists() {
            return Ok(Vec::new());
        }
        let source = fs::read_to_string(path)?;
        if source.trim().is_empty() {
            return Ok(Vec::new());
        }
        Ok(serde_json::from_str(&source)?)
    }

    pub fn install(&self, id: &str) -> Result<Installed, MarketError> {
        let package = find(id).ok_or_else(|| MarketError::NotFound(id.into()))?;
        let manifest = package.manifest()?;
        let dest = self.root.join(&manifest.name);
        for (relative, content) in package.files {
            let path = dest.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, content)?;
        }
        self.record(&manifest, &dest)
    }

    /// Install a package directory produced by an author or a git checkout.
    pub fn install_dir(&self, source: &Path) -> Result<Installed, MarketError> {
        let manifest = manifest_in(source)?;
        let dest = self.root.join(&manifest.name);
        copy_tree(source, &dest)?;
        self.record(&manifest, &dest)
    }

    fn record(&self, manifest: &Manifest, dest: &Path) -> Result<Installed, MarketError> {
        fs::create_dir_all(&self.root)?;
        let entry = Installed {
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            kind: manifest.kind,
            path: dest.display().to_string(),
        };
        let mut installed = self.installed()?;
        installed.retain(|item| item.name != entry.name);
        installed.push(entry.clone());
        installed.sort_by(|a, b| a.name.cmp(&b.name));
        let json = serde_json::to_string_pretty(&installed)?;
        fs::write(self.root.join("installed.json"), json)?;
        Ok(entry)
    }
}

pub fn manifest_in(dir: &Path) -> Result<Manifest, MarketError> {
    let noema = dir.join("noema.json");
    if noema.exists() {
        return Ok(Manifest::from_json(&fs::read_to_string(noema)?)?);
    }
    let skill = dir.join("SKILL.md");
    if skill.exists() {
        return Ok(parse_skill_md(&fs::read_to_string(skill)?)?.manifest);
    }
    Err(MarketError::NotFound(dir.display().to_string()))
}

fn copy_tree(source: &Path, dest: &Path) -> Result<(), MarketError> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
