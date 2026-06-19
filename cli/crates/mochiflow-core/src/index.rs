//! Index generation: produce INDEX.md dashboard + state/index.json.

use std::path::Path;

use crate::config::Config;
use crate::spec_meta::read_spec_metadata;

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
    risk: String,
    module: String,
    docs: String,
}

/// Done spec entry.
struct DoneEntry {
    slug: String,
    title: String,
    spec_type: String,
    module: String,
    updated: String,
}

fn status_emoji(status: &str) -> &str {
    match status {
        "draft" => "📝",
        "approved" => "🟢",
        "done" => "✅",
        "seed" => "🌱",
        "triaged" => "📌",
        "ready-for-think" => "💭",
        "parked" => "⏸️",
        "spec-ready" => "📥",
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
    let mut fields = std::collections::BTreeMap::new();
    for line in fm_block.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || !line.contains(':') {
            continue;
        }
        let (key, value) = line.split_once(':')?;
        fields.insert(key.trim().to_string(), value.trim().to_string());
    }

    let title = fields.get("title").cloned().unwrap_or_else(|| slug.clone());

    Some(SeedInfo {
        slug: fields.get("slug").cloned().unwrap_or_else(|| slug.clone()),
        title,
        maturity: fields
            .get("maturity")
            .cloned()
            .unwrap_or_else(|| "—".to_string()),
        source: fields
            .get("source")
            .cloned()
            .unwrap_or_else(|| "—".to_string()),
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
        if let Ok(m) = read_spec_metadata(&d) {
            let mut docs_parts = vec!["spec".to_string()];
            if d.join("design.md").exists() {
                docs_parts.push("design".to_string());
            }
            if d.join("tasks.md").exists() {
                docs_parts.push("tasks".to_string());
            }
            active.push(ActiveEntry {
                slug: entry.file_name().to_string_lossy().to_string(),
                title: m.title().to_string(),
                status: m.status().to_string(),
                risk: m.risk().to_string(),
                module: m.module().unwrap_or("—").to_string(),
                docs: docs_parts.join("+"),
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
            if let Ok(m) = read_spec_metadata(&d) {
                let updated = m.updated().unwrap_or("").to_string();
                let updated = if updated.is_empty() {
                    mtime_isodate(&d)
                } else {
                    updated
                };
                done.push(DoneEntry {
                    slug: entry.file_name().to_string_lossy().to_string(),
                    title: m.title().to_string(),
                    spec_type: m.spec_type().to_string(),
                    module: m.module().unwrap_or("—").to_string(),
                    updated,
                });
            }
        }
        // Sort: by slug asc first (stable), then by updated desc
        done.sort_by(|a, b| a.slug.cmp(&b.slug));
        done.sort_by(|a, b| b.updated.cmp(&a.updated));
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
        format!(
            "| 📝 draft | {} |",
            active.iter().filter(|a| a.status == "draft").count()
        ),
        format!(
            "| 🟢 approved | {} |",
            active.iter().filter(|a| a.status == "approved").count()
        ),
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
        for s in &seeds {
            lines.push(format!(
                "| [{}]({specs_rel}/_backlog/{}.md) | {} | {} {} | {} |",
                s.slug,
                s.slug,
                s.title,
                status_emoji(&s.maturity),
                s.maturity,
                s.source
            ));
        }
    }

    lines.push(String::new());
    lines.push("## Active specs".to_string());
    lines.push(String::new());

    if active.is_empty() {
        lines.push("（なし）".to_string());
    } else {
        lines.push("| Spec | Status | Risk | Docs | Module |".to_string());
        lines.push("|:-----|:-------|:-----|:-----|:-------|".to_string());
        for a in &active {
            lines.push(format!(
                "| [{}]({specs_rel}/{}/) | {} {} | {} | {} | {} |",
                a.slug,
                a.slug,
                status_emoji(&a.status),
                a.status,
                a.risk,
                a.docs,
                a.module
            ));
        }
    }

    lines.push(String::new());
    lines.push("## Done (chronological)".to_string());
    lines.push(String::new());

    if done.is_empty() {
        lines.push("（なし）".to_string());
    } else {
        let mut current_month: Option<String> = None;
        for d in &done {
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
                "| {} | [{}]({specs_rel}/_done/{}/) | {} | {} |",
                d.updated, d.slug, d.slug, d.title, d.spec_type
            ));
        }
        lines.push(String::new());
        lines.push(format!("> done total: {}", done.len()));
    }

    lines.join("\n") + "\n"
}

pub fn is_index_stale(cfg: &Config) -> bool {
    let index_path = cfg.index_path();
    let actual = match std::fs::read_to_string(&index_path) {
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
pub fn generate_index(cfg: &Config) {
    generate_index_inner(cfg, true);
}

pub fn generate_index_quiet(cfg: &Config) {
    generate_index_inner(cfg, false);
}

fn generate_index_inner(cfg: &Config, print_summary: bool) {
    let (active, done, seeds) = collect(cfg);
    let now = utc_now_formatted();
    let content = render_index(cfg, &now);

    // Write INDEX.md
    let index_path = cfg.index_path();
    if let Some(parent) = index_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::write(&index_path, &content).ok();

    // Write state/index.json
    let state_dir = cfg.state_dir();
    std::fs::create_dir_all(&state_dir).ok();
    let json_data = build_json(&now, &active, &done, &seeds);
    std::fs::write(state_dir.join("index.json"), json_data).ok();

    if print_summary {
        println!(
            "index: {} + {}",
            index_path.display(),
            state_dir.join("index.json").display()
        );
    }
}

fn build_json(now: &str, active: &[ActiveEntry], done: &[DoneEntry], seeds: &[SeedInfo]) -> String {
    use serde_json::{Value, json};

    let active_json: Vec<Value> = active
        .iter()
        .map(|a| {
            json!({
                "slug": a.slug,
                "title": a.title,
                "status": a.status,
                "risk": a.risk,
                "module": a.module,
                "docs": a.docs,
            })
        })
        .collect();

    let done_json: Vec<Value> = done
        .iter()
        .map(|d| {
            json!({
                "slug": d.slug,
                "title": d.title,
                "type": d.spec_type,
                "module": d.module,
                "updated": d.updated,
            })
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
