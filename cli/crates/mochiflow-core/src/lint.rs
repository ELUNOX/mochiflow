//! Structural lint for spec directories.

use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::spec_meta::{SpecMeta, SpecMetaError, YamlValue, read_spec_metadata};

use regex::Regex;

use std::sync::LazyLock;

#[allow(clippy::expect_used)]
static AC_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bAC-\d{2,}\b").expect("AC_RE"));
#[allow(clippy::expect_used)]
static TASK_AC_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^対応 AC:\s*(.+)$").expect("TASK_AC_RE"));
#[allow(clippy::expect_used)]
static EARS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(?:SHALL|WHEN|WHILE|WHERE|THEN)\b").expect("EARS_RE"));
#[allow(clippy::expect_used)]
static VERDICT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)Verdict:\s*(pass|pass-with-comments)").expect("VERDICT_RE"));

pub struct Issue {
    pub severity: String,
    pub path: PathBuf,
    pub message: String,
}

fn section<'a>(text: &'a str, heading: &str) -> Option<&'a str> {
    let pattern = format!("## {heading}");
    let start = text.find(&pattern)?;
    let after = &text[start..];
    // Find next ## heading or end
    let body_start = after
        .find('\n')
        .map(|i| start + i + 1)
        .unwrap_or(text.len());
    let body = &text[body_start..];
    let end = body
        .find("\n## ")
        .map(|i| body_start + i)
        .unwrap_or(text.len());
    Some(&text[body_start..end])
}

fn ac_ids(text: &str) -> Vec<String> {
    AC_RE
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect()
}

fn ac_ids_in_spec(text: &str) -> BTreeSet<String> {
    let body = section(text, "受入基準").unwrap_or(text);
    ac_ids(body).into_iter().collect()
}

fn ac_ids_in_tasks(text: &str) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for cap in TASK_AC_RE.captures_iter(text) {
        let line = &cap[1];
        if line.contains("なし") {
            continue;
        }
        for id in ac_ids(line) {
            ids.insert(id);
        }
    }
    ids
}

fn ac_lines_missing_ears(text: &str) -> BTreeSet<String> {
    let body = section(text, "受入基準").unwrap_or("");
    let mut missing = BTreeSet::new();
    for line in body.lines() {
        let found: Vec<String> = AC_RE
            .find_iter(line)
            .map(|m| m.as_str().to_string())
            .collect();
        if !found.is_empty() && !EARS_RE.is_match(line) {
            for id in found {
                missing.insert(id);
            }
        }
    }
    missing
}

fn has_frontmatter(text: &str) -> bool {
    text.starts_with("---\n") && text[4..].contains("\n---\n")
}

fn design_required(meta: &SpecMeta) -> bool {
    let risk = meta.risk();
    let integration = meta.integration();
    risk == "elevated" || risk == "critical" || integration != "none" || meta.surfaces().len() > 1
}

