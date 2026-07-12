//! Join: local setup for an existing MochiFlow project.

use std::path::PathBuf;

use crate::adapter;
use crate::config::{Config, load_config};
use crate::doctor;
use crate::index;
use crate::init::EngineExtractFn;
use crate::manifest::read_engine_version;
use crate::upgrade::install_engine_staged;

#[derive(Default)]
struct JoinReport {
    mode: &'static str,
    target: String,
    dry_run: bool,
    actions: Vec<String>,
    warnings: Vec<String>,
    errors: Vec<String>,
    adapter_drift: Vec<String>,
    blocked_adapters: Vec<BlockedAdapterReport>,
    index_stale: bool,
    doctor_exit: Option<i32>,
}

#[derive(Default)]
struct BlockedAdapterReport {
    target: String,
    candidate: String,
}

impl JoinReport {
    fn exit_code(&self) -> i32 {
        if !self.errors.is_empty() {
            1
        } else if self.doctor_exit.unwrap_or(0) != 0 {
            self.doctor_exit.unwrap_or(1)
        } else {
            0
        }
    }
}

fn render_json(report: &JoinReport) -> String {
    let blocked_adapters: Vec<serde_json::Value> = report
        .blocked_adapters
        .iter()
        .map(|blocked| {
            serde_json::json!({
                "target": blocked.target,
                "candidate": blocked.candidate,
            })
        })
        .collect();
    let doc = serde_json::json!({
        "schema_version": 1,
        "mode": report.mode,
        "target": report.target,
        "dry_run": report.dry_run,
        "actions": report.actions,
        "warnings": report.warnings,
        "errors": report.errors,
        "adapter_drift": report.adapter_drift,
        "blocked_adapters": blocked_adapters,
        "index_stale": report.index_stale,
        "doctor_exit": report.doctor_exit,
        "exit_code": report.exit_code(),
    });
    let mut s = serde_json::to_string_pretty(&doc).unwrap_or_else(|_| "{}".to_string());
    s.push('\n');
    s
}

fn print_human(report: &JoinReport) {
    println!("target : {}", report.target);
    println!("mode   : join");
    if report.dry_run {
        println!("dry-run: true");
    }
    println!("\nActions:");
    if report.actions.is_empty() {
        println!("  OK: no local setup changes needed");
    } else {
        for action in &report.actions {
            println!("  OK: {action}");
        }
    }
    if !report.warnings.is_empty() {
        println!("\nWarnings:");
        for warning in &report.warnings {
            println!("  WARN: {warning}");
        }
    }
    if !report.adapter_drift.is_empty() {
        println!("\nAdapter drift:");
        for drift in &report.adapter_drift {
            println!("  DRIFT: {drift}");
        }
        println!("  dry-run: join would regenerate these adapter files");
    }
    if !report.blocked_adapters.is_empty() {
        println!("\nBlocked adapters:");
        for blocked in &report.blocked_adapters {
            println!(
                "  BLOCKED: {} (candidate: {})",
                blocked.target, blocked.candidate
            );
        }
    }
    if report.index_stale {
        println!("\nINDEX.md is stale");
        if report.dry_run {
            println!("  dry-run: join would regenerate INDEX.md");
        }
    }
    if !report.errors.is_empty() {
        println!("\nErrors:");
        for error in &report.errors {
            println!("  FAIL: {error}");
        }
    }
    println!("\nJoin: {} fail", report.exit_code());
}

fn log_json(json: bool, message: &str) {
    if json {
        eprintln!("{message}");
    } else {
        println!("{message}");
    }
}

fn config_path_for_target(target: &str) -> PathBuf {
    PathBuf::from(target).join(".mochiflow").join("config.toml")
}

fn load_target_config(target: &str) -> Result<Config, String> {
    let config_path = config_path_for_target(target);
    if !config_path.exists() {
        return Err(format!(
            "config.toml not found at {}; run `mochiflow init` first",
            config_path.display()
        ));
    }
    load_config(&config_path).map_err(|e| format!("{e}"))
}

fn required_gitignore_entries_missing(cfg: &Config) -> Vec<&'static str> {
    let path = cfg.install_dir_path().join(".gitignore");
    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(_) => return vec!["state/", "constitution.local.md"],
    };
    ["state/", "constitution.local.md"]
        .into_iter()
        .filter(|entry| !content.lines().any(|line| line.trim() == *entry))
        .collect()
}

fn engine_needs_install(cfg: &Config, bundled_engine_version: &str) -> bool {
    let engine_dir = cfg.engine_dir();
    if !engine_dir.exists() || !engine_dir.join("MANIFEST.json").exists() {
        return true;
    }
    if doctor::check_engine_with_bundled(cfg, Some(bundled_engine_version))
        .iter()
        .any(|issue| issue.severity == "FAIL")
    {
        return true;
    }
    match read_engine_version(&engine_dir) {
        Ok(version) => {
            bundled_engine_version != "unknown"
                && !bundled_engine_version.is_empty()
                && version != bundled_engine_version
        }
        Err(_) => true,
    }
}

