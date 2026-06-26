//! Project detection: surfaces, git, and verify commands.
//!
//! Machine detection of *facts* (CLI, deterministic, pinnable). Judgement stays
//! explicit: detection never auto-adopts a git provider or auto-sets a
//! `pr_command` (AC-03), and uncertain fields are surfaced via
//! `# mochiflow: confirm` markers (AC-02) rather than guessed.
//!
//! All subprocesses are invoked via argv arrays (never the shell — AC-10); a
//! detection failure falls back to a low-confidence default and never corrupts
//! `config.toml`.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::{Command, Stdio};

/// Confidence level for confirm-marker gating (design.md decision table).
/// Ordered: Low < Medium < High (lower = less confident). `High` writes no
/// marker; `Medium` / `Low` attach `# mochiflow: confirm`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
    Low,
    Medium,
    High,
}

impl Confidence {
    /// True when a `# mochiflow: confirm` marker should be attached.
    pub fn needs_confirm(self) -> bool {
        self < Confidence::High
    }
}

/// A detected build surface (name + human description + verify command).
#[derive(Debug, Clone)]
pub struct DetectedSurface {
    pub name: String,
    pub description: String,
    pub verify: String,
    pub confidence: Confidence,
}

/// Detected git facts. The provider is recorded as a *fact* for presentation
/// only — it is never auto-adopted into config (provider stays `none`, manual
/// PR remains the first-class default; AC-03). No `pr_command` is inferred.
#[derive(Debug, Clone)]
pub struct DetectedGit {
    /// Detected remote provider (`github` / `gitlab` / `azure-devops` /
    /// `none`). Presentation-only; config keeps `provider = "none"`.
    pub provider: String,
    pub base_branch: String,
    pub branch_confidence: Confidence,
}

impl DetectedGit {
    /// True when a known remote provider was detected (presented as a confirm
    /// item, never auto-adopted).
    pub fn has_known_provider(&self) -> bool {
        self.provider != "none"
    }
}

/// Detect git provider (fact only) and base branch via argv subprocesses.
/// Never sets a `pr_command` and never adopts the provider into config (AC-03).
pub fn detect_git(root: &Path) -> DetectedGit {
    let mut result = DetectedGit {
        provider: "none".into(),
        base_branch: "main".into(),
        branch_confidence: Confidence::Medium, // fallback main
    };

    let url = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    if url.contains("github") {
        result.provider = "github".into();
    } else if url.contains("gitlab") {
        result.provider = "gitlab".into();
    } else if url.contains("dev.azure") || url.contains("visualstudio") {
        result.provider = "azure-devops".into();
    }

    if let Some(branch) = detect_default_branch(root) {
        result.base_branch = branch;
        result.branch_confidence = Confidence::High;
    }

    result
}

fn git_stdout(root: &Path, args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

fn detect_default_branch(root: &Path) -> Option<String> {
    if let Some(symref) = git_stdout(
        root,
        &[
            "symbolic-ref",
            "--quiet",
            "--short",
            "refs/remotes/origin/HEAD",
        ],
    ) && let Some(branch) = symref.strip_prefix("origin/")
        && !branch.is_empty()
    {
        return Some(branch.to_string());
    }

    for branch in ["main", "master"] {
        if Command::new("git")
            .args([
                "show-ref",
                "--verify",
                "--quiet",
                &format!("refs/heads/{branch}"),
            ])
            .current_dir(root)
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return Some(branch.to_string());
        }
    }

    None
}

/// Read scripts from package.json.
fn read_pkg_scripts(pkg: &Path) -> BTreeMap<String, String> {
    let text = match std::fs::read_to_string(pkg) {
        Ok(t) => t,
        Err(_) => return BTreeMap::new(),
    };
    let v: serde_json::Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(_) => return BTreeMap::new(),
    };
    v.get("scripts")
        .and_then(|s| s.as_object())
        .map(|obj| {
            obj.iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                .collect()
        })
        .unwrap_or_default()
}

/// Infer a verify command from project files.
///
/// `runner` is the resolved package runner for this project (detected once at
/// the root level by `detect_runner`). This function never re-derives the
/// runner from lock files — that decision is made in one place.
///
/// Returns `Medium` confidence (not `High`) when a package manifest exposes
/// more than one candidate script (e.g. both `test` and `lint`), so the
/// ambiguity is surfaced as a confirm marker; `Low` when nothing is detected.
fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn subdir_command_prefix(subdir: Option<&str>) -> String {
    subdir
        .map(|s| format!("cd {} && ", shell_single_quote(s)))
        .unwrap_or_default()
}

