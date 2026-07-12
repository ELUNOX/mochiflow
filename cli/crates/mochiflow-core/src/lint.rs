//! Structural lint for spec directories.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::spec_meta::{SpecMeta, SpecMetaError, YamlValue, read_spec_metadata};

use regex::Regex;

use std::sync::LazyLock;

#[allow(clippy::expect_used)]
static AC_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bAC-\d{2,}\b").expect("AC_RE"));
#[allow(clippy::expect_used)]
static TASK_AC_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^(?:Covers AC|対応 AC):\s*(.+)$").expect("TASK_AC_RE"));
#[allow(clippy::expect_used)]
static TASK_LINE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^- \[(?P<checked>[ xX])\] (?P<id>T-\d{3,})(?:\s+\[P\])?(?P<refs>(?:\s+\[[^\]]+\])*)\s+.+$",
    )
        .expect("TASK_LINE_RE")
});
#[allow(clippy::expect_used)]
static TASK_ID_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bT-\d{3,}\b").expect("TASK_ID_RE"));
#[allow(clippy::expect_used)]
static BACKTICK_PATH_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"`([^`]+)`").expect("BACKTICK_PATH_RE"));
#[allow(clippy::expect_used)]
static NFR_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bNFR-\d{2,}\b").expect("NFR_RE"));
#[allow(clippy::expect_used)]
static EARS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(?:SHALL|WHEN|WHILE|WHERE|THEN)\b").expect("EARS_RE"));
#[allow(clippy::expect_used)]
static TEMPLATE_PLACEHOLDER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{[^{}\n]{1,80}\}").expect("TEMPLATE_PLACEHOLDER_RE"));
#[allow(clippy::expect_used)]
static HTML_COMMENT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)<!--.*?-->").expect("HTML_COMMENT_RE"));
#[allow(clippy::expect_used)]
static BARE_TBD_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bTBD\b").expect("BARE_TBD_RE"));
#[allow(clippy::expect_used)]
static EXAMPLE_CELL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)\|\s*\.\.\.\s*\|").expect("EXAMPLE_CELL_RE"));
#[allow(clippy::expect_used)]
static VERDICT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Verdict:\s*(pass-with-comments|pass|fail)").expect("VERDICT_RE")
});
#[allow(clippy::expect_used)]
static TASK_HEADING_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^## Task\b").expect("TASK_HEADING_RE"));
#[allow(clippy::expect_used)]
static COMPLETED_RE: LazyLock<Regex> = LazyLock::new(|| {
    // Date `YYYY-MM-DD`, optionally followed by a `T`/space time component.
    Regex::new(r"^\d{4}-\d{2}-\d{2}([T ].*)?$").expect("COMPLETED_RE")
});

fn is_valid_completed(value: &str) -> bool {
    COMPLETED_RE.is_match(value.trim())
}

pub struct Issue {
    pub severity: String,
    pub path: PathBuf,
    pub message: String,
}

/// Pure structured lint report used by machine-facing state consumers.
pub fn report(cfg: &Config, target: &str) -> Vec<Issue> {
    let allowed: HashSet<String> = cfg.surface_names().into_iter().collect();
    let dirty_paths = dirty_worktree_paths(&cfg.repo_root);
    match resolve_lint_targets(cfg, target) {
        Ok(paths) => paths
            .iter()
            .flat_map(|root| discover_spec_dirs(root))
            .flat_map(|dir| lint_spec_dir(&dir, &allowed, dirty_paths.as_ref()))
            .collect(),
        Err(issue) => vec![issue],
    }
}

fn section<'a>(text: &'a str, heading: &str) -> Option<&'a str> {
    let pattern = format!("## {heading}");
    let matches_heading = |line: &str| {
        let line = line.trim_end();
        line == pattern
            || line
                .strip_prefix(&pattern)
                .is_some_and(|suffix| suffix.starts_with(' '))
    };

    let mut body_start = None;
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let line_without_newline = line.strip_suffix('\n').unwrap_or(line);
        if matches_heading(line_without_newline) {
            body_start = Some(offset + line.len());
            break;
        }
        offset += line.len();
    }

    let body_start = body_start?;
    let body = &text[body_start..];
    let end = body
        .find("\n## ")
        .map(|i| body_start + i)
        .unwrap_or(text.len());
    Some(&text[body_start..end])
}

