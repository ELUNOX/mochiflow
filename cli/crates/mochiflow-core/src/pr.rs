//! PR creation: assemble a normalized pr-request, run agnostic pre-flight,
//! push, then dispatch to a resolved backend (custom driver / provider
//! built-in / legacy command / manual handoff).
//!
//! `mochiflow pr` is the single place that runs `git push`. Exit codes:
//!   0  = PR created (URL captured)
//!   10 = manual handoff (human creates the PR)
//!   3  = pre-flight failed (nothing pushed/dispatched)
//!   1  = backend failure (assembly/driver/gh/push)
//! Exit 2 is reserved for config-load errors (global convention).

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;

use crate::config::Config;

pub const EXIT_OK: i32 = 0;
pub const EXIT_BACKEND_FAIL: i32 = 1;
pub const EXIT_PREFLIGHT_FAIL: i32 = 3;
pub const EXIT_MANUAL: i32 = 10;

/// Normalized PR request (mirrors contracts/pr-request.schema.json).
#[derive(Debug, Serialize)]
pub struct PrRequest {
    pub title: String,
    pub body: String,
    pub base: String,
    pub head: String,
    pub draft: bool,
    pub labels: Vec<String>,
    pub reviewers: Vec<String>,
}

/// Run a git command in `dir`, returning trimmed stdout on success.
fn git_capture(dir: &Path, args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

/// Backend selected by the resolution chain.
enum Backend {
    Driver(String),
    Github,
    Command(String),
    Manual,
}

fn resolve_backend(cfg: &Config) -> Backend {
    if let Some(driver) = cfg.git.pr_driver.as_deref()
        && !driver.trim().is_empty()
    {
        return Backend::Driver(driver.to_string());
    }
    if cfg.git.provider == "github" {
        return Backend::Github;
    }
    let cmd = cfg.git.pr_command.trim();
    if !cmd.is_empty() && !cmd.starts_with("TODO") {
        return Backend::Command(cfg.git.pr_command.clone());
    }
    Backend::Manual
}

/// Resolve the request directory for delivery artifacts. A bare token is always
/// a slug → gitignored `state/{slug}` (so a slug can never collide with a
/// same-named tracked top-level directory). Only a path-like input (absolute or
/// containing a separator) is honored as an explicit pre-staged dir; `None`
/// resolves to `state/`. `run_pr` still rejects any request-dir under `specs_dir`.
fn resolve_request_dir(cfg: &Config, spec: Option<&str>) -> PathBuf {
    match spec {
        Some(s) => {
            let p = PathBuf::from(s);
            let path_like =
                p.is_absolute() || s.contains('/') || s.contains(std::path::MAIN_SEPARATOR);
            if path_like {
                if p.is_absolute() {
                    p
                } else {
                    cfg.repo_root.join(s)
                }
            } else {
                cfg.state_dir().join(s)
            }
        }
        None => cfg.state_dir(),
    }
}

/// Extract `{"url": "..."}` from the last JSON object line of driver stdout.
fn parse_url(stdout: &str) -> Option<String> {
    for line in stdout.lines().rev() {
        let line = line.trim();
        if line.starts_with('{')
            && let Ok(v) = serde_json::from_str::<serde_json::Value>(line)
            && let Some(u) = v.get("url").and_then(|u| u.as_str())
        {
            return Some(u.to_string());
        }
    }
    None
}

/// Entry point for `mochiflow pr`. Returns the process exit code.
#[allow(clippy::too_many_arguments)]
pub fn run_pr(
    cfg: &Config,
    spec: Option<&str>,
    title: Option<&str>,
    body_file: Option<&str>,
    draft: bool,
    dry_run: bool,
) -> i32 {
    let root = &cfg.repo_root;
    let request_dir = resolve_request_dir(cfg, spec);
    let body_file_path = match body_file {
        Some(bf) => match std::fs::canonicalize(bf) {
            Ok(path) => Some(path),
            Err(e) => {
                eprintln!("FAIL: could not resolve --body-file {bf}: {e}");
                return EXIT_BACKEND_FAIL;
            }
        },
        None => None,
    };

    // Hard stop: never write PR delivery artifacts into the tracked spec tree.
    if request_dir.starts_with(cfg.specs_dir_path()) {
        eprintln!(
            "FAIL: refusing to write PR delivery artifacts under {} (tracked spec tree); pass a slug or a directory outside specs_dir.",
            cfg.specs_dir_path().display()
        );
        return EXIT_BACKEND_FAIL;
    }

    // Resolve head/base deterministically.
    let head = match git_capture(root, &["rev-parse", "--abbrev-ref", "HEAD"]) {
        Some(h) if h != "HEAD" && !h.is_empty() => h,
        _ => {
            eprintln!("FAIL: could not determine current branch (detached HEAD?).");
            return EXIT_BACKEND_FAIL;
        }
    };
    let base = cfg.git.base_branch.clone();

    // Title from flag; body only from --body-file (no implicit spec-dir read).
    let title = title.map(|s| s.to_string()).unwrap_or_default();
    let body = match body_file_path.as_ref() {
        Some(bf) => match std::fs::read_to_string(bf) {
            Ok(body) => body,
            Err(e) => {
                eprintln!("FAIL: could not read --body-file {}: {e}", bf.display());
                return EXIT_BACKEND_FAIL;
            }
        },
        None => String::new(),
    };

    let req = PrRequest {
        title,
        body,
        base: base.clone(),
        head: head.clone(),
        draft,
        labels: Vec::new(),
        reviewers: Vec::new(),
    };

    if req.title.trim().is_empty() || req.base.trim().is_empty() || req.head.trim().is_empty() {
        eprintln!(
            "FAIL: assembled pr-request invalid (title/base/head must be non-empty per contracts/pr-request.schema.json)."
        );
        return EXIT_BACKEND_FAIL;
    }

    let backend = resolve_backend(cfg);

    if dry_run {
        println!("(dry-run) request-dir : {}", request_dir.display());
        println!("(dry-run) head -> base: {head} -> {base}");
        println!("(dry-run) backend     : {}", backend_label(&backend));
        println!("(dry-run) no pre-flight, push, or dispatch performed.");
        return EXIT_OK;
    }

    // Agnostic pre-flight runs BEFORE writing any artifact, so mochiflow's own
    // pr-request.json does not dirty the tree and trip its own clean check.
    if let Some(code) = preflight(
        root,
        &head,
        &base,
        &cfg.state_dir(),
        &cfg.install_dir_path(),
    ) {
        return code;
    }
    if let Some(slug) = spec
        && !crate::accept::is_path_like_spec_arg(slug)
        && let Err(message) = crate::accept::validate_pr_spec_closeout_committed(cfg, slug)
    {
        eprintln!("{message}");
        return EXIT_PREFLIGHT_FAIL;
    }

    // Write pr-request.json only for the pr_driver backend (its sole consumer).
    // github / legacy command / manual never read it, so nothing is written for
    // them — keeping the working tree free of delivery scratch.
    if matches!(backend, Backend::Driver(_)) {
        if let Err(e) = std::fs::create_dir_all(&request_dir) {
            eprintln!("FAIL: could not create request dir: {e}");
            return EXIT_BACKEND_FAIL;
        }
        match serde_json::to_string_pretty(&req) {
            Ok(json) => {
                if let Err(e) = std::fs::write(request_dir.join("pr-request.json"), json + "\n") {
                    eprintln!("FAIL: could not write pr-request.json: {e}");
                    return EXIT_BACKEND_FAIL;
                }
            }
            Err(e) => {
                eprintln!("FAIL: could not serialize pr-request: {e}");
                return EXIT_BACKEND_FAIL;
            }
        }
    }

    // Push (the only git push in mochiflow). Skip only for manual without a remote.
    let has_remote = git_capture(root, &["remote", "get-url", "origin"]).is_some();
    let mut pushed = false;
    if has_remote {
        pushed = Command::new("git")
            .args(["push", "-u", "origin", &head])
            .current_dir(root)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !pushed {
            if matches!(backend, Backend::Manual) {
                eprintln!("WARN: git push failed; continue with manual handoff.");
            } else {
                eprintln!("FAIL: git push failed for branch {head}.");
                return EXIT_BACKEND_FAIL;
            }
        }
    } else if !matches!(backend, Backend::Manual) {
        eprintln!("FAIL: no 'origin' remote to push {head}.");
        return EXIT_BACKEND_FAIL;
    }

    // Dispatch.
    let language = cfg.conversation_output_language();
    match backend {
        Backend::Driver(driver) => dispatch_driver(root, &driver, &request_dir, language),
        Backend::Github => dispatch_github(root, &req, body_file_path.as_deref(), language),
        Backend::Command(cmd) => dispatch_command(root, &cmd, &request_dir, language),
        Backend::Manual => dispatch_manual(&req, pushed, language),
    }
}

fn backend_label(b: &Backend) -> String {
    match b {
        Backend::Driver(d) => format!("driver ({d})"),
        Backend::Github => "github built-in".into(),
        Backend::Command(_) => "legacy pr_command".into(),
        Backend::Manual => "manual handoff".into(),
    }
}

/// Conversation-language next action printed after a successful PR handoff:
/// merge the PR in the provider, then report the merge back so post-merge local
/// cleanup can run. `language` is already resolved for CLI-only output via
/// `Config::conversation_output_language()` (so `auto` has become a concrete
/// tag). The next action is local workflow guidance and is never written into
/// the PR body, which stays artifact-language.
fn pr_next_action(language: &str) -> String {
    if language == "ja" {
        "次の一手: プロバイダ上で PR をマージし、完了したらチャットで「マージした」と教えてください。ローカルの後片付け（ブランチと一時ファイルの整理）を行います。".to_string()
    } else {
        "Next: merge the PR in your provider, then come back and tell me it merged so I can run local cleanup.".to_string()
    }
}

/// Returns Some(exit_code) on failure, None when all checks pass.
fn preflight(
    root: &Path,
    head: &str,
    base: &str,
    state_dir: &Path,
    install_dir: &Path,
) -> Option<i32> {
    let status = git_capture(root, &["status", "--porcelain"]).unwrap_or_default();
    let dirty = !status.is_empty();
    let state_rel = state_dir
        .strip_prefix(root)
        .unwrap_or(state_dir)
        .to_string_lossy()
        .replace('\\', "/");
    let install_rel = install_dir
        .strip_prefix(root)
        .unwrap_or(install_dir)
        .to_string_lossy()
        .replace('\\', "/");
    let state_dirty = status.lines().any(|line| {
        line.get(3..)
            .is_some_and(|path| path.starts_with(&state_rel))
    });
    if dirty {
        eprintln!("pre-flight FAIL: working tree not clean.");
        if state_dirty {
            eprintln!(
                "pre-flight hint: {state_rel}/ is runtime state; add `state/` to {install_rel}/.gitignore."
            );
        }
        return Some(EXIT_PREFLIGHT_FAIL);
    }
    if head == base {
        eprintln!("pre-flight FAIL: source == target ({head}).");
        return Some(EXIT_PREFLIGHT_FAIL);
    }
    None
}

fn dispatch_driver(root: &Path, driver: &str, request_dir: &Path, language: &str) -> i32 {
    let output = Command::new(driver)
        .arg(request_dir)
        .current_dir(root)
        .output();
    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            match parse_url(&stdout) {
                Some(url) => {
                    println!("PR created: {url}");
                    println!("{}", pr_next_action(language));
                    EXIT_OK
                }
                None => {
                    eprintln!("FAIL: driver succeeded but emitted no {{\"url\"}}.");
                    EXIT_BACKEND_FAIL
                }
            }
        }
        Ok(o) => {
            eprintln!(
                "FAIL: pr_driver exited {}: {}",
                o.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&o.stderr).trim()
            );
            EXIT_BACKEND_FAIL
        }
        Err(e) => {
            eprintln!("FAIL: could not run pr_driver '{driver}': {e}");
            EXIT_BACKEND_FAIL
        }
    }
}

