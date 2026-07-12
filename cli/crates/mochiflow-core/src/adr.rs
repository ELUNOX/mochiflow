//! Addressable ADR store: per-file decision / pitfall records, a generated
//! gitignored content index, supersession lifecycle, and deterministic lint.
//!
//! The store is directory-rooted (`[adr].decisions` / `[adr].pitfalls`). Each
//! record is one immutable markdown file with a `---` front-matter block and a
//! verbatim body. The `INDEX.md` is a derived cache, never the only read path
//! and never staged.

use std::path::{Path, PathBuf};

use crate::config::{Config, adr_record_files};

/// Which store a record belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdrKind {
    Decisions,
    Pitfalls,
}

impl AdrKind {
    pub fn as_str(self) -> &'static str {
        match self {
            AdrKind::Decisions => "decisions",
            AdrKind::Pitfalls => "pitfalls",
        }
    }

    /// Directory root for this store.
    pub fn dir(self, cfg: &Config) -> PathBuf {
        match self {
            AdrKind::Decisions => cfg.decisions_dir(),
            AdrKind::Pitfalls => cfg.pitfalls_dir(),
        }
    }

    /// Generated, gitignored index path for this store.
    pub fn index_path(self, cfg: &Config) -> PathBuf {
        match self {
            AdrKind::Decisions => cfg.decisions_index(),
            AdrKind::Pitfalls => cfg.pitfalls_index(),
        }
    }

    /// Heading rendered at the top of the store's INDEX.md.
    pub fn index_heading(self) -> &'static str {
        match self {
            AdrKind::Decisions => "Decision Records",
            AdrKind::Pitfalls => "Pitfall Records",
        }
    }

    /// Status tokens valid for this store's records.
    pub fn valid_statuses(self) -> &'static [&'static str] {
        match self {
            AdrKind::Decisions => &["active", "superseded", "deprecated"],
            AdrKind::Pitfalls => &["active", "resolved"],
        }
    }
}

/// Lifecycle status of a record. `Resolved` is pitfalls-only; `Superseded` /
/// `Deprecated` are decisions-only, but the enum carries all of them so a single
/// type serves both stores.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdrStatus {
    Active,
    Superseded,
    Deprecated,
    Resolved,
}

impl AdrStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            AdrStatus::Active => "active",
            AdrStatus::Superseded => "superseded",
            AdrStatus::Deprecated => "deprecated",
            AdrStatus::Resolved => "resolved",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "active" => Some(AdrStatus::Active),
            "superseded" => Some(AdrStatus::Superseded),
            "deprecated" => Some(AdrStatus::Deprecated),
            "resolved" => Some(AdrStatus::Resolved),
            _ => None,
        }
    }

    /// A record is in the active set unless it has been superseded, deprecated,
    /// or (for pitfalls) resolved.
    pub fn is_active(self) -> bool {
        matches!(self, AdrStatus::Active)
    }
}

/// One parsed record.
#[derive(Debug, Clone)]
pub struct AdrRecord {
    pub kind: AdrKind,
    pub path: PathBuf,
    pub id: String,
    pub date: String,
    pub area: Vec<String>,
    pub spec: Option<String>,
    pub status: AdrStatus,
    pub supersedes: Option<String>,
    pub superseded_by: Option<String>,
    pub title: String,
    pub body: String,
}

impl AdrRecord {
    /// File stem (used as the fallback id and for traversal checks).
    pub fn file_stem(&self) -> String {
        self.path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string()
    }
}

/// Classification of a structural problem, used by lint to decide gating vs
/// warning and by readers to skip-with-warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdrProblemKind {
    /// Malformed front-matter, missing required key, unknown status/area.
    Schema,
    /// `superseded_by` points at an id that does not exist.
    DanglingSupersededBy,
    /// `supersedes` points at an id that does not exist.
    DanglingSupersedes,
    /// One-sided cross-reference (supersedes without reciprocal superseded_by).
    MissingCrossRef,
    /// A superseded/deprecated record still advertises `status: active`, or an
    /// active record is referenced as superseded without updating its status.
    Stale,
    /// A record references nothing and is referenced by nothing (informational).
    Orphan,
    /// Supersession cycle (A supersedes B, B supersedes A).
    Cycle,
    /// Record name contains a path separator or `..`.
    PathTraversal,
    /// The store's INDEX.md is absent or out of date.
    IndexStale,
}

impl AdrProblemKind {
    /// Doctor gates only on dangling / missing cross-ref / schema. Orphan,
    /// stale, and index freshness are non-blocking warnings.
    pub fn is_gating(self) -> bool {
        matches!(
            self,
            AdrProblemKind::Schema
                | AdrProblemKind::DanglingSupersededBy
                | AdrProblemKind::DanglingSupersedes
                | AdrProblemKind::MissingCrossRef
                | AdrProblemKind::Cycle
                | AdrProblemKind::PathTraversal
        )
    }
}

/// A structural problem found while loading or linting a store.
#[derive(Debug, Clone)]
pub struct AdrProblem {
    pub kind: AdrProblemKind,
    pub store: AdrKind,
    pub id: Option<String>,
    pub message: String,
}

/// A loaded store: successfully parsed records plus parse-time problems
/// (malformed records are skipped, not fatal — readers warn, lint gates).
#[derive(Debug)]
pub struct AdrStore {
    pub kind: AdrKind,
    pub dir: PathBuf,
    pub records: Vec<AdrRecord>,
    pub problems: Vec<AdrProblem>,
}

impl AdrStore {
    /// Find a record by id.
    pub fn find(&self, id: &str) -> Option<&AdrRecord> {
        self.records.iter().find(|r| r.id == id)
    }