fn section_any<'a>(text: &'a str, headings: &[&str]) -> Option<&'a str> {
    headings.iter().find_map(|heading| section(text, heading))
}

fn ac_ids(text: &str) -> Vec<String> {
    AC_RE
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect()
}

fn ac_ids_in_spec(text: &str) -> BTreeSet<String> {
    let body = section_any(
        text,
        &[
            "Requirements / Acceptance Criteria",
            "Acceptance Criteria",
            "要件 / 受入基準",
            "受入基準",
        ],
    )
    .unwrap_or(text);
    ac_ids(body).into_iter().collect()
}

fn ac_ids_in_tasks(text: &str) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for cap in TASK_LINE_RE.captures_iter(text) {
        if let Some(refs) = cap.name("refs") {
            for id in ac_ids(refs.as_str()) {
                ids.insert(id);
            }
        }
    }
    for cap in TASK_AC_RE.captures_iter(text) {
        let line = &cap[1];
        if line.contains("なし") || line.trim().eq_ignore_ascii_case("none") {
            continue;
        }
        for id in ac_ids(line) {
            ids.insert(id);
        }
    }
    ids
}

fn review_verdicts(design_text: &str) -> Vec<String> {
    let Some(review_results) = section(design_text, "Review Results") else {
        return Vec::new();
    };
    VERDICT_RE
        .captures_iter(review_results)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_ascii_lowercase()))
        .collect()
}

fn latest_review_verdict(design_text: &str) -> Option<String> {
    review_verdicts(design_text).into_iter().next_back()
}

fn task_count(tasks_text: Option<&str>) -> usize {
    tasks_text
        .map(|text| TASK_HEADING_RE.find_iter(text).count())
        .unwrap_or(0)
}

fn markdown_text_without_code(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut in_fence = false;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            result.push('\n');
            continue;
        }
        if in_fence {
            result.push('\n');
            continue;
        }

        let mut in_inline_code = false;
        for ch in line.chars() {
            if ch == '`' {
                in_inline_code = !in_inline_code;
                result.push(' ');
            } else if in_inline_code {
                result.push(' ');
            } else {
                result.push(ch);
            }
        }
    }
    result
}

fn template_residue_messages(text: &str) -> Vec<String> {
    let check_text = markdown_text_without_code(text);
    let mut messages = Vec::new();

    if TEMPLATE_PLACEHOLDER_RE.is_match(&check_text) {
        messages.push("template residue remains: unreplaced `{...}` placeholder".to_string());
    }
    if HTML_COMMENT_RE.is_match(&check_text) {
        messages.push("template residue remains: template-only HTML comment".to_string());
    }
    if EXAMPLE_CELL_RE.is_match(&check_text) {
        messages.push("template residue remains: example-only table row".to_string());
    }
    if BARE_TBD_RE.is_match(&check_text) {
        messages.push("template residue remains: bare `TBD`".to_string());
    }

    messages
}

fn push_template_residue_issues(issues: &mut Vec<Issue>, path: &Path, text: &str) {
    for message in template_residue_messages(text) {
        issues.push(Issue {
            severity: "FAIL".into(),
            path: path.to_path_buf(),
            message,
        });
    }
}

fn line_starts_section(line: &str) -> bool {
    line.trim_start().starts_with("## ")
}

fn line_declares_ac(line: &str) -> bool {
    AC_RE.is_match(line)
}

fn ac_lines_missing_ears(text: &str) -> BTreeSet<String> {
    let body = section_any(
        text,
        &[
            "Requirements / Acceptance Criteria",
            "Acceptance Criteria",
            "要件 / 受入基準",
            "受入基準",
        ],
    )
    .unwrap_or("");
    let mut missing = BTreeSet::new();
    let mut current_ids: Vec<String> = Vec::new();
    let mut current_block = String::new();
    for line in body.lines() {
        if line_starts_section(line) || (line_declares_ac(line) && !current_ids.is_empty()) {
            if !EARS_RE.is_match(&current_block) {
                missing.extend(current_ids.iter().cloned());
            }
            current_ids.clear();
            current_block.clear();
        }

        let found: Vec<String> = AC_RE
            .find_iter(line)
            .map(|m| m.as_str().to_string())
            .collect();
        if !found.is_empty() {
            current_ids = found;
        }
        if !current_ids.is_empty() {
            current_block.push_str(line);
            current_block.push('\n');
        }
    }
    if !current_ids.is_empty() && !EARS_RE.is_match(&current_block) {
        missing.extend(current_ids);
    }
    missing
}