fn dispatch_github(root: &Path, req: &PrRequest, body_file: Option<&Path>, language: &str) -> i32 {
    let mut args: Vec<String> = vec![
        "pr".into(),
        "create".into(),
        "--base".into(),
        req.base.clone(),
        "--head".into(),
        req.head.clone(),
        "--title".into(),
        req.title.clone(),
    ];
    // Forward the caller's --body-file path straight to gh (avoids ARG_MAX from
    // re-passing a large body via --body); fall back to --body only when absent.
    match body_file {
        Some(bf) => {
            args.push("--body-file".into());
            args.push(bf.to_string_lossy().to_string());
        }
        None => {
            args.push("--body".into());
            args.push(req.body.clone());
        }
    }
    if req.draft {
        args.push("--draft".into());
    }
    let output = Command::new("gh").args(&args).current_dir(root).output();
    match output {
        Ok(o) if o.status.success() => {
            let url = String::from_utf8_lossy(&o.stdout).trim().to_string();
            println!("PR created: {url}");
            println!("{}", pr_next_action(language));
            EXIT_OK
        }
        Ok(o) => {
            eprintln!(
                "FAIL: gh pr create failed: {}",
                String::from_utf8_lossy(&o.stderr).trim()
            );
            EXIT_BACKEND_FAIL
        }
        Err(e) => {
            eprintln!("FAIL: could not run `gh` (install GitHub CLI or set [git].pr_driver): {e}");
            EXIT_BACKEND_FAIL
        }
    }
}

