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

/// A repository-owned path proven to remain below the canonical repository
/// root. `operation_path` deliberately preserves symlink semantics; the
/// canonical path is only a containment witness.
#[derive(Debug, Clone)]
pub struct CheckedRepoPath {
    operation_path: PathBuf,
    witness: PathBuf,
}

impl CheckedRepoPath {
    pub fn operation_path(&self) -> &Path {
        &self.operation_path
    }

    pub fn witness(&self) -> &Path {
        &self.witness
    }

    pub fn into_operation_path(self) -> PathBuf {
        self.operation_path
    }
}

/// Validate the lexical part of a repository-owned path contract on every
/// platform. Backslashes are treated as separators even on Unix so a checked-in
/// Windows traversal cannot become valid merely because validation ran on Linux.
pub fn validate_repo_relative_path(field: &str, value: &str) -> Result<(), ConfigError> {
    use std::path::Component;

    let value = value.trim();
    if value.is_empty() {
        return Err(ConfigError::Invalid(format!("{field} must not be empty")));
    }
    if Path::new(value).is_absolute()
        || value.starts_with(['/', '\\'])
        || value.as_bytes().get(1).is_some_and(|byte| *byte == b':')
    {
        return Err(ConfigError::Invalid(format!(
            "{field} must be a relative path: {value}"
        )));
    }
    for spelling in [value.to_string(), value.replace('\\', "/")] {
        for component in Path::new(&spelling).components() {
            match component {
                Component::ParentDir => {
                    return Err(ConfigError::Invalid(format!(
                        "{field} must not contain `..`: {value}"
                    )));
                }
                Component::Prefix(_) | Component::RootDir => {
                    return Err(ConfigError::Invalid(format!(
                        "{field} must be a relative path: {value}"
                    )));
                }
                Component::Normal(_) | Component::CurDir => {}
            }
        }
    }
    Ok(())
}

/// Resolve the deepest existing ancestor and prove that its canonical target is
/// contained by `repo_root`. Missing tails remain lexical operation paths.
pub fn checked_repo_path(
    repo_root: &Path,
    field: &str,
    relative: &str,
) -> Result<CheckedRepoPath, ConfigError> {
    validate_repo_relative_path(field, relative)?;
    let canonical_root = repo_root.canonicalize().map_err(|error| {
        ConfigError::Invalid(format!(
            "cannot resolve repository root {}: {error}",
            repo_root.display()
        ))
    })?;
    let operation_path = repo_root.join(relative.trim());
    let mut ancestor = operation_path.as_path();
    loop {
        match std::fs::symlink_metadata(ancestor) {
            Ok(_) => break,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                ancestor = ancestor.parent().ok_or_else(|| {
                    ConfigError::Invalid(format!("cannot resolve existing ancestor for {field}"))
                })?;
            }
            Err(error) => {
                return Err(ConfigError::Invalid(format!(
                    "cannot inspect {field} ancestor {}: {error}",
                    ancestor.display()
                )));
            }
        }
    }
    let canonical_ancestor = ancestor.canonicalize().map_err(|error| {
        ConfigError::Invalid(format!(
            "cannot resolve {field} ancestor {}: {error}",
            ancestor.display()
        ))
    })?;
    if !canonical_ancestor.starts_with(&canonical_root) {
        return Err(ConfigError::Invalid(format!(
            "{field} escapes repository root: {relative}"
        )));
    }
    let tail = operation_path
        .strip_prefix(ancestor)
        .map_err(|_| ConfigError::Invalid(format!("cannot resolve repository path for {field}")))?
        .to_path_buf();
    Ok(CheckedRepoPath {
        operation_path,
        witness: canonical_ancestor.join(tail),
    })
}

/// Raw deserialized config.toml.
#[derive(Debug, Deserialize)]
pub struct RawConfig {
    pub schema_version: u32,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub i18n: Option<RawI18nConfig>,
    pub install_dir: String,
    pub specs_dir: String,
    /// Generated spec index path (top-level; not part of a living-spec layer).
    pub index: String,
    /// Always-loaded user-authored rules: project / local.
    pub constitution: RawConstitution,
    /// Foundational living-spec layer (refresh targets; loaded on demand when a
    /// workflow or repository-specific task needs orientation):
    /// product / structure / tech.
    pub context: RawContext,
    /// Decision records and active operational pitfalls.
    pub adr: RawAdr,
    #[serde(default)]
    pub git: RawGit,
    #[serde(default)]
    pub adapter: RawAdapter,
    #[serde(default)]
    pub surfaces: BTreeMap<String, RawSurface>,
}

