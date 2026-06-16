//! Config loading and types for config.toml.
//!
//! All project paths in config.toml are relative to the repo root.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config.toml not found: {0}")]
    NotFound(PathBuf),
    #[error("config.toml parse error: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("config.toml invalid: {0}")]
    Invalid(String),
}

/// Raw deserialized config.toml.
#[derive(Debug, Deserialize)]
pub struct RawConfig {
    pub schema_version: u32,
    #[serde(default = "default_language")]
    pub language: String,
    pub install_dir: String,
    pub specs_dir: String,
    /// Generated spec index path (top-level; not part of a living-spec layer).
    pub index: String,
    /// Always-loaded user-authored rules: project / local.
    pub constitution: RawConstitution,
    /// Foundational living-spec layer (refresh targets, always-loaded):
    /// product / structure / tech.
    pub context: RawContext,
    /// Decision records and active operational pitfalls.
    pub adr: RawAdr,
    #[serde(default)]
    pub git: RawGit,
    #[serde(default)]
    pub adapter: RawAdapter,
    #[serde(default)]
    pub write: RawWrite,
    #[serde(default)]
    pub surfaces: BTreeMap<String, RawSurface>,
}

fn default_language() -> String {
    "en".to_string()
}

/// Always-loaded layer — user-authored project and local rules.
#[derive(Debug, Deserialize)]
pub struct RawConstitution {
    pub project: String,
    pub local: String,
}

/// Foundational layer — refresh targets (`onboard` / `refresh-context`
/// regenerate from code; always-loaded orientation map).
#[derive(Debug, Deserialize)]
pub struct RawContext {
    pub product: String,
    pub structure: String,
    pub tech: String,
}

/// ADR layer — fold targets (`ship` appends durable decisions and pitfalls).
#[derive(Debug, Deserialize)]
pub struct RawAdr {
    pub decisions: String,
    pub pitfalls: String,
}

#[derive(Debug, Deserialize)]
pub struct RawGit {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default = "default_branch")]
    pub base_branch: String,
    #[serde(default)]
    pub pr_command: String,
    /// Optional custom PR driver (path to an executable implementing the
    /// pr-request contract). Takes precedence over provider built-ins and
    /// legacy pr_command.
    #[serde(default)]
    pub pr_driver: Option<String>,
}

impl Default for RawGit {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            base_branch: default_branch(),
            pr_command: String::new(),
            pr_driver: None,
        }
    }
}

fn default_provider() -> String {
    "none".to_string()
}
fn default_branch() -> String {
    "main".to_string()
}

#[derive(Debug, Default, Deserialize)]
pub struct RawAdapter {
    /// Legacy single tool (backward compat).
    #[serde(default)]
    pub tool: Option<String>,
    /// Preferred: list of tools.
    #[serde(default)]
    pub tools: Option<Vec<String>>,
}

impl RawAdapter {
    /// Resolve to a list of tools. `tools` takes precedence over `tool`.
    pub fn resolved_tools(&self) -> Vec<String> {
        if let Some(ref tools) = self.tools {
            tools.clone()
        } else if let Some(ref tool) = self.tool {
            vec![tool.clone()]
        } else {
            vec![default_tool()]
        }
    }
}

fn default_tool() -> String {
    "agents".to_string()
}

#[derive(Debug, Default, Deserialize)]
pub struct RawWrite {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RawSurface {
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub verify: BTreeMap<String, String>,
}

/// Resolved configuration with absolute paths.
#[derive(Debug)]
pub struct Config {
    pub schema_version: u32,
    pub language: String,
    pub install_dir: String,
    pub specs_dir: String,
    pub index: String,
    pub constitution: RawConstitution,
    pub context: RawContext,
    pub adr: RawAdr,
    pub git: RawGit,
    pub adapter: RawAdapter,
    pub write: RawWrite,
    pub surfaces: BTreeMap<String, RawSurface>,
    pub repo_root: PathBuf,
    pub config_path: PathBuf,
}

impl Config {
    pub fn install_dir_path(&self) -> PathBuf {
        self.repo_root.join(&self.install_dir)
    }

    pub fn engine_dir(&self) -> PathBuf {
        self.install_dir_path().join("engine")
    }