fn task_ids(text: &str) -> BTreeSet<String> {
    TASK_LINE_RE
        .captures_iter(text)
        .filter_map(|cap| cap.name("id").map(|m| m.as_str().to_string()))
        .collect()
}

fn unchecked_task_ids(text: &str) -> Vec<String> {
    TASK_LINE_RE
        .captures_iter(text)
        .filter_map(|cap| {
            let checked = cap.name("checked")?.as_str();
            let id = cap.name("id")?.as_str();
            if checked.eq_ignore_ascii_case("x") {
                None
            } else {
                Some(id.to_string())
            }
        })
        .collect()
}

#[derive(Debug)]
struct TaskFile {
    path: String,
    planned_deletion: bool,
}

#[derive(Debug)]
struct TaskProgress {
    id: String,
    checked: bool,
    files: Vec<TaskFile>,
}

#[derive(Debug)]
struct DirtyPathStatus {
    deleted: bool,
}

fn normalize_task_path(path: &str) -> String {
    path.trim()
        .trim_matches('`')
        .trim()
        .strip_prefix("./")
        .unwrap_or(path.trim().trim_matches('`').trim())
        .replace('\\', "/")
}

fn paths_from_files_value(value: &str) -> Vec<String> {
    let backtick_paths: Vec<_> = BACKTICK_PATH_RE
        .captures_iter(value)
        .filter_map(|cap| cap.get(1).map(|m| normalize_task_path(m.as_str())))
        .filter(|path| !path.is_empty())
        .collect();
    if !backtick_paths.is_empty() {
        return backtick_paths;
    }

    let cleaned = value
        .trim()
        .trim_start_matches("- ")
        .trim()
        .trim_matches(['[', ']']);
    if cleaned.is_empty() || cleaned.eq_ignore_ascii_case("none") {
        Vec::new()
    } else {
        cleaned
            .split(',')
            .map(normalize_task_path)
            .filter(|path| !path.is_empty())
            .collect()
    }
}

fn task_files_from_files_value(value: &str) -> Vec<TaskFile> {
    let cleaned = value.trim().trim_start_matches("- ").trim();
    let (planned_deletion, path_value) = cleaned
        .strip_prefix("deleted:")
        .map_or((false, cleaned), |rest| (true, rest.trim()));

    paths_from_files_value(path_value)
        .into_iter()
        .map(|path| TaskFile {
            path,
            planned_deletion,
        })
        .collect()
}

fn is_task_block_label(trimmed: &str, label: &str) -> bool {
    trimmed.starts_with(&format!("{label}:")) || trimmed.starts_with(&format!("- {label}:"))
}

fn files_for_task_body(body: &str) -> Vec<TaskFile> {
    let mut files = Vec::new();
    let mut in_files = false;

    for line in body.lines() {
        let trimmed = line.trim_start();
        if is_task_block_label(trimmed, "Files") {
            in_files = true;
            if let Some((_, value)) = trimmed.split_once(':') {
                files.extend(task_files_from_files_value(value));
            }
            continue;
        }

        if in_files
            && (is_task_block_label(trimmed, "Depends on")
                || is_task_block_label(trimmed, "Done")
                || is_task_block_label(trimmed, "Stop"))
        {
            in_files = false;
        }

        if in_files && trimmed.starts_with('-') {
            files.extend(task_files_from_files_value(trimmed));
        }
    }

    files.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then(a.planned_deletion.cmp(&b.planned_deletion))
    });
    files.dedup_by(|a, b| a.path == b.path && a.planned_deletion == b.planned_deletion);
    files
}

fn task_progress(text: &str) -> Vec<TaskProgress> {
    let captures: Vec<_> = TASK_LINE_RE.captures_iter(text).collect();
    let mut tasks = Vec::new();

    for (idx, cap) in captures.iter().enumerate() {
        let Some(task_match) = cap.get(0) else {
            continue;
        };
        let Some(id) = cap.name("id").map(|m| m.as_str().to_string()) else {
            continue;
        };
        let checked = cap
            .name("checked")
            .is_some_and(|m| m.as_str().eq_ignore_ascii_case("x"));
        let start = task_match.end();
        let end = captures
            .get(idx + 1)
            .and_then(|next| next.get(0))
            .map_or(text.len(), |m| m.start());
        let files = files_for_task_body(&text[start..end]);
        tasks.push(TaskProgress { id, checked, files });
    }

    tasks
}