fn infer_verify(root: &Path, subdir: Option<&str>, runner: &str) -> (String, Confidence) {
    let base = match subdir {
        Some(s) => root.join(s),
        None => root.to_path_buf(),
    };
    let pkg = base.join("package.json");
    if pkg.exists() {
        let scripts = read_pkg_scripts(&pkg);
        let candidates = ["test", "check", "lint", "build"];
        let present: Vec<&str> = candidates
            .iter()
            .copied()
            .filter(|c| scripts.contains_key(*c))
            .collect();
        if let Some(cmd) = present.first() {
            let prefix = subdir_command_prefix(subdir);
            let run_cmd = format!("{runner} run");
            let confidence = if present.len() == 1 {
                Confidence::High
            } else {
                Confidence::Medium
            };
            return (format!("{prefix}{run_cmd} {cmd}"), confidence);
        }
    }
    let cargo = base.join("Cargo.toml");
    if cargo.exists() {
        let prefix = subdir_command_prefix(subdir);
        return (format!("{prefix}cargo test"), Confidence::High);
    }
    let pyproject = base.join("pyproject.toml");
    if pyproject.exists() {
        let prefix = subdir_command_prefix(subdir);
        return (format!("{prefix}uv run ruff check ."), Confidence::High);
    }
    ("TODO: define test command".into(), Confidence::Low)
}

/// Detect the package runner from lock files at a given base path.
fn detect_runner(base: &Path) -> &'static str {
    if base.join("bun.lock").exists() || base.join("bun.lockb").exists() {
        "bun"
    } else if base.join("pnpm-lock.yaml").exists() {
        "pnpm"
    } else {
        "npm"
    }
}

/// For a monorepo (multiple pkg subdirs), infer a root-level workspace verify
/// command rather than `cd subdir && ...`. Always returns medium confidence
/// (confirm) because "which script to verify with" is a judgement call.
fn infer_workspace_verify(root: &Path) -> (String, Confidence) {
    // Turborepo takes precedence — its own task runner.
    if root.join("turbo.json").exists() {
        return ("turbo run build".into(), Confidence::Medium);
    }
    // nx.json → Nx workspace.
    if root.join("nx.json").exists() {
        return ("nx run-many -t build".into(), Confidence::Medium);
    }
    // Fallback: use the detected package runner with --filter.
    let runner = detect_runner(root);
    let cmd = match runner {
        "bun" => "bun --filter '*' build".to_string(),
        "pnpm" => "pnpm -r run build".to_string(),
        _ => "npm run build".to_string(),
    };
    (cmd, Confidence::Medium)
}

