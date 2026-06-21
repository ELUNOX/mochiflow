//! Init: deterministic bootstrap of mochiflow into a project.
//!
//! `mochiflow init` lays down the MochiFlow skeleton and a documented,
//! schema-valid config template with TODO sentinels. Project-specific judgement
//! is surfaced as a `Needs AI review` next step when machine detection cannot
//! complete the setup.

use std::path::{Path, PathBuf};

use crate::adapter;
use crate::config::{
    I18nValueSource, is_valid_artifact_language, is_valid_conversation_language, load_config,
};
use crate::doctor;
use crate::upgrade::{install_engine_staged, stage_source_engine};

const DEFAULT_INSTALL: &str = ".mochiflow";

/// Boilerplate body of an unmodified `constitution` stub.
pub const CONSTITUTION_STUB_BODY: &str =
    "User-authored always-loaded rules go here. MochiFlow does not fill this during onboarding.";
/// Boilerplate body of an unmodified `context/` (foundational) living-spec stub.
pub const CONTEXT_STUB_BODY: &str =
    "mochiflow `onboard` / `refresh-context` regenerates this foundational map from code.";
/// Boilerplate body of an unmodified `adr/` stub.
pub const ADR_STUB_BODY: &str =
    "mochiflow `ship` folds durable decisions and active pitfalls here when needed.";

/// Living-spec layer — determines stub wording and lifecycle.
#[derive(Clone, Copy)]
pub enum LivingSpecLayer {
    /// User-authored rules, always-loaded, not filled by onboard.
    Constitution,
    /// Foundational: refresh targets, always-loaded (product / structure / tech).
    Context,
    /// ADR: fold targets (decisions / pitfalls).
    Adr,
}

/// Render the deterministic stub for a living-spec file.
pub fn living_spec_stub(title: &str, layer: LivingSpecLayer) -> String {
    match layer {
        LivingSpecLayer::Constitution => {
            format!("# {title} (constitution — user-authored)\n\n{CONSTITUTION_STUB_BODY}\n")
        }
        LivingSpecLayer::Context => {
            format!("# {title} (context — foundational)\n\n{CONTEXT_STUB_BODY}\n")
        }
        LivingSpecLayer::Adr => format!("# {title} (adr — durable)\n\n{ADR_STUB_BODY}\n"),
    }
}

/// True when `content` is an unmodified generated stub or otherwise carries no
/// substantive authored content (empty / heading-only / boilerplate-only).
/// Shared with `doctor` so the WARN check stays locked to what `init` writes.
pub fn is_living_spec_stub(content: &str) -> bool {
    for line in content.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        if t == CONSTITUTION_STUB_BODY || t == CONTEXT_STUB_BODY || t == ADR_STUB_BODY {
            continue;
        }
        return false; // substantive authored content
    }
    true
}

/// Type alias for the engine extractor function (owned by the binary crate
/// which holds the `include_dir!` data).
pub type EngineExtractFn<'a> = &'a dyn Fn(&Path) -> std::io::Result<()>;

/// TOML comment marker for values that still need human confirmation (AC-02).
/// Kept as a comment so it is visible in the raw config text reviewed after
/// init (it is dropped by serde round-trips — AC-09).
const CONFIRM: &str = "# mochiflow: confirm";

/// Render a TOML string literal.
fn toml_string(value: &str) -> String {
    toml::Value::String(value.to_string()).to_string()
}

/// Render a TOML string array literal.
fn toml_array(items: &[String]) -> String {
    toml::Value::Array(
        items
            .iter()
            .map(|s| toml::Value::String(s.clone()))
            .collect(),
    )
    .to_string()
}

/// Render one TOML dotted-key segment. Bare keys keep the generated config easy
/// to read; everything else is quoted so detected names cannot break tables.
fn toml_key_segment(value: &str) -> String {
    if !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        value.to_string()
    } else {
        toml_string(value)
    }
}

/// Render the `[git]` section, keeping `provider = "none"` (AC-03) and adding a
/// confirm marker that records any detected provider as a fact to verify.
fn render_git_section(git: &crate::detect::DetectedGit) -> String {
    let mut out = String::from("[git]\n");
    if git.has_known_provider() {
        out.push_str(&format!(
            "{CONFIRM} detected remote provider \"{}\"; keep provider=none for manual PR (default) or set provider/pr_driver to automate.\n",
            git.provider
        ));
    }
    out.push_str(
        "provider = \"none\"          # none | github (built-in `gh`). none = manual PR\n",
    );
    if git.branch_confidence.needs_confirm() {
        out.push_str(&format!(
            "{CONFIRM} base branch fell back to a default; confirm it.\n"
        ));
    }
    out.push_str(&format!(
        "base_branch = {}\n",
        toml_string(&git.base_branch)
    ));
    out.push_str(
        "# PR creation: default is manual handoff (mochiflow pr pushes + hands off).\n\
         # To automate, set provider = \"github\", or pr_driver = \"path/to/driver\".\n\
         # pr_driver = \"...\"\n",
    );
    out
}

/// Render the `[surfaces.*]` sections from detected surfaces, attaching a
/// confirm marker to the verify command when its confidence is below High.
fn render_surfaces_section(surfaces: &[crate::detect::DetectedSurface]) -> String {
    let mut out = String::new();
    for s in surfaces {
        let surface_key = toml_key_segment(&s.name);
        out.push_str(&format!("\n[surfaces.{surface_key}]\n"));
        out.push_str(&format!("description = {}\n", toml_string(&s.description)));
        out.push_str(&format!("\n[surfaces.{surface_key}.verify]\n"));
        if s.confidence.needs_confirm() {
            let reason = if s.verify.starts_with("TODO:") {
                "no verify command detected"
            } else {
                "multiple candidate scripts; confirm the verify command"
            };
            out.push_str(&format!("{CONFIRM} {reason}\n"));
        }
        out.push_str(&format!("default = {}\n", toml_string(&s.verify)));
    }
    out
}