fn lint_spec_dir(spec_dir: &Path, allowed_surfaces: &HashSet<String>) -> Vec<Issue> {
    let mut issues = Vec::new();
    let spec_md = spec_dir.join("spec.md");
    let design_md = spec_dir.join("design.md");
    let tasks_md = spec_dir.join("tasks.md");

    let meta = match read_spec_metadata(spec_dir) {
        Ok(m) => m,
        Err(SpecMetaError::NotFound(_)) => {
            issues.push(Issue {
                severity: "FAIL".into(),
                path: spec_dir.join("spec.yaml"),
                message: "spec.yaml is required".into(),
            });
            return issues;
        }
        Err(e) => {
            issues.push(Issue {
                severity: "FAIL".into(),
                path: spec_dir.join("spec.yaml"),
                message: e.to_string(),
            });
            return issues;
        }
    };

    // metadata checks
    let required = [
        "version",
        "slug",
        "title",
        "type",
        "surfaces",
        "integration",
        "risk",
        "status",
    ];
    let missing: Vec<_> = required
        .iter()
        .filter(|k| !meta.data.contains_key(**k))
        .copied()
        .collect();
    if !missing.is_empty() {
        issues.push(Issue {
            severity: "FAIL".into(),
            path: meta.path.clone(),
            message: format!("spec.yaml missing required keys: {}", missing.join(", ")),
        });
    }
    if meta.data.get("version") != Some(&YamlValue::Int(1)) {
        issues.push(Issue {
            severity: "FAIL".into(),
            path: meta.path.clone(),
            message: "version must be 1".into(),
        });
    }
    let dir_name = spec_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if meta.slug() != dir_name {
        issues.push(Issue {
            severity: "FAIL".into(),
            path: meta.path.clone(),
            message: format!("slug must match directory name: {dir_name}"),
        });
    }
    let allowed_types = ["feature", "fix", "refactor", "docs", "chore"];
    if !allowed_types.contains(&meta.spec_type()) {
        issues.push(Issue {
            severity: "FAIL".into(),
            path: meta.path.clone(),
            message: "type must be one of: feature, fix, refactor, docs, chore".into(),
        });
    }
    if meta.surfaces().is_empty() {
        issues.push(Issue {
            severity: "FAIL".into(),
            path: meta.path.clone(),
            message: "surfaces must include at least one surface".into(),
        });
    }
    for s in meta.surfaces() {
        if !allowed_surfaces.contains(s) {
            issues.push(Issue {
                severity: "FAIL".into(),
                path: meta.path.clone(),
                message: format!("surface not defined in config.toml [surfaces.*]: {s}"),
            });
        }
    }
    let allowed_integrations = ["none", "contract", "workflow"];
    if !allowed_integrations.contains(&meta.integration()) {
        issues.push(Issue {
            severity: "FAIL".into(),
            path: meta.path.clone(),
            message: "integration must be one of: none, contract, workflow".into(),
        });
    }
    let allowed_risks = ["standard", "elevated", "critical"];
    if !allowed_risks.contains(&meta.risk()) {
        issues.push(Issue {
            severity: "FAIL".into(),
            path: meta.path.clone(),
            message: "risk must be one of: standard, elevated, critical".into(),
        });
    }
    let allowed_statuses = ["draft", "approved", "done"];
    if !allowed_statuses.contains(&meta.status()) {
        issues.push(Issue {
            severity: "FAIL".into(),
            path: meta.path.clone(),
            message: "status must be one of: draft, approved, done".into(),
        });
    }

    // spec.md
    if !spec_md.exists() {
        issues.push(Issue {
            severity: "FAIL".into(),
            path: spec_md,
            message: "spec.md is required".into(),
        });
        return issues;
    }
    let spec_text = std::fs::read_to_string(&spec_md).unwrap_or_default();
    if has_frontmatter(&spec_text) {
        issues.push(Issue {
            severity: "FAIL".into(),
            path: spec_md.clone(),
            message: "Markdown frontmatter is not allowed; use spec.yaml".into(),
        });
    }
    let spec_acs = ac_ids_in_spec(&spec_text);
    if spec_text.contains("[NEEDS-CLARIFICATION") {
        issues.push(Issue {
            severity: "WARN".into(),
            path: spec_md.clone(),
            message: "[NEEDS-CLARIFICATION] remains; resolve before approved".into(),
        });
    }
    let missing_ears = ac_lines_missing_ears(&spec_text);
    if !missing_ears.is_empty() {
        let list: Vec<_> = missing_ears.into_iter().collect();
        issues.push(Issue {
            severity: "WARN".into(),
            path: spec_md.clone(),
            message: format!(
                "AC without EARS keyword (SHALL/WHEN/WHILE/WHERE/THEN): {}",
                list.join(", ")
            ),
        });
    }

    // design.md
    if design_required(&meta) && !design_md.exists() {
        issues.push(Issue {
            severity: "FAIL".into(),
            path: design_md.clone(),
            message: "design.md is required for risk>=elevated / integration!=none / multi-surface"
                .into(),
        });
    }
    if design_md.exists() {
        let design_text = std::fs::read_to_string(&design_md).unwrap_or_default();
        if has_frontmatter(&design_text) {
            issues.push(Issue {
                severity: "FAIL".into(),
                path: design_md.clone(),
                message: "Markdown frontmatter is not allowed; use spec.yaml".into(),
            });
        }
        if (meta.integration() == "contract" || meta.integration() == "workflow")
            && !design_text.contains("## Integration Contract")
        {
            issues.push(Issue {
                severity: "FAIL".into(),
                path: design_md.clone(),
                message: "integration: contract/workflow requires Integration Contract section"
                    .into(),
            });
        }
    }

    // tasks.md
    let mut matrix_text = spec_text.clone();
    let mut tasks_text: Option<String> = None;
    if tasks_md.exists() {
        let tt = std::fs::read_to_string(&tasks_md).unwrap_or_default();
        if has_frontmatter(&tt) {
            issues.push(Issue {
                severity: "FAIL".into(),
                path: tasks_md.clone(),
                message: "Markdown frontmatter is not allowed; use spec.yaml".into(),
            });
        }
        let unknown: BTreeSet<_> = ac_ids_in_tasks(&tt)
            .difference(&spec_acs)
            .cloned()
            .collect();
        if !unknown.is_empty() {
            let list: Vec<_> = unknown.into_iter().collect();
            issues.push(Issue {
                severity: "FAIL".into(),
                path: tasks_md.clone(),
                message: format!("tasks reference AC IDs not in spec.md: {}", list.join(", ")),
            });
        }
        matrix_text = tt.clone();
        tasks_text = Some(tt);
    }

    // status guards
    if meta.status() == "done" {
        let matrix = section(&matrix_text, "AC Verification Matrix");
        if matrix.is_none() {
            issues.push(Issue {
                severity: "FAIL".into(),
                path: spec_dir.join("spec.yaml"),
                message: "status is done but AC Verification Matrix is missing".into(),
            });
        } else if let Some(m) = matrix {
            if m.contains("FAIL") {
                issues.push(Issue {
                    severity: "FAIL".into(),
                    path: spec_dir.join("spec.yaml"),
                    message: "status is done but AC Verification Matrix contains FAIL".into(),
                });
            }
            let matrix_acs: BTreeSet<String> = ac_ids(m).into_iter().collect();
            let uncovered: Vec<_> = spec_acs.difference(&matrix_acs).cloned().collect();
            if !uncovered.is_empty() {
                issues.push(Issue {
                    severity: "FAIL".into(),
                    path: spec_md.clone(),
                    message: format!(
                        "AC not present in AC Verification Matrix: {}",
                        uncovered.join(", ")
                    ),
                });
            }
        }
        if let Some(ref tt) = tasks_text {
            let untasked: Vec<_> = spec_acs.difference(&ac_ids_in_tasks(tt)).cloned().collect();
            if !untasked.is_empty() {
                issues.push(Issue {
                    severity: "FAIL".into(),
                    path: tasks_md.clone(),
                    message: format!(
                        "AC not covered by any task 対応 AC: {}",
                        untasked.join(", ")
                    ),
                });
            }
        }
        let risk_order = |r: &str| match r {
            "standard" => 0,
            "elevated" => 1,
            "critical" => 2,
            _ => 0,
        };
        if risk_order(meta.risk()) >= 1 {
            let design_text = std::fs::read_to_string(&design_md).unwrap_or_default();
            if !VERDICT_RE.is_match(&design_text) {
                issues.push(Issue { severity: "FAIL".into(), path: spec_dir.join("spec.yaml"), message: "status is done but reviewer verdict (pass/pass-with-comments) is not recorded for risk>=elevated".into() });
            }
        }
    }

    issues
}

