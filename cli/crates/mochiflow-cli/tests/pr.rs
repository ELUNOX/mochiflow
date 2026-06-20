//! Integration tests for `mochiflow pr`: pre-flight, push ownership, backend
//! resolution (driver / legacy command / manual), and exit-code regime.
//!
//! Each test builds a throwaway git repo (with a bare `origin`) so `git push`
//! inside `mochiflow pr` succeeds without touching any real remote. The github
//! built-in success path needs a real `gh` + GitHub remote and is covered by
//! human QA (QA-09), not here.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::Path;
use std::process::Command as Proc;

use assert_cmd::Command;

fn bin() -> Command {
    Command::cargo_bin("mochiflow").unwrap()
}

fn git(dir: &Path, args: &[&str]) {
    let ok = Proc::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .unwrap()
        .success();
    assert!(ok, "git {args:?} failed in {}", dir.display());
}

/// Build a repo with an initial commit on `main`, a bare origin, and (unless
/// `stay_on_main`) a checked-out `feature` branch. `git_block` is spliced into
/// `[git]`. Returns the config.toml path.
fn setup(dir: &Path, git_block: &str, stay_on_main: bool) -> std::path::PathBuf {
    // bare remote
    let bare = dir.join("origin.git");
    fs::create_dir_all(&bare).unwrap();
    git(&bare, &["init", "--bare", "-q"]);

    // work repo
    let repo = dir.join("repo");
    fs::create_dir_all(&repo).unwrap();
    git(&repo, &["init", "-q", "-b", "main"]);
    git(&repo, &["config", "user.email", "t@example.com"]);
    git(&repo, &["config", "user.name", "Test"]);
    fs::write(repo.join("README.md"), "hi\n").unwrap();
    fs::write(repo.join(".gitignore"), ".mochiflow/\n").unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-q", "-m", "init"]);
    git(&repo, &["remote", "add", "origin", bare.to_str().unwrap()]);
    git(&repo, &["push", "-q", "-u", "origin", "main"]);
    if !stay_on_main {
        git(&repo, &["checkout", "-q", "-b", "feature"]);
    }

    let mf = repo.join(".mochiflow");
    fs::create_dir_all(mf.join("specs")).unwrap();
    let config = format!(
        "schema_version = 1\n\
         install_dir = \".mochiflow\"\nspecs_dir = \".mochiflow/specs\"\nindex = \".mochiflow/INDEX.md\"\n\n\
         [constitution]\nproject = \".mochiflow/constitution.md\"\nlocal = \".mochiflow/constitution.local.md\"\n\n\
         [context]\nproduct = \".mochiflow/context/product.md\"\nstructure = \".mochiflow/context/structure.md\"\ntech = \".mochiflow/context/tech.md\"\n\n\
         [adr]\ndecisions = \".mochiflow/adr/decisions.md\"\npitfalls = \".mochiflow/adr/pitfalls.md\"\n\n\
         [git]\nbase_branch = \"main\"\n{git_block}\n\n\
         [adapter]\ntools = [\"kiro\"]\n\n\
         [surfaces.app]\ndescription = \"app\"\n\n[surfaces.app.verify]\ndefault = \"echo ok\"\n"
    );
    let config_path = mf.join("config.toml");
    fs::write(&config_path, config).unwrap();
    config_path
}

fn run_pr(config: &Path, extra: &[&str]) -> assert_cmd::assert::Assert {
    let mut args = vec!["--config", config.to_str().unwrap(), "pr"];
    args.extend_from_slice(extra);
    bin().args(&args).assert()
}

/// provider=none, no driver/command → manual handoff (exit 10), branch pushed.
/// Manual backend writes no pr-request.json (driver-only artifact).
#[test]
fn pr_manual_handoff_exits_10() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = setup(dir.path(), "provider = \"none\"", false);
    run_pr(&cfg, &["--title", "Add feature"]).failure().code(10);
    assert!(
        !dir.path()
            .join("repo/.mochiflow/state/pr-request.json")
            .exists()
    );
    assert!(!dir.path().join("repo/pr-request.json").exists());
    assert!(
        !dir.path()
            .join("repo/.mochiflow/specs/pr-request.json")
            .exists()
    );
}

