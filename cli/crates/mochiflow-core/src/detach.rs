//! Project detach: remove MochiFlow runtime/adapter integration while
//! preserving durable project knowledge by default.

use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::adapter::{MARKER_PREFIX, is_embeddable_target, load_manifest, managed_block_bounds};
use crate::config::Config;

pub const PURGE_CONFIRM_PHRASE: &str = "delete mochiflow data";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DetachMode {
    #[default]
    Detach,
    Purge,
}

#[derive(Debug, Default, Serialize)]
pub struct DetachReport {
    pub mode: DetachMode,
    pub dry_run: bool,
    pub removed: Vec<String>,
    pub updated: Vec<String>,
    pub kept: Vec<String>,
    pub skipped: Vec<String>,
    pub errors: Vec<String>,
    pub exit_code: i32,
}

enum AdapterAction {
    RemoveFile {
        path: PathBuf,
        label: String,
    },
    UpdateFile {
        path: PathBuf,
        label: String,
        content: String,
    },
    Skip {
        label: String,
        reason: String,
    },
}

struct AdapterPlan {
    actions: Vec<AdapterAction>,
    known_parents: Vec<PathBuf>,
}

#[allow(clippy::too_many_arguments)]
pub fn run_detach(
    cfg: &Config,
    purge: bool,
    dry_run: bool,
    json: bool,
    confirm: Option<&str>,
) -> i32 {
    let mode = if purge {
        DetachMode::Purge
    } else {
        DetachMode::Detach
    };
    let mut report = DetachReport {
        mode,
        dry_run,
        ..DetachReport::default()
    };

    if purge && !purge_confirmed(json, confirm) {
        report.errors.push(format!(
            "purge requires exact confirmation: --confirm \"{PURGE_CONFIRM_PHRASE}\""
        ));
        report.exit_code = 1;
        present_report(&report, json);
        return report.exit_code;
    }

    let plan = plan_adapter_cleanup(cfg);
    apply_adapter_plan(cfg, &plan, dry_run, &mut report);

    if purge {
        remove_or_report(
            &cfg.install_dir_path(),
            &cfg.install_dir,
            dry_run,
            &mut report,
        );
    } else {
        remove_or_report(
            &cfg.engine_dir(),
            &format!("{}/engine", cfg.install_dir),
            dry_run,
            &mut report,
        );
        remove_or_report(
            &cfg.state_dir(),
            &format!("{}/state", cfg.install_dir),
            dry_run,
            &mut report,
        );
        keep_default_project_data(cfg, &mut report);
    }

    report.exit_code = if report.errors.is_empty() { 0 } else { 1 };
    present_report(&report, json);
    report.exit_code
}

fn purge_confirmed(json: bool, confirm: Option<&str>) -> bool {
    if confirm == Some(PURGE_CONFIRM_PHRASE) {
        return true;
    }
    if json || !std::io::stdin().is_terminal() {
        return false;
    }

    eprintln!("This will permanently remove all MochiFlow project data.");
    eprintln!("Type \"{PURGE_CONFIRM_PHRASE}\" to continue:");
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .is_ok_and(|_| line.trim() == PURGE_CONFIRM_PHRASE)
}

fn plan_adapter_cleanup(cfg: &Config) -> AdapterPlan {
    let mut actions = Vec::new();
    let mut known_parents = Vec::new();

    for tool in &cfg.adapter_tools() {
        let Some((_adapter_dir, files)) = load_manifest(cfg, tool) else {
            continue;
        };
        for (out_rel, _tpl_rel) in files {
            let target = cfg.repo_root.join(&out_rel);
            if let Some(parent) = target.parent() {
                known_parents.push(parent.to_path_buf());
            }

            let current = match std::fs::read_to_string(&target) {
                Ok(content) => content,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    actions.push(AdapterAction::Skip {
                        label: out_rel,
                        reason: "missing".into(),
                    });
                    continue;
                }
                Err(e) => {
                    actions.push(AdapterAction::Skip {
                        label: out_rel,
                        reason: format!("could not read: {e}"),
                    });
                    continue;
                }
            };

            if is_embeddable_target(&out_rel)
                && let Some((start, end)) = managed_block_bounds(&current, tool)
            {
                let next = remove_range_and_tidy(&current, start, end);
                if next.trim().is_empty() {
                    actions.push(AdapterAction::RemoveFile {
                        path: target,
                        label: out_rel,
                    });
                } else {
                    actions.push(AdapterAction::UpdateFile {
                        path: target,
                        label: out_rel,
                        content: next,
                    });
                }
                continue;
            }

            if current.contains(MARKER_PREFIX) {
                actions.push(AdapterAction::RemoveFile {
                    path: target,
                    label: out_rel,
                });
            } else {
                actions.push(AdapterAction::Skip {
                    label: out_rel,
                    reason: "unmanaged".into(),
                });
            }
        }
    }

    known_parents.sort();
    known_parents.dedup();
    AdapterPlan {
        actions,
        known_parents,
    }
}

