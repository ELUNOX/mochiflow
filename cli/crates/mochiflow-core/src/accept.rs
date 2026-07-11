//! Deterministic accept close-out mechanics.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::adr::{self, AdrKind};
use crate::config::Config;
use crate::lint;
use crate::spec_meta::{SpecMeta, read_spec_metadata};
use crate::spec_mode::{SpecPersistence, SpecPersistenceMode, classify_spec_dir};

const EXIT_OK: i32 = 0;
const EXIT_FAIL: i32 = 1;

#[derive(Debug, Clone)]
struct Target {
    slug: String,
    active_dir: PathBuf,
    done_dir: PathBuf,
}

#[derive(Debug)]
struct MatrixRow {
    line_index: usize,
    ac: String,
    scope: String,
    method: String,
    result: String,
    evidence: String,
    result_idx: usize,
    evidence_idx: usize,
    cells: Vec<String>,
}

#[derive(Debug)]
struct StatusEntry {
    index: char,
    worktree: char,
    paths: Vec<String>,
}

#[derive(Debug)]
struct AcceptPaths {
    dir_prefixes: BTreeSet<String>,
    exact_files: BTreeSet<String>,
    rejected_files: BTreeSet<String>,
}

/// Entry point for `mochiflow accept`. Returns the process exit code.
pub fn run_accept(cfg: &Config, slug_arg: Option<&str>, dry_run: bool) -> i32 {
    if let Err(error) = cfg.validate_repository_paths_now() {
        eprintln!("FAIL: {error}");
        return EXIT_FAIL;
    }
    let target = match resolve_target(cfg, slug_arg) {
        Ok(target) => target,
        Err(message) => {
            eprintln!("FAIL: {message}");
            return EXIT_FAIL;
        }
    };
    let active_exists = target.active_dir.exists();
    let done_exists = target.done_dir.exists();
    if active_exists && done_exists {
        eprintln!(
            "FAIL: both active and archived spec directories exist for `{}`; remove one before retrying.",
            target.slug
        );
        return EXIT_FAIL;
    }
    if !active_exists && !done_exists {
        eprintln!("FAIL: spec `{}` was not found.", target.slug);
        return EXIT_FAIL;
    }
    if !active_exists {
        eprintln!(
            "FAIL: spec `{}` is archived under _done/; accept operates on active flat specs.",
            target.slug
        );
        return EXIT_FAIL;
    }

    let meta = match read_spec_metadata(&target.active_dir) {
        Ok(meta) => meta,
        Err(e) => {
            eprintln!("FAIL: could not read spec metadata: {e}");
            return EXIT_FAIL;
        }
    };
    let persistence = match classify_spec_dir(cfg, &target.active_dir) {
        Ok(persistence) => persistence,
        Err(message) => {
            eprintln!("FAIL: {message}");
            return EXIT_FAIL;
        }
    };

    let status_entries = match git_status_z(&cfg.repo_root) {
        Ok(entries) => entries,
        Err(message) => {
            eprintln!("FAIL: {message}");
            return EXIT_FAIL;
        }
    };
    let (accept_paths, mut blockers) = match accept_paths(cfg, &target, &status_entries) {
        Ok(paths) => paths,
        Err(message) => {
            eprintln!("FAIL: {message}");
            return EXIT_FAIL;
        }
    };
    blockers.extend(unexpected_status_paths(&status_entries, &accept_paths));
    let readiness = readiness_blockers(cfg, &target, &meta);

    if dry_run {
        println!("accept target : {}", target.slug);
        println!("state       : active");
        println!("persistence: {}", persistence.mode.as_str());
        println!("persistence reason: {}", persistence.reason);
        println!("planned actions:");
        println!("  - run final verification for declared surfaces");
        println!("  - update automated AC Matrix rows when eligible");
        println!("  - set accepted metadata (no _done move, no INDEX write)");
        match persistence.mode {
            SpecPersistenceMode::Tracked => {
                println!("planned stage paths:");
                for path in accept_paths.stage_path_strings() {
                    println!("  - {path}");
                }
                println!(
                    "  - stage the target spec and linked ADR fold records, then create the close-out commit"
                );
            }
            SpecPersistenceMode::Local => {
                println!(
                    "  - skip close-out commit, spec staging, and ADR staging because local mode keeps spec artifacts ignored"
                );
            }
        }
        if blockers.is_empty() && readiness.is_empty() {
            println!("readiness blockers: none");
        } else {
            println!("readiness blockers:");
            for blocker in blockers.iter().chain(readiness.iter()) {
                println!("  - {blocker}");
            }
        }
        println!("dry-run: no verification, mutation, staging, or commit performed.");
        return if blockers.is_empty() && readiness.is_empty() {
            EXIT_OK
        } else {
            EXIT_FAIL
        };
    }

    if !blockers.is_empty() {
        eprintln!("FAIL: unrelated working tree or staged changes exist:");
        for blocker in blockers {
            eprintln!("  - {blocker}");
        }
        return EXIT_FAIL;
    }
    if !readiness.is_empty() {
        eprintln!("FAIL: spec is not ready to accept:");
        for blocker in readiness {
            eprintln!("  - {blocker}");
        }
        return EXIT_FAIL;
    }

    let verify_evidence = match run_final_verification(cfg, &meta) {
        Ok(evidence) => evidence,
        Err(message) => {
            eprintln!("FAIL: {message}");
            return EXIT_FAIL;
        }
    };

    if let Err(message) = update_spec_md(&target.active_dir.join("spec.md"), &verify_evidence) {
        eprintln!("FAIL: {message}");
        return EXIT_FAIL;
    }
    let updated = utc_now_timestamp();
    if let Err(message) = update_spec_yaml(&target.active_dir.join("spec.yaml"), &updated) {
        eprintln!("FAIL: {message}");
        return EXIT_FAIL;
    }
    // The spec stays flat at its active path: no `_done/` move, no `done` write,
    // and no INDEX regeneration in the close-out (the board is gitignored and
    // refreshed by the shared post-command step).
    if lint::run_lint(cfg, Some(&target.slug), true) != 0 {
        eprintln!(
            "FAIL: lint failed after setting accepted; lifecycle files are left for inspection."
        );
        return EXIT_FAIL;
    }
    match persistence.mode {
        SpecPersistenceMode::Tracked => stage_validate_commit(cfg, &target, &accept_paths, &meta),
        SpecPersistenceMode::Local => finish_local_accept(&target, &persistence),
    }
}

