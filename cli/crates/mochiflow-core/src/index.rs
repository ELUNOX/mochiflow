//! Index generation: produce INDEX.md dashboard + state/index.json.

use std::path::Path;

use crate::config::Config;
use crate::delivery::{DeliveryColumn, NextActionKind};
use crate::inspect::{ProcessRunner, legacy_snapshot};
use crate::spec_meta::{SpecMeta, parse_yaml_subset};

/// Backlog seed metadata (from markdown frontmatter).
pub struct SeedInfo {
    pub slug: String,
    pub title: String,
    pub maturity: String,
    pub source: String,
}

/// Active spec entry.
struct ActiveEntry {
    slug: String,
    title: String,
    status: String,
    column: DeliveryColumn,
    risk: String,
    module: String,
    docs: String,
    path: String,
    /// Derived conversational next action (e.g. report-merge while in review).
    next_action: Option<NextActionKind>,
}

/// Done spec entry.
struct DoneEntry {
    slug: String,
    title: String,
    path: String,
    spec_type: String,
    module: String,
    /// Date shown in the table and used for month grouping (YYYY-MM-DD).
    updated: String,
    /// Completion timestamp written at the `done` transition (ISO 8601), if any.
    completed: Option<String>,
    /// Ordering key: completed (full precision) → updated → mtime date.
    /// Sorting by this descending yields chronological completion order, with
    /// intra-day order preserved when `completed` carries a time component.
    sort_key: String,
    /// Derived conversational next action (e.g. local-cleanup-pending for a
    /// done-derived flat spec whose local branch / scratch remain).
    next_action: Option<NextActionKind>,
}

fn status_emoji(status: &str) -> &str {
    match status {
        "draft" => "📝",
        "approved" => "🟢",
        "accepted" => "🔵",
        "done" => "✅",
        "seed" => "🌱",
        _ => "❓",
    }
}

/// Parse frontmatter from a backlog seed .md file (matching Python's read_memo_metadata).
fn read_seed(path: &Path) -> Option<SeedInfo> {
    let text = std::fs::read_to_string(path).ok()?;
    let slug = path.file_stem()?.to_str()?.to_string();

    if !text.starts_with("---\n") {
        // No frontmatter — derive title from first heading
        let title = text
            .lines()
            .find(|l| l.starts_with("# "))
            .map(|l| l.trim_start_matches("# ").trim().to_string())
            .unwrap_or_else(|| slug.clone());
        return Some(SeedInfo {
            slug,
            title,
            maturity: "—".to_string(),
            source: "—".to_string(),
        });
    }

    let end = text[4..].find("\n---\n")?;
    let fm_block = &text[4..4 + end];
    // Parse the front-matter through the canonical YAML-subset parser so quoted
    // values (`slug: "foo"`) and inline comments are handled identically to
    // spec.yaml. A hand-rolled `split_once(':')` here previously leaked the
    // surrounding quotes into the rendered INDEX.md.
    let fields = parse_yaml_subset(fm_block).unwrap_or_default();
    let get = |key: &str| fields.get(key).and_then(|v| v.as_str()).map(str::to_string);

    let title = get("title").unwrap_or_else(|| slug.clone());

    Some(SeedInfo {
        slug: get("slug").unwrap_or_else(|| slug.clone()),
        title,
        maturity: get("maturity").unwrap_or_else(|| "—".to_string()),
        source: get("source").unwrap_or_else(|| "—".to_string()),
    })
}

