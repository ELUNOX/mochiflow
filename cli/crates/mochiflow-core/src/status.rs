//! `mochiflow status`: render the live delivery board (read-only).
//!
//! The board is **computed, never stored**. It unions the asserted `spec.yaml`
//! state with the derived delivery state (`delivery.rs`) and groups specs into
//! five columns: Backlog, Active, Ready, In Review, Done. `status` writes no
//! file (it never regenerates `INDEX.md`) and exits 0 even when derivation
//! degrades (offline / `provider = none` / detached HEAD).

use std::path::Path;
use std::process::Command;

use crate::config::Config;
use crate::delivery::{self, DeliveryColumn, NextActionKind};
use crate::index::read_seed_public;
use crate::spec_meta::read_spec_metadata;

/// One board row.
#[derive(Debug, Clone)]
pub struct BoardEntry {
    pub slug: String,
    pub title: String,
    /// Derived conversational next action (in review / local cleanup pending),
    /// or `None` for columns with no action.
    pub next_action: Option<NextActionKind>,
}

/// The computed board, one bucket per column.
#[derive(Debug, Default)]
pub struct Board {
    pub backlog: Vec<BoardEntry>,
    pub active: Vec<BoardEntry>,
    pub ready: Vec<BoardEntry>,
    pub in_review: Vec<BoardEntry>,
    pub done: Vec<BoardEntry>,
}

/// Entry point for `mochiflow status`. Read-only: prints the board and returns
/// 0 even when derivation degrades. `--fetch` performs one `git fetch` first.
pub fn run_status(cfg: &Config, fetch: bool) -> i32 {
    if fetch {
        // Best-effort network refresh; never fail the command on a fetch error.
        let _ = Command::new("git")
            .args(["fetch", "origin"])
            .current_dir(&cfg.repo_root)
            .status();
    }
    let board = compute_board(cfg);
    print!(
        "{}",
        render_board(&board, cfg.conversation_output_language())
    );
    0
}

/// Compute the board by unioning asserted spec state with derived delivery
/// state. Pure with respect to the filesystem/VCS it reads; it writes nothing.
pub fn compute_board(cfg: &Config) -> Board {
    let specs_dir = cfg.specs_dir_path();
    let mut board = Board::default();
    if !specs_dir.exists() {
        return board;
    }

    // Active specs: direct children that are not dot/underscore prefixed.
    let mut active_dirs = child_dirs(&specs_dir, |name| {
        !name.starts_with('.') && !name.starts_with('_')
    });
    active_dirs.sort();
    for dir in &active_dirs {
        if let Ok(meta) = read_spec_metadata(dir) {
            let column = delivery::derive_column(cfg, meta.slug(), meta.status(), meta.spec_type());
            let next_action = delivery::derive_next_action(
                cfg,
                meta.slug(),
                meta.status(),
                meta.spec_type(),
                column,
            );
            push_entry(&mut board, column, meta.slug(), meta.title(), next_action);
        }
    }

    // Legacy archived specs under `_done/` always render in Done (status `done`
    // resolves to Done in delivery derivation).
    let done_dir = specs_dir.join("_done");
    let mut done_dirs = child_dirs(&done_dir, |_| true);
    done_dirs.sort();
    for dir in &done_dirs {
        if let Ok(meta) = read_spec_metadata(dir) {
            let column = delivery::derive_column(cfg, meta.slug(), meta.status(), meta.spec_type());
            let next_action = delivery::derive_next_action(
                cfg,
                meta.slug(),
                meta.status(),
                meta.spec_type(),
                column,
            );
            push_entry(&mut board, column, meta.slug(), meta.title(), next_action);
        }
    }

    // Backlog seeds (`_backlog/*.md`).
    let backlog_dir = specs_dir.join("_backlog");
    if backlog_dir.exists()
        && let Ok(entries) = std::fs::read_dir(&backlog_dir)
    {
        let mut seed_paths: Vec<_> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.extension().is_some_and(|e| e == "md")
                    && p.file_name().and_then(|n| n.to_str()) != Some("README.md")
            })
            .collect();
        seed_paths.sort();
        for path in &seed_paths {
            if let Some(seed) = read_seed_public(path) {
                board.backlog.push(BoardEntry {
                    slug: seed.slug,
                    title: seed.title,
                    next_action: None,
                });
            }
        }
    }

    board
}

fn push_entry(
    board: &mut Board,
    column: DeliveryColumn,
    slug: &str,
    title: &str,
    next_action: Option<NextActionKind>,
) {
    let entry = BoardEntry {
        slug: slug.to_string(),
        title: title.to_string(),
        next_action,
    };
    match column {
        DeliveryColumn::Done => board.done.push(entry),
        DeliveryColumn::InReview => board.in_review.push(entry),
        DeliveryColumn::Ready => board.ready.push(entry),
        DeliveryColumn::Active => board.active.push(entry),
    }
}

fn child_dirs(dir: &Path, keep: impl Fn(&str) -> bool) -> Vec<std::path::PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter(|e| {
            let name = e.file_name();
            keep(&name.to_string_lossy())
        })
        .map(|e| e.path())
        .collect()
}