/// Render the deterministic config.toml template, injecting machine-detected
/// values and `# mochiflow: confirm` markers for fields that need human
/// confirmation (AC-01 / AC-02). Detection failures fall back to TODO sentinels
/// and confirm markers — the config is never corrupted (AC-10).
fn render_config(
    artifact_language: &str,
    conversation_language: &str,
    adapter_tools: &[String],
    report: &crate::detect::DetectionReport,
) -> String {
    let tools_literal = toml_array(adapter_tools);
    let git_section = render_git_section(&report.git);
    let surfaces_section = render_surfaces_section(&report.surfaces);
    let allow_literal = toml_array(&report.write_scope.allow);
    let deny_literal = toml_array(&report.write_scope.deny);
    let write_confirm = if report.write_scope.confidence.needs_confirm() {
        format!("{CONFIRM} write scope fell back to a default; confirm the AI edit boundary.\n")
    } else {
        String::new()
    };
    format!(
        r##"# MochiFlow project config. `mochiflow init` writes this skeleton with
# machine-detected values. Lines flagged with a MochiFlow confirmation marker need
# review before the setup is considered complete. `mochiflow doctor` reports
# remaining TODOs and confirmation markers.
schema_version = 1

install_dir = ".mochiflow"
specs_dir = ".mochiflow/specs"
index = ".mochiflow/INDEX.md"

[i18n]
artifact_language = {artifact_language_literal}
conversation_language = {conversation_language_literal}

[constitution]
project = ".mochiflow/constitution.md"
local = ".mochiflow/constitution.local.md"

[context]
product = ".mochiflow/context/product.md"
structure = ".mochiflow/context/structure.md"
tech = ".mochiflow/context/tech.md"

[adr]
decisions = ".mochiflow/adr/decisions.md"
pitfalls = ".mochiflow/adr/pitfalls.md"

{git_section}
[adapter]
tools = {tools_literal}

[write]
{write_confirm}allow = {allow_literal}
deny = {deny_literal}
{surfaces_section}"##,
        artifact_language_literal = toml_string(artifact_language),
        conversation_language_literal = toml_string(conversation_language),
    )
}

/// Adapter selection menu: (id, human description). Order is the menu order.
const ADAPTER_MENU: &[(&str, &str)] = &[
    (
        "agents",
        "AGENTS.md (generic, industry standard — the default)",
    ),
    ("codex", "AGENTS.md (OpenAI Codex)"),
    ("kiro", ".kiro/steering + .kiro/agents"),
    ("claude-code", "CLAUDE.md"),
    ("copilot", ".github/copilot-instructions.md"),
];

/// Parse a comma/space separated number selection (e.g. "1,3") against the
/// menu, resolving labels to canonical IDs and de-duplicating while preserving
/// order. Empty / all-invalid input falls back to `["agents"]`. Pure function
/// (no I/O) so the selection logic is unit-testable independent of the TTY.
fn parse_selection(input: &str, options: &[(&str, &str)]) -> Vec<String> {
    let mut labels: Vec<String> = Vec::new();
    for tok in input.split(|c: char| c == ',' || c.is_whitespace()) {
        let tok = tok.trim();
        if tok.is_empty() {
            continue;
        }
        if let Ok(n) = tok.parse::<usize>()
            && n >= 1
            && n <= options.len()
        {
            labels.push(options[n - 1].0.to_string());
        }
    }
    let resolved = adapter::resolve_adapter_labels(&labels);
    if resolved.is_empty() {
        vec!["agents".to_string()]
    } else {
        resolved
    }
}

/// Interactively prompt for adapter selection on a TTY. Reads one line from
/// stdin and delegates parsing to `parse_selection`.
fn prompt_adapters() -> Vec<String> {
    use std::io::Write;
    // Prompts are UI, not output — write them to stderr so a `--json` run keeps
    // stdout a single JSON document even on an interactive TTY.
    eprintln!("\nSelect AI tool adapter(s) to generate (comma-separated, e.g. 1,3):");
    for (i, (id, desc)) in ADAPTER_MENU.iter().enumerate() {
        eprintln!("  {}) {id:<12} {desc}", i + 1);
    }
    eprint!("Selection [Enter = 1) agents]: ");
    let _ = std::io::stderr().flush();
    let mut line = String::new();
    if std::io::stdin().read_line(&mut line).is_err() {
        return vec!["agents".to_string()];
    }
    parse_selection(&line, ADAPTER_MENU)
}