    /// The set of ids that some other record validly supersedes (target exists).
    fn superseded_target_ids(&self) -> std::collections::BTreeSet<String> {
        self.records
            .iter()
            .filter_map(|r| r.supersedes.clone())
            .filter(|target| self.find(target).is_some())
            .collect()
    }

    /// Active set: records whose own status is active AND that are not the
    /// target of a valid `supersedes` from another record (status may lag — the
    /// supersession still wins for active computation; lint flags the lag).
    pub fn active_set(&self) -> Vec<&AdrRecord> {
        let superseded = self.superseded_target_ids();
        self.records
            .iter()
            .filter(|r| r.status.is_active() && !superseded.contains(&r.id))
            .collect()
    }

    /// Deterministic structural analysis of the supersession lifecycle: dangling
    /// references, one-sided cross-refs, cycles, stale status, and orphans. Does
    /// not include schema problems (those come from parsing) or INDEX staleness
    /// (checked separately).
    pub fn analyze(&self) -> Vec<AdrProblem> {
        let mut problems = Vec::new();
        let superseded = self.superseded_target_ids();

        for record in &self.records {
            // Dangling references.
            if let Some(target) = &record.supersedes
                && self.find(target).is_none()
            {
                problems.push(self.problem(
                    AdrProblemKind::DanglingSupersedes,
                    &record.id,
                    format!(
                        "`{}` supersedes `{target}`, which does not exist",
                        record.id
                    ),
                ));
            }
            if let Some(target) = &record.superseded_by
                && self.find(target).is_none()
            {
                problems.push(self.problem(
                    AdrProblemKind::DanglingSupersededBy,
                    &record.id,
                    format!(
                        "`{}` is superseded_by `{target}`, which does not exist",
                        record.id
                    ),
                ));
            }

            // One-sided cross-reference (both records exist but reciprocal field
            // is absent or mismatched).
            if let Some(target) = &record.supersedes
                && let Some(other) = self.find(target)
                && other.superseded_by.as_deref() != Some(record.id.as_str())
            {
                problems.push(self.problem(
                    AdrProblemKind::MissingCrossRef,
                    &record.id,
                    format!(
                        "`{}` supersedes `{target}` but `{target}` is missing the reciprocal `superseded_by: {}`",
                        record.id, record.id
                    ),
                ));
            }
            if let Some(target) = &record.superseded_by
                && let Some(other) = self.find(target)
                && other.supersedes.as_deref() != Some(record.id.as_str())
            {
                problems.push(self.problem(
                    AdrProblemKind::MissingCrossRef,
                    &record.id,
                    format!(
                        "`{}` is superseded_by `{target}` but `{target}` is missing the reciprocal `supersedes: {}`",
                        record.id, record.id
                    ),
                ));
            }

            // Stale: target of a valid supersedes but status still active.
            if superseded.contains(&record.id) && record.status.is_active() {
                problems.push(self.problem(
                    AdrProblemKind::Stale,
                    &record.id,
                    format!(
                        "`{}` is superseded by another record but still has `status: active`",
                        record.id
                    ),
                ));
            }
            // Stale: status superseded but no superseded_by recorded.
            if record.status == AdrStatus::Superseded && record.superseded_by.is_none() {
                problems.push(self.problem(
                    AdrProblemKind::Stale,
                    &record.id,
                    format!(
                        "`{}` has `status: superseded` but no `superseded_by` link",
                        record.id
                    ),
                ));
            }

            // Orphan: non-active record with no successor link and not referenced.
            if !record.status.is_active()
                && record.superseded_by.is_none()
                && !superseded.contains(&record.id)
            {
                problems.push(self.problem(
                    AdrProblemKind::Orphan,
                    &record.id,
                    format!(
                        "`{}` is `{}` but is not linked to a successor and nothing supersedes it",
                        record.id,
                        record.status.as_str()
                    ),
                ));
            }
        }

        problems.extend(self.detect_cycles());
        problems
    }

    /// Detect supersession cycles by following `supersedes` edges.
    fn detect_cycles(&self) -> Vec<AdrProblem> {
        let mut problems = Vec::new();
        let mut reported: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for start in &self.records {
            let mut slow = Some(start);
            let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
            while let Some(node) = slow {
                if !seen.insert(node.id.clone()) {
                    // Returned to a node already visited on this walk → cycle.
                    if reported.insert(node.id.clone()) {
                        problems.push(self.problem(
                            AdrProblemKind::Cycle,
                            &node.id,
                            format!("supersession cycle detected involving `{}`", node.id),
                        ));
                    }
                    break;
                }
                slow = node.supersedes.as_deref().and_then(|t| self.find(t));
            }
        }
        problems
    }

    fn problem(&self, kind: AdrProblemKind, id: &str, message: String) -> AdrProblem {
        AdrProblem {
            kind,
            store: self.kind,
            id: Some(id.to_string()),
            message,
        }
    }
}

/// Load every record in a store directory. Returns a config error only when the
/// configured path resolves to a file where a directory is expected (AC-10);
/// an absent or empty directory is simply zero records (AC-02).
pub fn load_store(cfg: &Config, kind: AdrKind) -> Result<AdrStore, crate::config::ConfigError> {
    match kind {
        AdrKind::Decisions => cfg.checked_decisions_dir()?,
        AdrKind::Pitfalls => cfg.checked_pitfalls_dir()?,
    };
    cfg.validate_adr_dirs()?;
    let dir = kind.dir(cfg);
    let mut records = Vec::new();
    let mut problems = Vec::new();
    for path in adr_record_files(&dir) {
        match parse_record(&path, kind) {
            Ok(record) => records.push(record),
            Err(problem) => problems.push(problem),
        }
    }
    records.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.id.cmp(&b.id)));
    Ok(AdrStore {
        kind,
        dir,
        records,
        problems,
    })
}