#[derive(Debug, Deserialize)]
pub struct RawI18nConfig {
    #[serde(default)]
    pub artifact_language: Option<String>,
    #[serde(default)]
    pub conversation_language: Option<String>,
}

/// Always-loaded layer — user-authored project and local rules.
#[derive(Debug, Deserialize)]
pub struct RawConstitution {
    pub project: String,
    pub local: String,
}

/// Foundational layer — refresh targets (`onboard` / `refresh-context`
/// regenerate from code; a conditional orientation map loaded on demand).
#[derive(Debug, Deserialize)]
pub struct RawContext {
    pub product: String,
    pub structure: String,
    pub tech: String,
}

/// ADR layer — fold targets (`open` appends durable decision/pitfall records).
///
/// `decisions` and `pitfalls` are **directory roots**, not monolith files. Each
/// holds one immutable record per decision/pitfall plus a generated, gitignored
/// `INDEX.md`. There is no legacy monolith read path: an absent or empty
/// directory is simply zero records.
#[derive(Debug, Deserialize)]
pub struct RawAdr {
    pub decisions: String,
    pub pitfalls: String,
}

/// List the record file paths (`*.md` except the generated `INDEX.md`) directly
/// under an ADR store directory, sorted by file name. An absent or empty
/// directory yields an empty list — there is no monolith fallback.
pub fn adr_record_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path.extension().is_some_and(|ext| ext == "md")
                && path.file_name().is_some_and(|name| name != "INDEX.md")
        })
        .collect();
    files.sort();
    files
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
    pub i18n: I18nConfig,
    pub i18n_meta: I18nMeta,
    pub install_dir: String,
    pub specs_dir: String,
    pub index: String,
    pub constitution: RawConstitution,
    pub context: RawContext,
    pub adr: RawAdr,
    pub git: RawGit,
    pub adapter: RawAdapter,
    pub surfaces: BTreeMap<String, RawSurface>,
    pub repo_root: PathBuf,
    pub config_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct I18nConfig {
    pub artifact_language: String,
    pub conversation_language: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum I18nValueSource {
    I18n,
    LegacyLanguage,
    Default,
}

#[derive(Debug, Clone)]
pub struct I18nMeta {
    pub has_i18n_table: bool,
    pub has_legacy_language: bool,
    pub missing_artifact_language: bool,
    pub missing_conversation_language: bool,
    pub artifact_source: I18nValueSource,
    pub conversation_source: I18nValueSource,
}

impl Config {
    pub fn checked_path(
        &self,
        field: &str,
        relative: &str,
    ) -> Result<CheckedRepoPath, ConfigError> {
        checked_repo_path(&self.repo_root, field, relative)
    }

    pub fn checked_install_dir(&self) -> Result<CheckedRepoPath, ConfigError> {
        self.checked_path("install_dir", &self.install_dir)
    }

    pub fn checked_engine_dir(&self) -> Result<CheckedRepoPath, ConfigError> {
        self.checked_path(
            "install_dir.engine",
            &format!("{}/engine", self.install_dir),
        )
    }

    pub fn checked_state_dir(&self) -> Result<CheckedRepoPath, ConfigError> {
        self.checked_path("install_dir.state", &format!("{}/state", self.install_dir))
    }

    pub fn checked_specs_dir(&self) -> Result<CheckedRepoPath, ConfigError> {
        self.checked_path("specs_dir", &self.specs_dir)
    }

    pub fn checked_index_path(&self) -> Result<CheckedRepoPath, ConfigError> {
        self.checked_path("index", &self.index)
    }

    pub fn checked_decisions_dir(&self) -> Result<CheckedRepoPath, ConfigError> {
        self.checked_path("adr.decisions", &self.adr.decisions)
    }

    pub fn checked_pitfalls_dir(&self) -> Result<CheckedRepoPath, ConfigError> {
        self.checked_path("adr.pitfalls", &self.adr.pitfalls)
    }

    /// Recheck every configured repository-owned path immediately before a
    /// command performs a group of mutations.
    pub fn validate_repository_paths_now(&self) -> Result<(), ConfigError> {
        for (field, value) in self.repository_path_fields() {
            self.checked_path(field, value)?;
        }
        Ok(())
    }

    fn repository_path_fields(&self) -> [(&'static str, &str); 10] {
        [
            ("install_dir", self.install_dir.as_str()),
            ("specs_dir", self.specs_dir.as_str()),
            ("index", self.index.as_str()),
            ("constitution.project", self.constitution.project.as_str()),
            ("constitution.local", self.constitution.local.as_str()),
            ("context.product", self.context.product.as_str()),
            ("context.structure", self.context.structure.as_str()),
            ("context.tech", self.context.tech.as_str()),
            ("adr.decisions", self.adr.decisions.as_str()),
            ("adr.pitfalls", self.adr.pitfalls.as_str()),
        ]
    }

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

    /// Directory root holding the decision records and their generated INDEX.md.
    pub fn decisions_dir(&self) -> PathBuf {
        self.repo_root.join(&self.adr.decisions)
    }

    /// Directory root holding the pitfall records and their generated INDEX.md.
    pub fn pitfalls_dir(&self) -> PathBuf {
        self.repo_root.join(&self.adr.pitfalls)
    }

    /// Generated, gitignored content index for the decisions store.
    pub fn decisions_index(&self) -> PathBuf {
        self.decisions_dir().join("INDEX.md")
    }

    /// Generated, gitignored content index for the pitfalls store.
    pub fn pitfalls_index(&self) -> PathBuf {
        self.pitfalls_dir().join("INDEX.md")
    }

    /// AC-10: an `[adr]` value that resolves to an existing **file** where a
    /// record directory is expected is a config error, never a silently empty
    /// store. An absent path is fine (it is just zero records).
    pub fn validate_adr_dirs(&self) -> Result<(), ConfigError> {
        for (key, dir) in [
            ("adr.decisions", self.decisions_dir()),
            ("adr.pitfalls", self.pitfalls_dir()),
        ] {
            if dir.exists() && !dir.is_dir() {
                return Err(ConfigError::Invalid(format!(
                    "{key} must resolve to a record directory, but {} is a file",
                    dir.display()
                )));
            }
        }
        Ok(())
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

    /// Language to use for CLI text when conversation language cannot be inferred
    /// from a current user utterance. Agents still treat `auto` dynamically.
    pub fn conversation_output_language(&self) -> &str {
        if self.i18n.conversation_language == "auto" {
            &self.i18n.artifact_language
        } else {
            &self.i18n.conversation_language
        }
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

pub fn is_valid_language_tag(value: &str) -> bool {
    regex::Regex::new(r"^[A-Za-z]{2,3}(-[A-Za-z0-9]{2,8})*$")
        .map(|re| re.is_match(value))
        .unwrap_or(false)
}

pub fn is_valid_artifact_language(value: &str) -> bool {
    value != "auto" && is_valid_language_tag(value)
}

pub fn is_valid_conversation_language(value: &str) -> bool {
    value == "auto" || is_valid_language_tag(value)
}

fn validate_config_paths(raw: &RawConfig) -> Result<(), ConfigError> {
    for (field, value) in [
        ("install_dir", raw.install_dir.as_str()),
        ("specs_dir", raw.specs_dir.as_str()),
        ("index", raw.index.as_str()),
        ("constitution.project", raw.constitution.project.as_str()),
        ("constitution.local", raw.constitution.local.as_str()),
        ("context.product", raw.context.product.as_str()),
        ("context.structure", raw.context.structure.as_str()),
        ("context.tech", raw.context.tech.as_str()),
        ("adr.decisions", raw.adr.decisions.as_str()),
        ("adr.pitfalls", raw.adr.pitfalls.as_str()),
    ] {
        validate_repo_relative_path(field, value)?;
    }
    Ok(())
}

fn validate_adapter_config(adapter: &RawAdapter) -> Result<(), ConfigError> {
    if adapter.tools.as_ref().is_some_and(Vec::is_empty) {
        return Err(ConfigError::Invalid(
            "adapter.tools must contain at least one tool".into(),
        ));
    }
    if adapter
        .tools
        .as_ref()
        .is_some_and(|tools| tools.iter().any(|tool| tool.trim().is_empty()))
        || adapter
            .tool
            .as_ref()
            .is_some_and(|tool| tool.trim().is_empty())
    {
        return Err(ConfigError::Invalid(
            "adapter tools must not be empty strings".into(),
        ));
    }
    const SUPPORTED: [&str; 4] = ["kiro", "agents", "copilot", "claude-code"];
    for tool in adapter.resolved_tools() {
        if !SUPPORTED.contains(&tool.as_str()) {
            return Err(ConfigError::Invalid(format!(
                "unsupported adapter tool: {tool}"
            )));
        }
    }
    Ok(())
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
    validate_config_paths(&raw)?;
    validate_adapter_config(&raw.adapter)?;

    let config_abs = config_path
        .canonicalize()
        .unwrap_or_else(|_| config_path.to_path_buf());
    // config.toml lives at <install_dir>/config.toml, so its parent is the
    // install dir and that parent's parent is the repo root. Do not infer the
    // repo root from the configured install_dir text; that value is data to
    // validate, not an authority for path traversal.
    let install_dir_abs = config_abs.parent().unwrap_or(Path::new("."));
    let repo_root = install_dir_abs
        .parent()
        .unwrap_or(Path::new("/"))
        .to_path_buf();

    let has_i18n_table = raw.i18n.is_some();
    let has_legacy_language = raw.language.is_some();
    let missing_artifact_language = raw
        .i18n
        .as_ref()
        .is_some_and(|i18n| i18n.artifact_language.is_none());
    let missing_conversation_language = raw
        .i18n
        .as_ref()
        .is_some_and(|i18n| i18n.conversation_language.is_none());

    let (artifact_language, artifact_source) = if let Some(language) = raw
        .i18n
        .as_ref()
        .and_then(|i18n| i18n.artifact_language.clone())
    {
        (language, I18nValueSource::I18n)
    } else if let Some(language) = raw.language.clone() {
        (language, I18nValueSource::LegacyLanguage)
    } else {
        ("en".to_string(), I18nValueSource::Default)
    };
    let (conversation_language, conversation_source) = if let Some(language) = raw
        .i18n
        .as_ref()
        .and_then(|i18n| i18n.conversation_language.clone())
    {
        (language, I18nValueSource::I18n)
    } else {
        ("auto".to_string(), I18nValueSource::Default)
    };

    for (field, value) in [
        ("install_dir", raw.install_dir.as_str()),
        ("specs_dir", raw.specs_dir.as_str()),
        ("index", raw.index.as_str()),
        ("constitution.project", raw.constitution.project.as_str()),
        ("constitution.local", raw.constitution.local.as_str()),
        ("context.product", raw.context.product.as_str()),
        ("context.structure", raw.context.structure.as_str()),
        ("context.tech", raw.context.tech.as_str()),
        ("adr.decisions", raw.adr.decisions.as_str()),
        ("adr.pitfalls", raw.adr.pitfalls.as_str()),
    ] {
        checked_repo_path(&repo_root, field, value)?;
    }

    Ok(Config {
        schema_version: raw.schema_version,
        i18n: I18nConfig {
            artifact_language,
            conversation_language,
        },
        i18n_meta: I18nMeta {
            has_i18n_table,
            has_legacy_language,
            missing_artifact_language,
            missing_conversation_language,
            artifact_source,
            conversation_source,
        },
        install_dir: raw.install_dir,
        specs_dir: raw.specs_dir,
        index: raw.index,
        constitution: raw.constitution,
        context: raw.context,
        adr: raw.adr,
        git: raw.git,
        adapter: raw.adapter,
        surfaces: raw.surfaces,
        repo_root,
        config_path: config_abs,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn validates_bcp47_style_language_tags() {
        for value in [
            "en", "ja", "ko", "fr", "de", "zh-Hans", "zh-Hant", "pt-BR", "es-419",
        ] {
            assert!(is_valid_language_tag(value), "{value}");
        }
        for value in ["", " ", "ja JP", "ja\n", "../ja"] {
            assert!(!is_valid_language_tag(value), "{value:?}");
        }
    }

    #[test]
    fn artifact_language_rejects_auto() {
        assert!(is_valid_artifact_language("ja"));
        assert!(!is_valid_artifact_language("auto"));
    }

    #[test]
    fn conversation_language_allows_auto() {
        assert!(is_valid_conversation_language("auto"));
        assert!(is_valid_conversation_language("pt-BR"));
        assert!(!is_valid_conversation_language(""));
        assert!(!is_valid_conversation_language("../ja"));
    }

    #[test]
    fn repository_paths_reject_cross_platform_escapes() {
        for value in [
            "",
            " ",
            "/tmp/out",
            "../out",
            "a/../out",
            r"a\..\out",
            r"C:\out",
            r"\\server\share",
        ] {
            assert!(
                validate_repo_relative_path("test.path", value).is_err(),
                "{value:?}"
            );
        }
        for value in [".mochiflow", "a/b", "./a", r"a\b"] {
            assert!(
                validate_repo_relative_path("test.path", value).is_ok(),
                "{value:?}"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn checked_repository_path_allows_local_symlink_and_rejects_escape() {
        use std::os::unix::fs::symlink;

        let repo = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(repo.path().join("inside")).unwrap();
        symlink(repo.path().join("inside"), repo.path().join("local-link")).unwrap();
        symlink(outside.path(), repo.path().join("outside-link")).unwrap();

        let local = checked_repo_path(repo.path(), "test.path", "local-link/new/file").unwrap();
        assert_eq!(
            local.operation_path(),
            repo.path().join("local-link/new/file")
        );
        assert!(
            local
                .witness()
                .starts_with(repo.path().canonicalize().unwrap())
        );

        let error = checked_repo_path(repo.path(), "test.path", "outside-link/file").unwrap_err();
        assert!(error.to_string().contains("escapes repository root"));
    }

    fn write_min_config(repo: &Path, decisions: &str, pitfalls: &str) -> PathBuf {
        let install = repo.join(".mochiflow");
        std::fs::create_dir_all(&install).unwrap();
        let text = format!(
            "schema_version = 1\ninstall_dir = \".mochiflow\"\nspecs_dir = \".mochiflow/specs\"\nindex = \".mochiflow/INDEX.md\"\n\n[constitution]\nproject = \".mochiflow/constitution.md\"\nlocal = \".mochiflow/constitution.local.md\"\n\n[context]\nproduct = \".mochiflow/context/product.md\"\nstructure = \".mochiflow/context/structure.md\"\ntech = \".mochiflow/context/tech.md\"\n\n[adr]\ndecisions = \"{decisions}\"\npitfalls = \"{pitfalls}\"\n\n[surfaces.app]\ndescription = \"app\"\n\n[surfaces.app.verify]\ndefault = \"echo ok\"\n"
        );
        let path = install.join("config.toml");
        std::fs::write(&path, text).unwrap();
        path
    }

    #[test]
    fn adr_accessors_resolve_directory_roots_and_indexes() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        let path = write_min_config(repo, ".mochiflow/adr/decisions", ".mochiflow/adr/pitfalls");
        let cfg = load_config(&path).unwrap();
        assert!(cfg.decisions_dir().ends_with(".mochiflow/adr/decisions"));
        assert!(cfg.pitfalls_dir().ends_with(".mochiflow/adr/pitfalls"));
        assert!(
            cfg.decisions_index()
                .ends_with(".mochiflow/adr/decisions/INDEX.md")
        );
        assert!(
            cfg.pitfalls_index()
                .ends_with(".mochiflow/adr/pitfalls/INDEX.md")
        );
    }

    #[test]
    fn adr_absent_or_empty_store_yields_zero_records_no_fallback() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        let path = write_min_config(repo, ".mochiflow/adr/decisions", ".mochiflow/adr/pitfalls");
        let cfg = load_config(&path).unwrap();

        // Absent directory: zero records, and validation passes (absent is fine).
        assert!(adr_record_files(&cfg.decisions_dir()).is_empty());
        cfg.validate_adr_dirs().unwrap();

        // Empty directory plus a monolith-named file elsewhere is never read as
        // a fallback: only `*.md` records inside the directory count.
        std::fs::create_dir_all(cfg.decisions_dir()).unwrap();
        std::fs::write(cfg.decisions_dir().join("INDEX.md"), "# index\n").unwrap();
        assert!(adr_record_files(&cfg.decisions_dir()).is_empty());

        std::fs::write(
            cfg.decisions_dir().join("2026-06-28-example.md"),
            "---\nid: 2026-06-28-example\n---\nbody\n",
        )
        .unwrap();
        let records = adr_record_files(&cfg.decisions_dir());
        assert_eq!(records.len(), 1);
        assert!(records[0].ends_with("2026-06-28-example.md"));
    }

    #[test]
    fn adr_value_resolving_to_file_is_a_config_error() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        let path = write_min_config(repo, ".mochiflow/adr/decisions", ".mochiflow/adr/pitfalls");
        let cfg = load_config(&path).unwrap();

        std::fs::create_dir_all(repo.join(".mochiflow/adr")).unwrap();
        std::fs::write(cfg.decisions_dir(), "# legacy monolith\n").unwrap();

        let err = cfg.validate_adr_dirs().unwrap_err();
        assert!(
            matches!(err, ConfigError::Invalid(ref m) if m.contains("adr.decisions")),
            "expected file-where-directory config error, got {err:?}"
        );
    }
}