/// Resolve the effective adapter list from CLI input.
///
/// - non-empty `--adapter` (repeatable + comma-separated): resolve + dedupe.
/// - empty + interactive TTY: prompt.
/// - empty + non-interactive: default `["agents"]` (never blocks CI).
fn resolve_adapters(adapter_flags: &[String], yes: bool) -> Vec<String> {
    use std::io::IsTerminal;
    let split: Vec<String> = adapter_flags
        .iter()
        .flat_map(|a| a.split(',').map(|s| s.trim().to_string()))
        .filter(|s| !s.is_empty())
        .collect();
    if !split.is_empty() {
        return adapter::resolve_adapter_labels(&split);
    }
    if !yes && std::io::stdin().is_terminal() {
        prompt_adapters()
    } else {
        vec!["agents".to_string()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedLanguage {
    value: String,
    source: crate::present::InitLanguageSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedI18n {
    artifact_language: ResolvedLanguage,
    conversation_language: ResolvedLanguage,
}

#[cfg(test)]
fn parse_language_selection(input: &str, default: &str) -> String {
    let trimmed = input.trim().to_ascii_lowercase();
    match trimmed.as_str() {
        "" => default.to_string(),
        "1" | "en" | "english" => "en".to_string(),
        "2" | "ja" | "japanese" | "日本語" => "ja".to_string(),
        _ => default.to_string(),
    }
}

#[cfg(test)]
fn is_japanese_locale(value: &str) -> bool {
    value.to_ascii_lowercase().starts_with("ja")
}

#[cfg(test)]
fn locale_language_default_from_values<'a>(
    values: impl IntoIterator<Item = Option<&'a str>>,
) -> ResolvedLanguage {
    let mut saw_locale = false;
    for value in values.into_iter().flatten() {
        if value.is_empty() {
            continue;
        }
        saw_locale = true;
        if is_japanese_locale(value) {
            return ResolvedLanguage {
                value: "ja".to_string(),
                source: crate::present::InitLanguageSource::Locale,
            };
        }
    }

    ResolvedLanguage {
        value: "en".to_string(),
        source: if saw_locale {
            crate::present::InitLanguageSource::Locale
        } else {
            crate::present::InitLanguageSource::Default
        },
    }
}

fn contains_japanese(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(
            c,
            '\u{3040}'..='\u{30ff}' | '\u{3400}'..='\u{9fff}' | '\u{f900}'..='\u{faff}'
        )
    })
}

fn scan_file_for_japanese(path: &Path) -> bool {
    const MAX_BYTES: usize = 64 * 1024;
    let Ok(bytes) = std::fs::read(path) else {
        return false;
    };
    let end = bytes.len().min(MAX_BYTES);
    std::str::from_utf8(&bytes[..end])
        .map(contains_japanese)
        .unwrap_or(false)
}

fn scan_dir_for_japanese(dir: &Path, depth: usize, visited: &mut usize) -> bool {
    const MAX_FILES: usize = 64;
    if depth == 0 || *visited >= MAX_FILES {
        return false;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in entries.flatten() {
        if *visited >= MAX_FILES {
            return false;
        }
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') && name != ".mochiflow" {
            continue;
        }
        if path.is_dir() {
            if scan_dir_for_japanese(&path, depth - 1, visited) {
                return true;
            }
        } else if matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("md" | "mdx" | "txt" | "rst" | "adoc")
        ) {
            *visited += 1;
            if scan_file_for_japanese(&path) {
                return true;
            }
        }
    }
    false
}

fn detect_artifact_language_from_docs(root: &Path) -> Option<String> {
    let file_candidates = [
        "README.md",
        "CONTRIBUTING.md",
        "AGENTS.md",
        "CLAUDE.md",
        ".github/copilot-instructions.md",
    ];
    for rel in file_candidates {
        if scan_file_for_japanese(&root.join(rel)) {
            return Some("ja".to_string());
        }
    }
    let mut visited = 0;
    for rel in ["docs", "specs", ".mochiflow/specs"] {
        if scan_dir_for_japanese(&root.join(rel), 3, &mut visited) {
            return Some("ja".to_string());
        }
    }
    None
}

fn display_language(artifact_language: &str, conversation_language: &str) -> String {
    if conversation_language == "auto" {
        artifact_language.to_string()
    } else {
        conversation_language.to_string()
    }
}

fn present_source_from_i18n(source: I18nValueSource) -> crate::present::InitLanguageSource {
    match source {
        I18nValueSource::I18n => crate::present::InitLanguageSource::ExistingConfig,
        I18nValueSource::LegacyLanguage => crate::present::InitLanguageSource::LegacyConfig,
        I18nValueSource::Default => crate::present::InitLanguageSource::Default,
    }
}

fn resolve_i18n(
    root: &Path,
    config_path: &Path,
    artifact_language_flag: Option<&str>,
    conversation_language_flag: Option<&str>,
) -> Result<ResolvedI18n, String> {
    let existing = if config_path.exists() {
        load_config(config_path).ok()
    } else {
        None
    };

    let artifact_language = if let Some(language) = artifact_language_flag {
        if !is_valid_artifact_language(language) {
            return Err(format!(
                "invalid --artifact-language `{language}`; expected a BCP 47-style tag and not `auto`"
            ));
        }
        ResolvedLanguage {
            value: language.to_string(),
            source: crate::present::InitLanguageSource::Flag,
        }
    } else if let Some(cfg) = existing.as_ref() {
        match cfg.i18n_meta.artifact_source {
            I18nValueSource::I18n | I18nValueSource::LegacyLanguage => ResolvedLanguage {
                value: cfg.i18n.artifact_language.clone(),
                source: present_source_from_i18n(cfg.i18n_meta.artifact_source),
            },
            I18nValueSource::Default => detect_artifact_language_from_docs(root)
                .map(|value| ResolvedLanguage {
                    value,
                    source: crate::present::InitLanguageSource::Docs,
                })
                .unwrap_or_else(|| ResolvedLanguage {
                    value: "en".to_string(),
                    source: crate::present::InitLanguageSource::Default,
                }),
        }
    } else {
        detect_artifact_language_from_docs(root)
            .map(|value| ResolvedLanguage {
                value,
                source: crate::present::InitLanguageSource::Docs,
            })
            .unwrap_or_else(|| ResolvedLanguage {
                value: "en".to_string(),
                source: crate::present::InitLanguageSource::Default,
            })
    };

    let conversation_language = if let Some(language) = conversation_language_flag {
        if !is_valid_conversation_language(language) {
            return Err(format!(
                "invalid --conversation-language `{language}`; expected `auto` or a BCP 47-style tag"
            ));
        }
        ResolvedLanguage {
            value: language.to_string(),
            source: crate::present::InitLanguageSource::Flag,
        }
    } else if let Some(cfg) = existing.as_ref()
        && cfg.i18n_meta.conversation_source == I18nValueSource::I18n
    {
        ResolvedLanguage {
            value: cfg.i18n.conversation_language.clone(),
            source: crate::present::InitLanguageSource::ExistingConfig,
        }
    } else {
        ResolvedLanguage {
            value: "auto".to_string(),
            source: crate::present::InitLanguageSource::Default,
        }
    };

    Ok(ResolvedI18n {
        artifact_language,
        conversation_language,
    })
}