/// Parse a single record file. A malformed record becomes a `Schema` problem.
pub fn parse_record(path: &Path, kind: AdrKind) -> Result<AdrRecord, AdrProblem> {
    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();
    let schema_problem = |message: String| AdrProblem {
        kind: AdrProblemKind::Schema,
        store: kind,
        id: Some(file_stem.clone()),
        message,
    };

    let text = std::fs::read_to_string(path)
        .map_err(|e| schema_problem(format!("cannot read {}: {e}", path.display())))?;

    let (front, body) = split_front_matter(&text).ok_or_else(|| {
        schema_problem(format!(
            "{}: missing `---` front-matter block",
            path.display()
        ))
    })?;
    let fields = parse_front_matter(front);

    let id = match fields
        .iter()
        .find(|(k, _)| k == "id")
        .map(|(_, v)| v.trim().to_string())
    {
        Some(v) if !v.is_empty() => v,
        _ => {
            return Err(schema_problem(format!(
                "{}: required front-matter key `id` is missing or empty",
                path.display()
            )));
        }
    };
    if !record_name_is_safe(&file_stem) || !record_name_is_safe(&id) {
        return Err(AdrProblem {
            kind: AdrProblemKind::PathTraversal,
            store: kind,
            id: Some(file_stem.clone()),
            message: format!(
                "{}: record id / file name must not contain path separators or `..`",
                path.display()
            ),
        });
    }
    let date = fields
        .iter()
        .find(|(k, _)| k == "date")
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    if date.is_empty() {
        return Err(schema_problem(format!(
            "{}: required front-matter key `date` is missing",
            path.display()
        )));
    }
    let status_raw = fields
        .iter()
        .find(|(k, _)| k == "status")
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    let status = AdrStatus::parse(&status_raw).ok_or_else(|| {
        schema_problem(format!(
            "{}: status `{status_raw}` is not one of {:?}",
            path.display(),
            kind.valid_statuses()
        ))
    })?;
    if !kind.valid_statuses().contains(&status.as_str()) {
        return Err(schema_problem(format!(
            "{}: status `{}` is not valid for the {} store",
            path.display(),
            status.as_str(),
            kind.as_str()
        )));
    }
    let area = fields
        .iter()
        .find(|(k, _)| k == "area")
        .map(|(_, v)| parse_list(v))
        .unwrap_or_default();
    if area.is_empty() {
        return Err(schema_problem(format!(
            "{}: required front-matter key `area` is missing or empty",
            path.display()
        )));
    }
    let spec = fields
        .iter()
        .find(|(k, _)| k == "spec")
        .map(|(_, v)| v.clone())
        .filter(|v| !v.is_empty());
    let supersedes = fields
        .iter()
        .find(|(k, _)| k == "supersedes")
        .map(|(_, v)| v.clone())
        .filter(|v| !v.is_empty());
    let superseded_by = fields
        .iter()
        .find(|(k, _)| k == "superseded_by")
        .map(|(_, v)| v.clone())
        .filter(|v| !v.is_empty());

    let title = derive_title(body, &id);

    Ok(AdrRecord {
        kind,
        path: path.to_path_buf(),
        id,
        date,
        area,
        spec,
        status,
        supersedes,
        superseded_by,
        title,
        body: body.to_string(),
    })
}

/// Split `text` into (front-matter-body, record-body) when it opens with a
/// `---` fence. Returns `None` when there is no closing fence.
fn split_front_matter(text: &str) -> Option<(&str, &str)> {
    let rest = text
        .strip_prefix("---\n")
        .or_else(|| text.strip_prefix("---\r\n"))?;
    // Find the closing fence line.
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        let trimmed = line.strip_suffix('\n').unwrap_or(line);
        let trimmed = trimmed.strip_suffix('\r').unwrap_or(trimmed);
        if trimmed == "---" {
            let front = &rest[..offset];
            let body = &rest[offset + line.len()..];
            return Some((front, body));
        }
        offset += line.len();
    }
    None
}

/// Parse simple `key: value` front-matter lines into ordered pairs.
fn parse_front_matter(front: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in front.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        out.push((key.trim().to_string(), value.trim().to_string()));
    }
    out
}

