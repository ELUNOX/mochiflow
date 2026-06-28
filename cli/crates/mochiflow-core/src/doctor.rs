//! Doctor: quality gate over config / specs / adapter / engine.

use std::path::Path;

use crate::config::Config;
use crate::index;
use crate::lint;
use crate::manifest::{load_manifest, read_engine_version};

pub struct DoctorIssue {
    pub severity: String,
    pub message: String,
}

const TERMINAL_CLI_COMMAND_REFERENCES: &[&str] = &[
    "accept",
    "adapter",
    "adr",
    "backlog",
    "completions",
    "config",
    "detach",
    "doctor",
    "freeze",
    "guide",
    "index",
    "init",
    "join",
    "lint",
    "pr",
    "ready",
    "status",
    "upgrade",
];

const WORKFLOW_COMMAND_REFERENCES: &[&str] = &[
    "build",
    "close",
    "discuss",
    "onboard",
    "open",
    "patch",
    "plan",
    "refresh-context",
    "review",
    "update",
];

struct TargetReport {
    name: String,
    issues: Vec<DoctorIssue>,
    fails: usize,
    warns: usize,
}

pub fn terminal_cli_command_references() -> &'static [&'static str] {
    TERMINAL_CLI_COMMAND_REFERENCES
}

pub fn workflow_command_references() -> &'static [&'static str] {
    WORKFLOW_COMMAND_REFERENCES
}

pub fn validate_config(cfg: &Config) -> Vec<DoctorIssue> {
    let mut issues = Vec::new();
    if cfg.schema_version != 1 {
        issues.push(DoctorIssue {
            severity: "FAIL".into(),
            message: "schema_version must be 1".into(),
        });
    }
    if cfg.i18n_meta.has_legacy_language {
        issues.push(DoctorIssue {
            severity: "WARN".into(),
            message: "top-level `language` is deprecated; use `[i18n].artifact_language` instead."
                .into(),
        });
    }
    if !cfg.i18n_meta.has_i18n_table {
        issues.push(DoctorIssue {
            severity: "WARN".into(),
            message: "missing `[i18n]`; using artifact_language=\"en\" and conversation_language=\"auto\" defaults unless legacy `language` is present.".into(),
        });
    }
    if cfg.i18n_meta.missing_artifact_language {
        issues.push(DoctorIssue {
            severity: "FAIL".into(),
            message: "`[i18n].artifact_language` is required".into(),
        });
    }
    if cfg.i18n_meta.missing_conversation_language {
        issues.push(DoctorIssue {
            severity: "FAIL".into(),
            message: "`[i18n].conversation_language` is required".into(),
        });
    }
    if !crate::config::is_valid_artifact_language(&cfg.i18n.artifact_language) {
        issues.push(DoctorIssue {
            severity: "FAIL".into(),
            message: format!(
                "`[i18n].artifact_language` must be a BCP 47-style language tag and must not be `auto`: {}",
                cfg.i18n.artifact_language
            ),
        });
    }
    if !crate::config::is_valid_conversation_language(&cfg.i18n.conversation_language) {
        issues.push(DoctorIssue {
            severity: "FAIL".into(),
            message: format!(
                "`[i18n].conversation_language` must be `auto` or a BCP 47-style language tag: {}",
                cfg.i18n.conversation_language
            ),
        });
    }
    if cfg.specs_dir.is_empty() {
        issues.push(DoctorIssue {
            severity: "FAIL".into(),
            message: "specs_dir is required".into(),
        });
    }
    if !cfg.specs_dir_path().exists() {
        issues.push(DoctorIssue {
            severity: "WARN".into(),
            message: format!(
                "specs_dir does not exist: {}",
                cfg.specs_dir_path().display()
            ),
        });
    }
    if is_mochiflow_source_repo(&cfg.repo_root) {
        issues.push(DoctorIssue {
            severity: "WARN".into(),
            message: "source repo detected: `doctor` checks project health; run `mochiflow freeze --check` to verify source derived files."
                .into(),
        });
    }
    // Context files must be filled from code during onboarding / refresh.
    // Constitution and ADR files may remain generated stubs: they are
    // user-authored or ship-grown, not onboarding completion gates.
    let context = [
        ("context.product", cfg.product_path()),
        ("context.structure", cfg.structure_path()),
        ("context.tech", cfg.tech_path()),
    ];
    for (label, path) in &context {
        match std::fs::read_to_string(path) {
            Err(_) => issues.push(DoctorIssue {
                severity: "WARN".into(),
                message: format!("{label} does not exist: {}", path.display()),
            }),
            Ok(content) if crate::init::is_living_spec_stub(&content) => {
                issues.push(DoctorIssue {
                    severity: "WARN".into(),
                    message: format!(
                        "{label} is an unfilled stub — ask your AI agent to refresh project context from code using the refresh-context workflow: {}",
                        path.display()
                    ),
                });
            }
            Ok(content) => {
                for command in stale_context_command_references(&content) {
                    issues.push(DoctorIssue {
                        severity: "WARN".into(),
                        message: format!(
                            "{label} references unknown MochiFlow command `mochiflow {command}`; refresh project context from code using the refresh-context workflow: {}",
                            path.display()
                        ),
                    });
                }
            }
        }
    }
    if cfg.surfaces.is_empty() {
        issues.push(DoctorIssue {
            severity: "FAIL".into(),
            message: "no surfaces defined".into(),
        });
    }
    for (name, surface) in &cfg.surfaces {
        if !surface.verify.contains_key("default") {
            issues.push(DoctorIssue {
                severity: "FAIL".into(),
                message: format!("surface {name} missing verify.default"),
            });
        }
        for (profile, cmd) in &surface.verify {
            if cmd.starts_with("TODO") {
                issues.push(DoctorIssue {
                    severity: "WARN".into(),
                    message: format!("surface {name}.{profile} is a TODO placeholder"),
                });
            }
        }
    }
    let valid_tools = ["agents", "kiro", "copilot", "claude-code"];
    let adapter_tools = cfg.adapter_tools();
    if adapter_tools.is_empty() {
        issues.push(DoctorIssue {
            severity: "FAIL".into(),
            message: "adapter.tools must contain at least one tool".into(),
        });
    }
    for tool in &adapter_tools {
        if !valid_tools.contains(&tool.as_str()) {
            issues.push(DoctorIssue {
                severity: "FAIL".into(),
                message: format!("unknown adapter tool: {tool}"),
            });
        }
    }
    issues.extend(check_state_ignored(cfg));
    if index::is_index_stale(cfg) {
        issues.push(DoctorIssue {
            severity: "WARN".into(),
            message: "INDEX.md is stale; run `mochiflow index`".into(),
        });
    }
    // Note: an unconfigured PR backend (pr_command TODO / provider none) is a
    // valid default — `mochiflow pr` falls back to manual handoff — so it is NOT
    // reported as incomplete setup. Only verify TODOs gate readiness.
    // Aggregate summary for TODO placeholders
    let todo_count = issues
        .iter()
        .filter(|i| i.severity == "WARN" && i.message.contains("TODO placeholder"))
        .count();
    if todo_count > 0 {
        issues.push(DoctorIssue {
            severity: "WARN".into(),
            message: format!(
                "setup needs review: {todo_count} TODO value(s) remain — edit config or use the init prompt with your AI agent"
            ),
        });
    }
    issues
}