/// Detect surfaces from project structure.
pub fn detect_surfaces(root: &Path) -> Vec<DetectedSurface> {
    let mut surfaces = Vec::new();

    // Resolve the package runner once from the project root; all infer_verify
    // calls share this instead of re-deriving it per subdir.
    let runner = detect_runner(root);

    // iOS / Apple
    if root.join("Project.swift").exists()
        || std::fs::read_dir(root)
            .into_iter()
            .flatten()
            .flatten()
            .any(|e| e.file_name().to_string_lossy().ends_with(".xcodeproj"))
    {
        surfaces.push(DetectedSurface {
            name: "ios".into(),
            description: "iOS / Apple app".into(),
            verify: "TODO: define test command".into(),
            confidence: Confidence::Low,
        });
    }

    // Rust crate / workspace at root.
    if root.join("Cargo.toml").exists() {
        let (v, c) = infer_verify(root, None, runner);
        surfaces.push(DetectedSurface {
            name: "cli".into(),
            description: "Rust crate / workspace".into(),
            verify: v,
            confidence: c,
        });
    }

    // Python backend
    if root.join("pyproject.toml").exists() {
        let (v, c) = infer_verify(root, None, runner);
        surfaces.push(DetectedSurface {
            name: "api".into(),
            description: "Python backend".into(),
            verify: v,
            confidence: c,
        });
    } else if root.join("backend").is_dir() && root.join("backend/pyproject.toml").exists() {
        let (v, c) = infer_verify(root, Some("backend"), runner);
        surfaces.push(DetectedSurface {
            name: "api".into(),
            description: "Python backend".into(),
            verify: v,
            confidence: c,
        });
    }

    // Web / monorepo (package.json in subdirs).
    let mut subdirs_with_pkg: Vec<String> = std::fs::read_dir(root)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let n = e.file_name();
            let name = n.to_string_lossy().to_string();
            e.path().is_dir() && !name.starts_with('.') && e.path().join("package.json").exists()
        })
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    subdirs_with_pkg.sort();

    if subdirs_with_pkg.len() > 1 {
        // Monorepo: use a workspace-level verify command (always medium/confirm).
        let (ws_verify, ws_confidence) = infer_workspace_verify(root);

        let api_dirs: Vec<_> = subdirs_with_pkg
            .iter()
            .filter(|d| d.contains("api"))
            .cloned()
            .collect();
        let web_dirs: Vec<_> = subdirs_with_pkg
            .iter()
            .filter(|d| !d.contains("api"))
            .cloned()
            .collect();
        if !web_dirs.is_empty() {
            let desc = if web_dirs.len() > 1 {
                format!("Web apps ({})", web_dirs.join(", "))
            } else {
                format!("Web app ({})", web_dirs[0])
            };
            surfaces.push(DetectedSurface {
                name: "web".into(),
                description: desc,
                verify: ws_verify.clone(),
                confidence: ws_confidence,
            });
        }
        if !api_dirs.is_empty() && !surfaces.iter().any(|s| s.name == "api") {
            let first = &api_dirs[0];
            let desc = if api_dirs.len() > 1 {
                format!("API ({})", api_dirs.join(", "))
            } else {
                format!("API ({first})")
            };
            // API may have its own runtime (e.g. Workers); use per-subdir but
            // always medium confidence (confirm).
            let (v, _) = infer_verify(root, Some(first), runner);
            surfaces.push(DetectedSurface {
                name: "api".into(),
                description: desc,
                verify: v,
                confidence: Confidence::Medium,
            });
        }
    } else if root.join("package.json").exists() && !surfaces.iter().any(|s| s.name == "web") {
        let (v, c) = infer_verify(root, None, runner);
        surfaces.push(DetectedSurface {
            name: "web".into(),
            description: "Web frontend".into(),
            verify: v,
            confidence: c,
        });
    }

    if surfaces.is_empty() {
        surfaces.push(DetectedSurface {
            name: "app".into(),
            description: "main project".into(),
            verify: "TODO: define test command".into(),
            confidence: Confidence::Low,
        });
    }

    surfaces
}

/// Aggregate of all detection results — the single source of truth shared by
/// the config writer (WS-2) and the output presenter (WS-3).
#[derive(Debug, Clone)]
pub struct DetectionReport {
    pub surfaces: Vec<DetectedSurface>,
    pub git: DetectedGit,
}

impl DetectionReport {
    /// Run all detectors against `root` (best-effort; never fails).
    pub fn detect(root: &Path) -> Self {
        DetectionReport {
            surfaces: detect_surfaces(root),
            git: detect_git(root),
        }
    }