pub fn is_path_like_spec_arg(value: &str) -> bool {
    let path = PathBuf::from(value);
    path.is_absolute() || value.contains('/') || value.contains(std::path::MAIN_SEPARATOR)
}

pub fn validate_pr_spec_closeout_committed(cfg: &Config, slug: &str) -> Result<(), String> {
    let target = target_for_slug(cfg, slug);
    if !target.active_dir.exists() {
        return Err(format!(
            "pre-flight FAIL: spec `{slug}` was not found at {}.",
            target.active_dir.display()
        ));
    }
    let meta = read_spec_metadata(&target.active_dir)
        .map_err(|e| format!("pre-flight FAIL: could not read spec `{slug}`: {e}"))?;
    if meta.status() != "accepted" {
        return Err(format!(
            "pre-flight FAIL: spec `{slug}` status is `{}`; run `mochiflow accept {slug}` to reach `accepted` before PR handoff.",
            meta.status()
        ));
    }
    let active_rel = rel_path(&cfg.repo_root, &target.active_dir);
    let grep = format!("^Spec: {}$", regex::escape(slug));
    let output = Command::new("git")
        .args([
            "log",
            "--format=%H",
            "--max-count=1",
            "--extended-regexp",
            "--grep",
            &grep,
            "--",
        ])
        .arg(&active_rel)
        .current_dir(&cfg.repo_root)
        .output()
        .map_err(|e| format!("pre-flight FAIL: git log failed: {e}"))?;
    if !output.status.success() || output.stdout.is_empty() {
        return Err(format!(
            "pre-flight FAIL: spec `{slug}` is not committed with a `Spec: {slug}` trailer."
        ));
    }
    Ok(())
}

pub fn validate_pr_local_acceptance(cfg: &Config, slug: &str) -> Result<(), String> {
    let target = target_for_slug(cfg, slug);
    if !target.active_dir.exists() {
        return Err(format!(
            "pre-flight FAIL: spec `{slug}` was not found at {}.",
            target.active_dir.display()
        ));
    }
    let meta = read_spec_metadata(&target.active_dir)
        .map_err(|e| format!("pre-flight FAIL: could not read spec `{slug}`: {e}"))?;
    let mut blockers = Vec::new();
    if meta.status() != "accepted" {
        blockers.push(format!(
            "local spec status is `{}`; expected `accepted`",
            meta.status()
        ));
    }
    if lint::run_lint(cfg, Some(slug), true) != 0 {
        blockers.push("lint failed for local accepted spec".to_string());
    }
    blockers.extend(matrix_completion_blockers(
        &target.active_dir.join("spec.md"),
    ));
    if risk_order(meta.risk()) >= 1 && !has_passing_review(&target.active_dir.join("design.md")) {
        blockers.push(
            "reviewer verdict (pass/pass-with-comments) is not recorded in design.md ## Review Results"
                .to_string(),
        );
    }
    if blockers.is_empty() {
        return Ok(());
    }
    let mut message =
        "pre-flight FAIL: local accepted state or verification evidence is incomplete:".to_string();
    for blocker in blockers {
        message.push_str("\n  - ");
        message.push_str(&blocker);
    }
    Err(message)
}