fn dirty_worktree_paths(repo_root: &Path) -> Option<HashMap<String, DirtyPathStatus>> {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=all"])
        .current_dir(repo_root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut paths = HashMap::new();
    for line in stdout.lines() {
        let status = line.get(..2).unwrap_or("");
        let deleted = status.as_bytes().contains(&b'D');
        let Some(raw_path) = line.get(3..) else {
            continue;
        };
        for path in raw_path.split(" -> ") {
            let normalized = normalize_task_path(path);
            if !normalized.is_empty() {
                paths.insert(normalized, DirtyPathStatus { deleted });
            }
        }
    }
    Some(paths)
}

fn dirty_files_for_unchecked_task<'a>(
    task: &'a TaskProgress,
    dirty_paths: &HashMap<String, DirtyPathStatus>,
) -> Vec<&'a str> {
    if task.checked {
        return Vec::new();
    }

    task.files
        .iter()
        .filter_map(|file| {
            let dirty_status = dirty_paths.get(&file.path)?;
            if file.planned_deletion && dirty_status.deleted {
                None
            } else {
                Some(file.path.as_str())
            }
        })
        .collect()
}

fn task_refs_are_present(refs: &str) -> bool {
    AC_RE.is_match(refs) || NFR_RE.is_match(refs) || refs.contains("[chore:")
}

fn parse_depends_on(line: &str) -> Vec<String> {
    let Some((_, value)) = line.split_once(':') else {
        return Vec::new();
    };
    if value.trim().eq_ignore_ascii_case("none")
        || value
            .trim()
            .eq_ignore_ascii_case("all implementation tasks")
    {
        return Vec::new();
    }
    TASK_ID_RE
        .find_iter(value)
        .map(|m| m.as_str().to_string())
        .collect()
}

fn lint_task_structure(text: &str) -> Vec<String> {
    let mut issues = Vec::new();
    let ids = task_ids(text);
    let mut seen = BTreeSet::new();
    let matches: Vec<_> = TASK_LINE_RE.find_iter(text).collect();

    if matches.is_empty() {
        issues.push("tasks.md must contain top-level T-### checkbox tasks".to_string());
        return issues;
    }

    if text.contains("- [ ]") || text.contains("- [x]") || text.contains("- [X]") {
        for line in text.lines().filter(|line| line.starts_with("- [")) {
            if !TASK_LINE_RE.is_match(line) {
                issues.push(format!(
                    "top-level task is missing T-### ID or title: {line}"
                ));
            }
        }
    }

    for cap in TASK_LINE_RE.captures_iter(text) {
        let Some(task_id) = cap.name("id").map(|m| m.as_str()) else {
            continue;
        };
        if !seen.insert(task_id.to_string()) {
            issues.push(format!("duplicate task ID: {task_id}"));
        }
        let refs = cap.name("refs").map(|m| m.as_str()).unwrap_or("");
        if !task_refs_are_present(refs) {
            issues.push(format!(
                "task {task_id} must reference AC, NFR, or chore reason"
            ));
        }
    }

    for (idx, task_match) in matches.iter().enumerate() {
        let task_line = task_match.as_str();
        let task_id = TASK_ID_RE
            .find(task_line)
            .map(|m| m.as_str())
            .unwrap_or("task");
        let start = task_match.end();
        let end = matches.get(idx + 1).map_or(text.len(), regex::Match::start);
        let body = &text[start..end];
        for required in ["Depends on:", "Files:", "Done:", "Stop:"] {
            if !body.contains(required) {
                issues.push(format!("task {task_id} missing {required}"));
            }
        }
        for line in body.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("- Depends on:") || trimmed.starts_with("Depends on:") {
                for dep in parse_depends_on(trimmed) {
                    if !ids.contains(&dep) {
                        issues.push(format!("task {task_id} depends on unknown task {dep}"));
                    }
                }
            }
        }
    }

    issues
}

fn has_frontmatter(text: &str) -> bool {
    text.starts_with("---\n") && text[4..].contains("\n---\n")
}