/// dirty working tree → pre-flight FAIL (exit 3), nothing dispatched.
#[test]
fn pr_dirty_tree_exits_3() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = setup(dir.path(), "provider = \"none\"", false);
    fs::write(dir.path().join("repo/dirty.txt"), "x\n").unwrap();
    run_pr(&cfg, &["--title", "X"]).failure().code(3);
}

/// On the base branch (head == base) → pre-flight FAIL (exit 3).
#[test]
fn pr_base_equals_head_exits_3() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = setup(dir.path(), "provider = \"none\"", true); // stay on main
    run_pr(&cfg, &["--title", "X"]).failure().code(3);
}

/// Missing title → exit 1 (assembly failure).
#[test]
fn pr_missing_title_exits_1() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = setup(dir.path(), "provider = \"none\"", false);
    run_pr(&cfg, &[]).failure().code(1);
}

/// Custom pr_driver that emits {"url"} → exit 0, URL captured.
#[test]
fn pr_driver_success_exits_0() {
    let dir = tempfile::tempdir().unwrap();
    let driver = dir.path().join("driver.sh");
    fs::write(
        &driver,
        "#!/bin/sh\necho 'log line'\necho '{\"url\": \"https://example/pr/1\"}'\n",
    )
    .unwrap();
    Proc::new("chmod")
        .args(["+x", driver.to_str().unwrap()])
        .status()
        .unwrap();
    let block = format!(
        "provider = \"none\"\npr_driver = \"{}\"",
        driver.to_str().unwrap()
    );
    let cfg = setup(dir.path(), &block, false);
    run_pr(&cfg, &["--title", "Add"])
        .success()
        .stdout(predicates::str::contains("https://example/pr/1"));
}

/// pr_driver wins over pr_command (precedence): command must NOT run.
#[test]
fn pr_driver_beats_command() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("command-ran.txt");
    let driver = dir.path().join("driver.sh");
    fs::write(&driver, "#!/bin/sh\necho '{\"url\": \"https://e/pr/9\"}'\n").unwrap();
    Proc::new("chmod")
        .args(["+x", driver.to_str().unwrap()])
        .status()
        .unwrap();
    let block = format!(
        "provider = \"none\"\npr_driver = \"{}\"\npr_command = \"touch {}\"",
        driver.to_str().unwrap(),
        marker.to_str().unwrap()
    );
    let cfg = setup(dir.path(), &block, false);
    run_pr(&cfg, &["--title", "Add"]).success();
    assert!(
        !marker.exists(),
        "legacy pr_command must not run when pr_driver is set"
    );
}

/// Legacy pr_command-only path runs the command (exit 0).
#[test]
fn pr_command_only_runs_exits_0() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("command-ran.txt");
    let block = format!(
        "provider = \"none\"\npr_command = \"touch {}\"",
        marker.to_str().unwrap()
    );
    let cfg = setup(dir.path(), &block, false);
    run_pr(&cfg, &["--title", "Add"]).success();
    assert!(marker.exists(), "pr_command should have run");
}

#[test]
fn pr_command_escapes_spec_dir_placeholder() {
    let dir = tempfile::tempdir().unwrap();
    let block =
        "provider = \"none\"\npr_command = \"mkdir -p {spec_dir} && touch {spec_dir}/marker\"";
    let cfg = setup(dir.path(), block, false);
    run_pr(&cfg, &["--title", "Add", "--spec", "semi;touch hacked"]).success();
    assert!(
        dir.path()
            .join("repo/.mochiflow/state/semi;touch hacked/marker")
            .exists()
    );
    assert!(!dir.path().join("repo/hacked").exists());
}

