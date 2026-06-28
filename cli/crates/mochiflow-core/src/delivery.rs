//! Delivery-state derivation.
//!
//! Delivery state is **observed, never stored**. For a given spec this module
//! resolves exactly one delivery column from VCS/provider reality plus the
//! asserted `spec.yaml` status. Resolution precedence (highest first):
//!
//!   Done > In Review > Ready > Active
//!
//! - **Done**: the provider reports the PR merged, OR a `Spec: {slug}` trailer is
//!   reachable from `origin/{base_branch}`, OR the spec is a legacy archived
//!   `_done/` spec (status `done`). Only two live signals (provider, trailer);
//!   the human merge report is never persisted as a merged signal.
//! - **In Review**: not Done, and either the provider reports an open PR, or
//!   (`provider = none`) the spec branch is pushed to `origin` and unmerged.
//! - **Ready**: `status: accepted`, not Done, not In Review (accepted-unpushed).
//! - **Active**: `draft` / `approved` (and any other asserted state).
//!
//! Derivation degrades gracefully: provider-unavailable and `provider = none`
//! fall back to local-git signals and never error.

use std::path::Path;
use std::process::Command;

use crate::config::Config;

/// The single delivery column a spec resolves to. Backlog (a `_backlog/` seed)
/// is not a spec and is handled by the board layer, not here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryColumn {
    /// Merged (provider) or trailer-in-base, or legacy archived `_done/`.
    Done,
    /// An open PR (provider) or a pushed-and-unmerged branch (`provider = none`).
    InReview,
    /// `accepted` but not yet delivered.
    Ready,
    /// `draft` / `approved`.
    Active,
}

impl DeliveryColumn {
    /// Stable lowercase identifier for the column.
    pub fn as_str(self) -> &'static str {
        match self {
            DeliveryColumn::Done => "done",
            DeliveryColumn::InReview => "in_review",
            DeliveryColumn::Ready => "ready",
            DeliveryColumn::Active => "active",
        }
    }
}

/// Observed delivery signals for one spec. Provider signals are best-effort: a
/// provider that is unavailable yields `false` for both, so resolution falls
/// back to the local-git signals.
#[derive(Debug, Clone, Copy, Default)]
pub struct DeliverySignals {
    /// The provider reports the spec's PR merged.
    pub provider_merged: bool,
    /// The provider reports an open PR for the spec branch.
    pub provider_open_pr: bool,
    /// A `Spec: {slug}` trailer is reachable from `origin/{base_branch}`.
    pub trailer_in_base: bool,
    /// The spec branch is pushed to `origin` and not merged into the base.
    pub branch_pushed_unmerged: bool,
}

/// Pure resolution of a spec's delivery column from its asserted `status` and
/// observed signals. This is the single source of the precedence rule and is
/// the unit-tested core; all I/O lives in `gather_signals` / `derive_column`.
pub fn resolve_column(status: &str, signals: &DeliverySignals) -> DeliveryColumn {
    // Done outranks every other column (merged outranks an open PR).
    if status == "done" || signals.provider_merged || signals.trailer_in_base {
        return DeliveryColumn::Done;
    }
    // In Review: a live open PR, or the local-git pushed-and-unmerged signal.
    if signals.provider_open_pr || signals.branch_pushed_unmerged {
        return DeliveryColumn::InReview;
    }
    // Ready: accepted but neither merged nor in review.
    if status == "accepted" {
        return DeliveryColumn::Ready;
    }
    // Active: draft / approved (and defensively any other asserted state).
    DeliveryColumn::Active
}

/// Derive the delivery column for a spec, gathering signals from git/provider.
/// Never errors; degrades to local/last-known signals.
pub fn derive_column(cfg: &Config, slug: &str, status: &str, spec_type: &str) -> DeliveryColumn {
    // Legacy archived specs already assert `done`; they resolve to Done without
    // any git/provider probe (and never re-trigger network calls).
    if status == "done" {
        return DeliveryColumn::Done;
    }
    let branch = branch_name(spec_type, slug);
    let signals = gather_signals(cfg, slug, &branch);
    resolve_column(status, &signals)
}

/// Branch name `{prefix}/{slug}` per the git branch convention: `feature` maps
/// to `feat`; every other type is used as-is.
pub fn branch_name(spec_type: &str, slug: &str) -> String {
    let prefix = match spec_type {
        "feature" => "feat",
        other => other,
    };
    format!("{prefix}/{slug}")
}