    /// True when any detected field is below `High` confidence and thus needs a
    /// `# mochiflow: confirm` marker, or when a known git provider was detected
    /// (always a confirm item, never auto-adopted).
    pub fn needs_any_confirm(&self) -> bool {
        self.surfaces.iter().any(|s| s.confidence.needs_confirm())
            || self.git.branch_confidence.needs_confirm()
            || self.git.has_known_provider()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    fn tmp() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn confidence_is_ordered_and_gates_confirm() {
        assert!(Confidence::Low < Confidence::Medium);
        assert!(Confidence::Medium < Confidence::High);
        assert!(Confidence::Low.needs_confirm());
        assert!(Confidence::Medium.needs_confirm());
        assert!(!Confidence::High.needs_confirm());
    }

    #[test]
    fn surfaces_empty_project_falls_back_to_app_low() {
        let d = tmp();
        let surfaces = detect_surfaces(d.path());
        assert_eq!(surfaces.len(), 1);
        assert_eq!(surfaces[0].name, "app");
        assert_eq!(surfaces[0].confidence, Confidence::Low);
        assert!(surfaces[0].verify.starts_with("TODO:"));
    }

    #[test]
    fn surfaces_single_npm_script_is_high() {
        let d = tmp();
        std::fs::write(
            d.path().join("package.json"),
            r#"{"scripts": {"test": "vitest"}}"#,
        )
        .unwrap();
        let surfaces = detect_surfaces(d.path());
        let web = surfaces.iter().find(|s| s.name == "web").unwrap();
        assert_eq!(web.verify, "npm run test");
        assert_eq!(web.confidence, Confidence::High);
    }

    #[test]
    fn surfaces_multiple_npm_scripts_is_medium_confirm() {
        let d = tmp();
        std::fs::write(
            d.path().join("package.json"),
            r#"{"scripts": {"test": "vitest", "lint": "eslint ."}}"#,
        )
        .unwrap();
        let surfaces = detect_surfaces(d.path());
        let web = surfaces.iter().find(|s| s.name == "web").unwrap();
        // first candidate by priority order is `test`
        assert_eq!(web.verify, "npm run test");
        assert_eq!(web.confidence, Confidence::Medium);
        assert!(web.confidence.needs_confirm());
    }

    #[test]
    fn surfaces_bun_lockfile_uses_bun_runner() {
        let d = tmp();
        std::fs::write(
            d.path().join("package.json"),
            r#"{"scripts": {"test": "vitest"}}"#,
        )
        .unwrap();
        std::fs::write(d.path().join("bun.lockb"), "").unwrap();
        let surfaces = detect_surfaces(d.path());
        let web = surfaces.iter().find(|s| s.name == "web").unwrap();
        assert_eq!(web.verify, "bun run test");
    }

    #[test]
    fn surfaces_cargo_is_high() {
        let d = tmp();
        std::fs::write(d.path().join("Cargo.toml"), "[package]\nname=\"x\"\n").unwrap();
        let surfaces = detect_surfaces(d.path());
        let cli = surfaces.iter().find(|s| s.name == "cli").unwrap();
        assert_eq!(cli.verify, "cargo test");
        assert_eq!(cli.confidence, Confidence::High);
    }

    #[test]
    fn git_no_remote_keeps_provider_none() {
        // A bare temp dir has no git origin; provider must stay none and no
        // pr_command is ever produced (the struct carries none).
        let d = tmp();
        let git = detect_git(d.path());
        assert_eq!(git.provider, "none");
        assert!(!git.has_known_provider());
    }

    #[test]
    fn monorepo_bun_uses_filter_and_medium_confidence() {
        let d = tmp();
        // Simulate a bun monorepo with 2 web dirs + 1 api dir.
        for sub in ["site-a", "site-b", "my-api"] {
            let dir = d.path().join(sub);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(
                dir.join("package.json"),
                r#"{"scripts":{"build":"vite build"}}"#,
            )
            .unwrap();
        }
        std::fs::write(d.path().join("bun.lockb"), "").unwrap();
        let surfaces = detect_surfaces(d.path());
        let web = surfaces.iter().find(|s| s.name == "web").unwrap();
        assert_eq!(web.verify, "bun --filter '*' build");
        assert_eq!(web.confidence, Confidence::Medium);
        assert!(web.description.contains("site-a"));
        assert!(web.description.contains("site-b"));
        let api = surfaces.iter().find(|s| s.name == "api").unwrap();
        assert_eq!(api.confidence, Confidence::Medium);
    }

    #[test]
    fn monorepo_api_subdir_with_space_is_shell_quoted() {
        let d = tmp();
        for sub in ["site", "api app"] {
            let dir = d.path().join(sub);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("package.json"), r#"{"scripts":{"test":"vitest"}}"#).unwrap();
        }

        let surfaces = detect_surfaces(d.path());
        let api = surfaces.iter().find(|s| s.name == "api").unwrap();
        assert_eq!(api.verify, "cd 'api app' && npm run test");
        assert_eq!(api.confidence, Confidence::Medium);
    }

    #[test]
    fn monorepo_turbo_uses_turbo_run() {
        let d = tmp();
        for sub in ["app", "admin"] {
            let dir = d.path().join(sub);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("package.json"), r#"{"scripts":{}}"#).unwrap();
        }
        std::fs::write(d.path().join("turbo.json"), "{}").unwrap();
        let surfaces = detect_surfaces(d.path());
        let web = surfaces.iter().find(|s| s.name == "web").unwrap();
        assert_eq!(web.verify, "turbo run build");
        assert_eq!(web.confidence, Confidence::Medium);
    }
}