fn remove_range_and_tidy(content: &str, start: usize, end: usize) -> String {
    let mut out = String::new();
    out.push_str(&content[..start]);
    out.push_str(&content[end..]);
    while out.contains("\n\n\n") {
        out = out.replace("\n\n\n", "\n\n");
    }
    out.trim_end_matches([' ', '\t', '\r', '\n']).to_string() + "\n"
}

fn apply_adapter_plan(cfg: &Config, plan: &AdapterPlan, dry_run: bool, report: &mut DetachReport) {
    for action in &plan.actions {
        match action {
            AdapterAction::RemoveFile { path, label } => {
                if !dry_run
                    && let Err(e) = std::fs::remove_file(path)
                    && e.kind() != std::io::ErrorKind::NotFound
                {
                    report.errors.push(format!("could not remove {label}: {e}"));
                    continue;
                }
                report.removed.push(label.clone());
            }
            AdapterAction::UpdateFile {
                path,
                label,
                content,
            } => {
                if !dry_run && let Err(e) = std::fs::write(path, content) {
                    report.errors.push(format!("could not update {label}: {e}"));
                    continue;
                }
                report.updated.push(label.clone());
            }
            AdapterAction::Skip { label, reason } => {
                report.skipped.push(format!("{label} ({reason})"));
            }
        }
    }

    if !dry_run {
        prune_known_adapter_dirs(cfg, &plan.known_parents, report);
    }
}

fn prune_known_adapter_dirs(cfg: &Config, parents: &[PathBuf], report: &mut DetachReport) {
    let mut dirs: Vec<PathBuf> = parents.to_vec();
    dirs.sort_by_key(|p| std::cmp::Reverse(p.components().count()));
    dirs.dedup();

    for dir in dirs {
        let mut current = Some(dir.as_path());
        while let Some(path) = current {
            if path == cfg.repo_root {
                break;
            }
            match std::fs::remove_dir(path) {
                Ok(()) => report.removed.push(rel_label(cfg, path)),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) if e.kind() == std::io::ErrorKind::DirectoryNotEmpty => break,
                Err(_) => break,
            }
            current = path.parent();
        }
    }
}

fn remove_or_report(path: &Path, label: &str, dry_run: bool, report: &mut DetachReport) {
    if !path.exists() {
        report.skipped.push(format!("{label} (missing)"));
        return;
    }
    if dry_run {
        report.removed.push(label.to_string());
        return;
    }

    let result = if path.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    };
    match result {
        Ok(()) => report.removed.push(label.to_string()),
        Err(e) => report.errors.push(format!("could not remove {label}: {e}")),
    }
}

fn keep_default_project_data(cfg: &Config, report: &mut DetachReport) {
    let paths = vec![
        cfg.config_path.clone(),
        cfg.specs_dir_path(),
        cfg.decisions_path(),
        cfg.pitfalls_path(),
        cfg.product_path(),
        cfg.structure_path(),
        cfg.tech_path(),
        cfg.constitution_path(),
        cfg.constitution_local_path(),
        cfg.index_path(),
    ];
    for path in paths {
        if path.exists() {
            report.kept.push(rel_label(cfg, &path));
        }
    }
}

fn rel_label(cfg: &Config, path: &Path) -> String {
    path.strip_prefix(&cfg.repo_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn present_report(report: &DetachReport, json: bool) {
    if json {
        let mut text = serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".into());
        text.push('\n');
        print!("{text}");
        return;
    }

    let action = match (report.mode, report.dry_run) {
        (DetachMode::Detach, false) => "Detached MochiFlow project integration.",
        (DetachMode::Detach, true) => "Would detach MochiFlow project integration.",
        (DetachMode::Purge, false) => "Purged MochiFlow project data.",
        (DetachMode::Purge, true) => "Would purge MochiFlow project data.",
    };
    println!("{action}");
    for item in &report.updated {
        println!("updated: {item}");
    }
    for item in &report.removed {
        println!("removed: {item}");
    }
    for item in &report.kept {
        println!("kept: {item}");
    }
    for item in &report.skipped {
        println!("skipped: {item}");
    }
    for error in &report.errors {
        println!("FAIL: {error}");
    }
    println!(
        "\nSummary: {} removed, {} updated, {} kept, {} skipped, {} failed",
        report.removed.len(),
        report.updated.len(),
        report.kept.len(),
        report.skipped.len(),
        report.errors.len()
    );
}