fn install_engine(
    cfg: &Config,
    force: bool,
    embedded_engine_extract: Option<EngineExtractFn<'_>>,
) -> Result<(), Vec<String>> {
    let Some(extract_fn) = embedded_engine_extract else {
        return Err(vec![
            "engine source not found and no embedded engine extractor was provided".to_string(),
        ]);
    };
    install_engine_staged(cfg, "bundled engine", force, extract_fn).map_err(|e| e.report_lines())
}

#[allow(clippy::too_many_arguments)]
pub fn run_join(
    target: &str,
    dry_run: bool,
    json: bool,
    force: bool,
    bundled_engine_version: &str,
    embedded_engine_extract: Option<EngineExtractFn<'_>>,
) -> i32 {
    let root = PathBuf::from(target)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(target));
    let mut report = JoinReport {
        mode: "join",
        target: root.display().to_string(),
        dry_run,
        ..JoinReport::default()
    };

    let cfg = match load_target_config(&root.display().to_string()) {
        Ok(cfg) => cfg,
        Err(e) => {
            report.errors.push(e);
            if json {
                print!("{}", render_json(&report));
            } else {
                print_human(&report);
            }
            return report.exit_code();
        }
    };
    if let Err(error) = cfg.validate_repository_paths_now() {
        report.errors.push(error.to_string());
        if json {
            print!("{}", render_json(&report));
        } else {
            print_human(&report);
        }
        return report.exit_code();
    }

    for entry in required_gitignore_entries_missing(&cfg) {
        report.warnings.push(format!(
            "{} missing `{entry}`; generated local files may be visible to git",
            cfg.install_dir_path().join(".gitignore").display()
        ));
    }

    let needs_engine = engine_needs_install(&cfg, bundled_engine_version);
    if needs_engine {
        report
            .actions
            .push(format!("install engine at {}", cfg.engine_dir().display()));
        if !dry_run {
            match install_engine(&cfg, force, embedded_engine_extract) {
                Ok(()) => log_json(json, "installed engine <- bundled engine"),
                Err(lines) => report.errors.extend(lines),
            }
        }
    }

    if !dry_run {
        match cfg.checked_state_dir() {
            Err(e) => report
                .errors
                .push(format!("could not validate state directory: {e}")),
            Ok(state) => {
                if let Err(e) = std::fs::create_dir_all(state.operation_path()) {
                    report.errors.push(format!(
                        "could not create {}: {e}",
                        state.operation_path().display()
                    ));
                } else {
                    report
                        .actions
                        .push(format!("ensured {}", state.operation_path().display()));
                }
            }
        }
    } else {
        report
            .actions
            .push(format!("ensure {}", cfg.state_dir().display()));
    }

    if report.errors.is_empty() {
        report.index_stale = index::is_index_stale(&cfg);
        if dry_run {
            let adapter_result = adapter::generate(&cfg, true, false);
            report.adapter_drift = adapter_result.drift;
            report.errors.extend(adapter_result.errors);
            if !report.adapter_drift.is_empty() {
                report
                    .actions
                    .push("would regenerate adapter entrypoints".to_string());
            }
            if report.index_stale {
                report.actions.push(format!(
                    "would regenerate {} and state/index.json",
                    cfg.index_path().display()
                ));
            }
        } else {
            let adapter_result = adapter::generate(&cfg, false, false);
            for f in &adapter_result.wrote {
                report.actions.push(format!("wrote adapter {f}"));
            }
            for blocked in &adapter_result.blocked {
                report.blocked_adapters.push(BlockedAdapterReport {
                    target: blocked.target.clone(),
                    candidate: blocked.candidate.clone(),
                });
                report.errors.push(format!(
                    "adapter blocked: {} (candidate: {})",
                    blocked.target, blocked.candidate
                ));
            }
            report.errors.extend(adapter_result.errors);
            if report.index_stale {
                match index::generate_index_quiet(&cfg) {
                    Ok(()) => {
                        report.actions.push(format!(
                            "regenerated {} and state/index.json",
                            cfg.index_path().display()
                        ));
                        report.index_stale = false;
                    }
                    Err(e) => report.errors.push(format!(
                        "could not regenerate {} and state/index.json: {e}",
                        cfg.index_path().display()
                    )),
                }
            }
        }
    }

    if report.errors.is_empty() && !dry_run {
        let doctor_exit =
            doctor::run_doctor_with_bundled(&cfg, None, json, Some(bundled_engine_version));
        report.doctor_exit = Some(doctor_exit);
    }

    if json {
        print!("{}", render_json(&report));
    } else {
        print_human(&report);
    }
    report.exit_code()
}