pub fn is_mochiflow_source_repo(root: &Path) -> bool {
    root.join("cli/Cargo.toml").is_file() && root.join("engine/VERSION").is_file()
}

fn is_known_command_reference(command: &str) -> bool {
    TERMINAL_CLI_COMMAND_REFERENCES.contains(&command)
        || WORKFLOW_COMMAND_REFERENCES.contains(&command)
}

fn stale_context_command_references(content: &str) -> Vec<String> {
    let Ok(re) = regex::Regex::new(r"\bmochiflow\s+([A-Za-z][A-Za-z0-9_-]*)") else {
        return Vec::new();
    };
    let mut stale = std::collections::BTreeSet::new();
    for captures in re.captures_iter(content) {
        let Some(command) = captures.get(1).map(|m| m.as_str()) else {
            continue;
        };
        if !is_known_command_reference(command) {
            stale.insert(command.to_string());
        }
    }
    stale.into_iter().collect()
}

/// FAIL when `{install_dir}/state/` is not gitignored. PR/QA delivery artifacts
/// are written under state/, and `mochiflow pr` requires a clean working tree.
/// `init` writes `{install_dir}/.gitignore` for fresh projects; this catches
/// drift or older projects. Skipped when the project is not a git repo.
pub fn check_state_ignored(cfg: &Config) -> Vec<DoctorIssue> {
    use std::process::Command;
    let root = &cfg.repo_root;
    let is_git = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(root)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !is_git {
        return Vec::new();
    }
    let state = cfg.state_dir();
    let state_probe = state.join(".gitignore-probe");
    let ignored = Command::new("git")
        .arg("check-ignore")
        .arg("-q")
        .arg(&state_probe)
        .current_dir(root)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if ignored {
        Vec::new()
    } else {
        vec![DoctorIssue {
            severity: "FAIL".into(),
            message: format!(
                "{} is not gitignored; add `state/` to {}/.gitignore (init does this for new projects)",
                state.display(),
                cfg.install_dir_path().display()
            ),
        }]
    }
}