fn stage_validate_commit(
    cfg: &Config,
    target: &Target,
    accept_paths: &AcceptPaths,
    meta: &SpecMeta,
) -> i32 {
    if let Err(message) = git_add_accept_paths(cfg, accept_paths) {
        eprintln!("FAIL: {message}");
        return EXIT_FAIL;
    }
    let staged = match git_cached_name_status_z(&cfg.repo_root) {
        Ok(staged) => staged,
        Err(message) => {
            eprintln!("FAIL: {message}");
            return EXIT_FAIL;
        }
    };
    let unexpected: Vec<_> = staged
        .iter()
        .flat_map(|entry| entry.paths.iter())
        .filter(|path| !accept_paths.allows(path))
        .cloned()
        .collect();
    if !unexpected.is_empty() {
        eprintln!("FAIL: staged result includes unexpected paths:");
        for path in unexpected {
            eprintln!("  - {path}");
        }
        return EXIT_FAIL;
    }
    if staged.is_empty() {
        println!(
            "accept: no lifecycle changes are staged for `{}`.",
            target.slug
        );
        return EXIT_OK;
    }
    let subject = format!(
        "{}: complete delivery record",
        commit_type(meta.spec_type())
    );
    let trailer = format!("Spec: {}", target.slug);
    let status = Command::new("git")
        .args(["commit", "-m", &subject, "-m", &trailer])
        .current_dir(&cfg.repo_root)
        .status();
    match status {
        Ok(status) if status.success() => {
            println!("accept: committed close-out for `{}`.", target.slug);
            EXIT_OK
        }
        Ok(_) => {
            eprintln!("FAIL: git commit failed.");
            EXIT_FAIL
        }
        Err(e) => {
            eprintln!("FAIL: could not run git commit: {e}");
            EXIT_FAIL
        }
    }
}

fn finish_local_accept(target: &Target, persistence: &SpecPersistence) -> i32 {
    println!(
        "accept: local mode for `{}`; final verification passed and local spec metadata is accepted.",
        target.slug
    );
    println!(
        "accept: skipped close-out commit, spec staging, and ADR staging: {}",
        persistence.reason
    );
    EXIT_OK
}

fn resolve_target(cfg: &Config, slug_arg: Option<&str>) -> Result<Target, String> {
    if let Some(slug) = slug_arg {
        if slug.trim().is_empty() || is_path_like_spec_arg(slug) {
            return Err("accept expects a spec slug, not a path.".to_string());
        }
        return Ok(target_for_slug(cfg, slug));
    }
    let branch = git_capture(&cfg.repo_root, &["branch", "--show-current"])
        .ok_or_else(|| "could not determine current branch; pass an explicit slug.".to_string())?;
    let slug = branch
        .split_once('/')
        .and_then(|(prefix, slug)| {
            matches!(prefix, "feat" | "fix" | "refactor" | "docs" | "chore").then_some(slug)
        })
        .ok_or_else(|| {
            format!("current branch `{branch}` does not identify a spec; pass an explicit slug.")
        })?;
    Ok(target_for_slug(cfg, slug))
}

fn target_for_slug(cfg: &Config, slug: &str) -> Target {
    let specs_dir = cfg.specs_dir_path();
    Target {
        slug: slug.to_string(),
        active_dir: specs_dir.join(slug),
        done_dir: specs_dir.join("_done").join(slug),
    }
}

fn readiness_blockers(cfg: &Config, target: &Target, meta: &SpecMeta) -> Vec<String> {
    let mut blockers = Vec::new();
    // accept operates only on active flat specs (the archived path is rejected
    // earlier in run_accept), so readiness always targets the active spec dir.
    let spec_dir = &target.active_dir;
    if meta.status() != "approved" {
        blockers.push(format!(
            "active spec status is `{}`; expected `approved`",
            meta.status()
        ));
    }
    for surface in meta.surfaces() {
        match cfg.verify_command(surface, "default", None) {
            Ok(cmd) if cmd.trim().is_empty() || cmd.trim_start().starts_with("TODO") => blockers
                .push(format!(
                    "verification command for surface `{surface}` is not runnable"
                )),
            Ok(_) => {}
            Err(e) => blockers.push(e.to_string()),
        }
    }
    if lint::run_lint(cfg, Some(&target.slug), true) != 0 {
        blockers.push("lint failed before mutation".to_string());
    }
    blockers.extend(matrix_completion_blockers(&spec_dir.join("spec.md")));
    if risk_order(meta.risk()) >= 1 && !has_passing_review(&spec_dir.join("design.md")) {
        blockers.push(
            "reviewer verdict (pass/pass-with-comments) is not recorded in design.md ## Review Results"
                .to_string(),
        );
    }
    blockers
}

