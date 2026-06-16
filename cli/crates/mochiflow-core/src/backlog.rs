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

/// Validate a seed exists.
pub fn validate_seed(cfg: &Config, slug: &str) -> i32 {
    let path = cfg
        .specs_dir_path()
        .join("_backlog")
        .join(format!("{slug}.md"));
    if !path.exists() {
        println!("FAIL: backlog seed not found: {slug}");
        return 1;
    }
    println!("OK: backlog seed {slug}");
    0
}
