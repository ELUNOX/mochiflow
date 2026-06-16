//! Integration tests for first-run setup: machine detection + confirm
//! markers in config, the clig.dev presenter (plain / json), and the static
//! `guide` card. Golden snapshots live test-locally (under `tests/golden/`),
//! intentionally outside the version-gated `tests/conformance/golden/` surface
//! — this UX text is not a frozen consumer contract.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::Path;

use assert_cmd::Command;

fn bin() -> Command {
    Command::cargo_bin("mochiflow").unwrap()
}

/// init --force in a project dir; returns the written config.toml text.
fn init_force(dir: &Path) -> String {
    bin()
        .args(["init", "--force", "--target", dir.path_str()])
        .write_stdin("")
        .assert()
        .success();
    fs::read_to_string(dir.join(".mochiflow/config.toml")).unwrap()
}

trait PathStr {
    fn path_str(&self) -> &str;
}
impl PathStr for Path {
    fn path_str(&self) -> &str {
        self.to_str().unwrap()
    }
}

/// QA-01 / AC-01 / AC-02: a project with multiple candidate scripts detects a
/// verify command and attaches a confirm marker (ambiguous candidates).
#[test]
fn qa01_scripts_detected_with_confirm_on_ambiguity() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"scripts": {"test": "vitest", "lint": "eslint ."}}"#,
    )
    .unwrap();
    fs::write(dir.path().join("bun.lockb"), "").unwrap();
    let cfg = init_force(dir.path());

    assert!(cfg.contains("bun run test"), "detected verify cmd:\n{cfg}");
    assert!(
        cfg.contains("# mochiflow: confirm"),
        "multiple candidates must attach a confirm marker:\n{cfg}"
    );
    // schema-valid TOML
    let _: toml::Value = toml::from_str(&cfg).unwrap();
}

/// QA-02 / AC-03: a github remote never auto-switches provider; it stays none
/// and is surfaced as a confirm item.
#[test]
fn qa02_github_remote_keeps_provider_none() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    for args in [
        ["init", "-q"].as_slice(),
        [
            "remote",
            "add",
            "origin",
            "https://github.com/example/repo.git",
        ]
        .as_slice(),
    ] {
        std::process::Command::new("git")
            .args(args)
            .current_dir(root)
            .status()
            .unwrap();
    }
    let cfg = init_force(root);

    let parsed: toml::Value = toml::from_str(&cfg).unwrap();
    assert_eq!(
        parsed["git"]["provider"].as_str(),
        Some("none"),
        "provider must stay none:\n{cfg}"
    );
    assert!(
        cfg.contains("# mochiflow: confirm") && cfg.contains("github"),
        "detected provider must be a confirm item:\n{cfg}"
    );
}

/// QA-03 / AC-10 / Edge: no manifest → verify stays a TODO sentinel + confirm,
/// and the config is still valid (never corrupted).
#[test]
fn qa03_no_manifest_keeps_todo_and_valid_config() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_force(dir.path());
    assert!(cfg.contains("TODO: define test command"), "{cfg}");
    assert!(cfg.contains("# mochiflow: confirm"), "{cfg}");
    let _: toml::Value = toml::from_str(&cfg).unwrap();
}

/// QA-05 / AC-05: NO_COLOR + piped stdout → plain renderer (ASCII glyphs, no
/// ANSI escapes).
#[test]
fn qa05_no_color_pipe_is_plain() {
    let dir = tempfile::tempdir().unwrap();
    let out = bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .env("NO_COLOR", "1")
        .write_stdin("")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(
        stdout.contains("[ok]") || stdout.contains("[!]"),
        "{stdout}"
    );
    assert!(
        !stdout.contains('\u{1b}'),
        "plain output must contain no ANSI escapes:\n{stdout}"
    );
}

/// QA-06 / AC-05: init --json emits one structured JSON document on stdout
/// matching the golden; progress logs go to stderr only.
#[test]
fn qa06_json_matches_golden_and_stdout_is_pure_json() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"scripts":{"test":"vitest"}}"#,
    )
    .unwrap();
    let out = bin()
        .args(["init", "--target", dir.path().to_str().unwrap(), "--json"])
        .env("LANG", "en_US.UTF-8")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .write_stdin("")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    // stdout parses as a single JSON document with automation-facing status.
    let mut parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(parsed["schema_version"].as_i64(), Some(1));
    assert_eq!(parsed["status"].as_str(), Some("needs_ai_review"));
    assert_eq!(parsed["exit_code"].as_i64(), Some(0));
    assert_eq!(parsed["dry_run"].as_bool(), Some(false));
    assert_eq!(parsed["language"].as_str(), Some("en"));
    assert_eq!(parsed["language_source"].as_str(), Some("locale"));
    assert_eq!(parsed["review"]["required"].as_bool(), Some(true));
    assert!(
        parsed["review"]["prompt"]
            .as_str()
            .unwrap()
            .contains("Complete MochiFlow setup")
    );
    let target = parsed["target"].as_str().unwrap().to_string();
    parsed["target"] = serde_json::Value::String("<target>".to_string());
    if let Some(items) = parsed["created_updated"].as_array_mut() {
        for item in items {
            if let Some(text) = item.as_str() {
                *item = serde_json::Value::String(text.replace(&target, "<target>"));
            }
        }
    }
    let mut normalized = serde_json::to_string_pretty(&parsed).unwrap();
    normalized.push('\n');
    let golden = include_str!("golden/init_npm.json");
    assert_eq!(normalized, golden, "init --json must match golden");
}

/// QA-07 / AC-07: the guide card matches the golden in each language, and the
/// content (verbs + explicit commands) is identical across languages (static).
#[test]
fn qa07_guide_card_matches_golden_en_and_ja() {
    // en: no config → default English card
    let out = bin().args(["guide"]).assert().success();
    let en = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert_eq!(en, include_str!("golden/guide_en.txt"));

    // ja: config with language = ja
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args([
            "init",
            "--target",
            dir.path().to_str().unwrap(),
            "--language",
            "ja",
        ])
        .write_stdin("")
        .assert()
        .success();
    let config = dir.path().join(".mochiflow/config.toml");
    let out = bin()
        .args(["--config", config.to_str().unwrap(), "guide"])
        .assert()
        .success();
    let ja = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert_eq!(ja, include_str!("golden/guide_ja.txt"));

    for cmd in [
        "mochiflow-discuss",
        "mochiflow-plan",
        "mochiflow-build",
        "mochiflow-ship",
    ] {
        assert!(en.contains(cmd) && ja.contains(cmd), "missing {cmd}");
    }
}