/// Collect active, done, and seed entries from the specs directory.
fn collect(cfg: &Config) -> (Vec<ActiveEntry>, Vec<DoneEntry>, Vec<SeedInfo>) {
    let specs_dir = cfg.specs_dir_path();
    let mut active = Vec::new();
    let mut done = Vec::new();
    let mut seeds = Vec::new();

    if !specs_dir.exists() {
        return (active, done, seeds);
    }
    let snapshot = legacy_snapshot(cfg, &ProcessRunner);

    // Active specs (direct children excluding . and _ prefixed)
    let mut dirs: Vec<_> = std::fs::read_dir(&specs_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let n = name.to_string_lossy();
            e.path().is_dir() && !n.starts_with('.') && !n.starts_with('_')
        })
        .collect();
    dirs.sort_by_key(|e| e.file_name());

    for entry in &dirs {
        let d = entry.path();
        if let Some(observed) = snapshot.iter().find(|item| item.dir == d) {
            let m = &observed.meta;
            let slug = entry.file_name().to_string_lossy().to_string();
            // Column membership comes from the same derivation the board uses
            // (asserted ∪ derived), not from directory location: a flat spec
            // whose PR has merged renders in Done even though it is not in _done.
            let column = observed.column;
            let next_action = observed.next_action;
            if column == DeliveryColumn::Done {
                done.push(make_done_entry(&d, &slug, &slug, m, next_action));
                continue;
            }
            let mut docs_parts = vec!["spec".to_string()];
            if d.join("design.md").exists() {
                docs_parts.push("design".to_string());
            }
            if d.join("tasks.md").exists() {
                docs_parts.push("tasks".to_string());
            }
            active.push(ActiveEntry {
                slug: slug.clone(),
                title: m.title().to_string(),
                status: m.status().to_string(),
                column,
                risk: m.risk().to_string(),
                module: m.module().unwrap_or("—").to_string(),
                docs: docs_parts.join("+"),
                path: slug,
                next_action,
            });
        }
    }

    // Done specs
    let done_dir = specs_dir.join("_done");
    if done_dir.exists() {
        let mut done_dirs: Vec<_> = std::fs::read_dir(&done_dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .collect();
        done_dirs.sort_by_key(|e| e.file_name());

        for entry in &done_dirs {
            let d = entry.path();
            if let Some(observed) = snapshot.iter().find(|item| item.dir == d) {
                let m = &observed.meta;
                let slug = entry.file_name().to_string_lossy().to_string();
                let next_action = observed.next_action;
                done.push(make_done_entry(
                    &d,
                    &slug,
                    &format!("_done/{slug}"),
                    m,
                    next_action,
                ));
            }
        }
        // Sort: slug asc first (stable tiebreak), then completion key desc so the
        // most recently completed spec leads each day. Equal keys keep slug asc.
        done.sort_by(|a, b| a.slug.cmp(&b.slug));
        done.sort_by(|a, b| b.sort_key.cmp(&a.sort_key));
    } else if !done.is_empty() {
        // Derived-done flat specs (no _done dir present): keep deterministic order.
        done.sort_by(|a, b| a.slug.cmp(&b.slug));
        done.sort_by(|a, b| b.sort_key.cmp(&a.sort_key));
    }

    // Backlog seeds
    let backlog = specs_dir.join("_backlog");
    if backlog.exists() {
        let mut seed_files: Vec<_> = std::fs::read_dir(&backlog)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name();
                let n = name.to_string_lossy();
                n.ends_with(".md") && n != "README.md"
            })
            .collect();
        seed_files.sort_by_key(|e| e.file_name());

        for entry in &seed_files {
            if let Some(seed) = read_seed(&entry.path()) {
                seeds.push(seed);
            }
        }
    }

    (active, done, seeds)
}