/// Parse a front-matter list value: inline `[a, b]` or a single scalar.
fn parse_list(value: &str) -> Vec<String> {
    let trimmed = value.trim();
    let inner = trimmed
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(trimmed);
    inner
        .split(',')
        .map(|item| item.trim().trim_matches(['"', '\'']).to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

/// Derive a display title from the first markdown heading in the body, trimming
/// a leading `YYYY-MM-DD` date and a trailing `(YYYY-MM-DD)`. Falls back to the
/// record id when the body has no heading.
fn derive_title(body: &str, id: &str) -> String {
    let heading = body
        .lines()
        .find(|line| line.trim_start().starts_with('#'))
        .map(|line| line.trim_start().trim_start_matches('#').trim());
    let Some(heading) = heading else {
        return id.to_string();
    };
    let mut title = heading.to_string();
    // Strip a leading date token and separator: "2026-06-22 — Foo" / "... - Foo".
    if let Some(rest) = strip_leading_date(&title) {
        title = rest;
    }
    // Strip a trailing "(2026-06-22)".
    if let Some(open) = title.rfind('(') {
        let tail = &title[open + 1..];
        if let Some(close) = tail.find(')') {
            let inside = tail[..close].trim();
            if is_iso_date(inside) {
                title = title[..open].trim_end().to_string();
            }
        }
    }
    if title.is_empty() {
        id.to_string()
    } else {
        title
    }
}

fn strip_leading_date(title: &str) -> Option<String> {
    let trimmed = title.trim_start();
    // Slice the candidate date prefix on a char boundary. `get` returns None
    // when byte 10 is out of range or inside a multibyte char (e.g. a
    // non-ASCII-leading title in a `ja` artifact-language project), so this
    // never panics the way `&trimmed[..10]` would.
    let prefix = trimmed.get(..10)?;
    if !is_iso_date(prefix) {
        return None;
    }
    let rest = trimmed[10..].trim_start();
    let rest = rest
        .strip_prefix('—')
        .or_else(|| rest.strip_prefix('-'))
        .or_else(|| rest.strip_prefix(':'))
        .unwrap_or(rest);
    Some(rest.trim().to_string())
}

fn is_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[..4].iter().all(u8::is_ascii_digit)
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[8..10].iter().all(u8::is_ascii_digit)
}

fn escape_cell(value: &str) -> String {
    value.replace(['\r', '\n'], " ").replace('|', r"\|")
}

/// Render the deterministic INDEX.md content for a store. No timestamp is
/// emitted, so the rendered text is purely content-derived and the staleness
/// check is an exact comparison.
pub fn render_index(store: &AdrStore) -> String {
    let mut lines = vec![format!("# {}", store.kind.index_heading()), String::new()];
    if store.records.is_empty() {
        lines.push("_No records._".to_string());
        return lines.join("\n") + "\n";
    }
    lines.push("| Date | Title | Area | Status |".to_string());
    lines.push("|:-----|:------|:-----|:-------|".to_string());
    for record in &store.records {
        lines.push(format!(
            "| {} | {} | {} | {} |",
            escape_cell(&record.date),
            escape_cell(&record.title),
            escape_cell(&record.area.join(", ")),
            escape_cell(record.status.as_str()),
        ));
    }
    lines.join("\n") + "\n"
}

/// True when the store's INDEX.md is absent or does not match the rendered
/// content. An absent index is stale (regenerated by callers, never a failure).
pub fn is_index_stale(store: &AdrStore) -> bool {
    match std::fs::read_to_string(store.dir.join("INDEX.md")) {
        Ok(content) => content != render_index(store),
        Err(_) => true,
    }
}

/// Write the store's INDEX.md (creating the directory if needed). The index is
/// gitignored and must never be staged.
pub fn generate_index(store: &AdrStore) -> std::io::Result<PathBuf> {
    let index_path = store.dir.join("INDEX.md");
    if let Some(parent) = index_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&index_path, render_index(store))?;
    Ok(index_path)
}

/// Regenerate INDEX.md for both stores. Best-effort; returns the written paths.
pub fn regenerate_all_indexes(cfg: &Config) -> Vec<PathBuf> {
    let mut written = Vec::new();
    for kind in [AdrKind::Decisions, AdrKind::Pitfalls] {
        if let Ok(store) = load_store(cfg, kind)
            && let Ok(path) = generate_index(&store)
        {
            written.push(path);
        }
    }
    written
}

/// All deterministic structural problems for a store: parse problems (schema /
/// traversal), supersession analysis (dangling / cross-ref / cycle / stale /
/// orphan), and INDEX staleness. Pure: it does not regenerate the index.
pub fn lint_store(
    cfg: &Config,
    kind: AdrKind,
) -> Result<Vec<AdrProblem>, crate::config::ConfigError> {
    let store = load_store(cfg, kind)?;
    let mut problems = store.problems.clone();
    problems.extend(store.analyze());
    // Unknown `area`: a record tag that is not a configured surface is a schema
    // violation (gating). `area` defaults to the spec's surfaces, so any value
    // outside the known surface set is a mistake the lint must catch.
    let known: std::collections::BTreeSet<String> = cfg.surface_names().into_iter().collect();
    if !known.is_empty() {
        for record in &store.records {
            for area in &record.area {
                if !known.contains(area) {
                    problems.push(AdrProblem {
                        kind: AdrProblemKind::Schema,
                        store: kind,
                        id: Some(record.id.clone()),
                        message: format!(
                            "`{}` has unknown area `{area}` (not a configured surface)",
                            record.id
                        ),
                    });
                }
            }
        }
    }
    if is_index_stale(&store) {
        problems.push(AdrProblem {
            kind: AdrProblemKind::IndexStale,
            store: kind,
            id: None,
            message: format!(
                "{} INDEX.md is absent or stale; regenerate it (it is a gitignored cache)",
                kind.as_str()
            ),
        });
    }
    Ok(problems)
}

/// `mochiflow adr lint`: report deterministic structural problems for one or
/// both stores, classified into gating (FAIL) vs non-blocking (WARN). Returns a
/// non-zero exit code only when a gating problem is present.
pub fn run_adr_lint(cfg: &Config, kind_filter: Option<AdrKind>) -> i32 {
    let kinds = match kind_filter {
        Some(kind) => vec![kind],
        None => vec![AdrKind::Decisions, AdrKind::Pitfalls],
    };
    let mut fail = 0usize;
    let mut warn = 0usize;
    for kind in kinds {
        match lint_store(cfg, kind) {
            Err(e) => {
                eprintln!("FAIL: [{}] {e}", kind.as_str());
                fail += 1;
            }
            Ok(problems) => {
                for problem in problems {
                    let id = problem
                        .id
                        .as_deref()
                        .map(|s| format!(" {s}:"))
                        .unwrap_or_default();
                    if problem.kind.is_gating() {
                        println!("FAIL: [{}]{id} {}", kind.as_str(), problem.message);
                        fail += 1;
                    } else {
                        println!("WARN: [{}]{id} {}", kind.as_str(), problem.message);
                        warn += 1;
                    }
                }
            }
        }
    }
    println!("\nadr lint: {fail} fail, {warn} warn");
    if fail > 0 { 1 } else { 0 }
}