#[derive(Debug)]
struct MatrixRow {
    ac: String,
    result: String,
}

fn matrix_section(text: &str) -> Option<&str> {
    section_any(
        text,
        &[
            "Verification Plan / AC Matrix",
            "AC Matrix",
            "AC Verification Matrix",
        ],
    )
}

fn parse_matrix_rows(text: &str) -> Vec<MatrixRow> {
    let Some(matrix) = matrix_section(text) else {
        return Vec::new();
    };
    let mut lines = matrix
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with('|') && line.ends_with('|'));
    let Some(header) = lines.next() else {
        return Vec::new();
    };
    let headers: Vec<_> = header
        .trim_matches('|')
        .split('|')
        .map(|cell| cell.trim())
        .collect();
    let ac_idx = headers.iter().position(|cell| *cell == "AC");
    let result_idx = headers
        .iter()
        .position(|cell| *cell == "Result" || *cell == "結果");
    let (Some(ac_idx), Some(result_idx)) = (ac_idx, result_idx) else {
        return Vec::new();
    };

    let mut rows = Vec::new();
    for line in lines {
        let cells: Vec<_> = line
            .trim_matches('|')
            .split('|')
            .map(|cell| cell.trim())
            .collect();
        if cells
            .iter()
            .all(|cell| cell.chars().all(|c| c == '-' || c == ':'))
        {
            continue;
        }
        let Some(ac) = cells.get(ac_idx) else {
            continue;
        };
        let Some(result) = cells.get(result_idx) else {
            continue;
        };
        if AC_RE.is_match(ac) {
            rows.push(MatrixRow {
                ac: (*ac).to_string(),
                result: (*result).to_string(),
            });
        }
    }
    rows
}

fn is_canonical_matrix_result(result: &str) -> bool {
    matches!(
        result,
        "UNVERIFIED" | "PASS" | "CONFIRMED" | "PENDING_HUMAN" | "人間確認済み" | "FAIL"
    ) || result
        .strip_prefix("N/A: ")
        .is_some_and(|reason| !reason.trim().is_empty())
        || result
            .strip_prefix("対象外（")
            .and_then(|s| s.strip_suffix('）'))
            .is_some_and(|reason| !reason.trim().is_empty() && reason.trim() != "理由")
}

fn is_done_matrix_result(result: &str) -> bool {
    if result == "PASS" || result == "CONFIRMED" || result == "人間確認済み" {
        return true;
    }
    result
        .strip_prefix("N/A: ")
        .is_some_and(|reason| !reason.trim().is_empty())
        || result
            .strip_prefix("対象外（")
            .and_then(|s| s.strip_suffix('）'))
            .is_some_and(|reason| !reason.trim().is_empty() && reason.trim() != "理由")
}

fn design_required(meta: &SpecMeta) -> bool {
    let risk = meta.risk();
    let integration = meta.integration();
    risk == "elevated" || risk == "critical" || integration != "none" || meta.surfaces().len() > 1
}

fn pitchless_micro_eligible(meta: &SpecMeta, design_exists: bool, tasks_exists: bool) -> bool {
    meta.risk() == "standard"
        && meta.integration() == "none"
        && meta.surfaces().len() == 1
        && !design_exists
        && !tasks_exists
}