fn render_board(board: &Board, language: &str) -> String {
    let mut out = String::from("# 📋 Spec Board\n");
    for (heading, entries) in [
        ("Backlog", &board.backlog),
        ("Active", &board.active),
        ("Ready", &board.ready),
        ("In Review", &board.in_review),
        ("Done", &board.done),
    ] {
        out.push_str(&format!("\n## {} ({})\n\n", heading, entries.len()));
        if entries.is_empty() {
            out.push_str("（なし）\n");
        } else {
            for entry in entries {
                out.push_str(&format!("- {} — {}\n", entry.slug, entry.title));
                if let Some(action) = entry.next_action {
                    out.push_str(&format!("    ↳ {}\n", action.message(language)));
                }
            }
        }
    }
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::config::load_config;

    fn write_config(repo: &Path) {
        std::fs::create_dir_all(repo.join(".mochiflow/adr")).unwrap();
        std::fs::create_dir_all(repo.join(".mochiflow/specs")).unwrap();
        std::fs::write(
            repo.join(".mochiflow/config.toml"),
            "schema_version = 1\ninstall_dir = \".mochiflow\"\nspecs_dir = \".mochiflow/specs\"\nindex = \".mochiflow/INDEX.md\"\n\n[constitution]\nproject = \".mochiflow/constitution.md\"\nlocal = \".mochiflow/constitution.local.md\"\n\n[context]\nproduct = \".mochiflow/context/product.md\"\nstructure = \".mochiflow/context/structure.md\"\ntech = \".mochiflow/context/tech.md\"\n\n[adr]\ndecisions = \".mochiflow/adr/decisions.md\"\npitfalls = \".mochiflow/adr/pitfalls.md\"\n\n[git]\nbase_branch = \"main\"\n\n[adapter]\ntool = \"agents\"\n\n[surfaces.app]\ndescription = \"app\"\n\n[surfaces.app.verify]\ndefault = \"echo ok\"\n",
        )
        .unwrap();
    }

    fn write_spec(repo: &Path, rel: &str, slug: &str, status: &str) {
        let dir = repo.join(".mochiflow/specs").join(rel);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("spec.yaml"),
            format!(
                "version: 1\nslug: {slug}\ntitle: {slug} title\ntype: feature\nsurfaces:\n  - app\nintegration: none\nrisk: standard\nstatus: {status}\n"
            ),
        )
        .unwrap();
        std::fs::write(dir.join("spec.md"), format!("# {slug}\n")).unwrap();
    }

    fn slugs(entries: &[BoardEntry]) -> Vec<&str> {
        entries.iter().map(|e| e.slug.as_str()).collect()
    }

    #[test]
    fn board_places_specs_by_asserted_and_derived_state() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        write_config(repo);
        // No remote: accepted → Ready, draft/approved → Active, archived done → Done.
        write_spec(repo, "draft-spec", "draft-spec", "draft");
        write_spec(repo, "approved-spec", "approved-spec", "approved");
        write_spec(repo, "accepted-spec", "accepted-spec", "accepted");
        write_spec(repo, "_done/old-spec", "old-spec", "done");
        std::fs::create_dir_all(repo.join(".mochiflow/specs/_backlog")).unwrap();
        std::fs::write(
            repo.join(".mochiflow/specs/_backlog/idea.md"),
            "# Idea seed\n",
        )
        .unwrap();

        let cfg = load_config(&repo.join(".mochiflow/config.toml")).unwrap();
        let board = compute_board(&cfg);

        assert_eq!(slugs(&board.active), vec!["approved-spec", "draft-spec"]);
        assert_eq!(slugs(&board.ready), vec!["accepted-spec"]);
        assert!(board.in_review.is_empty());
        assert_eq!(slugs(&board.done), vec!["old-spec"]);
        assert_eq!(slugs(&board.backlog), vec!["idea"]);
    }

    #[test]
    fn status_is_read_only_and_writes_no_index() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        write_config(repo);
        write_spec(repo, "accepted-spec", "accepted-spec", "accepted");

        let cfg = load_config(&repo.join(".mochiflow/config.toml")).unwrap();
        let code = run_status(&cfg, false);
        assert_eq!(code, 0);
        assert!(
            !repo.join(".mochiflow/INDEX.md").exists(),
            "status must not write INDEX.md"
        );
        assert!(
            !repo.join(".mochiflow/state/index.json").exists(),
            "status must not write any board file"
        );
    }

    #[test]
    fn status_fetch_degrades_without_remote() {
        // `--fetch` performs a best-effort `git fetch` before computing; with no
        // remote it degrades gracefully (exit 0) and still writes no file.
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        write_config(repo);
        write_spec(repo, "draft-spec", "draft-spec", "draft");

        let cfg = load_config(&repo.join(".mochiflow/config.toml")).unwrap();
        let code = run_status(&cfg, true);
        assert_eq!(code, 0, "status --fetch must not fail without a remote");
        assert!(!repo.join(".mochiflow/INDEX.md").exists());
    }

    #[test]
    fn render_board_shows_next_action_lines() {
        let mut board = Board::default();
        board.in_review.push(BoardEntry {
            slug: "rev".into(),
            title: "Reviewing".into(),
            next_action: Some(NextActionKind::ReportMerge),
        });
        board.done.push(BoardEntry {
            slug: "cln".into(),
            title: "Cleanup".into(),
            next_action: Some(NextActionKind::LocalCleanupPending),
        });
        board.active.push(BoardEntry {
            slug: "act".into(),
            title: "Active".into(),
            next_action: None,
        });

        let en = render_board(&board, "en");
        assert!(en.contains("- rev — Reviewing"), "{en}");
        assert!(en.contains("↳ Merge the PR in your provider"), "{en}");
        assert!(en.contains("↳ Local cleanup pending"), "{en}");
        // An entry without a next action gets no hint line.
        assert!(!en.contains("- act — Active\n    ↳"), "{en}");

        let ja = render_board(&board, "ja");
        assert!(ja.contains("マージ"), "{ja}");
        assert!(ja.contains("後片付け"), "{ja}");
    }
}