/// Record names that contain a path separator or `..` segment must never be
/// accepted — they could escape the store directory.
pub fn record_name_is_safe(name: &str) -> bool {
    if name.is_empty() || name.contains('/') || name.contains('\\') {
        return false;
    }
    !Path::new(name)
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
}

/// Read-only retrieval filters for `adr list` / `show` / `search`.
#[derive(Debug, Default, Clone)]
pub struct AdrQuery {
    /// Restrict to one store; `None` covers both.
    pub kind: Option<AdrKind>,
    pub area: Option<String>,
    /// Status filter. `None` means the default active set; `Some("all")` widens
    /// to full history; any other value matches that exact status.
    pub status: Option<String>,
    pub spec: Option<String>,
}

impl AdrQuery {
    fn kinds(&self) -> Vec<AdrKind> {
        match self.kind {
            Some(kind) => vec![kind],
            None => vec![AdrKind::Decisions, AdrKind::Pitfalls],
        }
    }

    /// True when the query scans the full history (`--status all`) rather than
    /// the bounded default-active set.
    fn scans_all(&self) -> bool {
        self.status.as_deref() == Some("all")
    }
}

/// Collect records across the queried stores that match the filters. By default
/// only the active set is returned (bounded); `--status all` widens to the full
/// history, and an explicit `--status <s>` matches that status exactly.
fn collect_matching(
    cfg: &Config,
    query: &AdrQuery,
) -> Result<Vec<AdrRecord>, crate::config::ConfigError> {
    let mut out = Vec::new();
    for kind in query.kinds() {
        let store = load_store(cfg, kind)?;
        let base: Vec<AdrRecord> = match query.status.as_deref() {
            // Default and explicit `active` both mean the effective active set:
            // a record superseded by another is excluded even if its own status
            // string still reads `active` (status lag).
            None | Some("active") => store.active_set().into_iter().cloned().collect(),
            // Explicit wider scan over the full history.
            Some("all") => store.records.clone(),
            // Exact status match for the other lifecycle states.
            Some(status) => store
                .records
                .iter()
                .filter(|r| r.status.as_str() == status)
                .cloned()
                .collect(),
        };
        for record in base {
            if let Some(area) = &query.area
                && !record.area.iter().any(|a| a == area)
            {
                continue;
            }
            if let Some(spec) = &query.spec
                && record.spec.as_deref() != Some(spec.as_str())
            {
                continue;
            }
            out.push(record);
        }
    }
    out.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.id.cmp(&b.id)));
    Ok(out)
}

fn header_line(record: &AdrRecord) -> String {
    format!(
        "{}  {}  [{}]  ({}) {}",
        record.date,
        record.id,
        record.area.join(","),
        record.status.as_str(),
        record.title
    )
}

/// `mochiflow adr list`: header rows for the matched records.
pub fn run_adr_list(cfg: &Config, query: &AdrQuery) -> i32 {
    match collect_matching(cfg, query) {
        Err(e) => {
            eprintln!("FAIL: {e}");
            1
        }
        Ok(records) => {
            if records.is_empty() {
                println!("(no matching records)");
            }
            for record in &records {
                println!("{}", header_line(record));
            }
            0
        }
    }
}

/// `mochiflow adr search <term>`: header rows whose front-matter or body contain
/// `term` (case-insensitive), over the same bounded/default-active set as list.
pub fn run_adr_search(cfg: &Config, term: &str, query: &AdrQuery) -> i32 {
    let needle = term.to_lowercase();
    match collect_matching(cfg, query) {
        Err(e) => {
            eprintln!("FAIL: {e}");
            1
        }
        Ok(records) => {
            let matches: Vec<&AdrRecord> = records
                .iter()
                .filter(|r| {
                    r.id.to_lowercase().contains(&needle)
                        || r.title.to_lowercase().contains(&needle)
                        || r.body.to_lowercase().contains(&needle)
                        || r.area.iter().any(|a| a.to_lowercase().contains(&needle))
                })
                .collect();
            if matches.is_empty() {
                let scope = if query.scans_all() {
                    "full history"
                } else {
                    "active set"
                };
                println!("(no matches for `{term}` in the {scope})");
            }
            for record in matches {
                println!("{}", header_line(record));
            }
            0
        }
    }
}

/// `mochiflow adr show <id>`: full body plus supersession lineage. Resolves the
/// id across the queried stores (full history, so superseded records are
/// reachable for lineage tracing). Returns non-zero when the id is unknown.
pub fn run_adr_show(cfg: &Config, id: &str, kind_filter: Option<AdrKind>) -> i32 {
    if !record_name_is_safe(id) {
        eprintln!("FAIL: invalid record id `{id}`");
        return 1;
    }
    let kinds = match kind_filter {
        Some(kind) => vec![kind],
        None => vec![AdrKind::Decisions, AdrKind::Pitfalls],
    };
    for kind in kinds {
        let store = match load_store(cfg, kind) {
            Ok(store) => store,
            Err(e) => {
                eprintln!("FAIL: {e}");
                return 1;
            }
        };
        if let Some(record) = store.find(id) {
            print!("{}", render_show(&store, record));
            return 0;
        }
    }
    eprintln!("FAIL: no record with id `{id}`");
    1
}