fn matrix_completion_blockers(spec_md: &Path) -> Vec<String> {
    let text = fs::read_to_string(spec_md).unwrap_or_default();
    parse_matrix(&text)
        .into_iter()
        .filter_map(|row| match row.result.as_str() {
            "PASS" | "CONFIRMED" => None,
            result if result.starts_with("N/A: ") => None,
            other => Some(format!(
                "AC Matrix row {} has result `{}` and cannot be completed by final verification",
                row.ac, other
            )),
        })
        .collect()
}

fn run_final_verification(cfg: &Config, meta: &SpecMeta) -> Result<Vec<(String, String)>, String> {
    let mut evidence = Vec::new();
    for surface in meta.surfaces() {
        let cmd = cfg
            .verify_command(surface, "default", None)
            .map_err(|e| e.to_string())?;
        if cmd.trim().is_empty() || cmd.trim_start().starts_with("TODO") {
            return Err(format!(
                "verification command for surface `{surface}` is not runnable"
            ));
        }
        let status = Command::new("sh")
            .args(["-c", &cmd])
            .current_dir(&cfg.repo_root)
            .status()
            .map_err(|e| format!("could not run verification for `{surface}`: {e}"))?;
        if !status.success() {
            return Err(format!(
                "verification failed for surface `{surface}`: `{cmd}`"
            ));
        }
        evidence.push((surface.to_string(), cmd));
    }
    Ok(evidence)
}

fn update_spec_md(spec_md: &Path, verify_evidence: &[(String, String)]) -> Result<(), String> {
    let text = fs::read_to_string(spec_md)
        .map_err(|e| format!("could not read {}: {e}", spec_md.display()))?;
    let rows = parse_matrix(&text);
    if rows.is_empty() {
        return Err("AC Matrix is missing or unreadable".to_string());
    }
    let mut lines: Vec<String> = text.lines().map(ToString::to_string).collect();
    for row in rows {
        if row.result != "PASS" || row.method != "automated" {
            continue;
        }
        let Some((_, cmd)) = verify_evidence
            .iter()
            .find(|(surface, _)| surface == &row.scope)
        else {
            continue;
        };
        let mut cells = row.cells;
        cells[row.result_idx] = "PASS".to_string();
        let final_evidence = format!("final verification: `{}`", escape_table_cell(cmd));
        cells[row.evidence_idx] = if row.evidence.trim().is_empty() {
            final_evidence
        } else if row.evidence.contains(&final_evidence) {
            row.evidence
        } else {
            format!("{}<br>{final_evidence}", row.evidence)
        };
        lines[row.line_index] = format!("| {} |", cells.join(" | "));
    }
    fs::write(spec_md, lines.join("\n") + "\n")
        .map_err(|e| format!("could not write {}: {e}", spec_md.display()))
}

fn update_spec_yaml(spec_yaml: &Path, timestamp: &str) -> Result<(), String> {
    let text = fs::read_to_string(spec_yaml)
        .map_err(|e| format!("could not read {}: {e}", spec_yaml.display()))?;
    let date = timestamp.split('T').next().unwrap_or(timestamp);
    let mut saw_status = false;
    let mut saw_updated = false;
    let mut lines = Vec::new();
    for line in text.lines() {
        if line.starts_with("status:") {
            lines.push("status: \"accepted\"".to_string());
            saw_status = true;
        } else if line.starts_with("updated:") {
            lines.push(format!("updated: \"{date}\""));
            saw_updated = true;
        } else {
            // `accepted` never writes `completed`; any existing line is left as-is.
            lines.push(line.to_string());
        }
    }
    if !saw_status {
        lines.push("status: \"accepted\"".to_string());
    }
    if !saw_updated {
        lines.push(format!("updated: \"{date}\""));
    }
    fs::write(spec_yaml, lines.join("\n") + "\n")
        .map_err(|e| format!("could not write {}: {e}", spec_yaml.display()))
}

fn parse_matrix(text: &str) -> Vec<MatrixRow> {
    let lines: Vec<&str> = text.lines().collect();
    let Some(start) = lines.iter().position(|line| {
        matches!(
            line.trim(),
            "## Verification Plan / AC Matrix" | "## AC Matrix" | "## AC Verification Matrix"
        )
    }) else {
        return Vec::new();
    };
    let end = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find(|(_, line)| line.starts_with("## "))
        .map(|(idx, _)| idx)
        .unwrap_or(lines.len());
    let mut table_lines = Vec::new();
    for (idx, line) in lines.iter().enumerate().take(end).skip(start + 1) {
        let trimmed = line.trim();
        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            table_lines.push((idx, *line));
        }
    }
    let Some((_, header)) = table_lines.first() else {
        return Vec::new();
    };
    let headers = split_table_row(header);
    let ac_idx = headers.iter().position(|cell| cell == "AC");
    let scope_idx = headers.iter().position(|cell| cell == "Scope");
    let method_idx = headers
        .iter()
        .position(|cell| cell == "Verification method");
    let result_idx = headers.iter().position(|cell| cell == "Result");
    let evidence_idx = headers.iter().position(|cell| cell == "Evidence");
    let (Some(ac_idx), Some(scope_idx), Some(method_idx), Some(result_idx), Some(evidence_idx)) =
        (ac_idx, scope_idx, method_idx, result_idx, evidence_idx)
    else {
        return Vec::new();
    };
    table_lines
        .into_iter()
        .skip(1)
        .filter_map(|(line_index, line)| {
            let cells = split_table_row(line);
            if cells
                .iter()
                .all(|cell| cell.chars().all(|c| c == '-' || c == ':'))
            {
                return None;
            }
            let ac = cells.get(ac_idx)?.to_string();
            if !ac.starts_with("AC-") {
                return None;
            }
            Some(MatrixRow {
                line_index,
                ac,
                scope: cells.get(scope_idx).cloned().unwrap_or_default(),
                method: cells.get(method_idx).cloned().unwrap_or_default(),
                result: cells.get(result_idx).cloned().unwrap_or_default(),
                evidence: cells.get(evidence_idx).cloned().unwrap_or_default(),
                result_idx,
                evidence_idx,
                cells,
            })
        })
        .collect()
}