/// Find the engine source directory on the filesystem (fallback).
fn find_engine_source() -> Option<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        let mut p = exe.clone();
        for _ in 0..5 {
            if let Some(parent) = p.parent() {
                p = parent.to_path_buf();
            } else {
                break;
            }
            if p.join("engine").join("VERSION").exists() {
                return Some(p.join("engine"));
            }
        }
    }
    if let Ok(cwd) = std::env::current_dir()
        && cwd.join("engine").join("VERSION").exists()
    {
        return Some(cwd.join("engine"));
    }
    None
}

/// Write `{install_dir}/.gitignore` so the vendored engine copy and runtime
/// state are never tracked, guaranteeing the ignore that delivery-artifact
/// relocation relies on. Returns `Ok(true)` when written, `Ok(false)` when an
/// existing file is kept (no `--force`). Never touches the project's top-level
/// `.gitignore` (init's source-tree-inviolable rule).
fn write_install_gitignore(install_dir: &Path, force: bool) -> std::io::Result<bool> {
    let path = install_dir.join(".gitignore");
    if path.exists() && !force {
        return Ok(false);
    }
    std::fs::write(
        &path,
        "# Managed by mochiflow init. Regenerated, local, or runtime-derived — do not track.\nengine/\nstate/\nconstitution.local.md\n",
    )?;
    Ok(true)
}