#[test]
fn pr_github_body_file_is_canonicalized_from_calling_cwd() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = setup(dir.path(), "provider = \"github\"", false);
    let repo = dir.path().join("repo");
    let subdir = repo.join("subdir");
    fs::create_dir_all(&subdir).unwrap();
    let body = dir.path().join("body.md");
    fs::write(&body, "body\n").unwrap();

    let fakebin = dir.path().join("fakebin");
    fs::create_dir_all(&fakebin).unwrap();
    let capture = dir.path().join("gh-args.txt");
    let gh = fakebin.join("gh");
    fs::write(
        &gh,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\necho https://example/pr/42\n",
            capture.display()
        ),
    )
    .unwrap();
    Proc::new("chmod")
        .args(["+x", gh.to_str().unwrap()])
        .status()
        .unwrap();
    let old_path = std::env::var("PATH").unwrap_or_default();
    let path = format!("{}:{old_path}", fakebin.display());

    let result = bin()
        .current_dir(&subdir)
        .env("PATH", path)
        .args([
            "--config",
            cfg.to_str().unwrap(),
            "pr",
            "--title",
            "Add",
            "--body-file",
            "../../body.md",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(stdout.contains("https://example/pr/42"), "{stdout}");
    let args = fs::read_to_string(&capture).unwrap();
    let expected_body = body.canonicalize().unwrap().to_string_lossy().to_string();
    assert!(
        args.lines().any(|line| line == expected_body),
        "gh args should contain canonical body path, got:\n{args}"
    );
}

/// --dry-run writes/pushes/dispatches nothing.
#[test]
fn pr_dry_run_noop() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = setup(dir.path(), "provider = \"none\"", false);
    run_pr(&cfg, &["--title", "X", "--dry-run"]).success();
    assert!(!dir.path().join("repo/pr-request.json").exists());
}

/// provider=github but `gh` cannot be found on PATH → exit 1, and no
/// pr-request.json is written (github built-in is not the driver consumer).
#[test]
fn pr_github_without_gh_exits_1() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = setup(dir.path(), "provider = \"github\"", false);
    let mut cmd = bin();
    cmd.env("PATH", "");
    cmd.args(["--config", cfg.to_str().unwrap(), "pr", "--title", "X"]);
    cmd.assert().failure().code(1);
    assert!(
        !dir.path()
            .join("repo/.mochiflow/state/pr-request.json")
            .exists()
    );
}

/// Driver backend writes pr-request.json under state/ and it conforms to
/// contracts/pr-request.schema.json (only the driver consumes this file).
#[test]
fn pr_request_json_conforms_to_schema() {
    let dir = tempfile::tempdir().unwrap();
    let driver = dir.path().join("driver.sh");
    fs::write(&driver, "#!/bin/sh\necho '{\"url\": \"https://e/pr/1\"}'\n").unwrap();
    Proc::new("chmod")
        .args(["+x", driver.to_str().unwrap()])
        .status()
        .unwrap();
    let block = format!(
        "provider = \"none\"\npr_driver = \"{}\"",
        driver.to_str().unwrap()
    );
    let cfg = setup(dir.path(), &block, false);
    run_pr(&cfg, &["--title", "Add feature"]).success();
    // --spec omitted → request-dir is state/ root.
    let pr = dir.path().join("repo/.mochiflow/state/pr-request.json");
    assert!(
        pr.exists(),
        "pr-request.json should be written for driver backend"
    );
    let raw = fs::read_to_string(&pr).unwrap();
    for k in ["\"title\"", "\"body\"", "\"base\"", "\"head\""] {
        assert!(
            raw.contains(k),
            "pr-request.json missing required field {k}\n{raw}"
        );
    }
    assert!(
        raw.contains("\"Add feature\""),
        "title not in pr-request: {raw}"
    );
    assert!(
        raw.contains("\"main\""),
        "base 'main' not in pr-request: {raw}"
    );
    assert!(
        raw.contains("\"feature\""),
        "head 'feature' not in pr-request: {raw}"
    );
}

/// Driver + slug → pr-request.json lands under state/{slug}/, never specs/{slug}/.
#[test]
fn pr_driver_writes_under_state_slug_not_specs() {
    let dir = tempfile::tempdir().unwrap();
    let driver = dir.path().join("driver.sh");
    fs::write(&driver, "#!/bin/sh\necho '{\"url\": \"https://e/pr/2\"}'\n").unwrap();
    Proc::new("chmod")
        .args(["+x", driver.to_str().unwrap()])
        .status()
        .unwrap();
    let block = format!(
        "provider = \"none\"\npr_driver = \"{}\"",
        driver.to_str().unwrap()
    );
    let cfg = setup(dir.path(), &block, false);
    run_pr(&cfg, &["--spec", "my-slug", "--title", "Add"]).success();
    assert!(
        dir.path()
            .join("repo/.mochiflow/state/my-slug/pr-request.json")
            .exists()
    );
    assert!(!dir.path().join("repo/.mochiflow/specs/my-slug").exists());
}