fn lint_spec_dir(
    spec_dir: &Path,
    allowed_surfaces: &HashSet<String>,
    dirty_paths: Option<&HashMap<String, DirtyPathStatus>>,
) -> Vec<Issue> {
    let mut issues = Vec::new();
    let spec_md = spec_dir.join("spec.md");
    let pitch_md = spec_dir.join("pitch.md");
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
    let allowed_statuses = ["draft", "approved", "accepted", "done"];
    if !allowed_statuses.contains(&meta.status()) {
        issues.push(Issue {
            severity: "FAIL".into(),
            path: meta.path.clone(),
            message: "status must be one of: draft, approved, accepted, done".into(),
        });
    }
    // `done` is a legacy terminal status retained only for archived specs under
    // `_done/`. Active specs settle at `accepted`; the engine never writes `done`
    // for an active spec. Reject `done` on a spec that is not under `_done/`.
    let dir_is_archived = spec_dir.components().any(|c| c.as_os_str() == "_done");
    if meta.status() == "done" && !dir_is_archived {
        issues.push(Issue {
            severity: "FAIL".into(),
            path: meta.path.clone(),
            message: "status: done is reserved for archived specs under _done/; active specs use draft → approved → accepted".into(),
        });
    }
    // `completed` is the completion timestamp recorded at the `done` transition.
    // It drives chronological ordering in INDEX.md. Missing on a done spec is a
    // WARN (legacy specs predate it); a present-but-malformed value is a FAIL.
    match (meta.status(), meta.completed()) {
        ("done", None) => issues.push(Issue {
            severity: "WARN".into(),
            path: meta.path.clone(),
            message: "status is done but `completed` timestamp is missing; same-day INDEX ordering falls back to `updated`".into(),
        }),
        (_, Some(c)) if !is_valid_completed(c) => issues.push(Issue {
            severity: "FAIL".into(),
            path: meta.path.clone(),
            message: format!(
                "`completed` must be an ISO 8601 date or timestamp (YYYY-MM-DD[THH:MM:SSZ]), got `{c}`"
            ),
        }),
        _ => {}
    }

    // Draft has three valid shapes: pitch-only, expanded with pitch, and
    // pitchless micro. Without a stored depth field, an eligible pitchless
    // `spec.md` draft is intentionally treated as micro.
    if meta.status() == "draft" && !pitch_md.exists() {
        if !spec_md.exists() {
            issues.push(Issue {
                severity: "FAIL".into(),
                path: pitch_md.clone(),
                message: "pitch.md is required for draft status".into(),
            });
        } else if !pitchless_micro_eligible(&meta, design_md.exists(), tasks_md.exists()) {
            issues.push(Issue {
                severity: "FAIL".into(),
                path: pitch_md.clone(),
                message: "pitchless draft with spec.md must be micro eligible: risk standard, one surface, integration none, no design.md, no tasks.md".into(),
            });
        }
    }

    // spec.md
    if !spec_md.exists() {
        if meta.status() == "draft" {
            return issues;
        }
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
    push_template_residue_issues(&mut issues, &spec_md, &spec_text);
    let spec_acs = ac_ids_in_spec(&spec_text);
    if spec_text.contains("[NEEDS-CLARIFICATION") {
        let severity = if meta.status() == "draft" {
            "WARN"
        } else {
            "FAIL"
        };
        issues.push(Issue {
            severity: severity.into(),
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
        push_template_residue_issues(&mut issues, &design_md, &design_text);
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
        push_template_residue_issues(&mut issues, &tasks_md, &tt);
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
        tasks_text = Some(tt);
    }

    if meta.status() == "approved"
        && let (Some(tt), Some(dirty_paths)) = (tasks_text.as_deref(), dirty_paths)
    {
        for task in task_progress(tt) {
            let dirty_files = dirty_files_for_unchecked_task(&task, dirty_paths);
            if !dirty_files.is_empty() {
                issues.push(Issue {
                    severity: "WARN".into(),
                    path: tasks_md.clone(),
                    message: format!(
                        "status is approved but task {} has modified Files entries and is not checked: {}",
                        task.id,
                        dirty_files.join(", ")
                    ),
                });
            }
        }
    }

    // AC Matrix guards
    let spec_matrix = matrix_section(&spec_text);
    let spec_matrix_rows = parse_matrix_rows(&spec_text);
    let legacy_matrix_rows = if spec_matrix_rows.is_empty() {
        tasks_text
            .as_deref()
            .map(parse_matrix_rows)
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let using_legacy_matrix = spec_matrix_rows.is_empty() && !legacy_matrix_rows.is_empty();
    let matrix_path = if using_legacy_matrix {
        tasks_md.clone()
    } else {
        spec_md.clone()
    };
    let matrix_rows = if using_legacy_matrix {
        legacy_matrix_rows
    } else {
        spec_matrix_rows
    };
    let matrix = if spec_matrix.is_some() || !matrix_rows.is_empty() {
        Some(())
    } else {
        None
    };
    if matrix.is_some() {
        for row in &matrix_rows {
            if !is_canonical_matrix_result(&row.result) {
                issues.push(Issue {
                    severity: "FAIL".into(),
                    path: matrix_path.clone(),
                    message: format!(
                        "AC Matrix result for {} must be one of UNVERIFIED, PASS, CONFIRMED, PENDING_HUMAN, N/A: <reason>, FAIL (also accepted: 人間確認済み, 対象外（<reason>）)",
                        row.ac
                    ),
                });
            }
        }
    }
    if meta.status() == "approved" && matrix.is_none() {
        issues.push(Issue {
            severity: "FAIL".into(),
            path: spec_md.clone(),
            message: "status is approved but AC Matrix is missing".into(),
        });
    }
    if matches!(meta.status(), "approved" | "accepted" | "done") && matrix.is_some() {
        let matrix_acs: BTreeSet<String> = matrix_rows.iter().map(|row| row.ac.clone()).collect();
        let uncovered: Vec<_> = spec_acs.difference(&matrix_acs).cloned().collect();
        if !uncovered.is_empty() {
            issues.push(Issue {
                severity: "FAIL".into(),
                path: matrix_path.clone(),
                message: format!("AC not present in AC Matrix: {}", uncovered.join(", ")),
            });
        }
    }

    if matches!(meta.status(), "accepted" | "done") {
        let status_word = meta.status();
        if matrix.is_none() {
            issues.push(Issue {
                severity: "FAIL".into(),
                path: spec_dir.join("spec.yaml"),
                message: format!(
                    "status is {status_word} but AC Verification Matrix is missing; AC Matrix is missing"
                ),
            });
        } else {
            for row in &matrix_rows {
                if !is_done_matrix_result(&row.result) {
                    let shown = if row.result.is_empty() {
                        "<empty>"
                    } else {
                        row.result.as_str()
                    };
                    issues.push(Issue {
                        severity: "FAIL".into(),
                        path: spec_dir.join("spec.yaml"),
                        message: format!(
                            "status is {status_word} but AC Matrix row {} has invalid result `{shown}`; expected PASS, CONFIRMED, or N/A: <reason> (also accepted: 人間確認済み, 対象外（<reason>）)",
                            row.ac
                        ),
                    });
                }
            }
        }
        if let Some(ref tt) = tasks_text {
            for task_id in unchecked_task_ids(tt) {
                issues.push(Issue {
                    severity: "FAIL".into(),
                    path: tasks_md.clone(),
                    message: format!("status is {status_word} but task {task_id} is not checked"),
                });
            }
            let untasked: Vec<_> = spec_acs.difference(&ac_ids_in_tasks(tt)).cloned().collect();
            if !untasked.is_empty() {
                issues.push(Issue {
                    severity: "FAIL".into(),
                    path: tasks_md.clone(),
                    message: format!(
                        "AC not covered by any task Covers AC: {}",
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
            let verdicts = review_verdicts(&design_text);
            let latest_verdict = latest_review_verdict(&design_text);
            if latest_verdict.as_deref() == Some("fail") {
                issues.push(Issue {
                    severity: "FAIL".into(),
                    path: spec_dir.join("spec.yaml"),
                    message: format!(
                        "status is {status_word} but latest Review Results reviewer Verdict is fail"
                    ),
                });
            } else if latest_verdict.is_none() {
                issues.push(Issue { severity: "FAIL".into(), path: spec_dir.join("spec.yaml"), message: format!("status is {status_word} but reviewer verdict (pass/pass-with-comments) is not recorded in design.md ## Review Results for risk>=elevated") });
            }
            let pass_count = verdicts
                .iter()
                .filter(|v| v.as_str() == "pass" || v.as_str() == "pass-with-comments")
                .count();
            if meta.risk() == "critical" {
                let required = task_count(tasks_text.as_deref());
                if required > 0 && pass_count < required {
                    issues.push(Issue {
                        severity: "FAIL".into(),
                        path: spec_dir.join("spec.yaml"),
                        message: format!(
                            "status is {status_word} but critical risk requires at least {required} passing reviewer verdict(s), found {pass_count}"
                        ),
                    });
                }
            }
        }
    }

    if let Some(ref tt) = tasks_text
        && (meta.status() != "done" || !task_ids(tt).is_empty())
    {
        for message in lint_task_structure(tt) {
            issues.push(Issue {
                severity: "FAIL".into(),
                path: tasks_md.clone(),
                message,
            });
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

fn is_explicit_spec_path(target: &str) -> bool {
    let path = Path::new(target);
    path.is_absolute() || path.exists() || path.components().count() > 1
}

fn resolve_lint_targets(cfg: &Config, target: &str) -> Result<Vec<PathBuf>, Issue> {
    if is_explicit_spec_path(target) {
        let path = PathBuf::from(target);
        let spec_dir = if path.is_file() {
            path.parent().unwrap_or(&path).to_path_buf()
        } else {
            path
        };
        if spec_dir.join("spec.yaml").exists() {
            return Ok(vec![spec_dir]);
        }
        return Err(Issue {
            severity: "FAIL".into(),
            path: spec_dir.join("spec.yaml"),
            message: format!("spec not found: {target}"),
        });
    }

    let active = cfg.specs_dir_path().join(target);
    let archived = cfg.specs_dir_path().join("_done").join(target);
    let mut matches = Vec::new();
    for candidate in [&active, &archived] {
        if candidate.join("spec.yaml").exists() {
            matches.push(candidate.to_path_buf());
        }
    }

    match matches.len() {
        0 => Err(Issue {
            severity: "FAIL".into(),
            path: active.join("spec.yaml"),
            message: format!("spec not found: {target}"),
        }),
        1 => Ok(matches),
        _ => Err(Issue {
            severity: "FAIL".into(),
            path: active.join("spec.yaml"),
            message: format!(
                "spec target is ambiguous: {target} matches {} and {}",
                active.display(),
                archived.display()
            ),
        }),
    }
}

/// Run lint, print issues, return exit code (0=pass, 1=fail).
pub fn run_lint(cfg: &Config, spec_slug: Option<&str>, log_to_stderr: bool) -> i32 {
    macro_rules! report_ln {
        ($($arg:tt)*) => {
            if log_to_stderr { eprintln!($($arg)*) } else { println!($($arg)*) }
        };
    }
    let allowed: HashSet<String> = cfg.surface_names().into_iter().collect();
    let mut all_issues = Vec::new();
    let paths = if let Some(target) = spec_slug {
        match resolve_lint_targets(cfg, target) {
            Ok(paths) => paths,
            Err(issue) => {
                all_issues.push(issue);
                Vec::new()
            }
        }
    } else {
        vec![cfg.specs_dir_path()]
    };

    let dirty_paths = dirty_worktree_paths(&cfg.repo_root);
    let mut seen = HashSet::new();
    for root in &paths {
        let spec_dirs = discover_spec_dirs(root);
        for spec_dir in spec_dirs {
            if seen.contains(&spec_dir) {
                continue;
            }
            seen.insert(spec_dir.clone());
            all_issues.extend(lint_spec_dir(&spec_dir, &allowed, dirty_paths.as_ref()));
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

#[cfg(test)]
mod tests {
    use super::{
        DirtyPathStatus, TaskFile, TaskProgress, dirty_files_for_unchecked_task,
        files_for_task_body,
    };
    use std::collections::HashMap;

    fn dirty_status(deleted: bool) -> DirtyPathStatus {
        DirtyPathStatus { deleted }
    }

    #[test]
    fn deleted_files_entries_mark_every_path_on_the_line() {
        let files = files_for_task_body(
            "  - Files:\n    - deleted: `src/a.rs`, `src/b.rs`\n    - `src/c.rs`\n",
        );

        let planned: Vec<_> = files
            .iter()
            .map(|file| (file.path.as_str(), file.planned_deletion))
            .collect();
        assert_eq!(
            planned,
            vec![("src/a.rs", true), ("src/b.rs", true), ("src/c.rs", false),]
        );
    }

    #[test]
    fn dirty_deleted_path_is_not_reported_for_planned_deletion_entry() {
        let task = TaskProgress {
            id: "T-001".into(),
            checked: false,
            files: vec![TaskFile {
                path: "src/old.rs".into(),
                planned_deletion: true,
            }],
        };
        let dirty_paths = HashMap::from([("src/old.rs".into(), dirty_status(true))]);

        assert!(dirty_files_for_unchecked_task(&task, &dirty_paths).is_empty());
    }

    #[test]
    fn dirty_non_deleted_path_is_reported_for_planned_deletion_entry() {
        let task = TaskProgress {
            id: "T-001".into(),
            checked: false,
            files: vec![TaskFile {
                path: "src/old.rs".into(),
                planned_deletion: true,
            }],
        };
        let dirty_paths = HashMap::from([("src/old.rs".into(), dirty_status(false))]);

        assert_eq!(
            dirty_files_for_unchecked_task(&task, &dirty_paths),
            vec!["src/old.rs"]
        );
    }
}