/// Gather observed signals for a spec branch. All probes are best-effort and
/// return `false` on any failure (missing ref, no remote, missing provider CLI).
fn gather_signals(cfg: &Config, slug: &str, branch: &str) -> DeliverySignals {
    let root = &cfg.repo_root;
    let base = &cfg.git.base_branch;

    let trailer_in_base = trailer_reachable_from_base(root, slug, base);
    let (provider_merged, provider_open_pr) = match cfg.git.provider.as_str() {
        "github" => provider_pr_state(root, branch),
        // `provider = none` (and any unrecognized provider) uses local git only.
        _ => (false, false),
    };
    let branch_pushed_unmerged = if cfg.git.provider == "none" {
        let branch_pushed = remote_branch_exists(root, branch);
        branch_pushed && !remote_branch_merged(root, branch, base)
    } else {
        false
    };

    DeliverySignals {
        provider_merged,
        provider_open_pr,
        trailer_in_base,
        branch_pushed_unmerged,
    }
}

/// True when a commit carrying a `Spec: {slug}` trailer is reachable from
/// `origin/{base_branch}`. Requires the `origin/{base}` ref to be present
/// locally (i.e. fetched); absent ref → `false`, never an error.
fn trailer_reachable_from_base(root: &Path, slug: &str, base: &str) -> bool {
    let base_ref = format!("origin/{base}");
    let grep = format!("^Spec: {}$", regex::escape(slug));
    git_capture(
        root,
        &[
            "log",
            &base_ref,
            "--extended-regexp",
            "--grep",
            &grep,
            "--format=%H",
            "--max-count=1",
        ],
    )
    .is_some_and(|out| !out.trim().is_empty())
}

/// True when `origin/{branch}` exists locally.
fn remote_branch_exists(root: &Path, branch: &str) -> bool {
    git_capture(
        root,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("refs/remotes/origin/{branch}"),
        ],
    )
    .is_some()
}

/// True when `origin/{branch}` is an ancestor of `origin/{base}` (i.e. merged).
fn remote_branch_merged(root: &Path, branch: &str, base: &str) -> bool {
    Command::new("git")
        .args(["merge-base", "--is-ancestor"])
        .arg(format!("origin/{branch}"))
        .arg(format!("origin/{base}"))
        .current_dir(root)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Query the GitHub provider for the spec branch PR state. Returns
/// `(merged, open)`. Any failure (gh missing, no PR, offline) → `(false, false)`
/// so derivation falls back to local-git signals.
fn provider_pr_state(root: &Path, branch: &str) -> (bool, bool) {
    let out = Command::new("gh")
        .args(["pr", "view", branch, "--json", "state", "--jq", ".state"])
        .current_dir(root)
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let state = String::from_utf8_lossy(&o.stdout).trim().to_uppercase();
            match state.as_str() {
                "MERGED" => (true, false),
                "OPEN" => (false, true),
                _ => (false, false),
            }
        }
        _ => (false, false),
    }
}