    pub fn state_dir(&self) -> PathBuf {
        self.install_dir_path().join("state")
    }

    pub fn specs_dir_path(&self) -> PathBuf {
        self.repo_root.join(&self.specs_dir)
    }

    pub fn constitution_path(&self) -> PathBuf {
        self.repo_root.join(&self.constitution.project)
    }

    pub fn constitution_local_path(&self) -> PathBuf {
        self.repo_root.join(&self.constitution.local)
    }

    pub fn pitfalls_path(&self) -> PathBuf {
        self.repo_root.join(&self.adr.pitfalls)
    }

    pub fn decisions_path(&self) -> PathBuf {
        self.repo_root.join(&self.adr.decisions)
    }

    pub fn product_path(&self) -> PathBuf {
        self.repo_root.join(&self.context.product)
    }

    pub fn structure_path(&self) -> PathBuf {
        self.repo_root.join(&self.context.structure)
    }

    pub fn tech_path(&self) -> PathBuf {
        self.repo_root.join(&self.context.tech)
    }

    pub fn index_path(&self) -> PathBuf {
        self.repo_root.join(&self.index)
    }

    pub fn surface_names(&self) -> Vec<String> {
        self.surfaces.keys().cloned().collect()
    }

    /// All configured adapter tools (resolved from `tools` or `tool`).
    pub fn adapter_tools(&self) -> Vec<String> {
        self.adapter.resolved_tools()
    }

    /// Primary adapter tool (first in the list). Used for markers/subs.
    pub fn primary_tool(&self) -> &str {
        self.adapter
            .tools
            .as_ref()
            .and_then(|t| t.first())
            .or(self.adapter.tool.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("agents")
    }

    pub fn verify_command(
        &self,
        surface: &str,
        profile: &str,
        target: Option<&str>,
    ) -> Result<String, ConfigError> {
        let s = self
            .surfaces
            .get(surface)
            .ok_or_else(|| ConfigError::Invalid(format!("unknown surface: {surface}")))?;
        let cmd = s
            .verify
            .get(profile)
            .or_else(|| s.verify.get("default"))
            .ok_or_else(|| {
                ConfigError::Invalid(format!(
                    "surface {surface} has no verify profile: {profile}"
                ))
            })?;
        let cmd = if let Some(t) = target {
            cmd.replace("{target}", t)
        } else {
            cmd.clone()
        };
        Ok(cmd)
    }
}

/// Load config from a specific path.
pub fn load_config(config_path: &Path) -> Result<Config, ConfigError> {
    if !config_path.exists() {
        return Err(ConfigError::NotFound(config_path.to_path_buf()));
    }
    let text = std::fs::read_to_string(config_path)
        .map_err(|e| ConfigError::Invalid(format!("cannot read: {e}")))?;
    let raw: RawConfig = toml::from_str(&text)?;
    resolve(raw, config_path)
}

/// Derive repo_root from config_path and install_dir (matches Python logic).
fn resolve(raw: RawConfig, config_path: &Path) -> Result<Config, ConfigError> {
    let config_abs = config_path
        .canonicalize()
        .unwrap_or_else(|_| config_path.to_path_buf());
    // config.toml lives at <install_dir>/config.toml, so its parent IS the install dir.
    let install_dir_abs = config_abs.parent().unwrap_or(Path::new("."));
    // repo_root = install_dir_abs walked up by the number of path components in
    // install_dir (mirrors Python: `install_dir_abs.parents[len(parts) - 1]`,
    // i.e. `len(parts)` calls to `.parent()`). For install_dir = ".mochiflow"
    // that is one level up = the repo root.
    let component_count = Path::new(&raw.install_dir).components().count();
    let mut repo_root = install_dir_abs.to_path_buf();
    for _ in 0..component_count {
        repo_root = repo_root.parent().unwrap_or(Path::new("/")).to_path_buf();
    }

    Ok(Config {
        schema_version: raw.schema_version,
        language: raw.language,
        install_dir: raw.install_dir,
        specs_dir: raw.specs_dir,
        index: raw.index,
        constitution: raw.constitution,
        context: raw.context,
        adr: raw.adr,
        git: raw.git,
        adapter: raw.adapter,
        write: raw.write,
        surfaces: raw.surfaces,
        repo_root,
        config_path: config_abs,
    })
}