fn split_table_row(line: &str) -> Vec<String> {
    let inner = line
        .trim()
        .strip_prefix('|')
        .unwrap_or(line.trim())
        .strip_suffix('|')
        .unwrap_or_else(|| line.trim().strip_prefix('|').unwrap_or(line.trim()));
    let mut cells = Vec::new();
    let mut cell = String::new();
    let mut escaped = false;
    for ch in inner.chars() {
        if ch == '|' && !escaped {
            cells.push(cell.trim().to_string());
            cell.clear();
            escaped = false;
            continue;
        }
        cell.push(ch);
        escaped = ch == '\\' && !escaped;
        if ch != '\\' {
            escaped = false;
        }
    }
    cells.push(cell.trim().to_string());
    cells
}

fn escape_table_cell(value: &str) -> String {
    value.replace('|', "\\|")
}

fn accept_paths(
    cfg: &Config,
    target: &Target,
    entries: &[StatusEntry],
) -> Result<(AcceptPaths, Vec<String>), String> {
    let mut paths = AcceptPaths {
        dir_prefixes: BTreeSet::from([path_to_string(&rel_path(
            &cfg.repo_root,
            &target.active_dir,
        ))]),
        exact_files: BTreeSet::new(),
        rejected_files: BTreeSet::new(),
    };
    let dirty_paths: BTreeSet<String> = entries
        .iter()
        .flat_map(|entry| entry.paths.iter().cloned())
        .collect();
    let mut blockers = Vec::new();
    let mut records = Vec::new();
    for kind in [AdrKind::Decisions, AdrKind::Pitfalls] {
        let store = adr::load_store(cfg, kind)
            .map_err(|e| format!("could not load ADR {} store: {e}", kind.as_str()))?;
        for problem in &store.problems {
            if let Some(problem_path) = adr_problem_path(cfg, kind, problem.id.as_deref())
                && dirty_paths.contains(&problem_path)
            {
                paths.rejected_files.insert(problem_path.clone());
                blockers.push(format!(
                    "ADR record cannot be parsed for accept: {problem_path}: {}",
                    problem.message
                ));
            }
        }
        records.extend(store.records);
    }

    let mut selected_ids = BTreeSet::new();
    for record in records
        .iter()
        .filter(|record| record.spec.as_deref() == Some(target.slug.as_str()))
    {
        selected_ids.insert(record.id.clone());
        if let Some(id) = &record.supersedes {
            selected_ids.insert(id.clone());
        }
        if let Some(id) = &record.superseded_by {
            selected_ids.insert(id.clone());
        }
    }

    let mut changed = true;
    while changed {
        changed = false;
        let linked_ids: Vec<String> = records
            .iter()
            .filter(|record| selected_ids.contains(record.id.as_str()))
            .flat_map(|record| [&record.supersedes, &record.superseded_by])
            .flatten()
            .cloned()
            .collect();
        for id in linked_ids {
            if selected_ids.insert(id) {
                changed = true;
            }
        }
    }

    for record in records
        .iter()
        .filter(|record| selected_ids.contains(record.id.as_str()))
    {
        paths
            .exact_files
            .insert(path_to_string(&rel_path(&cfg.repo_root, &record.path)));
    }

    Ok((paths, blockers))
}

fn adr_problem_path(cfg: &Config, kind: AdrKind, id: Option<&str>) -> Option<String> {
    let id = id?;
    Some(path_to_string(&rel_path(
        &cfg.repo_root,
        &kind.dir(cfg).join(format!("{id}.md")),
    )))
}

impl AcceptPaths {
    fn allows(&self, path: &str) -> bool {
        self.exact_files.contains(path)
            || self
                .dir_prefixes
                .iter()
                .any(|allowed| path == allowed || path.starts_with(&format!("{allowed}/")))
    }