/// Run init. Returns the process exit code.
#[allow(clippy::too_many_arguments)]
pub fn run_init(
    target: &str,
    adapter_flags: &[String],
    artifact_language_flag: Option<&str>,
    conversation_language_flag: Option<&str>,
    force: bool,
    dry_run: bool,
    json: bool,
    yes: bool,
    embedded_engine_extract: Option<EngineExtractFn>,
) -> i32 {
    // In --json mode stdout carries a single JSON document; all progress logs go
    // to stderr (AC-05). `log!` routes every status/FAIL line accordingly.
    macro_rules! log {
        ($($arg:tt)*) => {
            if json { eprintln!($($arg)*) } else { println!($($arg)*) }
        };
    }
    let root = PathBuf::from(target)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(target));

    let install_dir = DEFAULT_INSTALL;
    let install_abs = root.join(install_dir);
    let config_path = install_abs.join("config.toml");

    let adapter_tools = resolve_adapters(adapter_flags, yes);
    let resolved_i18n = match resolve_i18n(
        &root,
        &config_path,
        artifact_language_flag,
        conversation_language_flag,
    ) {
        Ok(i18n) => i18n,
        Err(e) => {
            log!("FAIL: {e}");
            return 1;
        }
    };
    let artifact_language = resolved_i18n.artifact_language.value.as_str();
    let conversation_language = resolved_i18n.conversation_language.value.as_str();
    let output_language = display_language(artifact_language, conversation_language);
    let language = output_language.as_str();
    let mut done_items: Vec<String> = Vec::new();
    let mut confirmation_items: Vec<String> = Vec::new();
    let mut blocked_items: Vec<crate::present::InitBlockedItem> = Vec::new();

    // Detection is best-effort and read-only; run it once and reuse the report
    // for config rendering (when writing) and for the presenter output.
    let report = crate::detect::DetectionReport::detect(&root);

    log!("target       : {}", root.display());
    log!("install_dir  : {install_dir}");
    log!("adapters     : {}", adapter_tools.join(", "));
    log!("artifact_language     : {artifact_language}");
    log!("conversation_language : {conversation_language}");
    log!("config exists: {}", config_path.exists());

    if dry_run {
        let dry_run_items = dry_run_done_items(language);
        let mut dry_run_confirmation_items = Vec::new();
        let status = dry_run_status(&root, &config_path, force);
        if status == crate::present::InitStatus::NeedsAiReview {
            dry_run_confirmation_items.extend(dry_run_review_items(language, &config_path));
        }
        if json {
            print!(
                "{}",
                crate::present::render_init_json(crate::present::InitJsonInput {
                    report: &report,
                    target: &root.display().to_string(),
                    dry_run: true,
                    created_updated: &dry_run_items,
                    status,
                    extra_confirmation_items: &dry_run_confirmation_items,
                    blocked_items: &[],
                    artifact_language,
                    conversation_language,
                    display_language: language,
                    artifact_language_source: resolved_i18n.artifact_language.source,
                    conversation_language_source: resolved_i18n.conversation_language.source,
                },)
            );
        } else {
            present_results(
                &report,
                &root.display().to_string(),
                true,
                &dry_run_items,
                &dry_run_confirmation_items,
                &[],
                status,
                artifact_language,
                conversation_language,
                language,
                resolved_i18n.artifact_language.source,
                resolved_i18n.conversation_language.source,
                false,
            );
        }
        return status.exit_code();
    }

    if let Err(e) = std::fs::create_dir_all(&install_abs) {
        log!(
            "FAIL: could not create install dir {}: {e}",
            install_abs.display()
        );
        return 1;
    }

    // Guarantee the vendored engine + runtime state are gitignored (required for
    // the safety guarantee — a write failure fails init).
    match write_install_gitignore(&install_abs, force) {
        Ok(true) => {
            let path = install_abs.join(".gitignore");
            log!("wrote {}", path.display());
            done_items.push(done_item(
                language,
                &format!("wrote {}", path.display()),
                &format!("{} を作成", path.display()),
            ));
        }
        Ok(false) => {
            log!(".gitignore exists; keeping it (use --force to overwrite).");
            done_items.push(done_item(
                language,
                &format!("kept {}", install_abs.join(".gitignore").display()),
                &format!("{} を保持", install_abs.join(".gitignore").display()),
            ));
        }
        Err(e) => {
            log!(
                "FAIL: could not write {}: {e}",
                install_abs.join(".gitignore").display()
            );
            return 1;
        }
    }

    if config_path.exists() && !force {
        log!(
            "config.toml exists; keeping it (use --force to overwrite). Refreshing engine + adapter."
        );
        done_items.push(done_item(
            language,
            &format!("kept {}", config_path.display()),
            &format!("{} を保持", config_path.display()),
        ));
    } else {
        let content = render_config(
            artifact_language,
            conversation_language,
            &adapter_tools,
            &report,
        );
        if let Err(e) = std::fs::write(&config_path, &content) {
            log!("FAIL: could not write {}: {e}", config_path.display());
            return 1;
        }
        log!("wrote {}", config_path.display());
        done_items.push(done_item(
            language,
            &format!("wrote {}", config_path.display()),
            &format!("{} を作成", config_path.display()),
        ));
    }

    match load_config(&config_path) {
        Ok(cfg) => {
            // Install the vendored engine via the same staged path used by
            // `upgrade`, so dirty installed engines require --force.
            let engine_result = if let Some(extract_fn) = embedded_engine_extract {
                install_engine_staged(&cfg, "bundled engine", force, extract_fn)
            } else if let Some(source_engine) = find_engine_source() {
                let label = source_engine.display().to_string();
                install_engine_staged(&cfg, &label, force, |staging| {
                    stage_source_engine(&source_engine, staging)
                })
            } else {
                log!("FAIL: engine source not found and no embedded engine extractor was provided");
                return 1;
            };
            if let Err(e) = engine_result {
                for line in e.report_lines() {
                    log!("{line}");
                }
                return 1;
            }
            done_items.push(done_item(
                language,
                &format!("installed engine at {}", cfg.engine_dir().display()),
                &format!("engine を {} に配置", cfg.engine_dir().display()),
            ));

            // Scaffold directories + living-spec stubs.
            for d in &[
                cfg.specs_dir_path(),
                cfg.specs_dir_path().join("_done"),
                cfg.specs_dir_path().join("_backlog"),
            ] {
                if let Err(e) = std::fs::create_dir_all(d) {
                    log!("FAIL: could not create {}: {e}", d.display());
                    return 1;
                }
            }
            for (label, path, layer) in [
                (
                    "constitution",
                    cfg.constitution_path(),
                    LivingSpecLayer::Constitution,
                ),
                (
                    "constitution.local",
                    cfg.constitution_local_path(),
                    LivingSpecLayer::Constitution,
                ),
                ("product", cfg.product_path(), LivingSpecLayer::Context),
                ("structure", cfg.structure_path(), LivingSpecLayer::Context),
                ("tech", cfg.tech_path(), LivingSpecLayer::Context),
                ("decisions", cfg.decisions_path(), LivingSpecLayer::Adr),
                ("pitfalls", cfg.pitfalls_path(), LivingSpecLayer::Adr),
            ] {
                if !path.exists() {
                    if let Some(parent) = path.parent()
                        && let Err(e) = std::fs::create_dir_all(parent)
                    {
                        log!("FAIL: could not create {}: {e}", parent.display());
                        return 1;
                    }
                    let title = capitalize(label);
                    if let Err(e) = std::fs::write(&path, living_spec_stub(&title, layer)) {
                        log!("FAIL: could not write {}: {e}", path.display());
                        return 1;
                    }
                }
            }
            let adapter_result = adapter::generate(&cfg, false, force);
            for error in &adapter_result.errors {
                log!("FAIL: {error}");
            }
            if !adapter_result.errors.is_empty() {
                return 1;
            }
            if adapter_result.blocked.is_empty() {
                done_items.push(done_item(
                    language,
                    &format!("generated adapters: {}", adapter_tools.join(", ")),
                    &format!("adapter を生成: {}", adapter_tools.join(", ")),
                ));
            } else {
                if !adapter_result.wrote.is_empty() {
                    done_items.push(done_item(
                        language,
                        &format!(
                            "generated adapter files: {}",
                            adapter_result.wrote.join(", ")
                        ),
                        &format!(
                            "adapter ファイルを生成: {}",
                            adapter_result.wrote.join(", ")
                        ),
                    ));
                }
                for blocked in &adapter_result.blocked {
                    let message = if language == "ja" {
                        format!(
                            "{} は既に存在する構造化 adapter ファイルのため上書きしませんでした。{} を確認して手動で統合してください。置き換える場合は --force で再実行できます。",
                            blocked.target, blocked.candidate
                        )
                    } else {
                        format!(
                            "{} already exists as a structured adapter file and was not overwritten. Review {} and merge it manually, or re-run with --force to replace it.",
                            blocked.target, blocked.candidate
                        )
                    };
                    confirmation_items.push(confirmation_item(language, &message, &message));
                    blocked_items.push(crate::present::InitBlockedItem {
                        target: blocked.target.clone(),
                        candidate: blocked.candidate.clone(),
                        message,
                    });
                }
            }
            log!("\n--- doctor ---");
            let doctor_exit = run_init_doctor(&cfg, json, !adapter_result.blocked.is_empty());
            if doctor_exit != 0 {
                return doctor_exit;
            }
            let status = init_status(&cfg, &config_path, !adapter_result.blocked.is_empty());
            if status == crate::present::InitStatus::NeedsAiReview {
                confirmation_items.extend(init_review_items(&cfg, &config_path, language));
            }
            present_results(
                &report,
                &root.display().to_string(),
                false,
                &done_items,
                &confirmation_items,
                &blocked_items,
                status,
                &cfg.i18n.artifact_language,
                &cfg.i18n.conversation_language,
                cfg.conversation_output_language(),
                resolved_i18n.artifact_language.source,
                resolved_i18n.conversation_language.source,
                json,
            );
            status.exit_code()
        }
        Err(e) => {
            log!("FAIL: could not load written config: {e}");
            1
        }
    }
}