pub fn check_engine(cfg: &Config) -> Vec<DoctorIssue> {
    check_engine_with_bundled(cfg, None)
}

pub fn check_engine_with_bundled(cfg: &Config, bundled_version: Option<&str>) -> Vec<DoctorIssue> {
    let mut issues = Vec::new();
    let engine_dir = cfg.engine_dir();
    if !engine_dir.exists() {
        issues.push(DoctorIssue {
            severity: "FAIL".into(),
            message: "engine directory not found".into(),
        });
        return issues;
    }

    let installed_version = match read_engine_version(&engine_dir) {
        Ok(version) => Some(version),
        Err(e) => {
            issues.push(DoctorIssue {
                severity: "FAIL".into(),
                message: format!("engine VERSION error: {e}"),
            });
            None
        }
    };

    // Build current file hashes
    let current = build_manifest_files(&engine_dir);

    // Load existing MANIFEST.json
    let manifest_path = engine_dir.join("MANIFEST.json");
    if !manifest_path.exists() {
        issues.push(DoctorIssue {
            severity: "WARN".into(),
            message: "MANIFEST.json missing (run upgrade/init to generate)".into(),
        });
        return issues;
    }

    match load_manifest(&engine_dir) {
        Ok(manifest) => {
            if let Some(version) = &installed_version
                && manifest.version != *version
            {
                issues.push(DoctorIssue {
                    severity: "FAIL".into(),
                    message: format!(
                        "engine MANIFEST version mismatch: manifest={} VERSION={version}",
                        manifest.version
                    ),
                });
            }
            // Compare
            let old = &manifest.files;
            let mut all_keys: Vec<_> = old.keys().chain(current.keys()).cloned().collect();
            all_keys.sort();
            all_keys.dedup();
            for f in all_keys {
                if old.get(&f) != current.get(&f) {
                    issues.push(DoctorIssue {
                        severity: "FAIL".into(),
                        message: format!("engine MANIFEST drift: {f}"),
                    });
                }
            }
        }
        Err(e) => {
            issues.push(DoctorIssue {
                severity: "FAIL".into(),
                message: format!("MANIFEST.json error: {e}"),
            });
        }
    }
    if let Some(version) = installed_version {
        if let Some(bundled) = bundled_version
            && !bundled.is_empty()
            && bundled != "unknown"
            && bundled != version
        {
            issues.push(DoctorIssue {
                    severity: "WARN".into(),
                    message: format!(
                        "installed engine is {version}, bundled engine is {bundled}; run `mochiflow upgrade`"
                    ),
                });
        }
        let source_version_path = cfg.repo_root.join("engine").join("VERSION");
        let installed_version_path = engine_dir.join("VERSION");
        if source_version_path != installed_version_path
            && source_version_path.exists()
            && let Ok(source_version) = std::fs::read_to_string(&source_version_path)
        {
            let source_version = source_version.trim();
            if !source_version.is_empty() && source_version != version {
                issues.push(DoctorIssue {
                        severity: "WARN".into(),
                        message: format!(
                            "source engine is {source_version}, installed engine is {version}; run `mochiflow upgrade --source engine --force` if dogfooding latest engine"
                        ),
                    });
            }
        }
    }
    issues
}

fn build_manifest_files(engine_dir: &Path) -> std::collections::BTreeMap<String, String> {
    use sha2::{Digest, Sha256};
    let mut files = std::collections::BTreeMap::new();
    for entry in walkdir_files(engine_dir) {
        let rel = entry.strip_prefix(engine_dir).unwrap_or(&entry);
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if rel_str.contains("__pycache__") || rel_str == "MANIFEST.json" {
            continue;
        }
        if let Ok(bytes) = std::fs::read(&entry) {
            let hash = Sha256::digest(&bytes);
            files.insert(rel_str, format!("sha256:{hash:x}"));
        }
    }
    files
}

fn walkdir_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut result = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                result.extend(walkdir_files(&path));
            } else if path.is_file() {
                result.push(path);
            }
        }
    }
    result
}

/// Run the doctor command. Returns exit code.
pub fn run_doctor(cfg: &Config, target: Option<&str>, log_to_stderr: bool) -> i32 {
    run_doctor_with_bundled(cfg, target, log_to_stderr, None)
}

