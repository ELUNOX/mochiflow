//! Backlog seed operations (list / show / validate).

use crate::config::Config;
use crate::index::read_seed_public;

/// List all backlog seeds.
pub fn list_seeds(cfg: &Config) -> i32 {
    let backlog = cfg.specs_dir_path().join("_backlog");
    println!("Seeds:");
    if !backlog.exists() {
        println!("- none");
        return 0;
    }
    let mut files: Vec<_> = std::fs::read_dir(&backlog)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let n = e.file_name();
            let name = n.to_string_lossy();
            name.ends_with(".md") && name != "README.md"
        })
        .collect();
    files.sort_by_key(|e| e.file_name());

    if files.is_empty() {
        println!("- none");
        return 0;
    }
    for entry in &files {
        if let Some(seed) = read_seed_public(&entry.path()) {
            println!(
                "- {}: {} [{}, source={}]",
                seed.slug, seed.title, seed.maturity, seed.source
            );
        }
    }
    0
}

/// Show a single seed.
pub fn show_seed(cfg: &Config, slug: &str) -> i32 {
    let path = cfg
        .specs_dir_path()
        .join("_backlog")
        .join(format!("{slug}.md"));
    if !path.exists() {
        println!("FAIL: backlog seed not found: {slug}");
        return 1;
    }
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    if let Some(seed) = read_seed_public(&path) {
        println!(
            "Seed: {}\nTitle: {}\nMaturity: {}\nSource: {}",
            seed.slug, seed.title, seed.maturity, seed.source
        );
        println!("\n---");
        // Body after frontmatter
        let body = if let Some(rest) = text.strip_prefix("---\n") {
            if let Some(end) = rest.find("\n---\n") {
                &rest[end + 5..]
            } else {
                &text
            }
        } else {
            &text
        };
        print!("{}", body.trim_start_matches('\n'));
        if !body.ends_with('\n') {
            println!();
        }
    }
    0
}

/// Validate a seed's lifecycle invariants: maturity value, required frontmatter,
/// and required body headings. Returns 0 when valid, 1 on any FAIL.
pub fn validate_seed(cfg: &Config, slug: &str) -> i32 {
    let path = cfg
        .specs_dir_path()
        .join("_backlog")
        .join(format!("{slug}.md"));
    if !path.exists() {
        println!("FAIL: backlog seed not found: {slug}");
        return 1;
    }
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let failures = validate_seed_text(&text);
    if failures.is_empty() {
        println!("OK: backlog seed {slug}");
        0
    } else {
        for f in &failures {
            println!("FAIL: {slug}: {f}");
        }
        1
    }
}

/// Pure validation core: returns a list of failure messages (empty when valid).
/// Checks maturity value, maturity-specific frontmatter, and required headings.
fn validate_seed_text(text: &str) -> Vec<String> {
    let mut failures: Vec<String> = Vec::new();
    let fields = parse_frontmatter(text);
    let headings = body_headings(text);

    let maturity = fields.get("maturity").map(String::as_str).unwrap_or("");
    match maturity {
        "seed" => {
            for required in [
                "## Signal",
                "## Why It Matters",
                "## Evidence",
                "## Open Questions",
            ] {
                if !headings.iter().any(|h| h == required) {
                    failures.push(format!("seed is missing required heading {required}"));
                }
            }
        }
        "ready-for-plan" => {
            for (key, expected) in [("source", "conversation"), ("source_phase", "discuss")] {
                match fields.get(key) {
                    Some(v) if v == expected => {}
                    Some(v) => failures.push(format!(
                        "ready-for-plan handoff has {key}={v}, expected {expected}"
                    )),
                    None => failures.push(format!("ready-for-plan handoff is missing {key}")),
                }
            }
            for required in [
                "## Decision Summary",
                "## Decisions",
                "## Assumptions",
                "## Open Questions",
                "## Change Impact",
                "## Evidence",
            ] {
                if !headings.iter().any(|h| h == required) {
                    failures.push(format!(
                        "ready-for-plan handoff is missing required heading {required}"
                    ));
                }
            }
        }
        "" => failures.push("frontmatter is missing maturity".to_string()),
        other => failures.push(format!(
            "maturity must be one of: seed, ready-for-plan (got {other})"
        )),
    }

    for required in ["slug", "title", "created", "updated"] {
        if !fields.contains_key(required) {
            failures.push(format!("frontmatter is missing {required}"));
        }
    }

    failures
}