fn dry_run_done_items(language: &str) -> Vec<String> {
    vec![
        done_item(
            language,
            "(dry-run) would write or preserve .mochiflow/.gitignore",
            "(dry-run) .mochiflow/.gitignore を作成または保持",
        ),
        done_item(
            language,
            "(dry-run) would write or preserve .mochiflow/config.toml",
            "(dry-run) .mochiflow/config.toml を作成または保持",
        ),
        done_item(
            language,
            "(dry-run) would install engine, scaffold constitution/context/adr/specs, write MANIFEST.json, and generate adapters",
            "(dry-run) engine 配置、constitution/context/adr/specs 作成、MANIFEST.json 作成、adapter 生成を実行予定",
        ),
    ]
}

fn dry_run_status(root: &Path, config_path: &Path, force: bool) -> crate::present::InitStatus {
    if !force
        && config_path.exists()
        && let Ok(cfg) = load_config(config_path)
    {
        let adapter_result = adapter::generate(&cfg, true, false);
        if !adapter_result.drift.is_empty() {
            return crate::present::InitStatus::Blocked;
        }
        return init_status(&cfg, config_path, false);
    }

    if root.join(".mochiflow/context/product.md").exists()
        && root.join(".mochiflow/context/structure.md").exists()
        && root.join(".mochiflow/context/tech.md").exists()
        && std::fs::read_to_string(config_path)
            .map(|content| !content.contains(CONFIRM) && !content.contains("TODO:"))
            .unwrap_or(false)
    {
        crate::present::InitStatus::Ready
    } else {
        crate::present::InitStatus::NeedsAiReview
    }
}

fn dry_run_review_items(language: &str, config_path: &Path) -> Vec<String> {
    if config_path.exists() {
        vec![confirmation_item(
            language,
            "Resolve any remaining confirmation markers, TODO values, and context stubs",
            "残っている確認マーカー、TODO、context stub を解決",
        )]
    } else {
        vec![confirmation_item(
            language,
            "Fill .mochiflow/context/product.md, structure.md, and tech.md from code after init",
            "init 後に .mochiflow/context/product.md、structure.md、tech.md をコードから補完",
        )]
    }
}

fn init_review_items(
    cfg: &crate::config::Config,
    config_path: &Path,
    language: &str,
) -> Vec<String> {
    let mut items = Vec::new();
    let config_needs_review = std::fs::read_to_string(config_path)
        .map(|content| content.contains(CONFIRM) || content.contains("TODO:"))
        .unwrap_or(false);
    if config_needs_review {
        items.push(confirmation_item(
            language,
            "Resolve confirmation markers and TODO values in .mochiflow/config.toml",
            ".mochiflow/config.toml の確認マーカーと TODO を解決",
        ));
    }

    let context_stub = [cfg.product_path(), cfg.structure_path(), cfg.tech_path()]
        .iter()
        .any(|path| {
            std::fs::read_to_string(path)
                .map(|content| is_living_spec_stub(&content))
                .unwrap_or(true)
        });
    if context_stub {
        items.push(confirmation_item(
            language,
            "Fill .mochiflow/context/product.md, structure.md, and tech.md from code",
            ".mochiflow/context/product.md、structure.md、tech.md をコードから補完",
        ));
    }

    items
}

fn run_init_doctor(cfg: &crate::config::Config, json: bool, skip_adapter: bool) -> i32 {
    if !skip_adapter {
        return doctor::run_doctor(cfg, None, json);
    }

    let mut exit = 0;
    for target in ["config", "specs", "engine"] {
        let code = doctor::run_doctor(cfg, Some(target), json);
        if code != 0 {
            exit = code;
        }
    }
    exit
}

fn done_item(language: &str, en: &str, ja: &str) -> String {
    if language == "ja" {
        ja.to_string()
    } else {
        en.to_string()
    }
}

fn confirmation_item(language: &str, en: &str, ja: &str) -> String {
    if language == "ja" {
        ja.to_string()
    } else {
        en.to_string()
    }
}

fn init_status(
    cfg: &crate::config::Config,
    config_path: &Path,
    adapter_blocked: bool,
) -> crate::present::InitStatus {
    if adapter_blocked {
        return crate::present::InitStatus::Blocked;
    }

    let config_needs_review = std::fs::read_to_string(config_path)
        .map(|content| content.contains(CONFIRM) || content.contains("TODO:"))
        .unwrap_or(true);
    let context_stubs = [cfg.product_path(), cfg.structure_path(), cfg.tech_path()]
        .iter()
        .any(|path| {
            std::fs::read_to_string(path)
                .map(|content| is_living_spec_stub(&content))
                .unwrap_or(true)
        });

    if config_needs_review || context_stubs {
        crate::present::InitStatus::NeedsAiReview
    } else {
        crate::present::InitStatus::Ready
    }
}