pub fn run_doctor_json_with_bundled(
    cfg: &Config,
    target: Option<&str>,
    bundled_version: Option<&str>,
) -> i32 {
    let reports = collect_reports(cfg, target, true, bundled_version);
    let total_fail = reports.iter().map(|r| r.fails).sum();
    let mut warnings = Vec::new();
    let mut doc = doctor_json(&reports, total_fail, &warnings);
    if let Err(e) = write_doctor_state(cfg, &doc) {
        warnings.push(format!("could not write state/doctor.json: {e}"));
        doc = doctor_json(&reports, total_fail, &warnings);
    }
    println!("{}", serde_json::to_string_pretty(&doc).unwrap_or_default());
    if total_fail > 0 { 1 } else { 0 }
}

pub fn run_doctor_with_bundled(
    cfg: &Config,
    target: Option<&str>,
    log_to_stderr: bool,
    bundled_version: Option<&str>,
) -> i32 {
    // Route human report lines to stderr when stdout is reserved (e.g. init
    // --json emits a single JSON document on stdout — AC-05).
    macro_rules! report_ln {
        ($($arg:tt)*) => {
            if log_to_stderr { eprintln!($($arg)*) } else { println!($($arg)*) }
        };
    }

    let reports = collect_reports(cfg, target, log_to_stderr, bundled_version);
    let total_fail = reports.iter().map(|r| r.fails).sum();
    for report in &reports {
        let name = &report.name;
        let fails = report.fails;
        let warns = report.warns;
        report_ln!("\n[{name}]");
        for issue in &report.issues {
            report_ln!("  {}: {}", issue.severity, issue.message);
        }
        report_ln!("  -> {fails} fail, {warns} warn");
    }

    let mut warnings = Vec::new();
    let doc = doctor_json(&reports, total_fail, &warnings);
    if let Err(e) = write_doctor_state(cfg, &doc) {
        warnings.push(format!("could not write state/doctor.json: {e}"));
    }

    for warning in &warnings {
        report_ln!("WARN: {warning}");
    }
    report_ln!("\nDoctor: {total_fail} fail (state/doctor.json)");
    if total_fail > 0 { 1 } else { 0 }
}

fn collect_reports(
    cfg: &Config,
    target: Option<&str>,
    log_to_stderr: bool,
    bundled_version: Option<&str>,
) -> Vec<TargetReport> {
    let targets: Vec<&str> = match target {
        Some(t) => vec![t],
        None => vec!["config", "specs", "adapter", "engine"],
    };

    targets
        .iter()
        .map(|name| {
            let issues = match *name {
                "config" => validate_config(cfg),
                "specs" => {
                    if lint::run_lint(cfg, None, log_to_stderr) != 0 {
                        vec![DoctorIssue {
                            severity: "FAIL".into(),
                            message: "spec lint failed".into(),
                        }]
                    } else {
                        Vec::new()
                    }
                }
                "adapter" => {
                    let result = crate::adapter::generate(cfg, true, false);
                    let mut issues: Vec<DoctorIssue> = result
                        .drift
                        .iter()
                        .map(|f| DoctorIssue {
                            severity: "FAIL".into(),
                            message: format!("adapter drift: {f}"),
                        })
                        .collect();
                    issues.extend(result.errors.iter().map(|e| DoctorIssue {
                        severity: "FAIL".into(),
                        message: e.clone(),
                    }));
                    issues
                }
                "engine" => check_engine_with_bundled(cfg, bundled_version),
                other => vec![DoctorIssue {
                    severity: "FAIL".into(),
                    message: format!("unknown doctor target: {other}"),
                }],
            };
            let fails = issues.iter().filter(|i| i.severity == "FAIL").count();
            let warns = issues.iter().filter(|i| i.severity == "WARN").count();
            TargetReport {
                name: (*name).to_string(),
                issues,
                fails,
                warns,
            }
        })
        .collect()
}

fn doctor_json(
    reports: &[TargetReport],
    total_fail: usize,
    warnings: &[String],
) -> serde_json::Value {
    let checks = reports
        .iter()
        .map(|report| {
            let issues: Vec<_> = report
                .issues
                .iter()
                .map(|i| serde_json::json!({"severity": i.severity, "message": i.message}))
                .collect();
            (report.name.clone(), serde_json::Value::Array(issues))
        })
        .collect::<serde_json::Map<_, _>>();
    serde_json::json!({
        "total_fail": total_fail,
        "exit_code": if total_fail > 0 { 1 } else { 0 },
        "warnings": warnings,
        "checks": checks,
    })
}

fn write_doctor_state(cfg: &Config, doc: &serde_json::Value) -> std::io::Result<()> {
    let state_dir = cfg.state_dir();
    std::fs::create_dir_all(&state_dir)?;
    let body = serde_json::to_string_pretty(doc).map_err(std::io::Error::other)?;
    std::fs::write(state_dir.join("doctor.json"), body)
}