    fn stage_path_strings(&self) -> Vec<String> {
        self.dir_prefixes
            .iter()
            .chain(self.exact_files.iter())
            .cloned()
            .collect()
    }

    fn stage_paths(&self) -> Vec<PathBuf> {
        self.stage_path_strings()
            .into_iter()
            .map(PathBuf::from)
            .collect()
    }
}

fn unexpected_status_paths(entries: &[StatusEntry], accept_paths: &AcceptPaths) -> Vec<String> {
    entries
        .iter()
        .flat_map(|entry| {
            entry
                .paths
                .iter()
                .filter(|path| !accept_paths.rejected_files.contains(*path))
                .filter(|path| !accept_paths.allows(path))
                .map(move |path| format!("{}{} {path}", entry.index, entry.worktree))
        })
        .collect()
}

fn git_add_accept_paths(cfg: &Config, accept_paths: &AcceptPaths) -> Result<(), String> {
    let paths: Vec<_> = accept_paths
        .stage_paths()
        .into_iter()
        .filter(|path| {
            cfg.repo_root.join(path).exists() || git_path_exists_in_index(&cfg.repo_root, path)
        })
        .collect();
    let status = Command::new("git")
        .arg("add")
        .arg("-A")
        .args(paths.iter())
        .current_dir(&cfg.repo_root)
        .status()
        .map_err(|e| format!("could not run git add: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("git add -A for accept paths failed".to_string())
    }
}

fn git_path_exists_in_index(root: &Path, path: &Path) -> bool {
    Command::new("git")
        .args(["ls-files", "--error-unmatch", "--"])
        .arg(path)
        .current_dir(root)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn git_status_z(root: &Path) -> Result<Vec<StatusEntry>, String> {
    let output = Command::new("git")
        .args(["status", "--porcelain=v1", "-z", "-uall"])
        .current_dir(root)
        .output()
        .map_err(|e| format!("could not run git status: {e}"))?;
    if !output.status.success() {
        return Err("git status failed".to_string());
    }
    Ok(parse_porcelain_z(&output.stdout))
}

fn git_cached_name_status_z(root: &Path) -> Result<Vec<StatusEntry>, String> {
    let output = Command::new("git")
        .args(["diff", "--cached", "--name-status", "-z"])
        .current_dir(root)
        .output()
        .map_err(|e| format!("could not run git diff --cached: {e}"))?;
    if !output.status.success() {
        return Err("git diff --cached failed".to_string());
    }
    Ok(parse_name_status_z(&output.stdout))
}

fn parse_porcelain_z(bytes: &[u8]) -> Vec<StatusEntry> {
    let fields: Vec<&[u8]> = bytes
        .split(|b| *b == 0)
        .filter(|field| !field.is_empty())
        .collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < fields.len() {
        let field = fields[i];
        if field.len() < 4 {
            i += 1;
            continue;
        }
        let index = field[0] as char;
        let worktree = field[1] as char;
        let mut paths = vec![String::from_utf8_lossy(&field[3..]).to_string()];
        if (matches!(index, 'R' | 'C') || matches!(worktree, 'R' | 'C'))
            && let Some(next) = fields.get(i + 1)
        {
            paths.push(String::from_utf8_lossy(next).to_string());
            i += 1;
        }
        out.push(StatusEntry {
            index,
            worktree,
            paths,
        });
        i += 1;
    }
    out
}

fn parse_name_status_z(bytes: &[u8]) -> Vec<StatusEntry> {
    let fields: Vec<&[u8]> = bytes
        .split(|b| *b == 0)
        .filter(|field| !field.is_empty())
        .collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < fields.len() {
        let status = String::from_utf8_lossy(fields[i]).to_string();
        i += 1;
        let mut paths = vec![String::from_utf8_lossy(fields[i]).to_string()];
        i += 1;
        if (status.starts_with('R') || status.starts_with('C'))
            && let Some(next) = fields.get(i)
        {
            paths.push(String::from_utf8_lossy(next).to_string());
            i += 1;
        }
        let index = status.chars().next().unwrap_or(' ');
        out.push(StatusEntry {
            index,
            worktree: ' ',
            paths,
        });
    }
    out
}

fn git_capture(root: &Path, args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

fn rel_path(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn has_passing_review(design_md: &Path) -> bool {
    let text = fs::read_to_string(design_md).unwrap_or_default();
    let Some(review_results) = markdown_section(&text, "Review Results") else {
        return false;
    };
    review_results
        .lines()
        .filter_map(|line| {
            line.trim()
                .trim_start_matches("- ")
                .strip_prefix("Verdict:")
        })
        .map(|verdict| verdict.trim().to_ascii_lowercase())
        .next_back()
        .is_some_and(|verdict| verdict == "pass" || verdict == "pass-with-comments")
}

fn markdown_section<'a>(text: &'a str, heading: &str) -> Option<&'a str> {
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

fn risk_order(risk: &str) -> u8 {
    match risk {
        "elevated" => 1,
        "critical" => 2,
        _ => 0,
    }
}

fn commit_type(spec_type: &str) -> &str {
    match spec_type {
        "feature" => "feat",
        "fix" => "fix",
        "refactor" => "refactor",
        "docs" => "docs",
        "chore" => "chore",
        _ => "chore",
    }
}

fn utc_now_timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = rem / 3_600;
    let minute = (rem % 3_600) / 60;
    let second = rem % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if m <= 2 { 1 } else { 0 };
    (year, m, d)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::{
        accept_paths, git_add_accept_paths, has_passing_review, parse_name_status_z,
        target_for_slug, update_spec_md,
    };
    use crate::config::load_config;

    #[test]
    fn has_passing_review_uses_latest_verdict() {
        let tmp = tempfile::tempdir().unwrap();
        let design = tmp.path().join("design.md");
        std::fs::write(
            &design,
            "# Design\n\n## Review Results\n\n- Reviewer mode: delegated\n- Verdict: fail\n\n- Reviewer mode: delegated\n- Verdict: pass\n",
        )
        .unwrap();
        assert!(has_passing_review(&design));

        std::fs::write(
            &design,
            "# Design\n\n## Review Results\n\n- Reviewer mode: delegated\n- Verdict: pass\n\n- Reviewer mode: delegated\n- Verdict: fail\n",
        )
        .unwrap();
        assert!(!has_passing_review(&design));

        std::fs::write(
            &design,
            "# Design\n\nVerdict: pass\n\n## Review Results\n\n- Reviewer mode: delegated\n- Verdict: fail\n",
        )
        .unwrap();
        assert!(!has_passing_review(&design));
    }

    #[test]
    fn cached_name_status_parser_preserves_spaces_and_rejects_unexpected_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        std::fs::create_dir_all(repo.join(".mochiflow/adr")).unwrap();
        std::fs::create_dir_all(repo.join(".mochiflow/specs")).unwrap();
        std::fs::write(
            repo.join(".mochiflow/config.toml"),
            "schema_version = 1\ninstall_dir = \".mochiflow\"\nspecs_dir = \".mochiflow/specs\"\nindex = \".mochiflow/INDEX.md\"\n\n[constitution]\nproject = \".mochiflow/constitution.md\"\nlocal = \".mochiflow/constitution.local.md\"\n\n[context]\nproduct = \".mochiflow/context/product.md\"\nstructure = \".mochiflow/context/structure.md\"\ntech = \".mochiflow/context/tech.md\"\n\n[adr]\ndecisions = \".mochiflow/adr/decisions\"\npitfalls = \".mochiflow/adr/pitfalls\"\n\n[git]\nbase_branch = \"main\"\n\n[adapter]\ntool = \"agents\"\n\n[surfaces.app]\ndescription = \"app\"\n\n[surfaces.app.verify]\ndefault = \"echo ok\"\n",
        )
        .unwrap();
        let cfg = load_config(&repo.join(".mochiflow/config.toml")).unwrap();
        let target = target_for_slug(&cfg, "sample");
        let (paths, blockers) = accept_paths(&cfg, &target, &[]).unwrap();
        let parsed =
            parse_name_status_z(b"A\0.mochiflow/specs/sample/spec.yaml\0A\0src/with space.rs\0");

        assert!(blockers.is_empty(), "{blockers:?}");
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[1].paths, vec!["src/with space.rs"]);
        assert!(paths.allows(&parsed[0].paths[0]));
        assert!(!paths.allows(&parsed[1].paths[0]));
    }

    #[test]
    fn update_spec_md_appends_final_evidence_only_to_matching_automated_pass_rows() {
        let tmp = tempfile::tempdir().unwrap();
        let spec_md = tmp.path().join("spec.md");
        std::fs::write(
            &spec_md,
            "# S\n\n## Verification Plan / AC Matrix\n\n| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |\n| --- | --- | --- | --- | --- | --- | --- | --- |\n| AC-01 | app | automated | final verify | fixture | PASS | existing evidence |  |\n| AC-02 | app | automated | final verify | fixture | PASS | final verification: `cargo test \\| tee out` |  |\n| AC-03 | app | automated | final verify | fixture | UNVERIFIED | build evidence |  |\n| AC-04 | app | human | QA | fixture | PASS | human evidence |  |\n| AC-05 | app | automated | QA | fixture | CONFIRMED | confirmed evidence |  |\n| AC-06 | app | automated | n/a | fixture | N/A: not relevant | n/a evidence |  |\n| AC-07 | api | automated | final verify | fixture | PASS | api evidence |  |\n",
        )
        .unwrap();

        let evidence = vec![("app".to_string(), "cargo test | tee out".to_string())];
        update_spec_md(&spec_md, &evidence).unwrap();
        update_spec_md(&spec_md, &evidence).unwrap();

        let updated = std::fs::read_to_string(&spec_md).unwrap();
        assert!(
            updated.contains(
                "| AC-01 | app | automated | final verify | fixture | PASS | existing evidence<br>final verification: `cargo test \\| tee out` |  |"
            ),
            "{updated}"
        );
        assert_eq!(
            updated
                .matches("final verification: `cargo test \\| tee out`")
                .count(),
            2
        );
        assert!(updated.contains("| AC-03 | app | automated | final verify | fixture | UNVERIFIED | build evidence |  |"), "{updated}");
        assert!(
            updated.contains("| AC-04 | app | human | QA | fixture | PASS | human evidence |  |"),
            "{updated}"
        );
        assert!(
            updated.contains(
                "| AC-05 | app | automated | QA | fixture | CONFIRMED | confirmed evidence |  |"
            ),
            "{updated}"
        );
        assert!(
            updated.contains(
                "| AC-06 | app | automated | n/a | fixture | N/A: not relevant | n/a evidence |  |"
            ),
            "{updated}"
        );
        assert!(
            updated.contains(
                "| AC-07 | api | automated | final verify | fixture | PASS | api evidence |  |"
            ),
            "{updated}"
        );
    }

    #[test]
    fn accept_staging_includes_linked_adr_records_but_never_index() {
        use std::process::Command;
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(repo)
                    .status()
                    .unwrap()
                    .success(),
                "git {args:?}"
            );
        };
        git(&["init", "-q", "-b", "main"]);
        git(&["config", "user.email", "t@example.com"]);
        git(&["config", "user.name", "Test"]);

        std::fs::create_dir_all(repo.join(".mochiflow/adr/decisions")).unwrap();
        std::fs::create_dir_all(repo.join(".mochiflow/adr/pitfalls")).unwrap();
        std::fs::create_dir_all(repo.join(".mochiflow/specs/sample")).unwrap();
        // The bare `INDEX.md` ignore pattern already covers adr/**/INDEX.md.
        std::fs::write(repo.join(".mochiflow/.gitignore"), "INDEX.md\nstate/\n").unwrap();
        std::fs::write(
            repo.join(".mochiflow/config.toml"),
            "schema_version = 1\ninstall_dir = \".mochiflow\"\nspecs_dir = \".mochiflow/specs\"\nindex = \".mochiflow/INDEX.md\"\n\n[constitution]\nproject = \".mochiflow/constitution.md\"\nlocal = \".mochiflow/constitution.local.md\"\n\n[context]\nproduct = \".mochiflow/context/product.md\"\nstructure = \".mochiflow/context/structure.md\"\ntech = \".mochiflow/context/tech.md\"\n\n[adr]\ndecisions = \".mochiflow/adr/decisions\"\npitfalls = \".mochiflow/adr/pitfalls\"\n\n[git]\nbase_branch = \"main\"\n\n[adapter]\ntool = \"agents\"\n\n[surfaces.app]\ndescription = \"app\"\n\n[surfaces.app.verify]\ndefault = \"echo ok\"\n",
        )
        .unwrap();
        std::fs::write(
            repo.join(".mochiflow/adr/decisions/2026-06-22-x.md"),
            "---\nid: 2026-06-22-x\ndate: 2026-06-22\narea: [app]\nspec: sample\nstatus: active\n---\n## body\n",
        )
        .unwrap();
        std::fs::write(
            repo.join(".mochiflow/adr/decisions/2026-06-22-unrelated.md"),
            "---\nid: 2026-06-22-unrelated\ndate: 2026-06-22\narea: [app]\nspec: other\nstatus: active\n---\n## body\n",
        )
        .unwrap();
        std::fs::write(
            repo.join(".mochiflow/adr/decisions/INDEX.md"),
            "# Decision Records\n",
        )
        .unwrap();

        let cfg = load_config(&repo.join(".mochiflow/config.toml")).unwrap();
        let target = target_for_slug(&cfg, "sample");
        let (paths, blockers) = accept_paths(&cfg, &target, &[]).unwrap();
        assert!(blockers.is_empty(), "{blockers:?}");
        git_add_accept_paths(&cfg, &paths).unwrap();

        let staged = Command::new("git")
            .args(["diff", "--cached", "--name-only"])
            .current_dir(repo)
            .output()
            .unwrap();
        let names = String::from_utf8_lossy(&staged.stdout);
        assert!(
            names.contains(".mochiflow/adr/decisions/2026-06-22-x.md"),
            "record must be staged:\n{names}"
        );
        assert!(
            !names.contains(".mochiflow/adr/decisions/2026-06-22-unrelated.md"),
            "unrelated ADR record must not be staged:\n{names}"
        );
        assert!(
            !names.contains("INDEX.md"),
            "gitignored ADR INDEX.md must never be staged:\n{names}"
        );
    }
}