/// Build a `DoneEntry` for a spec directory, resolving its display/grouping date
/// and ordering key from `completed` → `updated` → file mtime.
fn make_done_entry(
    dir: &Path,
    slug: &str,
    path: &str,
    m: &SpecMeta,
    next_action: Option<NextActionKind>,
) -> DoneEntry {
    let completed = m.completed().map(str::to_string).filter(|s| !s.is_empty());
    let updated_field = m.updated().unwrap_or("").to_string();
    let pick_source = || {
        completed
            .clone()
            .filter(|s| !s.is_empty())
            .or_else(|| {
                if updated_field.is_empty() {
                    None
                } else {
                    Some(updated_field.clone())
                }
            })
            .unwrap_or_else(|| mtime_isodate(dir))
    };
    let updated = isodate_only(&pick_source());
    let sort_key = pick_source();
    DoneEntry {
        slug: slug.to_string(),
        title: m.title().to_string(),
        path: path.to_string(),
        spec_type: m.spec_type().to_string(),
        module: m.module().unwrap_or("—").to_string(),
        updated,
        completed,
        sort_key,
        next_action,
    }
}

/// Reduce a date or ISO 8601 timestamp to its `YYYY-MM-DD` prefix for display
/// and month grouping. Values shorter than 10 chars are returned unchanged.
fn isodate_only(value: &str) -> String {
    if value.len() >= 10 {
        value[..10].to_string()
    } else {
        value.to_string()
    }
}

fn mtime_isodate(path: &Path) -> String {
    path.metadata()
        .and_then(|m| m.modified())
        .map(|t| {
            let secs = t
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            // Simple UTC date from epoch seconds
            let days = secs / 86400;
            epoch_days_to_iso(days)
        })
        .unwrap_or_default()
}