fn discover_spec_dirs(root: &Path) -> Vec<PathBuf> {
    if root.is_file() {
        return vec![root.parent().unwrap_or(root).to_path_buf()];
    }
    if root.join("spec.yaml").exists() {
        return vec![root.to_path_buf()];
    }
    let excluded = ["_done", "_backlog"];
    let mut dirs = Vec::new();
    for entry in walkdir(root) {
        if entry.join("spec.yaml").exists() {
            // Check no excluded component in relative path
            if let Ok(rel) = entry.strip_prefix(root)
                && !rel.components().any(|c| {
                    let s = c.as_os_str().to_string_lossy();
                    excluded.contains(&s.as_ref())
                })
            {
                dirs.push(entry);
            }
        }
    }
    dirs.sort();
    dirs
}

fn walkdir(root: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                result.push(path.clone());
                result.extend(walkdir(&path));
            }
        }
    }
    result
}

/// Run lint, print issues, return exit code (0=pass, 1=fail).
pub fn run_lint(cfg: &Config, spec_slug: Option<&str>, log_to_stderr: bool) -> i32 {
    macro_rules! report_ln {
        ($($arg:tt)*) => {
            if log_to_stderr { eprintln!($($arg)*) } else { println!($($arg)*) }
        };
    }
    let allowed: HashSet<String> = cfg.surface_names().into_iter().collect();
    let paths = if let Some(slug) = spec_slug {
        vec![cfg.specs_dir_path().join(slug)]
    } else {
        vec![cfg.specs_dir_path()]
    };

    let mut all_issues = Vec::new();
    let mut seen = HashSet::new();
    for root in &paths {
        for spec_dir in discover_spec_dirs(root) {
            if seen.contains(&spec_dir) {
                continue;
            }
            seen.insert(spec_dir.clone());
            all_issues.extend(lint_spec_dir(&spec_dir, &allowed));
        }
    }

    for issue in &all_issues {
        report_ln!(
            "{}: {}: {}",
            issue.severity,
            issue.path.display(),
            issue.message
        );
    }
    let fail_count = all_issues.iter().filter(|i| i.severity == "FAIL").count();
    let warn_count = all_issues.iter().filter(|i| i.severity == "WARN").count();
    report_ln!("\nSummary: {fail_count} fail, {warn_count} warn");
    if fail_count > 0 { 1 } else { 0 }
}