/// A bare --spec token is always a slug (resolves under state/), even when a
/// same-named tracked top-level directory exists — never writes into it.
#[test]
fn pr_bare_slug_never_targets_tracked_topdir() {
    let dir = tempfile::tempdir().unwrap();
    let driver = dir.path().join("driver.sh");
    fs::write(&driver, "#!/bin/sh\necho '{\"url\": \"https://e/pr/4\"}'\n").unwrap();
    Proc::new("chmod")
        .args(["+x", driver.to_str().unwrap()])
        .status()
        .unwrap();
    let block = format!(
        "provider = \"none\"\npr_driver = \"{}\"",
        driver.to_str().unwrap()
    );
    let cfg = setup(dir.path(), &block, false);
    let docs = dir.path().join("repo/docs");
    fs::create_dir_all(&docs).unwrap();
    run_pr(&cfg, &["--spec", "docs", "--title", "Add"]).success();
    assert!(
        dir.path()
            .join("repo/.mochiflow/state/docs/pr-request.json")
            .exists()
    );
    assert!(!docs.join("pr-request.json").exists());
}

/// AC-01b: an explicit --spec dir resolving under specs_dir is rejected (exit 1).
#[test]
fn pr_rejects_request_dir_under_specs() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = setup(dir.path(), "provider = \"none\"", false);
    let specdir = dir.path().join("repo/.mochiflow/specs/my-slug");
    fs::create_dir_all(&specdir).unwrap();
    run_pr(
        &cfg,
        &["--spec", ".mochiflow/specs/my-slug", "--title", "X"],
    )
    .failure()
    .code(1);
}

/// AC-03: body comes only from --body-file; a stray spec-dir pr-description.md
/// is ignored and an absent --body-file yields an empty body.
#[test]
fn pr_body_only_from_body_file() {
    let dir = tempfile::tempdir().unwrap();
    let driver = dir.path().join("driver.sh");
    fs::write(&driver, "#!/bin/sh\necho '{\"url\": \"https://e/pr/3\"}'\n").unwrap();
    Proc::new("chmod")
        .args(["+x", driver.to_str().unwrap()])
        .status()
        .unwrap();
    let block = format!(
        "provider = \"none\"\npr_driver = \"{}\"",
        driver.to_str().unwrap()
    );
    let cfg = setup(dir.path(), &block, false);
    // stray pr-description.md in the spec dir (gitignored under .mochiflow/) must be ignored
    let specdir = dir.path().join("repo/.mochiflow/specs/my-slug");
    fs::create_dir_all(&specdir).unwrap();
    fs::write(specdir.join("pr-description.md"), "STRAY BODY\n").unwrap();
    run_pr(&cfg, &["--spec", "my-slug", "--title", "Add"]).success();
    let raw = fs::read_to_string(
        dir.path()
            .join("repo/.mochiflow/state/my-slug/pr-request.json"),
    )
    .unwrap();
    assert!(
        !raw.contains("STRAY BODY"),
        "spec-dir pr-description.md must not be used:\n{raw}"
    );
    assert!(
        raw.contains("\"body\": \"\""),
        "body should be empty without --body-file:\n{raw}"
    );

    // with --body-file the provided content is used
    let bf = dir.path().join("body.md");
    fs::write(&bf, "REAL BODY\n").unwrap();
    run_pr(
        &cfg,
        &[
            "--spec",
            "my-slug",
            "--title",
            "Add",
            "--body-file",
            bf.to_str().unwrap(),
        ],
    )
    .success();
    let raw2 = fs::read_to_string(
        dir.path()
            .join("repo/.mochiflow/state/my-slug/pr-request.json"),
    )
    .unwrap();
    assert!(
        raw2.contains("REAL BODY"),
        "--body-file content should be used:\n{raw2}"
    );
}

#[test]
fn pr_missing_body_file_exits_1_before_dispatch() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = setup(dir.path(), "provider = \"none\"", false);
    let missing = dir.path().join("missing-body.md");

    run_pr(
        &cfg,
        &[
            "--spec",
            "my-slug",
            "--title",
            "Add",
            "--body-file",
            missing.to_str().unwrap(),
        ],
    )
    .failure()
    .code(1)
    .stderr(predicates::str::contains(
        "FAIL: could not resolve --body-file",
    ));

    assert!(
        !dir.path()
            .join("repo/.mochiflow/state/my-slug/pr-request.json")
            .exists(),
        "missing --body-file should fail before writing pr-request.json"
    );
}