fn epoch_days_to_iso(days: u64) -> String {
    // Compute date from days since 1970-01-01 (civil calendar)
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

const NORMALIZED_TIMESTAMP: &str = "{{TIMESTAMP}}";

fn normalize_index_timestamp(text: &str) -> String {
    let mut out = text
        .lines()
        .map(|line| {
            if line.starts_with("> updated: ") {
                format!("> updated: {NORMALIZED_TIMESTAMP}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    if text.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn render_index(cfg: &Config, now: &str) -> String {
    let (active, done, seeds) = collect(cfg);
    render_index_snapshot(cfg, now, &active, &done, &seeds)
}

fn render_index_snapshot(
    cfg: &Config,
    now: &str,
    active: &[ActiveEntry],
    done: &[DoneEntry],
    seeds: &[SeedInfo],
) -> String {
    let active_specs: Vec<_> = active
        .iter()
        .filter(|entry| entry.column == DeliveryColumn::Active)
        .collect();
    let ready_specs: Vec<_> = active
        .iter()
        .filter(|entry| entry.column == DeliveryColumn::Ready)
        .collect();
    let in_review_specs: Vec<_> = active
        .iter()
        .filter(|entry| entry.column == DeliveryColumn::InReview)
        .collect();

    // Compute relative path from index file to specs_dir
    let index_path_buf = cfg.index_path();
    let index_parent = index_path_buf.parent().unwrap_or(Path::new("."));
    let specs_rel = pathdiff_relative(&cfg.specs_dir_path(), index_parent);

    let mut lines: Vec<String> = vec![
        "# 📋 Spec Dashboard".to_string(),
        String::new(),
        format!("> updated: {now}"),
        String::new(),
        "## Pipeline".to_string(),
        String::new(),
        "| stage | count |".to_string(),
        "|:------|------:|".to_string(),
        format!("| 🌱 backlog seed | {} |", seeds.len()),
        format!("| 📝 active | {} |", active_specs.len()),
        format!("| 🔵 ready | {} |", ready_specs.len()),
        format!("| 🔎 in review | {} |", in_review_specs.len()),
        format!("| ✅ done | {} |", done.len()),
        String::new(),
        "## Backlog seeds".to_string(),
        String::new(),
    ];

    if seeds.is_empty() {
        lines.push("（なし）".to_string());
    } else {
        lines.push("| Slug | Title | Maturity | Source |".to_string());
        lines.push("|:-----|:------|:---------|:-------|".to_string());
        for s in seeds {
            lines.push(format!(
                "| [{}]({specs_rel}/_backlog/{}.md) | {} | {} {} | {} |",
                md_table_cell(&s.slug),
                url_path_segment(&s.slug),
                md_table_cell(&s.title),
                status_emoji(&s.maturity),
                md_table_cell(&s.maturity),
                md_table_cell(&s.source)
            ));
        }
    }

    push_spec_section(&mut lines, "Active specs", &active_specs, &specs_rel);
    push_spec_section(&mut lines, "Ready specs", &ready_specs, &specs_rel);
    push_spec_section(&mut lines, "In Review specs", &in_review_specs, &specs_rel);

    lines.push(String::new());
    lines.push("## Done (chronological)".to_string());
    lines.push(String::new());

    if done.is_empty() {
        lines.push("（なし）".to_string());
    } else {
        let mut current_month: Option<String> = None;
        for d in done {
            let month = if d.updated.len() >= 7 {
                &d.updated[..7]
            } else {
                "unknown"
            };
            if current_month.as_deref() != Some(month) {
                if current_month.is_some() {
                    lines.push(String::new());
                }
                current_month = Some(month.to_string());
                lines.push(format!("### {month}"));
                lines.push(String::new());
                lines.push("| Updated | Slug | Title | Type |".to_string());
                lines.push("|:--------|:-----|:------|:-----|".to_string());
            }
            lines.push(format!(
                "| {} | [{}]({specs_rel}/{}/) | {} | {} |",
                md_table_cell(&d.updated),
                md_table_cell(&d.slug),
                url_path(&d.path),
                md_table_cell(&d.title),
                md_table_cell(&d.spec_type)
            ));
        }
        lines.push(String::new());
        lines.push(format!("> done total: {}", done.len()));
    }

    lines.join("\n") + "\n"
}

fn push_spec_section(
    lines: &mut Vec<String>,
    heading: &str,
    entries: &[&ActiveEntry],
    specs_rel: &str,
) {
    lines.push(String::new());
    lines.push(format!("## {heading}"));
    lines.push(String::new());

    if entries.is_empty() {
        lines.push("（なし）".to_string());
        return;
    }

    lines.push("| Spec | Status | Risk | Docs | Module |".to_string());
    lines.push("|:-----|:-------|:-----|:-----|:-------|".to_string());
    for entry in entries {
        lines.push(format!(
            "| [{}]({specs_rel}/{}/) | {} {} | {} | {} | {} |",
            md_table_cell(&entry.slug),
            url_path(&entry.path),
            status_emoji(&entry.status),
            md_table_cell(&entry.status),
            md_table_cell(&entry.risk),
            md_table_cell(&entry.docs),
            md_table_cell(&entry.module)
        ));
    }
}

fn md_table_cell(value: &str) -> String {
    value.replace(['\r', '\n'], " ").replace('|', r"\|")
}

fn url_path_segment(value: &str) -> String {
    value.replace('\\', "%5C").replace(' ', "%20")
}

fn url_path(value: &str) -> String {
    value
        .split('/')
        .map(url_path_segment)
        .collect::<Vec<_>>()
        .join("/")
}

pub fn is_index_stale(cfg: &Config) -> bool {
    let Ok(index_path) = cfg.checked_index_path() else {
        return true;
    };
    let index_path = index_path.operation_path();
    let actual = match std::fs::read_to_string(index_path) {
        Ok(content) => content,
        Err(_) => return true,
    };
    let expected = render_index(cfg, NORMALIZED_TIMESTAMP);
    normalize_index_timestamp(&actual) != normalize_index_timestamp(&expected)
}

pub fn check_index(cfg: &Config) -> i32 {
    if is_index_stale(cfg) {
        println!("FAIL: INDEX.md is stale; run `mochiflow index`");
        1
    } else {
        println!("index: clean");
        0
    }
}

/// Generate INDEX.md and state/index.json (matching Python's index.main).
pub fn generate_index(cfg: &Config) -> std::io::Result<()> {
    generate_index_inner(cfg, true)
}

pub fn generate_index_quiet(cfg: &Config) -> std::io::Result<()> {
    generate_index_inner(cfg, false)
}

fn generate_index_inner(cfg: &Config, print_summary: bool) -> std::io::Result<()> {
    let (active, done, seeds) = collect(cfg);
    let now = utc_now_formatted();
    let content = render_index_snapshot(cfg, &now, &active, &done, &seeds);

    // Write INDEX.md
    let index_path = cfg
        .checked_index_path()
        .map_err(std::io::Error::other)?
        .into_operation_path();
    if let Some(parent) = index_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&index_path, &content)?;

    // Write state/index.json
    let state_dir = cfg
        .checked_state_dir()
        .map_err(std::io::Error::other)?
        .into_operation_path();
    std::fs::create_dir_all(&state_dir)?;
    let json_data = build_json(
        &now,
        &active,
        &done,
        &seeds,
        cfg.conversation_output_language(),
    );
    std::fs::write(state_dir.join("index.json"), json_data)?;

    if print_summary {
        println!(
            "index: {} + {}",
            index_path.display(),
            state_dir.join("index.json").display()
        );
    }
    Ok(())
}

fn build_json(
    now: &str,
    active: &[ActiveEntry],
    done: &[DoneEntry],
    seeds: &[SeedInfo],
    language: &str,
) -> String {
    use serde_json::{Value, json};

    // Stable JSON board contract for delivery next actions: `next_action` is
    // `null` or `{kind, message}` (kind ∈ {report_merge, local_cleanup_pending});
    // `local_cleanup_pending` is the boolean shortcut for the cleanup kind.
    let next_action_fields = |action: Option<NextActionKind>| -> (Value, bool) {
        match action {
            Some(kind) => (
                json!({ "kind": kind.as_str(), "message": kind.message(language) }),
                kind == NextActionKind::LocalCleanupPending,
            ),
            None => (Value::Null, false),
        }
    };

    let active_json: Vec<Value> = active
        .iter()
        .map(|a| {
            let (next_action, local_cleanup_pending) = next_action_fields(a.next_action);
            json!({
                "slug": a.slug,
                "title": a.title,
                "status": a.status,
                "column": a.column.as_str(),
                "risk": a.risk,
                "module": a.module,
                "docs": a.docs,
                "path": a.path,
                "next_action": next_action,
                "local_cleanup_pending": local_cleanup_pending,
            })
        })
        .collect();

    let done_json: Vec<Value> = done
        .iter()
        .map(|d| {
            let (next_action, local_cleanup_pending) = next_action_fields(d.next_action);
            let mut obj = json!({
                "slug": d.slug,
                "title": d.title,
                "path": d.path,
                "type": d.spec_type,
                "module": d.module,
                "updated": d.updated,
                "next_action": next_action,
                "local_cleanup_pending": local_cleanup_pending,
            });
            if let Some(completed) = &d.completed {
                obj["completed"] = json!(completed);
            }
            obj
        })
        .collect();

    let seeds_json: Vec<Value> = seeds
        .iter()
        .map(|s| {
            json!({
                "slug": s.slug,
                "title": s.title,
                "maturity": s.maturity,
                "source": s.source,
            })
        })
        .collect();

    let root = json!({
        "generated": now,
        "active": active_json,
        "done": done_json,
        "seeds": seeds_json,
    });

    serde_json::to_string_pretty(&root).unwrap_or_default()
}

/// Simple relative path computation (avoids pulling in pathdiff crate).
fn pathdiff_relative(target: &Path, base: &Path) -> String {
    // Normalize both to components
    let target_components: Vec<_> = target.components().collect();
    let base_components: Vec<_> = base.components().collect();

    // Find common prefix length
    let common = target_components
        .iter()
        .zip(base_components.iter())
        .take_while(|(a, b)| a == b)
        .count();

    let ups = base_components.len() - common;
    let mut parts: Vec<String> = std::iter::repeat_n("..".to_string(), ups).collect();
    for comp in &target_components[common..] {
        parts.push(comp.as_os_str().to_string_lossy().to_string());
    }

    if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join("/")
    }
}

/// UTC timestamp formatted like Python's: "2026-06-05 12:00 UTC"
fn utc_now_formatted() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86400;
    let day_secs = secs % 86400;
    let hours = day_secs / 3600;
    let minutes = (day_secs % 3600) / 60;
    let date = epoch_days_to_iso(days);
    format!("{date} {hours:02}:{minutes:02} UTC")
}

/// Public API for reading a seed file.
pub fn read_seed_public(path: &Path) -> Option<SeedInfo> {
    read_seed(path)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::config::load_config;
    use std::process::Command;

    #[test]
    fn build_json_exposes_next_action_contract() {
        let active = vec![
            ActiveEntry {
                slug: "rev".into(),
                title: "Reviewing".into(),
                status: "accepted".into(),
                column: DeliveryColumn::InReview,
                risk: "standard".into(),
                module: "—".into(),
                docs: "spec".into(),
                path: "rev".into(),
                next_action: Some(NextActionKind::ReportMerge),
            },
            ActiveEntry {
                slug: "act".into(),
                title: "Active".into(),
                status: "approved".into(),
                column: DeliveryColumn::Active,
                risk: "standard".into(),
                module: "—".into(),
                docs: "spec".into(),
                path: "act".into(),
                next_action: None,
            },
        ];
        let done = vec![
            DoneEntry {
                slug: "cln".into(),
                title: "Cleanup".into(),
                path: "cln".into(),
                spec_type: "feature".into(),
                module: "—".into(),
                updated: "2026-06-30".into(),
                completed: None,
                sort_key: "2026-06-30".into(),
                next_action: Some(NextActionKind::LocalCleanupPending),
            },
            DoneEntry {
                slug: "arch".into(),
                title: "Archived".into(),
                path: "_done/arch".into(),
                spec_type: "feature".into(),
                module: "—".into(),
                updated: "2026-05-01".into(),
                completed: None,
                sort_key: "2026-05-01".into(),
                next_action: None,
            },
        ];

        let json = build_json("now", &active, &done, &[], "en");
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        let rev = &v["active"][0];
        assert_eq!(rev["next_action"]["kind"], "report_merge");
        assert!(
            rev["next_action"]["message"]
                .as_str()
                .unwrap()
                .contains("Merge the PR")
        );
        assert_eq!(rev["local_cleanup_pending"], false);

        let act = &v["active"][1];
        assert!(act["next_action"].is_null());
        assert_eq!(act["local_cleanup_pending"], false);

        let cln = &v["done"][0];
        assert_eq!(cln["next_action"]["kind"], "local_cleanup_pending");
        assert_eq!(cln["local_cleanup_pending"], true);

        let arch = &v["done"][1];
        assert!(arch["next_action"].is_null());
        assert_eq!(arch["local_cleanup_pending"], false);
    }

    #[test]
    fn build_json_next_action_message_is_language_aware() {
        let active = vec![ActiveEntry {
            slug: "rev".into(),
            title: "Reviewing".into(),
            status: "accepted".into(),
            column: DeliveryColumn::InReview,
            risk: "standard".into(),
            module: "—".into(),
            docs: "spec".into(),
            path: "rev".into(),
            next_action: Some(NextActionKind::ReportMerge),
        }];
        let json = build_json("now", &active, &[], &[], "ja");
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["active"][0]["next_action"]["kind"], "report_merge");
        assert!(
            v["active"][0]["next_action"]["message"]
                .as_str()
                .unwrap()
                .contains("マージ")
        );
    }

    #[test]
    fn generate_index_json_carries_delivery_next_actions() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        git_ok(repo, &["init", "-q", "-b", "main"]);
        git_ok(repo, &["config", "user.email", "t@example.com"]);
        git_ok(repo, &["config", "user.name", "Test"]);
        std::fs::write(repo.join("README.md"), "base\n").unwrap();
        git_ok(repo, &["add", "README.md"]);
        git_ok(
            repo,
            &[
                "commit",
                "-q",
                "-m",
                "feat: merged work",
                "-m",
                "Spec: cleanup-done",
            ],
        );
        git_ok(repo, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
        // A local feature branch for the merged spec → cleanup pending.
        git_ok(repo, &["branch", "feat/cleanup-done"]);
        // A pushed-and-unmerged branch → in review.
        git_ok(repo, &["checkout", "-q", "-b", "feat/in-review"]);
        std::fs::write(repo.join("README.md"), "branch\n").unwrap();
        git_ok(repo, &["commit", "-q", "-am", "branch"]);
        git_ok(
            repo,
            &["update-ref", "refs/remotes/origin/feat/in-review", "HEAD"],
        );

        let config = write_config(repo);
        write_spec(repo, "cleanup-done", "cleanup-done", "accepted");
        write_spec(repo, "in-review", "in-review", "accepted");

        let cfg = load_config(&config).unwrap();
        generate_index_quiet(&cfg).unwrap();
        let json: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(repo.join(".mochiflow/state/index.json")).unwrap(),
        )
        .unwrap();

        let in_review = json["active"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["slug"] == "in-review")
            .unwrap();
        assert_eq!(in_review["column"], "in_review");
        assert_eq!(in_review["next_action"]["kind"], "report_merge");
        assert_eq!(in_review["local_cleanup_pending"], false);

        let cleanup = json["done"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["slug"] == "cleanup-done")
            .unwrap();
        assert_eq!(cleanup["next_action"]["kind"], "local_cleanup_pending");
        assert_eq!(cleanup["local_cleanup_pending"], true);

        // After the local branch and scratch are gone, the same done-derived spec
        // no longer carries a cleanup next action.
        git_ok(repo, &["checkout", "-q", "main"]);
        git_ok(repo, &["branch", "-D", "feat/cleanup-done"]);
        generate_index_quiet(&cfg).unwrap();
        let json: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(repo.join(".mochiflow/state/index.json")).unwrap(),
        )
        .unwrap();
        let cleanup = json["done"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["slug"] == "cleanup-done")
            .unwrap();
        assert!(cleanup["next_action"].is_null());
        assert_eq!(cleanup["local_cleanup_pending"], false);
    }

    #[test]
    fn md_table_cell_escapes_pipes_and_newlines() {
        assert_eq!(md_table_cell("a|b\nc\rd"), r"a\|b c d");
    }

    #[test]
    fn read_seed_strips_quoted_frontmatter_values() {
        let tmp = tempfile::tempdir().unwrap();
        let seed = tmp.path().join("quoted-seed.md");
        std::fs::write(
            &seed,
            "---\nslug: \"quoted-seed\"\ntitle: \"A quoted title\"\nmaturity: \"seed\"\nsource: \"conversation\"\n---\n\n# Heading\n",
        )
        .unwrap();

        let info = read_seed(&seed).unwrap();
        assert_eq!(info.slug, "quoted-seed");
        assert_eq!(info.title, "A quoted title");
        assert_eq!(info.maturity, "seed");
        assert_eq!(info.source, "conversation");
        // Maturity must drive the seed emoji, not fall through to the unknown glyph.
        assert_eq!(status_emoji(&info.maturity), "🌱");
    }

    fn git_ok(root: &Path, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(root)
            .status()
            .unwrap();
        assert!(
            status.success(),
            "git {args:?} failed in {}",
            root.display()
        );
    }

    fn write_config(repo: &Path) -> std::path::PathBuf {
        let install = repo.join(".mochiflow");
        std::fs::create_dir_all(install.join("specs")).unwrap();
        std::fs::write(
            install.join("config.toml"),
            "schema_version = 1\ninstall_dir = \".mochiflow\"\nspecs_dir = \".mochiflow/specs\"\nindex = \".mochiflow/INDEX.md\"\n\n[constitution]\nproject = \".mochiflow/constitution.md\"\nlocal = \".mochiflow/constitution.local.md\"\n\n[context]\nproduct = \".mochiflow/context/product.md\"\nstructure = \".mochiflow/context/structure.md\"\ntech = \".mochiflow/context/tech.md\"\n\n[adr]\ndecisions = \".mochiflow/adr/decisions.md\"\npitfalls = \".mochiflow/adr/pitfalls.md\"\n\n[git]\nprovider = \"none\"\nbase_branch = \"main\"\n\n[adapter]\ntool = \"agents\"\n\n[surfaces.app]\ndescription = \"app\"\n\n[surfaces.app.verify]\ndefault = \"echo ok\"\n",
        )
        .unwrap();
        install.join("config.toml")
    }

    fn write_spec(repo: &Path, rel: &str, slug: &str, status: &str) {
        let dir = repo.join(".mochiflow/specs").join(rel);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("spec.yaml"),
            format!(
                "version: 1\nslug: {slug}\ntitle: {slug} title\ntype: feature\nsurfaces:\n  - app\nintegration: none\nrisk: standard\nstatus: {status}\nupdated: 2026-06-27\n"
            ),
        )
        .unwrap();
        std::fs::write(dir.join("spec.md"), format!("# {slug}\n")).unwrap();
    }

    #[test]
    fn render_index_splits_derived_columns_and_preserves_flat_done_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        git_ok(repo, &["init", "-q", "-b", "main"]);
        git_ok(repo, &["config", "user.email", "t@example.com"]);
        git_ok(repo, &["config", "user.name", "Test"]);
        std::fs::write(repo.join("README.md"), "base\n").unwrap();
        git_ok(repo, &["add", "README.md"]);
        git_ok(
            repo,
            &[
                "commit",
                "-q",
                "-m",
                "feat: flat done",
                "-m",
                "Spec: flat-done",
            ],
        );
        git_ok(repo, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
        git_ok(repo, &["checkout", "-q", "-b", "feat/in-review"]);
        std::fs::write(repo.join("README.md"), "branch\n").unwrap();
        git_ok(repo, &["commit", "-q", "-am", "branch"]);
        git_ok(
            repo,
            &["update-ref", "refs/remotes/origin/feat/in-review", "HEAD"],
        );

        let config = write_config(repo);
        write_spec(repo, "active", "active", "approved");
        write_spec(repo, "ready", "ready", "accepted");
        write_spec(repo, "in-review", "in-review", "accepted");
        write_spec(repo, "flat-done", "flat-done", "accepted");
        write_spec(repo, "_done/archived", "archived", "done");

        let cfg = load_config(&config).unwrap();
        let out = render_index(&cfg, NORMALIZED_TIMESTAMP);

        assert!(out.contains("## Active specs"), "{out}");
        assert!(
            out.contains("| [active](specs/active/) | 🟢 approved |"),
            "{out}"
        );
        assert!(out.contains("## Ready specs"), "{out}");
        assert!(
            out.contains("| [ready](specs/ready/) | 🔵 accepted |"),
            "{out}"
        );
        assert!(out.contains("## In Review specs"), "{out}");
        assert!(
            out.contains("| [in-review](specs/in-review/) | 🔵 accepted |"),
            "{out}"
        );
        assert!(out.contains("| [flat-done](specs/flat-done/) |"), "{out}");
        assert!(
            out.contains("| [archived](specs/_done/archived/) |"),
            "{out}"
        );
        assert!(
            !out.contains("[flat-done](specs/_done/flat-done/)"),
            "{out}"
        );
    }
}