/// Run a git command in `root`, returning trimmed stdout on success, else None.
fn git_capture(root: &Path, args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::config::load_config;
    use std::process::Command;

    fn signals(
        provider_merged: bool,
        provider_open_pr: bool,
        trailer_in_base: bool,
        branch_pushed_unmerged: bool,
    ) -> DeliverySignals {
        DeliverySignals {
            provider_merged,
            provider_open_pr,
            trailer_in_base,
            branch_pushed_unmerged,
        }
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

    fn write_config(repo: &Path, provider: &str) -> std::path::PathBuf {
        let install = repo.join(".mochiflow");
        std::fs::create_dir_all(&install).unwrap();
        std::fs::write(
            install.join("config.toml"),
            format!(
                "schema_version = 1\ninstall_dir = \".mochiflow\"\nspecs_dir = \".mochiflow/specs\"\nindex = \".mochiflow/INDEX.md\"\n\n[constitution]\nproject = \".mochiflow/constitution.md\"\nlocal = \".mochiflow/constitution.local.md\"\n\n[context]\nproduct = \".mochiflow/context/product.md\"\nstructure = \".mochiflow/context/structure.md\"\ntech = \".mochiflow/context/tech.md\"\n\n[adr]\ndecisions = \".mochiflow/adr/decisions.md\"\npitfalls = \".mochiflow/adr/pitfalls.md\"\n\n[git]\nprovider = \"{provider}\"\nbase_branch = \"main\"\n\n[adapter]\ntool = \"agents\"\n\n[surfaces.app]\ndescription = \"app\"\n\n[surfaces.app.verify]\ndefault = \"echo ok\"\n"
            ),
        )
        .unwrap();
        install.join("config.toml")
    }

    fn materialize_repo_with_unmerged_remote_branch() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        git_ok(repo, &["init", "-q", "-b", "main"]);
        git_ok(repo, &["config", "user.email", "t@example.com"]);
        git_ok(repo, &["config", "user.name", "Test"]);
        std::fs::write(repo.join("README.md"), "base\n").unwrap();
        git_ok(repo, &["add", "README.md"]);
        git_ok(repo, &["commit", "-q", "-m", "base"]);
        git_ok(repo, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
        git_ok(repo, &["checkout", "-q", "-b", "feat/sample"]);
        std::fs::write(repo.join("README.md"), "branch\n").unwrap();
        git_ok(repo, &["commit", "-q", "-am", "branch"]);
        git_ok(
            repo,
            &["update-ref", "refs/remotes/origin/feat/sample", "HEAD"],
        );
        tmp
    }

    #[test]
    fn accepted_unpushed_is_ready() {
        assert_eq!(
            resolve_column("accepted", &signals(false, false, false, false)),
            DeliveryColumn::Ready
        );
    }

    #[test]
    fn accepted_pushed_unmerged_is_in_review() {
        assert_eq!(
            resolve_column("accepted", &signals(false, false, false, true)),
            DeliveryColumn::InReview
        );
    }

    #[test]
    fn github_pushed_without_open_pr_stays_ready() {
        let tmp = materialize_repo_with_unmerged_remote_branch();
        let config = write_config(tmp.path(), "github");
        let cfg = load_config(&config).unwrap();
        assert_eq!(
            derive_column(&cfg, "sample", "accepted", "feature"),
            DeliveryColumn::Ready
        );
    }

    #[test]
    fn provider_none_pushed_unmerged_is_in_review() {
        let tmp = materialize_repo_with_unmerged_remote_branch();
        let config = write_config(tmp.path(), "none");
        let cfg = load_config(&config).unwrap();
        assert_eq!(
            derive_column(&cfg, "sample", "accepted", "feature"),
            DeliveryColumn::InReview
        );
    }

    #[test]
    fn merged_trailer_is_done() {
        assert_eq!(
            resolve_column("approved", &signals(false, false, true, false)),
            DeliveryColumn::Done
        );
    }

    #[test]
    fn provider_merged_is_done() {
        assert_eq!(
            resolve_column("accepted", &signals(true, false, false, false)),
            DeliveryColumn::Done
        );
    }

    #[test]
    fn conflicting_open_pr_and_merge_trailer_done_wins() {
        // A spec with both an open PR and a merge trailer reachable from base
        // resolves to Done (Done outranks In Review).
        assert_eq!(
            resolve_column("accepted", &signals(false, true, true, true)),
            DeliveryColumn::Done
        );
    }

    #[test]
    fn provider_unavailable_falls_back_to_local_signals() {
        // Provider signals are false (unavailable); the local trailer signal
        // still resolves Done.
        assert_eq!(
            resolve_column("approved", &signals(false, false, true, false)),
            DeliveryColumn::Done
        );
        // And a pushed-unmerged branch resolves In Review without any provider.
        assert_eq!(
            resolve_column("accepted", &signals(false, false, false, true)),
            DeliveryColumn::InReview
        );
    }

    #[test]
    fn legacy_archived_done_is_done() {
        assert_eq!(
            resolve_column("done", &signals(false, false, false, false)),
            DeliveryColumn::Done
        );
    }

    #[test]
    fn draft_and_approved_are_active() {
        assert_eq!(
            resolve_column("draft", &signals(false, false, false, false)),
            DeliveryColumn::Active
        );
        assert_eq!(
            resolve_column("approved", &signals(false, false, false, false)),
            DeliveryColumn::Active
        );
    }

    #[test]
    fn open_pr_outranks_ready() {
        // An accepted spec with an open PR is In Review, not Ready.
        assert_eq!(
            resolve_column("accepted", &signals(false, true, false, false)),
            DeliveryColumn::InReview
        );
    }

    #[test]
    fn branch_name_maps_feature_to_feat() {
        assert_eq!(branch_name("feature", "my-spec"), "feat/my-spec");
        assert_eq!(branch_name("fix", "my-spec"), "fix/my-spec");
        assert_eq!(branch_name("chore", "my-spec"), "chore/my-spec");
    }

    #[test]
    fn git_probes_never_error_in_non_repo_dir() {
        // Degradation contract: probes against a non-git directory return false
        // rather than panicking or erroring.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        assert!(!trailer_reachable_from_base(root, "x", "main"));
        assert!(!remote_branch_exists(root, "feat/x"));
        assert!(!remote_branch_merged(root, "feat/x", "main"));
    }
}