fn render_show(store: &AdrStore, record: &AdrRecord) -> String {
    let mut out = String::new();
    out.push_str(&format!("id: {}\n", record.id));
    out.push_str(&format!("date: {}\n", record.date));
    out.push_str(&format!("area: {}\n", record.area.join(", ")));
    if let Some(spec) = &record.spec {
        out.push_str(&format!("spec: {spec}\n"));
    }
    out.push_str(&format!("status: {}\n", record.status.as_str()));
    // Supersession lineage.
    if let Some(target) = &record.supersedes {
        let known = if store.find(target).is_some() {
            ""
        } else {
            " (unknown)"
        };
        out.push_str(&format!("supersedes: {target}{known}\n"));
    }
    if let Some(target) = &record.superseded_by {
        let known = if store.find(target).is_some() {
            ""
        } else {
            " (unknown)"
        };
        out.push_str(&format!("superseded_by: {target}{known}\n"));
    }
    out.push('\n');
    out.push_str(&record.body);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn write_record(dir: &Path, name: &str, front: &str, body: &str) -> PathBuf {
        std::fs::create_dir_all(dir).unwrap();
        let path = dir.join(name);
        std::fs::write(&path, format!("---\n{front}---\n{body}")).unwrap();
        path
    }

    #[test]
    fn parses_front_matter_and_derives_title() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("decisions");
        let path = write_record(
            &dir,
            "2026-06-22-version-ssot.md",
            "id: 2026-06-22-version-ssot\ndate: 2026-06-22\narea: [cli, core]\nspec: version-ssot\nstatus: active\n",
            "## 2026-06-22 — Version SSOT: one source\n\nbody text\n",
        );
        let record = parse_record(&path, AdrKind::Decisions).unwrap();
        assert_eq!(record.id, "2026-06-22-version-ssot");
        assert_eq!(record.date, "2026-06-22");
        assert_eq!(record.area, vec!["cli", "core"]);
        assert_eq!(record.spec.as_deref(), Some("version-ssot"));
        assert_eq!(record.status, AdrStatus::Active);
        assert_eq!(record.title, "Version SSOT: one source");
        assert!(record.body.starts_with("## 2026-06-22"));
    }

    #[test]
    fn multibyte_leading_title_does_not_panic() {
        // A `ja` artifact-language project has non-ASCII-leading titles. The
        // title-deriving slice must stay on a char boundary instead of slicing
        // byte 10 inside a multibyte character.
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("pitfalls");

        // No leading date: the whole multibyte heading is the title.
        let no_date = write_record(
            &dir,
            "2026-06-18-source-origin.md",
            "id: 2026-06-18-source-origin\ndate: 2026-06-18\narea: ios\nstatus: active\n",
            "## 確定値の source を aggregation 種別から決めると手入力が ble に化ける\n\nbody\n",
        );
        let record = parse_record(&no_date, AdrKind::Pitfalls).unwrap();
        assert_eq!(
            record.title,
            "確定値の source を aggregation 種別から決めると手入力が ble に化ける"
        );

        // Leading ISO date in front of a multibyte body: date is stripped,
        // multibyte remainder survives.
        let dated = write_record(
            &dir,
            "2026-06-18-dated.md",
            "id: 2026-06-18-dated\ndate: 2026-06-18\narea: ios\nstatus: active\n",
            "## 2026-06-18 マルチバイト見出し\n\nbody\n",
        );
        let record = parse_record(&dated, AdrKind::Pitfalls).unwrap();
        assert_eq!(record.title, "マルチバイト見出し");
    }

    #[test]
    fn pitfall_trailing_date_title_and_status() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("pitfalls");
        let path = write_record(
            &dir,
            "2026-06-22-lock-format.md",
            "id: 2026-06-22-lock-format\ndate: 2026-06-22\narea: cli\nstatus: resolved\n",
            "## contracts.lock format is byte-sensitive (2026-06-22)\n\nbody\n",
        );
        let record = parse_record(&path, AdrKind::Pitfalls).unwrap();
        assert_eq!(record.title, "contracts.lock format is byte-sensitive");
        assert_eq!(record.status, AdrStatus::Resolved);
    }

    #[test]
    fn missing_front_matter_or_date_is_a_schema_problem() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("decisions");
        std::fs::create_dir_all(&dir).unwrap();
        let no_fm = dir.join("bad.md");
        std::fs::write(&no_fm, "## just a body\n").unwrap();
        let err = parse_record(&no_fm, AdrKind::Decisions).unwrap_err();
        assert_eq!(err.kind, AdrProblemKind::Schema);

        let no_date = write_record(&dir, "no-date.md", "id: x\nstatus: active\n", "## body\n");
        let err = parse_record(&no_date, AdrKind::Decisions).unwrap_err();
        assert_eq!(err.kind, AdrProblemKind::Schema);
    }

    #[test]
    fn invalid_status_for_store_is_schema_problem() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("decisions");
        let path = write_record(
            &dir,
            "2026-06-22-x.md",
            "id: 2026-06-22-x\ndate: 2026-06-22\nstatus: resolved\n",
            "## body\n",
        );
        // `resolved` is a pitfalls-only status; invalid for decisions.
        let err = parse_record(&path, AdrKind::Decisions).unwrap_err();
        assert_eq!(err.kind, AdrProblemKind::Schema);
    }

    #[test]
    fn missing_id_or_area_is_a_schema_problem() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("decisions");
        // Missing `id` key (filename is not a fallback).
        let no_id = write_record(
            &dir,
            "2026-06-22-x.md",
            "date: 2026-06-22\narea: [cli]\nstatus: active\n",
            "## body\n",
        );
        assert_eq!(
            parse_record(&no_id, AdrKind::Decisions).unwrap_err().kind,
            AdrProblemKind::Schema
        );
        // Missing `area` key.
        let no_area = write_record(
            &dir,
            "2026-06-22-y.md",
            "id: 2026-06-22-y\ndate: 2026-06-22\nstatus: active\n",
            "## body\n",
        );
        assert_eq!(
            parse_record(&no_area, AdrKind::Decisions).unwrap_err().kind,
            AdrProblemKind::Schema
        );
        // Empty `area` list.
        let empty_area = write_record(
            &dir,
            "2026-06-22-z.md",
            "id: 2026-06-22-z\ndate: 2026-06-22\narea: []\nstatus: active\n",
            "## body\n",
        );
        assert_eq!(
            parse_record(&empty_area, AdrKind::Decisions)
                .unwrap_err()
                .kind,
            AdrProblemKind::Schema
        );
    }

    #[test]
    fn status_active_filter_excludes_status_lagged_superseded() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("decisions");
        // `old` is superseded by `new` but its own status still reads active.
        write_record(
            &dir,
            "2026-06-01-old.md",
            "id: 2026-06-01-old\ndate: 2026-06-01\narea: cli\nstatus: active\n",
            "## 2026-06-01 — Old\n",
        );
        write_record(
            &dir,
            "2026-06-20-new.md",
            "id: 2026-06-20-new\ndate: 2026-06-20\narea: cli\nstatus: active\nsupersedes: 2026-06-01-old\n",
            "## 2026-06-20 — New\n",
        );
        let cfg = test_cfg(tmp.path());
        let query = AdrQuery {
            status: Some("active".to_string()),
            ..AdrQuery::default()
        };
        let matched = collect_matching(&cfg, &query).unwrap();
        let ids: Vec<&str> = matched.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["2026-06-20-new"],
            "--status active must use the effective active set, not raw status"
        );
    }

    #[test]
    fn render_index_is_deterministic_and_sorted_desc() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("decisions");
        write_record(
            &dir,
            "2026-06-22-a.md",
            "id: 2026-06-22-a\ndate: 2026-06-22\narea: cli\nstatus: active\n",
            "## 2026-06-22 — Alpha\n",
        );
        write_record(
            &dir,
            "2026-06-24-b.md",
            "id: 2026-06-24-b\ndate: 2026-06-24\narea: cli\nstatus: active\n",
            "## 2026-06-24 — Beta\n",
        );
        let cfg = test_cfg(tmp.path());
        let store = load_store(&cfg, AdrKind::Decisions).unwrap();
        let rendered = render_index(&store);
        let beta = rendered.find("Beta").unwrap();
        let alpha = rendered.find("Alpha").unwrap();
        assert!(beta < alpha, "newest record first:\n{rendered}");
        assert!(rendered.contains("| 2026-06-24 | Beta | cli | active |"));
    }

    #[test]
    fn index_staleness_and_generate_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("decisions");
        write_record(
            &dir,
            "2026-06-22-a.md",
            "id: 2026-06-22-a\ndate: 2026-06-22\narea: cli\nstatus: active\n",
            "## 2026-06-22 — Alpha\n",
        );
        let cfg = test_cfg(tmp.path());
        let store = load_store(&cfg, AdrKind::Decisions).unwrap();
        assert!(is_index_stale(&store), "absent index is stale");
        generate_index(&store).unwrap();
        let store = load_store(&cfg, AdrKind::Decisions).unwrap();
        assert!(!is_index_stale(&store), "regenerated index is fresh");
    }

    #[test]
    fn empty_store_renders_no_records_and_no_fallback() {
        let tmp = tempfile::tempdir().unwrap();
        let cfg = test_cfg(tmp.path());
        let store = load_store(&cfg, AdrKind::Decisions).unwrap();
        assert!(store.records.is_empty());
        assert!(render_index(&store).contains("_No records._"));
    }

    #[test]
    fn rejects_record_names_with_traversal() {
        assert!(record_name_is_safe("2026-06-22-ok"));
        assert!(!record_name_is_safe("../escape"));
        assert!(!record_name_is_safe("a/b"));
        assert!(!record_name_is_safe(".."));
    }

    #[test]
    fn traversal_in_id_is_rejected_during_parse() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("decisions");
        let path = write_record(
            &dir,
            "evil.md",
            "id: ../../etc/passwd\ndate: 2026-06-22\nstatus: active\n",
            "## body\n",
        );
        let err = parse_record(&path, AdrKind::Decisions).unwrap_err();
        assert_eq!(err.kind, AdrProblemKind::PathTraversal);
    }

    #[test]
    fn active_set_excludes_superseded_deprecated_and_status_lag() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("decisions");
        // old: status still active but superseded by new (status lags).
        write_record(
            &dir,
            "2026-06-01-old.md",
            "id: 2026-06-01-old\ndate: 2026-06-01\narea: cli\nstatus: active\n",
            "## 2026-06-01 — Old\n",
        );
        // new: supersedes old, with reciprocal on old missing (one-sided).
        write_record(
            &dir,
            "2026-06-20-new.md",
            "id: 2026-06-20-new\ndate: 2026-06-20\narea: cli\nstatus: active\nsupersedes: 2026-06-01-old\n",
            "## 2026-06-20 — New\n",
        );
        // deprecated standalone.
        write_record(
            &dir,
            "2026-06-10-dep.md",
            "id: 2026-06-10-dep\ndate: 2026-06-10\narea: cli\nstatus: deprecated\n",
            "## 2026-06-10 — Dep\n",
        );
        let cfg = test_cfg(tmp.path());
        let store = load_store(&cfg, AdrKind::Decisions).unwrap();
        let active: Vec<&str> = store.active_set().iter().map(|r| r.id.as_str()).collect();
        assert_eq!(
            active,
            vec!["2026-06-20-new"],
            "only the successor is active"
        );

        let kinds: Vec<AdrProblemKind> = store.analyze().iter().map(|p| p.kind).collect();
        assert!(kinds.contains(&AdrProblemKind::MissingCrossRef));
        assert!(kinds.contains(&AdrProblemKind::Stale)); // old still active
    }

    #[test]
    fn dangling_supersedes_is_detected() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("decisions");
        write_record(
            &dir,
            "2026-06-20-x.md",
            "id: 2026-06-20-x\ndate: 2026-06-20\narea: cli\nstatus: active\nsupersedes: does-not-exist\n",
            "## 2026-06-20 — X\n",
        );
        let cfg = test_cfg(tmp.path());
        let store = load_store(&cfg, AdrKind::Decisions).unwrap();
        let kinds: Vec<AdrProblemKind> = store.analyze().iter().map(|p| p.kind).collect();
        assert!(kinds.contains(&AdrProblemKind::DanglingSupersedes));
    }

    #[test]
    fn supersession_cycle_is_detected() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("decisions");
        write_record(
            &dir,
            "a.md",
            "id: a\ndate: 2026-06-20\narea: cli\nstatus: superseded\nsupersedes: b\nsuperseded_by: b\n",
            "## A\n",
        );
        write_record(
            &dir,
            "b.md",
            "id: b\ndate: 2026-06-21\narea: cli\nstatus: superseded\nsupersedes: a\nsuperseded_by: a\n",
            "## B\n",
        );
        let cfg = test_cfg(tmp.path());
        let store = load_store(&cfg, AdrKind::Decisions).unwrap();
        let kinds: Vec<AdrProblemKind> = store.analyze().iter().map(|p| p.kind).collect();
        assert!(kinds.contains(&AdrProblemKind::Cycle), "{kinds:?}");
    }

    #[test]
    fn well_formed_supersession_has_no_relational_problems() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("decisions");
        write_record(
            &dir,
            "2026-06-01-old.md",
            "id: 2026-06-01-old\ndate: 2026-06-01\narea: cli\nstatus: superseded\nsuperseded_by: 2026-06-20-new\n",
            "## Old\n",
        );
        write_record(
            &dir,
            "2026-06-20-new.md",
            "id: 2026-06-20-new\ndate: 2026-06-20\narea: cli\nstatus: active\nsupersedes: 2026-06-01-old\n",
            "## New\n",
        );
        let cfg = test_cfg(tmp.path());
        let store = load_store(&cfg, AdrKind::Decisions).unwrap();
        assert!(store.analyze().is_empty(), "{:?}", store.analyze());
        let active: Vec<&str> = store.active_set().iter().map(|r| r.id.as_str()).collect();
        assert_eq!(active, vec!["2026-06-20-new"]);
    }

    #[test]
    fn lint_classifies_gating_vs_warning_and_sets_exit() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("decisions");
        // gating: dangling supersedes.
        write_record(
            &dir,
            "2026-06-20-x.md",
            "id: 2026-06-20-x\ndate: 2026-06-20\narea: cli\nstatus: active\nsupersedes: ghost\n",
            "## 2026-06-20 — X\n",
        );
        let cfg = test_cfg(tmp.path());
        let problems = lint_store(&cfg, AdrKind::Decisions).unwrap();
        let kinds: Vec<AdrProblemKind> = problems.iter().map(|p| p.kind).collect();
        assert!(kinds.contains(&AdrProblemKind::DanglingSupersedes));
        // INDEX absent -> stale warning, never a gate.
        assert!(kinds.contains(&AdrProblemKind::IndexStale));
        assert!(AdrProblemKind::DanglingSupersedes.is_gating());
        assert!(!AdrProblemKind::IndexStale.is_gating());
        assert!(!AdrProblemKind::Orphan.is_gating());
        assert!(!AdrProblemKind::Stale.is_gating());
        // Command exit is non-zero because a gating problem is present.
        assert_eq!(run_adr_lint(&cfg, Some(AdrKind::Decisions)), 1);
    }

    #[test]
    fn lint_flags_unknown_area_as_gating_schema() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("decisions");
        write_record(
            &dir,
            "2026-06-20-x.md",
            "id: 2026-06-20-x\ndate: 2026-06-20\narea: [bogus]\nstatus: active\n",
            "## 2026-06-20 — X\n",
        );
        let cfg = test_cfg(tmp.path()); // surface = cli
        let problems = lint_store(&cfg, AdrKind::Decisions).unwrap();
        assert!(
            problems
                .iter()
                .any(|p| p.kind == AdrProblemKind::Schema && p.message.contains("unknown area")),
            "{problems:?}"
        );
    }

    #[test]
    fn lint_clean_store_with_fresh_index_passes() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("decisions");
        write_record(
            &dir,
            "2026-06-20-x.md",
            "id: 2026-06-20-x\ndate: 2026-06-20\narea: cli\nstatus: active\n",
            "## 2026-06-20 — X\n",
        );
        let cfg = test_cfg(tmp.path());
        let store = load_store(&cfg, AdrKind::Decisions).unwrap();
        generate_index(&store).unwrap();
        assert_eq!(run_adr_lint(&cfg, Some(AdrKind::Decisions)), 0);
    }

    /// Build a Config rooted at `repo` with adr dirs under `repo/decisions` and
    /// `repo/pitfalls` (matching `write_record`'s layout).
    fn test_cfg(repo: &Path) -> Config {
        let install = repo.join(".mochiflow");
        std::fs::create_dir_all(&install).unwrap();
        std::fs::write(
            install.join("config.toml"),
            "schema_version = 1\ninstall_dir = \".mochiflow\"\nspecs_dir = \".mochiflow/specs\"\nindex = \".mochiflow/INDEX.md\"\n\n[constitution]\nproject = \".mochiflow/constitution.md\"\nlocal = \".mochiflow/constitution.local.md\"\n\n[context]\nproduct = \".mochiflow/context/product.md\"\nstructure = \".mochiflow/context/structure.md\"\ntech = \".mochiflow/context/tech.md\"\n\n[adr]\ndecisions = \"decisions\"\npitfalls = \"pitfalls\"\n\n[surfaces.cli]\ndescription = \"cli\"\n\n[surfaces.cli.verify]\ndefault = \"echo ok\"\n",
        )
        .unwrap();
        crate::config::load_config(&install.join("config.toml")).unwrap()
    }
}