/// Present init results: in `--json` mode a single JSON document on stdout
/// (logs already went to stderr); otherwise the human detection report
/// (pretty on a colour TTY, plain otherwise) followed by the single next-step
/// trigger line.
#[allow(clippy::too_many_arguments)]
fn present_results(
    report: &crate::detect::DetectionReport,
    target: &str,
    dry_run: bool,
    done_items: &[String],
    confirmation_items: &[String],
    blocked_items: &[crate::present::InitBlockedItem],
    status: crate::present::InitStatus,
    artifact_language: &str,
    conversation_language: &str,
    display_language: &str,
    artifact_language_source: crate::present::InitLanguageSource,
    conversation_language_source: crate::present::InitLanguageSource,
    json: bool,
) {
    use crate::present::{ColorChoice, OutputMode, render_init_json, render_init_summary};
    use std::io::IsTerminal;

    if json {
        print!(
            "{}",
            render_init_json(crate::present::InitJsonInput {
                report,
                target,
                dry_run,
                created_updated: done_items,
                status,
                extra_confirmation_items: confirmation_items,
                blocked_items,
                artifact_language,
                conversation_language,
                display_language,
                artifact_language_source,
                conversation_language_source,
            })
        );
        return;
    }

    let is_tty = std::io::stdout().is_terminal();
    let color = ColorChoice::resolve(is_tty);
    let mode = if is_tty && color == ColorChoice::Always {
        OutputMode::Pretty
    } else {
        OutputMode::Plain
    };
    println!();
    print!(
        "{}",
        render_init_summary(
            report,
            done_items,
            confirmation_items,
            status,
            mode,
            color,
            artifact_language,
            conversation_language,
            display_language,
            artifact_language_source,
            conversation_language_source,
        )
    );
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn tools(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    use crate::detect::{
        Confidence, DetectedGit, DetectedSurface, DetectedWriteScope, DetectionReport,
    };

    /// A high-confidence report (no confirm markers): single clear surface,
    /// branch detected, write scope detected, no remote provider.
    fn clean_report() -> DetectionReport {
        DetectionReport {
            surfaces: vec![DetectedSurface {
                name: "cli".into(),
                description: "Rust crate / workspace".into(),
                verify: "cargo test".into(),
                confidence: Confidence::High,
            }],
            git: DetectedGit {
                provider: "none".into(),
                base_branch: "main".into(),
                branch_confidence: Confidence::High,
            },
            write_scope: DetectedWriteScope {
                allow: vec!["src/**".into(), "README.md".into()],
                deny: vec![".git/**".into()],
                confidence: Confidence::High,
            },
        }
    }

    /// A report with nothing detected (empty project): TODO verify + low/medium
    /// confidence everywhere → confirm markers expected.
    fn empty_report() -> DetectionReport {
        DetectionReport {
            surfaces: vec![DetectedSurface {
                name: "app".into(),
                description: "primary surface".into(),
                verify: "TODO: define test command".into(),
                confidence: Confidence::Low,
            }],
            git: DetectedGit {
                provider: "none".into(),
                base_branch: "main".into(),
                branch_confidence: Confidence::Medium,
            },
            write_scope: DetectedWriteScope {
                allow: vec!["src/**".into(), "README.md".into()],
                deny: vec![".git/**".into()],
                confidence: Confidence::Medium,
            },
        }
    }

    #[test]
    fn render_config_contains_todo_when_no_verify_detected() {
        let out = render_config("en", "auto", &tools(&["agents"]), &empty_report());
        assert!(out.contains("TODO: define test command"), "{out}");
    }

    #[test]
    fn render_config_clean_report_has_no_confirm_markers() {
        let out = render_config("en", "auto", &tools(&["agents"]), &clean_report());
        assert!(!out.contains("# mochiflow: confirm"), "{out}");
    }

    #[test]
    fn render_config_empty_report_adds_confirm_markers() {
        // AC-02: undetected verify + fallback write scope → confirm markers.
        let out = render_config("en", "auto", &tools(&["agents"]), &empty_report());
        assert!(out.contains("# mochiflow: confirm"), "{out}");
    }

    #[test]
    fn render_config_keeps_provider_none_with_confirm_for_known_provider() {
        // AC-03: a detected github remote never auto-switches provider.
        let mut report = clean_report();
        report.git.provider = "github".into();
        let out = render_config("en", "auto", &tools(&["agents"]), &report);
        let parsed: toml::Value = toml::from_str(&out).unwrap();
        assert_eq!(parsed["git"]["provider"].as_str(), Some("none"), "{out}");
        assert!(out.contains("# mochiflow: confirm"), "{out}");
        assert!(out.contains("github"), "{out}");
    }

    #[test]
    fn render_config_threads_parameters() {
        let out = render_config("ja", "auto", &tools(&["agents"]), &clean_report());
        assert!(!out.contains("engine_version"), "{out}");
        assert!(out.contains("[i18n]"), "{out}");
        assert!(out.contains("artifact_language = \"ja\""), "{out}");
        assert!(out.contains("conversation_language = \"auto\""), "{out}");
        assert!(!out.contains("\nlanguage = "), "{out}");
        assert!(out.contains("tools = [\"agents\"]"), "{out}");
    }

    #[test]
    fn render_config_emits_multiple_tools() {
        let out = render_config("en", "auto", &tools(&["agents", "kiro"]), &clean_report());
        assert!(out.contains("tools = [\"agents\", \"kiro\"]"), "{out}");
    }

    #[test]
    fn render_config_is_schema_valid_toml() {
        // Both clean and confirm-marked output must still parse as valid TOML.
        for report in [clean_report(), empty_report()] {
            let out = render_config("en", "auto", &tools(&["agents"]), &report);
            let parsed: toml::Value = toml::from_str(&out).unwrap();
            assert_eq!(parsed["schema_version"].as_integer(), Some(1));
            assert_eq!(parsed["git"]["provider"].as_str(), Some("none"));
            let arr = parsed["adapter"]["tools"].as_array().unwrap();
            assert_eq!(arr[0].as_str(), Some("agents"));
            // a verify default is present on the (sole) surface
            let surfaces = parsed["surfaces"].as_table().unwrap();
            assert_eq!(surfaces.len(), 1);
        }
    }

    #[test]
    fn render_config_escapes_toml_values() {
        let language = "ja\"bad\\lang\nnext";
        let adapter = "agent\"s\\next\nline";
        let surface_name = "front.end app]";
        let description = "A \"quoted\" app\nwith \\ path";
        let verify = "echo \"ok\"\nprintf '\\n'";
        let allow = "frontend app/\"module\n/**";
        let deny = "**/secret\\file\"";
        let base_branch = "main\"bad\\branch\nnext";
        let report = DetectionReport {
            surfaces: vec![DetectedSurface {
                name: surface_name.into(),
                description: description.into(),
                verify: verify.into(),
                confidence: Confidence::High,
            }],
            git: DetectedGit {
                provider: "none".into(),
                base_branch: base_branch.into(),
                branch_confidence: Confidence::High,
            },
            write_scope: DetectedWriteScope {
                allow: vec![allow.into()],
                deny: vec![deny.into()],
                confidence: Confidence::High,
            },
        };

        let out = render_config(language, "auto", &tools(&["agents", adapter]), &report);
        let parsed: toml::Value = toml::from_str(&out).unwrap();

        assert_eq!(
            parsed["i18n"]["artifact_language"].as_str(),
            Some(language),
            "{out}"
        );
        assert_eq!(
            parsed["i18n"]["conversation_language"].as_str(),
            Some("auto"),
            "{out}"
        );
        assert_eq!(parsed["git"]["base_branch"].as_str(), Some(base_branch));
        let tools = parsed["adapter"]["tools"].as_array().unwrap();
        assert_eq!(tools[1].as_str(), Some(adapter), "{out}");
        let allow_items = parsed["write"]["allow"].as_array().unwrap();
        assert_eq!(allow_items[0].as_str(), Some(allow), "{out}");
        let deny_items = parsed["write"]["deny"].as_array().unwrap();
        assert_eq!(deny_items[0].as_str(), Some(deny), "{out}");
        let surface = &parsed["surfaces"][surface_name];
        assert_eq!(surface["description"].as_str(), Some(description), "{out}");
        assert_eq!(surface["verify"]["default"].as_str(), Some(verify), "{out}");
    }

    #[test]
    fn render_config_emits_guidance_layer_tables() {
        let out = render_config("en", "auto", &tools(&["agents"]), &clean_report());
        assert!(out.contains("[constitution]"), "{out}");
        assert!(out.contains("[context]"), "{out}");
        assert!(out.contains("[adr]"), "{out}");
        assert!(
            out.contains("product = \".mochiflow/context/product.md\""),
            "{out}"
        );
        assert!(
            out.contains("tech = \".mochiflow/context/tech.md\""),
            "{out}"
        );
        assert!(!out.contains("[paths]"), "{out}");
        let parsed: toml::Value = toml::from_str(&out).unwrap();
        assert_eq!(parsed["index"].as_str(), Some(".mochiflow/INDEX.md"));
        assert!(parsed["constitution"]["project"].is_str());
        assert!(parsed["context"]["structure"].is_str());
        assert!(parsed["adr"]["decisions"].is_str());
    }

    #[test]
    fn generated_stubs_are_detected_as_stubs() {
        // init's stub output must be classified as a stub by doctor's check
        // (locks the init↔doctor shared contract).
        assert!(is_living_spec_stub(&living_spec_stub(
            "Constitution",
            LivingSpecLayer::Constitution
        )));
        assert!(is_living_spec_stub(&living_spec_stub(
            "Product",
            LivingSpecLayer::Context
        )));
        assert!(is_living_spec_stub(&living_spec_stub(
            "Decisions",
            LivingSpecLayer::Adr
        )));
    }

    #[test]
    fn empty_and_heading_only_are_stubs() {
        assert!(is_living_spec_stub(""));
        assert!(is_living_spec_stub("# Product\n\n"));
    }

    #[test]
    fn authored_content_is_not_a_stub() {
        assert!(!is_living_spec_stub(
            "# Product\n\nThis project is a CLI for spec-driven flows.\n"
        ));
    }

    #[test]
    fn parse_selection_single() {
        assert_eq!(parse_selection("1", ADAPTER_MENU), tools(&["agents"]));
    }

    #[test]
    fn parse_selection_multiple_resolves_and_dedupes() {
        // 1=agents, 2=codex(->agents), 3=kiro => agents+kiro (codex collapses)
        assert_eq!(
            parse_selection("1,2,3", ADAPTER_MENU),
            tools(&["agents", "kiro"])
        );
    }

    #[test]
    fn parse_selection_preserves_order() {
        // 3=kiro, 4=claude-code
        assert_eq!(
            parse_selection("3 4", ADAPTER_MENU),
            tools(&["kiro", "claude-code"])
        );
    }

    #[test]
    fn parse_selection_empty_defaults_to_agents() {
        assert_eq!(parse_selection("", ADAPTER_MENU), tools(&["agents"]));
        assert_eq!(parse_selection("   ", ADAPTER_MENU), tools(&["agents"]));
    }

    #[test]
    fn parse_selection_ignores_invalid_tokens() {
        // out-of-range and non-numeric ignored; nothing valid => agents
        assert_eq!(parse_selection("9,foo", ADAPTER_MENU), tools(&["agents"]));
        // mixed: 3=kiro kept, junk dropped
        assert_eq!(parse_selection("x,3,99", ADAPTER_MENU), tools(&["kiro"]));
    }

    #[test]
    fn parse_language_selection_accepts_numbers_labels_and_defaults() {
        assert_eq!(parse_language_selection("1", "ja"), "en");
        assert_eq!(parse_language_selection("2", "en"), "ja");
        assert_eq!(parse_language_selection("en", "ja"), "en");
        assert_eq!(parse_language_selection("ja", "en"), "ja");
        assert_eq!(parse_language_selection("", "ja"), "ja");
        assert_eq!(parse_language_selection("nope", "en"), "en");
    }

    #[test]
    fn locale_language_default_detects_japanese_or_falls_back() {
        assert_eq!(
            locale_language_default_from_values([Some("ja_JP.UTF-8"), None, None]),
            ResolvedLanguage {
                value: "ja".into(),
                source: crate::present::InitLanguageSource::Locale,
            }
        );
        assert_eq!(
            locale_language_default_from_values([Some("C"), Some("en_US.UTF-8"), None]),
            ResolvedLanguage {
                value: "en".into(),
                source: crate::present::InitLanguageSource::Locale,
            }
        );
        assert_eq!(
            locale_language_default_from_values([None, None, None]),
            ResolvedLanguage {
                value: "en".into(),
                source: crate::present::InitLanguageSource::Default,
            }
        );
    }

    #[test]
    fn resolve_adapters_flag_comma_and_repeat() {
        assert_eq!(
            resolve_adapters(&tools(&["codex,agents"]), false),
            tools(&["agents"])
        );
        assert_eq!(
            resolve_adapters(&tools(&["kiro", "claude-code"]), false),
            tools(&["kiro", "claude-code"])
        );
    }
}