/// Parse a backlog file's YAML frontmatter into key→value pairs, stripping
/// surrounding quotes from values. Returns empty when no frontmatter is present.
fn parse_frontmatter(text: &str) -> std::collections::BTreeMap<String, String> {
    let mut fields = std::collections::BTreeMap::new();
    let Some(rest) = text.strip_prefix("---\n") else {
        return fields;
    };
    let Some(end) = rest.find("\n---\n") else {
        return fields;
    };
    for line in rest[..end].lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || !line.contains(':') {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            let value = value.trim().trim_matches('"').trim_matches('\'').trim();
            fields.insert(key.trim().to_string(), value.to_string());
        }
    }
    fields
}

/// Collect the `##`-level body headings (trimmed) from a backlog file.
fn body_headings(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim_end)
        .filter(|l| l.starts_with("## "))
        .map(|l| l.trim().to_string())
        .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::validate_seed_text;

    const VALID_SEED: &str = "---\n\
slug: \"foo\"\n\
title: \"Foo\"\n\
maturity: \"seed\"\n\
source: \"conversation\"\n\
created: \"2026-06-21\"\n\
updated: \"2026-06-21\"\n\
---\n\n\
# Foo\n\n\
## Signal\n\nx\n\n\
## Why It Matters\n\nx\n\n\
## Evidence\n\n- x\n\n\
## Open Questions\n\n- x\n";

    const VALID_HANDOFF: &str = "---\n\
slug: \"foo\"\n\
title: \"Foo\"\n\
maturity: \"ready-for-plan\"\n\
source: \"conversation\"\n\
source_phase: \"discuss\"\n\
created: \"2026-06-21\"\n\
updated: \"2026-06-21\"\n\
---\n\n\
# Foo\n\n\
## Decision Summary\n\nx\n\n\
## Decisions\n\n- x\n\n\
## Assumptions\n\n- x\n\n\
## Open Questions\n\n- x\n\n\
## Change Impact\n\n- x\n\n\
## Evidence\n\n- x\n";

    #[test]
    fn valid_seed_passes() {
        assert!(validate_seed_text(VALID_SEED).is_empty());
    }

    #[test]
    fn valid_ready_for_plan_handoff_passes() {
        assert!(validate_seed_text(VALID_HANDOFF).is_empty());
    }

    #[test]
    fn unknown_maturity_fails() {
        let text = VALID_SEED.replace("maturity: \"seed\"", "maturity: \"triaged\"");
        let failures = validate_seed_text(&text);
        assert!(
            failures
                .iter()
                .any(|f| f.contains("maturity must be one of")),
            "{failures:?}"
        );
    }

    #[test]
    fn missing_maturity_fails() {
        let text = VALID_SEED.replace("maturity: \"seed\"\n", "");
        let failures = validate_seed_text(&text);
        assert!(
            failures.iter().any(|f| f.contains("missing maturity")),
            "{failures:?}"
        );
    }

    #[test]
    fn handoff_missing_source_phase_fails() {
        let text = VALID_HANDOFF.replace("source_phase: \"discuss\"\n", "");
        let failures = validate_seed_text(&text);
        assert!(
            failures.iter().any(|f| f.contains("missing source_phase")),
            "{failures:?}"
        );
    }

    #[test]
    fn seed_missing_required_heading_fails() {
        let text = VALID_SEED.replace("## Signal\n\nx\n\n", "");
        let failures = validate_seed_text(&text);
        assert!(
            failures
                .iter()
                .any(|f| f.contains("missing required heading ## Signal")),
            "{failures:?}"
        );
    }

    #[test]
    fn handoff_wrong_source_fails() {
        let text = VALID_HANDOFF.replace("source: \"conversation\"", "source: \"seed\"");
        let failures = validate_seed_text(&text);
        assert!(
            failures
                .iter()
                .any(|f| f.contains("source=seed, expected conversation")),
            "{failures:?}"
        );
    }
}