fn dispatch_command(root: &Path, pr_command: &str, request_dir: &Path, language: &str) -> i32 {
    // Legacy: substitute {spec_dir} and run via the shell.
    let spec_dir = shell_single_quote(&request_dir.to_string_lossy());
    let cmd = pr_command.replace("{spec_dir}", &spec_dir);
    eprintln!(
        "WARN: [git].pr_command is deprecated and runs through a shell; prefer [git].pr_driver."
    );
    let status = Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .current_dir(root)
        .status();
    match status {
        Ok(s) if s.success() => {
            println!("{}", pr_next_action(language));
            EXIT_OK
        }
        Ok(s) => {
            eprintln!("FAIL: pr_command exited {}.", s.code().unwrap_or(-1));
            EXIT_BACKEND_FAIL
        }
        Err(e) => {
            eprintln!("FAIL: could not run pr_command: {e}");
            EXIT_BACKEND_FAIL
        }
    }
}

fn shell_single_quote(value: &str) -> String {
    let escaped = value.replace('\'', "'\"'\"'");
    format!("'{escaped}'")
}

fn dispatch_manual(req: &PrRequest, pushed: bool, language: &str) -> i32 {
    println!("\n--- manual PR handoff ---");
    println!("No PR backend configured (set [git].provider / [git].pr_driver to automate).");
    if pushed {
        println!("Branch pushed. Create the PR with this content via your provider UI/CLI:");
    } else {
        println!(
            "Branch NOT pushed (no origin or push failed). Push manually first, then create the PR with this content:"
        );
    }
    println!("  base : {}", req.base);
    println!("  head : {}", req.head);
    println!("  title: {}", req.title);
    println!("{}", pr_next_action(language));
    EXIT_MANUAL
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn parse_url_takes_last_json_line() {
        let out = "log line\nmore logs\n{\"url\": \"https://x/pr/1\"}\n";
        assert_eq!(parse_url(out).as_deref(), Some("https://x/pr/1"));
    }

    #[test]
    fn parse_url_none_when_absent() {
        assert_eq!(parse_url("no json here\n"), None);
    }

    #[test]
    fn shell_quote_handles_metacharacters() {
        assert_eq!(
            shell_single_quote("/tmp/a b/it's;bad"),
            "'/tmp/a b/it'\"'\"'s;bad'"
        );
    }

    #[test]
    fn pr_next_action_is_language_aware() {
        let en = pr_next_action("en");
        assert!(en.starts_with("Next:"), "{en}");
        assert!(
            en.contains("merge the PR") && en.contains("local cleanup"),
            "{en}"
        );
        let ja = pr_next_action("ja");
        assert!(ja.contains("マージ") && ja.contains("後片付け"), "{ja}");
        assert_ne!(en, ja);
    }
}
