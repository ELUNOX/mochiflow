//! CLI integration tests for the deterministic init and config-error behavior.
//!
//! Broader Rust-native conformance for read/query subcommands lives in
//! `tests/conformance.rs`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

use assert_cmd::Command;

fn bin() -> Command {
    Command::cargo_bin("mochiflow").unwrap()
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn set_json_field(path: &Path, key: &str, value: serde_json::Value) {
    let text = fs::read_to_string(path).unwrap();
    let mut json: serde_json::Value = serde_json::from_str(&text).unwrap();
    json.as_object_mut().unwrap().insert(key.into(), value);
    fs::write(path, serde_json::to_string_pretty(&json).unwrap() + "\n").unwrap();
}

fn copy_dir_all(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let ty = entry.file_type().unwrap();
        let target = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.match_indices(needle).count()
}

fn assert_instructions_readme_contract(readme: &str) {
    assert!(
        readme.contains("explicitly provide its file path"),
        "{readme}"
    );
    assert!(readme.contains("does not automatically load"), "{readme}");
    assert!(readme.contains("does not parse frontmatter"), "{readme}");
    assert!(readme.contains("detach --purge"), "{readme}");
}

#[test]
fn inspect_json_is_one_schema_valid_document() {
    let output = bin()
        .current_dir(repo_root())
        .args(["inspect", "agent-context-api", "--json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let schema: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repo_root().join("contracts/agent-context.schema.json")).unwrap(),
    )
    .unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    assert!(validator.is_valid(&document));
}

#[test]
fn inspect_missing_slug_returns_structured_error() {
    let output = bin()
        .current_dir(repo_root())
        .args(["inspect", "missing-agent-context-spec", "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(document["result"], "error");
    assert_eq!(document["errors"][0]["code"], "spec_missing");
}

#[test]
fn inspect_human_output_uses_configured_japanese() {
    let output = bin()
        .current_dir(repo_root())
        .args(["inspect", "agent-context-api"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("エージェントコンテキスト"), "{stdout}");
    assert!(stdout.contains("状態"), "{stdout}");
}

/// Deterministic init: no flags, piped stdin (non-TTY) → exit 0, scaffolds
/// config from machine detection. A bare temp dir detects nothing concrete, so
/// the verify command stays a TODO sentinel and confirm markers are attached
/// (detection-zero = low confidence), engine/VERSION exists.
#[test]
fn init_scaffolds_deterministically() {
    let dir = tempfile::tempdir().unwrap();
    let result = bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();

    let config_path = dir.path().join(".mochiflow/config.toml");
    assert!(config_path.exists(), "config.toml should be created");
    let cfg = fs::read_to_string(&config_path).unwrap();
    assert!(
        cfg.contains("TODO: define test command"),
        "should have TODO sentinel for undetected verify, got:\n{cfg}"
    );
    assert!(
        cfg.contains("# mochiflow: confirm"),
        "detection-zero should attach confirm markers, got:\n{cfg}"
    );
    assert!(
        cfg.contains("tools = [\"agents\"]"),
        "non-interactive default should be agents, got:\n{cfg}"
    );
    assert!(dir.path().join(".mochiflow/engine/VERSION").exists());
    assert!(dir.path().join(".mochiflow/instructions").is_dir());
    assert!(dir.path().join(".mochiflow/instructions.local").is_dir());
    let readme = fs::read_to_string(dir.path().join(".mochiflow/instructions/README.md")).unwrap();
    assert_instructions_readme_contract(&readme);
    let ignore = fs::read_to_string(dir.path().join(".mochiflow/.gitignore")).unwrap();
    assert!(ignore.contains("\ninstructions.local/\n"), "{ignore}");
    assert!(!ignore.contains("\ninstructions/\n"), "{ignore}");
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(
        out.contains("prepared user instructions: .mochiflow/instructions/ (shareable), .mochiflow/instructions.local/ (local-only)"),
        "{out}"
    );
}

#[test]
fn init_yes_uses_defaults_without_prompting() {
    let dir = tempfile::tempdir().unwrap();
    let result = bin()
        .args(["init", "--yes", "--target", dir.path().to_str().unwrap()])
        .env("LANG", "en_US.UTF-8")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .assert()
        .success();

    let cfg = fs::read_to_string(dir.path().join(".mochiflow/config.toml")).unwrap();
    assert!(cfg.contains("tools = [\"agents\"]"), "got:\n{cfg}");
    assert!(cfg.contains("[i18n]"), "got:\n{cfg}");
    assert!(cfg.contains("artifact_language = \"en\""), "got:\n{cfg}");
    assert!(
        cfg.contains("conversation_language = \"auto\""),
        "got:\n{cfg}"
    );
    assert!(!cfg.contains("\nlanguage = "), "got:\n{cfg}");
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(out.contains("artifact language: en (default)"), "{out}");
    assert!(
        out.contains("conversation language: auto (default)"),
        "{out}"
    );
    assert!(out.contains("Status:"), "{out}");
    assert!(
        out.contains("paste the setup prompt below into your AI agent"),
        "{out}"
    );
    assert!(out.contains("not errors"), "{out}");
    assert!(out.contains("Paste this into your AI agent"), "{out}");
}

#[test]
fn init_uses_artifact_language_flag_without_prompting() {
    let dir = tempfile::tempdir().unwrap();
    let result = bin()
        .args([
            "init",
            "--yes",
            "--adapter",
            "kiro",
            "--artifact-language",
            "ja",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .env("LANG", "en_US.UTF-8")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .assert()
        .success();

    let cfg = fs::read_to_string(dir.path().join(".mochiflow/config.toml")).unwrap();
    assert!(cfg.contains("artifact_language = \"ja\""), "got:\n{cfg}");
    assert!(
        cfg.contains("conversation_language = \"auto\""),
        "got:\n{cfg}"
    );
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(out.contains("artifact language: ja (flag)"), "{out}");
    assert!(out.contains("初期設定を完了してください"), "{out}");
    assert!(out.contains("エラーではありません"), "{out}");
    assert!(
        out.contains("AI アシスタントにこの文を貼ってください"),
        "{out}"
    );
}

#[test]
fn init_existing_config_preserves_config_but_still_scaffolds_init_paths() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args([
            "init",
            "--artifact-language",
            "en",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .write_stdin("")
        .assert()
        .success();
    fs::write(
        dir.path().join(".mochiflow/context/product.md"),
        "# Product\n\nP.\n",
    )
    .unwrap();
    fs::write(
        dir.path().join(".mochiflow/context/structure.md"),
        "# Structure\n\nS.\n",
    )
    .unwrap();
    fs::write(
        dir.path().join(".mochiflow/context/tech.md"),
        "# Tech\n\nT.\n",
    )
    .unwrap();
    let config = dir.path().join(".mochiflow/config.toml");
    bin()
        .args(["--config", config.to_str().unwrap(), "index"])
        .assert()
        .success();
    fs::remove_dir_all(dir.path().join(".mochiflow/instructions")).unwrap();
    fs::remove_dir_all(dir.path().join(".mochiflow/instructions.local")).unwrap();

    let result = bin()
        .args([
            "init",
            "--yes",
            "--adapter",
            "kiro",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .env("LANG", "ja_JP.UTF-8")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .assert()
        .success();
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(!out.contains("running join-style local setup"), "{out}");
    assert!(
        out.contains(
            "Ignoring --adapter kiro because existing config is kept; configured adapters: agents"
        ),
        "{out}"
    );
    assert!(!out.contains("Join: 0 fail"), "{out}");
    assert!(out.contains("generated adapters: agents"), "{out}");
    assert!(
        out.contains("prepared user instructions: .mochiflow/instructions/ (shareable), .mochiflow/instructions.local/ (local-only)"),
        "{out}"
    );
    assert!(
        dir.path()
            .join(".mochiflow/instructions/README.md")
            .exists()
    );
    assert!(dir.path().join(".mochiflow/instructions.local").is_dir());
    assert!(dir.path().join("AGENTS.md").exists());
    assert!(!dir.path().join(".kiro/steering/mochiflow.md").exists());
}

/// Idempotent/non-destructive: running init twice preserves hand-edited config.
#[test]
fn init_is_idempotent_nondestructive() {
    let dir = tempfile::tempdir().unwrap();

    // First run
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();

    // Hand-edit a value in config
    let config_path = dir.path().join(".mochiflow/config.toml");
    let original = fs::read_to_string(&config_path).unwrap();
    let edited = original.replace("artifact_language = \"en\"", "artifact_language = \"ja\"");
    fs::write(&config_path, &edited).unwrap();

    // Second run without --force preserves config and still follows init setup.
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();

    // Config should be preserved (not overwritten)
    let after = fs::read_to_string(&config_path).unwrap();
    assert!(
        after.contains("artifact_language = \"ja\""),
        "hand-edited value should be preserved, got:\n{after}"
    );
}

#[test]
fn init_preserves_user_instruction_files_even_with_force() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();

    let readme = dir.path().join(".mochiflow/instructions/README.md");
    let shared_note = dir.path().join(".mochiflow/instructions/release.md");
    let local_note = dir.path().join(".mochiflow/instructions.local/private.md");
    fs::write(&readme, "# Custom\n\nKeep me.\n").unwrap();
    fs::write(&shared_note, "# Release\n").unwrap();
    fs::write(&local_note, "# Private\n").unwrap();

    for args in [
        ["init", "--target", dir.path().to_str().unwrap()].as_slice(),
        ["init", "--force", "--target", dir.path().to_str().unwrap()].as_slice(),
    ] {
        bin().args(args).write_stdin("").assert().success();
        assert_eq!(
            fs::read_to_string(&readme).unwrap(),
            "# Custom\n\nKeep me.\n"
        );
        assert_eq!(fs::read_to_string(&shared_note).unwrap(), "# Release\n");
        assert_eq!(fs::read_to_string(&local_note).unwrap(), "# Private\n");
    }
}

#[test]
fn init_appends_local_instructions_ignore_rule_without_hiding_shared_files() {
    let dir = tempfile::tempdir().unwrap();
    let install = dir.path().join(".mochiflow");
    fs::create_dir_all(&install).unwrap();
    fs::write(
        install.join(".gitignore"),
        "state/\n# custom\ninstructions.local/\n",
    )
    .unwrap();

    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();

    let ignore = fs::read_to_string(install.join(".gitignore")).unwrap();
    assert!(ignore.contains("# custom"), "{ignore}");
    assert_eq!(
        count_occurrences(&ignore, "instructions.local/"),
        1,
        "{ignore}"
    );
    assert_eq!(count_occurrences(&ignore, "instructions/"), 0, "{ignore}");

    fs::write(
        dir.path().join(".mochiflow/instructions/shared.md"),
        "# Shared\n",
    )
    .unwrap();
    fs::write(
        dir.path().join(".mochiflow/instructions.local/private.md"),
        "# Private\n",
    )
    .unwrap();
    StdCommand::new("git")
        .args(["init", "-q"])
        .current_dir(dir.path())
        .status()
        .unwrap();
    let status = StdCommand::new("git")
        .args(["status", "--short", "--untracked-files=all"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&status.stdout).into_owned();
    assert!(
        stdout.contains(".mochiflow/instructions/shared.md"),
        "{stdout}"
    );
    assert!(
        !stdout.contains(".mochiflow/instructions.local/private.md"),
        "{stdout}"
    );
}

#[test]
fn init_appends_missing_local_instructions_ignore_rule_to_custom_file() {
    let dir = tempfile::tempdir().unwrap();
    let install = dir.path().join(".mochiflow");
    fs::create_dir_all(&install).unwrap();
    fs::write(
        install.join(".gitignore"),
        "state/\n# custom without local rule\n",
    )
    .unwrap();

    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();

    let ignore = fs::read_to_string(install.join(".gitignore")).unwrap();
    assert!(ignore.contains("# custom without local rule"), "{ignore}");
    assert!(ignore.ends_with("instructions.local/\n"), "{ignore}");
    assert_eq!(
        count_occurrences(&ignore, "instructions.local/"),
        1,
        "{ignore}"
    );
    assert_eq!(count_occurrences(&ignore, "instructions/"), 0, "{ignore}");
}

#[test]
fn init_fails_when_instruction_paths_are_files() {
    for rel in [".mochiflow/instructions", ".mochiflow/instructions.local"] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "not a directory\n").unwrap();

        let result = bin()
            .args(["init", "--target", dir.path().to_str().unwrap()])
            .write_stdin("")
            .assert()
            .failure()
            .code(1);
        let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
        assert!(out.contains("FAIL: could not create"), "{out}");
        assert_eq!(fs::read_to_string(&path).unwrap(), "not a directory\n");
    }
}

#[test]
fn init_fails_when_instructions_readme_path_is_directory() {
    let dir = tempfile::tempdir().unwrap();
    let readme = dir.path().join(".mochiflow/instructions/README.md");
    fs::create_dir_all(&readme).unwrap();

    let result = bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .failure()
        .code(1);
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(
        out.contains("FAIL: could not create user instructions"),
        "{out}"
    );
    assert!(
        readme.is_dir(),
        "{} should remain a directory",
        readme.display()
    );
}

#[test]
fn join_requires_existing_config() {
    let dir = tempfile::tempdir().unwrap();
    let result = bin()
        .args(["join", "--target", dir.path().to_str().unwrap()])
        .assert()
        .failure()
        .code(1);
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(out.contains("config.toml not found"), "{out}");
    assert!(out.contains("mochiflow init"), "{out}");
}

#[test]
fn index_fails_when_index_path_is_not_writable_file() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();
    let config = dir.path().join(".mochiflow/config.toml");
    let original = fs::read_to_string(&config).unwrap();
    fs::write(
        &config,
        original.replace("index = \".mochiflow/INDEX.md\"", "index = \".mochiflow\""),
    )
    .unwrap();

    let result = bin()
        .args(["--config", config.to_str().unwrap(), "index"])
        .assert()
        .failure()
        .code(1);
    let out = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(out.contains("FAIL: could not write index files"), "{out}");
}

fn prepare_join_project(dir: &tempfile::TempDir) -> PathBuf {
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();
    let config = dir.path().join(".mochiflow/config.toml");
    bin()
        .args(["--config", config.to_str().unwrap(), "index"])
        .assert()
        .success();
    config
}

#[test]
fn join_restores_local_engine_without_touching_shared_files() {
    let dir = tempfile::tempdir().unwrap();
    prepare_join_project(&dir);
    let agents = dir.path().join("AGENTS.md");
    let index = dir.path().join(".mochiflow/INDEX.md");
    let agents_before = fs::read_to_string(&agents).unwrap();
    let index_before = fs::read_to_string(&index).unwrap();
    fs::remove_dir_all(dir.path().join(".mochiflow/engine")).unwrap();
    fs::remove_dir_all(dir.path().join(".mochiflow/state")).unwrap();

    bin()
        .args(["join", "--target", dir.path().to_str().unwrap()])
        .assert()
        .success();

    assert!(dir.path().join(".mochiflow/engine/VERSION").exists());
    assert!(dir.path().join(".mochiflow/state/doctor.json").exists());
    assert_eq!(fs::read_to_string(&agents).unwrap(), agents_before);
    assert_eq!(fs::read_to_string(&index).unwrap(), index_before);
}

#[test]
fn join_and_upgrade_do_not_backfill_user_instruction_dirs() {
    let dir = tempfile::tempdir().unwrap();
    let config = prepare_join_project(&dir);
    fs::remove_dir_all(dir.path().join(".mochiflow/instructions")).unwrap();
    fs::remove_dir_all(dir.path().join(".mochiflow/instructions.local")).unwrap();

    bin()
        .args(["join", "--target", dir.path().to_str().unwrap()])
        .assert()
        .success();
    assert!(!dir.path().join(".mochiflow/instructions").exists());
    assert!(!dir.path().join(".mochiflow/instructions.local").exists());

    bin()
        .args(["--config", config.to_str().unwrap(), "upgrade"])
        .assert()
        .success();
    assert!(!dir.path().join(".mochiflow/instructions").exists());
    assert!(!dir.path().join(".mochiflow/instructions.local").exists());
}

#[test]
fn join_regenerates_markdown_adapters_and_index() {
    let dir = tempfile::tempdir().unwrap();
    prepare_join_project(&dir);
    let agents = dir.path().join("AGENTS.md");
    let index = dir.path().join(".mochiflow/INDEX.md");
    fs::write(&agents, "CUSTOM AGENTS\n").unwrap();
    fs::write(&index, "# stale\n").unwrap();

    bin()
        .args(["join", "--target", dir.path().to_str().unwrap()])
        .assert()
        .success();

    let repaired = fs::read_to_string(&agents).unwrap();
    assert!(repaired.starts_with("CUSTOM AGENTS\n"), "{repaired}");
    assert!(
        repaired.contains("<!-- mochiflow:begin adapter=agents -->"),
        "{repaired}"
    );
    let regenerated_index = fs::read_to_string(&index).unwrap();
    assert!(regenerated_index.contains("# 📋 Spec Dashboard"));
}

#[test]
fn join_rejects_removed_repair_flag() {
    let dir = tempfile::tempdir().unwrap();
    prepare_join_project(&dir);

    bin()
        .args(["join", "--repair", "--target", dir.path().to_str().unwrap()])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn join_blocks_handwritten_structured_adapter_and_writes_candidate() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args([
            "init",
            "--adapter",
            "kiro",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .write_stdin("")
        .assert()
        .success();

    let structured = dir.path().join(".kiro/agents/spec-change-reviewer.json");
    fs::write(&structured, "{\"custom\": true}\n").unwrap();

    let result = bin()
        .args(["join", "--target", dir.path().to_str().unwrap()])
        .assert()
        .failure()
        .code(1);
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(out.contains("Blocked adapters"), "{out}");
    assert!(
        out.contains(".kiro/agents/spec-change-reviewer.json"),
        "{out}"
    );
    assert!(
        dir.path()
            .join(".mochiflow/state/adapters/.kiro/agents/spec-change-reviewer.json")
            .exists()
    );
    assert_eq!(
        fs::read_to_string(&structured).unwrap(),
        "{\"custom\": true}\n"
    );
}

#[test]
fn join_dry_run_does_not_write_shared_files() {
    let dir = tempfile::tempdir().unwrap();
    prepare_join_project(&dir);
    let agents = dir.path().join("AGENTS.md");
    let index = dir.path().join(".mochiflow/INDEX.md");
    fs::write(&agents, "CUSTOM AGENTS\n").unwrap();
    fs::write(&index, "# stale\n").unwrap();

    let result = bin()
        .args([
            "join",
            "--dry-run",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(
        out.contains("would regenerate adapter entrypoints"),
        "{out}"
    );
    assert!(out.contains("would regenerate"), "{out}");
    assert_eq!(fs::read_to_string(&agents).unwrap(), "CUSTOM AGENTS\n");
    assert_eq!(fs::read_to_string(&index).unwrap(), "# stale\n");
}

#[test]
fn join_json_emits_single_stdout_document() {
    let dir = tempfile::tempdir().unwrap();
    prepare_join_project(&dir);
    fs::remove_dir_all(dir.path().join(".mochiflow/engine")).unwrap();

    let result = bin()
        .args(["join", "--json", "--target", dir.path().to_str().unwrap()])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["mode"].as_str(), Some("join"), "{stdout}");
    assert_eq!(json["exit_code"].as_i64(), Some(0), "{stdout}");
    assert!(dir.path().join(".mochiflow/engine/VERSION").exists());
}

#[test]
fn join_blocks_dirty_engine_without_force() {
    let dir = tempfile::tempdir().unwrap();
    prepare_join_project(&dir);
    let router = dir.path().join(".mochiflow/engine/router.md");
    fs::write(&router, "local engine edit\n").unwrap();

    let result = bin()
        .args(["join", "--target", dir.path().to_str().unwrap()])
        .assert()
        .failure()
        .code(1);
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(out.contains("DIRTY:"), "{out}");
    assert_eq!(fs::read_to_string(&router).unwrap(), "local engine edit\n");

    bin()
        .args(["join", "--force", "--target", dir.path().to_str().unwrap()])
        .assert()
        .success();
    assert_ne!(fs::read_to_string(&router).unwrap(), "local engine edit\n");
}

#[test]
fn init_blocks_dirty_engine_without_force() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();

    let router = dir.path().join(".mochiflow/engine/router.md");
    fs::write(&router, "local engine edit\n").unwrap();

    let result = bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .failure()
        .code(1);

    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(out.contains("DIRTY:"), "{out}");
    assert!(out.contains("--force"), "{out}");
    assert_eq!(fs::read_to_string(&router).unwrap(), "local engine edit\n");
}

#[test]
fn init_force_replaces_dirty_engine_and_repairs_manifest() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();

    let router = dir.path().join(".mochiflow/engine/router.md");
    fs::write(&router, "local engine edit\n").unwrap();

    bin()
        .args(["init", "--force", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();

    assert_ne!(fs::read_to_string(&router).unwrap(), "local engine edit\n");
    let config = dir.path().join(".mochiflow/config.toml");
    bin()
        .args(["--config", config.to_str().unwrap(), "doctor", "engine"])
        .assert()
        .success();
}

#[test]
fn init_force_staged_swap_removes_stale_engine_files() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();

    let obsolete = dir.path().join(".mochiflow/engine/obsolete.txt");
    fs::write(&obsolete, "old\n").unwrap();
    assert!(obsolete.exists());

    bin()
        .args(["init", "--force", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();

    assert!(
        !obsolete.exists(),
        "{} should be removed",
        obsolete.display()
    );
}

#[test]
fn init_force_ignores_old_fixed_staging_path() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();

    let version = dir.path().join(".mochiflow/engine/VERSION");
    let before = fs::read_to_string(&version).unwrap();
    fs::write(dir.path().join(".mochiflow/.engine.upgrade"), "not a dir\n").unwrap();

    bin()
        .args(["init", "--force", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();

    assert_eq!(fs::read_to_string(&version).unwrap(), before);
    assert_eq!(
        fs::read_to_string(dir.path().join(".mochiflow/.engine.upgrade")).unwrap(),
        "not a dir\n"
    );
    assert!(
        fs::read_dir(dir.path().join(".mochiflow"))
            .unwrap()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .all(|name| !name.starts_with(".engine.backup-")
                && !name.starts_with(".engine.upgrade-")),
        "unique staging/backup directories should be cleaned up"
    );
}

/// --adapter resolves the `codex` label to the `agents` ID, and i18n flags are
/// written to config.
#[test]
fn init_respects_adapter_i18n_overrides() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args([
            "init",
            "--adapter",
            "codex",
            "--artifact-language",
            "ja",
            "--conversation-language",
            "auto",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .env("LANG", "en_US.UTF-8")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .write_stdin("")
        .assert()
        .success();

    let cfg = fs::read_to_string(dir.path().join(".mochiflow/config.toml")).unwrap();
    assert!(cfg.contains("tools = [\"agents\"]"), "got:\n{cfg}");
    assert!(cfg.contains("artifact_language = \"ja\""), "got:\n{cfg}");
    assert!(
        cfg.contains("conversation_language = \"auto\""),
        "got:\n{cfg}"
    );
    let result = bin()
        .args([
            "init",
            "--adapter",
            "codex",
            "--artifact-language",
            "ja",
            "--conversation-language",
            "auto",
            "--target",
            dir.path().to_str().unwrap(),
            "--force",
        ])
        .write_stdin("")
        .assert()
        .success();
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(out.contains("artifact language: ja (flag)"), "{out}");
    assert!(out.contains("conversation language: auto (flag)"), "{out}");
    assert!(
        out.contains("AI アシスタントにこの文を貼ってください"),
        "{out}"
    );
}

/// Repeated --adapter flags produce a multi-tool config, order preserved.
#[test]
fn init_accepts_multiple_adapters() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args([
            "init",
            "--adapter",
            "kiro",
            "--adapter",
            "claude-code",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .write_stdin("")
        .assert()
        .success();

    let cfg = fs::read_to_string(dir.path().join(".mochiflow/config.toml")).unwrap();
    let parsed: toml::Value = toml::from_str(&cfg).unwrap();
    let tools = parsed["adapter"]["tools"].as_array().unwrap();
    assert_eq!(tools[0].as_str(), Some("kiro"), "got:\n{cfg}");
    assert_eq!(tools[1].as_str(), Some("claude-code"), "got:\n{cfg}");
}

/// Comma-separated adapters are split, resolved, and de-duplicated
/// (`codex,agents` -> `agents`).
#[test]
fn init_dedupes_comma_separated_adapters() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args([
            "init",
            "--adapter",
            "codex,agents",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .write_stdin("")
        .assert()
        .success();

    let cfg = fs::read_to_string(dir.path().join(".mochiflow/config.toml")).unwrap();
    assert!(cfg.contains("tools = [\"agents\"]"), "got:\n{cfg}");
}

/// Existing Markdown adapter targets are preserved and extended with a managed
/// MochiFlow block.
#[test]
fn init_existing_markdown_adapter_target_appends_managed_block() {
    let dir = tempfile::tempdir().unwrap();
    let agents = dir.path().join("AGENTS.md");
    fs::write(&agents, "CUSTOM AGENTS\n").unwrap();

    let result = bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();
    let body = fs::read_to_string(&agents).unwrap();
    assert!(body.starts_with("CUSTOM AGENTS\n"), "{body}");
    assert!(
        body.contains("<!-- mochiflow:begin adapter=agents -->"),
        "{body}"
    );
    assert!(
        body.contains("generated by mochiflow adapter=agents"),
        "{body}"
    );
    assert!(body.contains("Load the router"), "{body}");
    assert!(
        !dir.path()
            .join(".mochiflow/state/adapters/AGENTS.md")
            .exists()
    );

    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(out.contains("Status:"), "{out}");
    assert!(out.contains("generated adapters: agents"), "{out}");
    assert!(!out.contains("Blocked"), "{out}");
}

#[test]
fn init_blocked_json_exits_1_and_includes_candidate() {
    let dir = tempfile::tempdir().unwrap();
    let existing = dir.path().join(".kiro/agents/spec-change-reviewer.json");
    fs::create_dir_all(existing.parent().unwrap()).unwrap();
    fs::write(&existing, "{\"custom\": true}\n").unwrap();

    let result = bin()
        .args([
            "init",
            "--json",
            "--adapter",
            "kiro",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .write_stdin("")
        .assert()
        .failure()
        .code(1);

    let stdout = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["status"].as_str(), Some("blocked"), "{stdout}");
    assert_eq!(json["exit_code"].as_i64(), Some(1), "{stdout}");
    assert_eq!(
        json["blocked"]["required"].as_bool(),
        Some(true),
        "{stdout}"
    );
    assert_eq!(
        json["blocked"]["items"][0]["candidate"].as_str(),
        Some(".mochiflow/state/adapters/.kiro/agents/spec-change-reviewer.json"),
        "{stdout}"
    );
}

/// Blocked adapter guidance follows the configured language.
#[test]
fn init_existing_structured_adapter_target_guidance_is_language_aware() {
    let dir = tempfile::tempdir().unwrap();
    let existing = dir.path().join(".kiro/agents/spec-change-reviewer.json");
    fs::create_dir_all(existing.parent().unwrap()).unwrap();
    fs::write(&existing, "{\"custom\": true}\n").unwrap();

    let result = bin()
        .args([
            "init",
            "--adapter",
            "kiro",
            "--artifact-language",
            "ja",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .write_stdin("")
        .assert()
        .failure()
        .code(1);

    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(out.contains("確認が必要:"), "{out}");
    assert!(out.contains("Blocked"), "{out}");
    assert!(
        out.contains(".kiro/agents/spec-change-reviewer.json は既に存在する構造化 adapter ファイルのため上書きしませんでした"),
        "{out}"
    );
    assert!(
        out.contains(".mochiflow/state/adapters/.kiro/agents/spec-change-reviewer.json"),
        "{out}"
    );
}

/// The blocked/candidate behavior applies to every adapter target, including
/// nested Kiro paths.
#[test]
fn init_existing_nested_adapter_target_writes_nested_candidate() {
    let dir = tempfile::tempdir().unwrap();
    let existing = dir.path().join(".kiro/agents/spec-change-reviewer.json");
    fs::create_dir_all(existing.parent().unwrap()).unwrap();
    fs::write(&existing, "{\"custom\": true}\n").unwrap();

    bin()
        .args([
            "init",
            "--adapter",
            "kiro",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .write_stdin("")
        .assert()
        .failure()
        .code(1);

    assert_eq!(
        fs::read_to_string(&existing).unwrap(),
        "{\"custom\": true}\n"
    );
    let candidate = dir
        .path()
        .join(".mochiflow/state/adapters/.kiro/agents/spec-change-reviewer.json");
    let candidate_body = fs::read_to_string(&candidate).unwrap();
    assert!(
        candidate_body.contains("generated by mochiflow"),
        "{candidate_body}"
    );
    assert!(
        candidate_body.contains("spec-change-reviewer"),
        "{candidate_body}"
    );
}

#[test]
fn kiro_adapter_ignores_existing_custom_hooks() {
    let dir = tempfile::tempdir().unwrap();
    let custom_hook = dir.path().join(".kiro/hooks/custom.kiro.hook");
    fs::create_dir_all(custom_hook.parent().unwrap()).unwrap();
    fs::write(&custom_hook, "custom hook\n").unwrap();

    bin()
        .args([
            "init",
            "--adapter",
            "kiro",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .write_stdin("")
        .assert()
        .success();

    assert_eq!(fs::read_to_string(&custom_hook).unwrap(), "custom hook\n");
    assert!(
        !dir.path()
            .join(".kiro/hooks/generate-project-index.kiro.hook")
            .exists()
    );

    let config = dir.path().join(".mochiflow/config.toml");
    bin()
        .args([
            "--config",
            config.to_str().unwrap(),
            "adapter",
            "generate",
            "--check",
        ])
        .assert()
        .success();
}

#[test]
fn detach_removes_managed_block_and_runtime_but_preserves_project_data() {
    let dir = tempfile::tempdir().unwrap();
    let agents = dir.path().join("AGENTS.md");
    fs::write(&agents, "CUSTOM AGENTS\n").unwrap();

    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();
    let with_block = fs::read_to_string(&agents).unwrap();
    assert!(
        with_block.contains("<!-- mochiflow:begin adapter=agents -->"),
        "{with_block}"
    );

    bin()
        .args(["detach", "--target", dir.path().to_str().unwrap()])
        .assert()
        .success();

    let detached = fs::read_to_string(&agents).unwrap();
    assert_eq!(detached, "CUSTOM AGENTS\n");
    assert!(dir.path().join(".mochiflow/engine").exists());
    assert!(!dir.path().join(".mochiflow/state").exists());
    assert!(dir.path().join(".mochiflow/config.toml").exists());
    assert!(dir.path().join(".mochiflow/specs").exists());
    assert!(dir.path().join(".mochiflow/adr/decisions").is_dir());
    assert!(dir.path().join(".mochiflow/adr/pitfalls").is_dir());
    assert!(!dir.path().join(".mochiflow/adr/decisions.md").exists());
    assert!(dir.path().join(".mochiflow/context/product.md").exists());
    assert!(dir.path().join(".mochiflow/constitution.md").exists());
    assert!(
        dir.path()
            .join(".mochiflow/instructions/README.md")
            .exists()
    );
    assert!(dir.path().join(".mochiflow/instructions.local").is_dir());

    bin()
        .args(["join", "--target", dir.path().to_str().unwrap()])
        .env("LANG", "en_US.UTF-8")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .assert()
        .success();

    let reinit = fs::read_to_string(&agents).unwrap();
    assert!(reinit.starts_with("CUSTOM AGENTS\n"), "{reinit}");
    assert!(
        reinit.contains("<!-- mochiflow:begin adapter=agents -->"),
        "{reinit}"
    );
    assert!(dir.path().join(".mochiflow/engine/VERSION").exists());
}

#[test]
fn detach_deletes_full_generated_adapter_files_for_all_tools() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args([
            "init",
            "--adapter",
            "agents,claude-code,copilot,kiro",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .write_stdin("")
        .assert()
        .success();

    for rel in [
        "AGENTS.md",
        "CLAUDE.md",
        ".github/copilot-instructions.md",
        ".kiro/steering/mochiflow.md",
        ".kiro/agents/spec-plan-auditor.json",
        ".kiro/agents/spec-change-reviewer.json",
    ] {
        assert!(
            dir.path().join(rel).exists(),
            "{rel} should exist before detach"
        );
    }

    bin()
        .args(["detach", "--target", dir.path().to_str().unwrap()])
        .assert()
        .success();

    for rel in [
        "AGENTS.md",
        "CLAUDE.md",
        ".github/copilot-instructions.md",
        ".kiro/steering/mochiflow.md",
        ".kiro/agents/spec-plan-auditor.json",
        ".kiro/agents/spec-change-reviewer.json",
    ] {
        assert!(!dir.path().join(rel).exists(), "{rel} should be removed");
    }
}

#[test]
fn detach_leaves_unmanaged_adapter_and_custom_kiro_files() {
    let dir = tempfile::tempdir().unwrap();
    let agents = dir.path().join("AGENTS.md");
    let custom_agent = dir.path().join(".kiro/agents/custom.json");
    let custom_hook = dir.path().join(".kiro/hooks/custom.kiro.hook");
    fs::write(&agents, "CUSTOM AGENTS\n").unwrap();
    fs::create_dir_all(custom_agent.parent().unwrap()).unwrap();
    fs::write(&custom_agent, "{\"custom\": true}\n").unwrap();
    fs::create_dir_all(custom_hook.parent().unwrap()).unwrap();
    fs::write(&custom_hook, "custom hook\n").unwrap();

    bin()
        .args([
            "init",
            "--adapter",
            "kiro",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .write_stdin("")
        .assert()
        .success();

    bin()
        .args(["detach", "--target", dir.path().to_str().unwrap()])
        .assert()
        .success();

    assert_eq!(fs::read_to_string(&agents).unwrap(), "CUSTOM AGENTS\n");
    assert_eq!(
        fs::read_to_string(&custom_agent).unwrap(),
        "{\"custom\": true}\n"
    );
    assert_eq!(fs::read_to_string(&custom_hook).unwrap(), "custom hook\n");
}

#[test]
fn detach_dry_run_and_json_report_without_writing() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();

    let result = bin()
        .args([
            "detach",
            "--dry-run",
            "--json",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["mode"].as_str(), Some("detach"), "{stdout}");
    assert_eq!(json["dry_run"].as_bool(), Some(true), "{stdout}");
    assert_eq!(json["exit_code"].as_i64(), Some(0), "{stdout}");
    assert!(
        json["removed"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == "AGENTS.md")
    );

    assert!(dir.path().join("AGENTS.md").exists());
    assert!(dir.path().join(".mochiflow/engine").exists());
}

#[test]
fn detach_purge_requires_exact_confirmation_and_deletes_all_project_data() {
    let dir = tempfile::tempdir().unwrap();
    let agents = dir.path().join("AGENTS.md");
    fs::write(&agents, "CUSTOM AGENTS\n").unwrap();
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();
    fs::write(
        dir.path().join(".mochiflow/instructions/shared.md"),
        "# Shared\n",
    )
    .unwrap();
    fs::write(
        dir.path().join(".mochiflow/instructions.local/private.md"),
        "# Private\n",
    )
    .unwrap();

    let result = bin()
        .args([
            "detach",
            "--purge",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .code(1);
    let stdout = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    let stderr = String::from_utf8_lossy(&result.get_output().stderr).into_owned();
    assert!(
        stdout.contains(".mochiflow/instructions/")
            && stdout.contains(".mochiflow/instructions.local/"),
        "{stdout}"
    );
    assert!(
        stderr.contains(".mochiflow/instructions/")
            && stderr.contains(".mochiflow/instructions.local/"),
        "{stderr}"
    );
    assert!(dir.path().join(".mochiflow").exists());
    assert!(
        fs::read_to_string(&agents)
            .unwrap()
            .contains("<!-- mochiflow:begin adapter=agents -->")
    );

    let result = bin()
        .args([
            "detach",
            "--purge",
            "--confirm",
            "delete mochiflow data",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    let stderr = String::from_utf8_lossy(&result.get_output().stderr).into_owned();
    assert!(
        stdout.contains(".mochiflow/instructions/")
            && stdout.contains(".mochiflow/instructions.local/"),
        "{stdout}"
    );
    assert!(
        stderr.contains(".mochiflow/instructions/")
            && stderr.contains(".mochiflow/instructions.local/"),
        "{stderr}"
    );

    assert!(!dir.path().join(".mochiflow").exists());
    assert_eq!(fs::read_to_string(&agents).unwrap(), "CUSTOM AGENTS\n");
}

#[test]
fn detach_json_purge_warning_stays_out_of_stdout_document() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();

    let result = bin()
        .args([
            "detach",
            "--purge",
            "--json",
            "--confirm",
            "delete mochiflow data",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    let stderr = String::from_utf8_lossy(&result.get_output().stderr).into_owned();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["mode"].as_str(), Some("purge"), "{stdout}");
    assert_eq!(json["warnings"].as_array().unwrap().len(), 1, "{stdout}");
    assert!(
        json["warnings"][0]
            .as_str()
            .unwrap()
            .contains(".mochiflow/instructions.local/"),
        "{stdout}"
    );
    assert!(
        stderr.contains(".mochiflow/instructions/")
            && stderr.contains(".mochiflow/instructions.local/"),
        "{stderr}"
    );
    assert!(!dir.path().join(".mochiflow").exists());
}

/// Markdown targets are extended, while structured unmanaged targets still
/// block with candidates. Kiro steering (mochiflow.md) is full-file managed
/// (overwrites a markerless file rather than appending a managed block).
#[test]
fn init_existing_adapter_targets_append_markdown_and_block_structured_files() {
    let dir = tempfile::tempdir().unwrap();
    for rel in [
        "AGENTS.md",
        "CLAUDE.md",
        ".github/copilot-instructions.md",
        ".kiro/agents/spec-change-reviewer.json",
    ] {
        let target = dir.path().join(rel);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(target, format!("CUSTOM {rel}\n")).unwrap();
    }
    // Also pre-create mochiflow.md (markerless): will be overwritten whole
    let mochiflow_md = dir.path().join(".kiro/steering/mochiflow.md");
    fs::create_dir_all(mochiflow_md.parent().unwrap()).unwrap();
    fs::write(&mochiflow_md, "CUSTOM mochiflow.md\n").unwrap();

    bin()
        .args([
            "init",
            "--adapter",
            "agents,claude-code,copilot,kiro",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .write_stdin("")
        .assert()
        .failure()
        .code(1);

    for rel in ["AGENTS.md", "CLAUDE.md", ".github/copilot-instructions.md"] {
        let body = fs::read_to_string(dir.path().join(rel)).unwrap();
        assert!(
            body.starts_with(&format!("CUSTOM {rel}\n")),
            "{rel}:\n{body}"
        );
        assert!(
            body.contains("<!-- mochiflow:begin adapter="),
            "{rel}:\n{body}"
        );
        assert!(
            !dir.path()
                .join(".mochiflow/state/adapters")
                .join(rel)
                .exists(),
            "{rel} should not write a candidate"
        );
    }

    // mochiflow.md is full-file managed: overwritten whole, not block-appended
    {
        let body = fs::read_to_string(&mochiflow_md).unwrap();
        assert!(
            body.starts_with("---\ninclusion: always\n---"),
            "mochiflow.md frontmatter at top:\n{body}"
        );
        assert!(
            !body.contains("CUSTOM mochiflow.md"),
            "markerless pre-existing must be overwritten:\n{body}"
        );
    }

    // The reviewer JSON is BLOCKED (custom, no marker)
    {
        let rel = ".kiro/agents/spec-change-reviewer.json";
        let target = dir.path().join(rel);
        assert_eq!(
            fs::read_to_string(&target).unwrap(),
            format!("CUSTOM {rel}\n")
        );

        let candidate = dir.path().join(".mochiflow/state/adapters").join(rel);
        let candidate_body = fs::read_to_string(&candidate).unwrap();
        assert!(
            candidate_body.contains("generated by mochiflow"),
            "{rel}:\n{candidate_body}"
        );
    }
}

/// `adapter generate --check` reports drift only; it must not write candidate files.
#[test]
fn adapter_generate_check_does_not_write_candidate() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();
    fs::write(dir.path().join("AGENTS.md"), "CUSTOM AGENTS\n").unwrap();
    let config = dir.path().join(".mochiflow/config.toml");

    bin()
        .args([
            "--config",
            config.to_str().unwrap(),
            "adapter",
            "generate",
            "--check",
        ])
        .assert()
        .failure();

    assert!(
        !dir.path()
            .join(".mochiflow/state/adapters/AGENTS.md")
            .exists()
    );
}

#[test]
fn adapter_generate_check_fails_on_missing_manifest() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args([
            "init",
            "--adapter",
            "kiro",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .write_stdin("")
        .assert()
        .success();
    let config = dir.path().join(".mochiflow/config.toml");
    fs::remove_file(
        dir.path()
            .join(".mochiflow/engine/adapters/kiro/manifest.toml"),
    )
    .unwrap();

    let result = bin()
        .args([
            "--config",
            config.to_str().unwrap(),
            "adapter",
            "generate",
            "--check",
        ])
        .assert()
        .failure();
    let out = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(out.contains("manifest missing"), "{out}");

    let result = bin()
        .args(["--config", config.to_str().unwrap(), "doctor", "adapter"])
        .assert()
        .failure();
    let out = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(out.contains("manifest missing"), "{out}");
}

#[test]
fn adapter_generate_check_fails_on_missing_template() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args([
            "init",
            "--adapter",
            "kiro",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .write_stdin("")
        .assert()
        .success();
    let config = dir.path().join(".mochiflow/config.toml");
    fs::write(
        dir.path()
            .join(".mochiflow/engine/adapters/kiro/manifest.toml"),
        "[files]\n\".kiro/agents/spec-builder.json\" = \"missing.json.tpl\"\n",
    )
    .unwrap();

    let result = bin()
        .args([
            "--config",
            config.to_str().unwrap(),
            "adapter",
            "generate",
            "--check",
        ])
        .assert()
        .failure();
    let out = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(out.contains("template missing or unreadable"), "{out}");
}

/// Normal adapter generation appends a managed block to existing Markdown files.
#[test]
fn adapter_generate_appends_managed_block_to_existing_markdown() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();
    fs::write(dir.path().join("AGENTS.md"), "CUSTOM AGENTS\n").unwrap();
    let config = dir.path().join(".mochiflow/config.toml");

    let result = bin()
        .args(["--config", config.to_str().unwrap(), "adapter", "generate"])
        .assert()
        .success();

    let agents = fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
    assert!(agents.starts_with("CUSTOM AGENTS\n"), "{agents}");
    assert_eq!(
        count_occurrences(&agents, "<!-- mochiflow:begin adapter=agents -->"),
        1,
        "{agents}"
    );
    assert!(
        !dir.path()
            .join(".mochiflow/state/adapters/AGENTS.md")
            .exists()
    );

    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(out.contains("wrote: AGENTS.md"), "{out}");
    assert!(
        out.contains("Summary: 1 written, 0 blocked, 0 failed"),
        "{out}"
    );

    bin()
        .args(["--config", config.to_str().unwrap(), "adapter", "generate"])
        .assert()
        .success();
    let agents = fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
    assert_eq!(
        count_occurrences(&agents, "<!-- mochiflow:begin adapter=agents -->"),
        1,
        "{agents}"
    );

    bin()
        .args([
            "--config",
            config.to_str().unwrap(),
            "adapter",
            "generate",
            "--check",
        ])
        .assert()
        .success();
    bin()
        .args(["--config", config.to_str().unwrap(), "doctor", "adapter"])
        .assert()
        .success();
}

#[test]
fn adapter_generate_appends_managed_blocks_for_all_markdown_targets() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args([
            "init",
            "--adapter",
            "agents,claude-code,copilot,kiro",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .write_stdin("")
        .assert()
        .success();

    for rel in ["AGENTS.md", "CLAUDE.md", ".github/copilot-instructions.md"] {
        let target = dir.path().join(rel);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(&target, format!("CUSTOM {rel}\n")).unwrap();
    }
    let config = dir.path().join(".mochiflow/config.toml");

    bin()
        .args(["--config", config.to_str().unwrap(), "adapter", "generate"])
        .assert()
        .success();

    for (rel, adapter) in [
        ("AGENTS.md", "agents"),
        ("CLAUDE.md", "claude-code"),
        (".github/copilot-instructions.md", "copilot"),
    ] {
        let body = fs::read_to_string(dir.path().join(rel)).unwrap();
        assert!(
            body.starts_with(&format!("CUSTOM {rel}\n")),
            "{rel}:\n{body}"
        );
        assert_eq!(
            count_occurrences(
                &body,
                &format!("<!-- mochiflow:begin adapter={adapter} -->")
            ),
            1,
            "{rel}:\n{body}"
        );
        assert!(body.contains("generated by mochiflow"), "{rel}:\n{body}");
    }

    bin()
        .args([
            "--config",
            config.to_str().unwrap(),
            "adapter",
            "generate",
            "--check",
        ])
        .assert()
        .success();
}

/// `--force` keeps its replacement semantics and does not produce a blocked candidate.
#[test]
fn adapter_generate_force_replaces_existing_target_without_candidate() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();
    fs::write(dir.path().join("AGENTS.md"), "CUSTOM AGENTS\n").unwrap();
    let config = dir.path().join(".mochiflow/config.toml");

    bin()
        .args([
            "--config",
            config.to_str().unwrap(),
            "adapter",
            "generate",
            "--force",
        ])
        .assert()
        .success();

    let agents = fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
    assert!(agents.contains("generated by mochiflow"), "{agents}");
    assert!(
        !dir.path()
            .join(".mochiflow/state/adapters/AGENTS.md")
            .exists()
    );
}

#[test]
fn kiro_agent_model_override_is_not_adapter_drift_and_survives_force() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args([
            "init",
            "--adapter",
            "kiro",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .write_stdin("")
        .assert()
        .success();
    let config = dir.path().join(".mochiflow/config.toml");
    let reviewer = dir.path().join(".kiro/agents/spec-change-reviewer.json");

    let generated = fs::read_to_string(&reviewer).unwrap();
    assert!(
        generated.contains("\"model\": \"claude-opus-4.8\""),
        "{generated}"
    );

    set_json_field(
        &reviewer,
        "model",
        serde_json::Value::String("custom-reviewer-model".into()),
    );

    bin()
        .args([
            "--config",
            config.to_str().unwrap(),
            "adapter",
            "generate",
            "--check",
        ])
        .assert()
        .success();

    bin()
        .args(["--config", config.to_str().unwrap(), "doctor", "adapter"])
        .assert()
        .success();

    bin()
        .args([
            "--config",
            config.to_str().unwrap(),
            "adapter",
            "generate",
            "--force",
        ])
        .assert()
        .success();

    let reviewer_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&reviewer).unwrap()).unwrap();
    assert_eq!(
        reviewer_json["model"].as_str(),
        Some("custom-reviewer-model")
    );
}

#[test]
fn kiro_agent_non_model_override_is_adapter_drift() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args([
            "init",
            "--adapter",
            "kiro",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .write_stdin("")
        .assert()
        .success();
    let config = dir.path().join(".mochiflow/config.toml");
    let reviewer = dir.path().join(".kiro/agents/spec-change-reviewer.json");
    set_json_field(&reviewer, "custom", serde_json::Value::Bool(true));

    let result = bin()
        .args([
            "--config",
            config.to_str().unwrap(),
            "adapter",
            "generate",
            "--check",
        ])
        .assert()
        .failure();
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(
        out.contains("DRIFT: .kiro/agents/spec-change-reviewer.json"),
        "{out}"
    );
}

#[test]
fn adapter_generate_fails_when_candidate_parent_cannot_be_created() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args([
            "init",
            "--adapter",
            "kiro",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .write_stdin("")
        .assert()
        .success();
    let builder = dir.path().join(".kiro/agents/spec-change-reviewer.json");
    fs::write(&builder, "{\"custom\": true}\n").unwrap();
    let state = dir.path().join(".mochiflow/state");
    if state.exists() {
        if state.is_dir() {
            fs::remove_dir_all(&state).unwrap();
        } else {
            fs::remove_file(&state).unwrap();
        }
    }
    fs::write(&state, "not a directory\n").unwrap();
    let config = dir.path().join(".mochiflow/config.toml");

    let result = bin()
        .args(["--config", config.to_str().unwrap(), "adapter", "generate"])
        .assert()
        .failure();

    assert_eq!(
        fs::read_to_string(&builder).unwrap(),
        "{\"custom\": true}\n"
    );
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(out.contains("FAIL:"), "{out}");
    assert!(out.contains(".mochiflow/state/adapters"), "{out}");
    assert!(
        out.contains("Summary: 2 written, 0 blocked, 1 failed"),
        "{out}"
    );
}

#[test]
fn adapter_generate_errors_make_init_fail_when_candidate_parent_cannot_be_created() {
    let dir = tempfile::tempdir().unwrap();
    let existing = dir.path().join(".kiro/agents/spec-change-reviewer.json");
    fs::create_dir_all(existing.parent().unwrap()).unwrap();
    fs::write(&existing, "{\"custom\": true}\n").unwrap();
    let install = dir.path().join(".mochiflow");
    fs::create_dir_all(&install).unwrap();
    fs::write(install.join("state"), "not a directory\n").unwrap();

    let result = bin()
        .args([
            "init",
            "--adapter",
            "kiro",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .write_stdin("")
        .assert()
        .failure();

    assert_eq!(
        fs::read_to_string(&existing).unwrap(),
        "{\"custom\": true}\n"
    );
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(out.contains("FAIL:"), "{out}");
    assert!(out.contains(".mochiflow/state/adapters"), "{out}");
}

#[test]
fn adapter_generate_force_fails_when_target_parent_is_a_file() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args([
            "init",
            "--adapter",
            "copilot",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .write_stdin("")
        .assert()
        .success();
    let github = dir.path().join(".github");
    fs::remove_dir_all(&github).unwrap();
    fs::write(&github, "not a directory\n").unwrap();
    let config = dir.path().join(".mochiflow/config.toml");

    let result = bin()
        .args([
            "--config",
            config.to_str().unwrap(),
            "adapter",
            "generate",
            "--force",
        ])
        .assert()
        .failure();

    assert_eq!(fs::read_to_string(&github).unwrap(), "not a directory\n");
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(out.contains("FAIL:"), "{out}");
    assert!(out.contains(".github/copilot-instructions.md"), "{out}");
    assert!(
        out.contains("Summary: 0 written, 0 blocked, 1 failed"),
        "{out}"
    );
}

/// doctor reports an unknown adapter tool as FAIL (exit != 0).
#[test]
fn doctor_fails_on_unknown_adapter_tool() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();

    let config_path = dir.path().join(".mochiflow/config.toml");
    let original = fs::read_to_string(&config_path).unwrap();
    let edited = original.replace("tools = [\"agents\"]", "tools = [\"bogus\"]");
    fs::write(&config_path, &edited).unwrap();

    bin()
        .args(["--config", config_path.to_str().unwrap(), "doctor"])
        .assert()
        .failure();
}

#[test]
fn config_validate_fails_on_empty_adapter_tools() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();

    let config_path = dir.path().join(".mochiflow/config.toml");
    let original = fs::read_to_string(&config_path).unwrap();
    let edited = original.replace("tools = [\"agents\"]", "tools = []");
    fs::write(&config_path, &edited).unwrap();

    let result = bin()
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "config",
            "validate",
        ])
        .assert()
        .failure();
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(
        out.contains("adapter.tools must contain at least one tool"),
        "{out}"
    );
}

#[test]
fn config_validate_warns_on_legacy_language() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();

    let config_path = dir.path().join(".mochiflow/config.toml");
    let original = fs::read_to_string(&config_path).unwrap();
    let edited = original.replace(
        "[i18n]\nartifact_language = \"en\"\nconversation_language = \"auto\"\n\n",
        "language = \"ja\"\n\n",
    );
    fs::write(&config_path, &edited).unwrap();

    let result = bin()
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "config",
            "validate",
        ])
        .assert()
        .success();
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(out.contains("top-level `language` is deprecated"), "{out}");
    assert!(out.contains("missing `[i18n]`"), "{out}");
    assert!(out.contains("0 fail"), "{out}");
}

#[test]
fn config_validate_fails_on_invalid_i18n() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();

    let config_path = dir.path().join(".mochiflow/config.toml");
    let original = fs::read_to_string(&config_path).unwrap();
    let edited = original
        .replace("artifact_language = \"en\"", "artifact_language = \"auto\"")
        .replace(
            "conversation_language = \"auto\"",
            "conversation_language = \"../ja\"",
        );
    fs::write(&config_path, &edited).unwrap();

    let result = bin()
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "config",
            "validate",
        ])
        .assert()
        .failure();
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(out.contains("artifact_language"), "{out}");
    assert!(out.contains("conversation_language"), "{out}");
}

#[test]
fn doctor_json_outputs_single_document() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();
    let config_path = dir.path().join(".mochiflow/config.toml");

    let result = bin()
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "doctor",
            "--json",
            "config",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["exit_code"].as_i64(), Some(0), "{stdout}");
    assert!(json["checks"]["config"].is_array(), "{stdout}");
    assert!(dir.path().join(".mochiflow/state/doctor.json").exists());
}

#[test]
fn doctor_json_warns_when_state_write_fails() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();
    let config_path = dir.path().join(".mochiflow/config.toml");
    let state = dir.path().join(".mochiflow/state");
    if state.is_dir() {
        fs::remove_dir_all(&state).unwrap();
    }
    fs::write(&state, "not a directory\n").unwrap();

    let result = bin()
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "doctor",
            "--json",
            "config",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(
        json["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| warning
                .as_str()
                .unwrap()
                .contains("could not write state/doctor.json")),
        "{stdout}"
    );
}

#[test]
fn completions_bash_prints_script_without_config() {
    let result = bin().args(["completions", "bash"]).assert().success();
    let stdout = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(stdout.contains("mochiflow"), "{stdout}");
    assert!(stdout.contains("doctor"), "{stdout}");
}

#[test]
fn version_matches_cargo_pkg() {
    let result = bin().arg("--version").assert().success();
    let stdout = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert_eq!(
        stdout.trim(),
        format!("mochiflow {}", env!("CARGO_PKG_VERSION"))
    );
}

/// Unknown doctor targets are rejected by clap instead of silently passing.
#[test]
fn doctor_rejects_unknown_target() {
    bin().args(["doctor", "typo"]).assert().failure().code(2);
}

/// Valid doctor targets remain accepted after target validation was tightened.
#[test]
fn doctor_valid_targets_still_work() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();
    let config_path = dir.path().join(".mochiflow/config.toml");

    for target in ["config", "specs", "adapter", "engine"] {
        bin()
            .args(["--config", config_path.to_str().unwrap(), "doctor", target])
            .assert()
            .success();
    }
}

/// Setup warnings must not suggest a non-existent Rust CLI subcommand.
#[test]
fn doctor_setup_message_points_to_review() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();
    let config_path = dir.path().join(".mochiflow/config.toml");

    let result = bin()
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "doctor",
            "config",
        ])
        .assert()
        .success();
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(!out.contains("mochiflow onboard"), "{out}");
    assert!(
        out.contains("setup needs review") || out.contains("unfilled stub"),
        "{out}"
    );
}

fn init_project_with_context(product: &str, structure: &str, tech: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();
    fs::write(dir.path().join(".mochiflow/context/product.md"), product).unwrap();
    fs::write(
        dir.path().join(".mochiflow/context/structure.md"),
        structure,
    )
    .unwrap();
    fs::write(dir.path().join(".mochiflow/context/tech.md"), tech).unwrap();
    dir
}

#[test]
fn doctor_config_warns_on_stale_context_command_reference() {
    let dir = init_project_with_context(
        "# Product\n\nRun `mochiflow engine` during setup.\n",
        "# Structure\n\nNo command reference.\n",
        "# Tech\n\nNo command reference.\n",
    );
    let config_path = dir.path().join(".mochiflow/config.toml");

    let result = bin()
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "doctor",
            "config",
        ])
        .assert()
        .success();
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(out.contains("context.product"), "{out}");
    assert!(
        out.contains("unknown MochiFlow command `mochiflow engine`"),
        "{out}"
    );
    assert!(out.contains("refresh-context workflow"), "{out}");
}

#[test]
fn doctor_config_allows_current_cli_and_workflow_command_references() {
    let dir = init_project_with_context(
        "# Product\n\nRun `mochiflow doctor` for project health.\n",
        "# Structure\n\nUse `mochiflow discuss` for workflow planning.\n",
        "# Tech\n\nNo command reference.\n",
    );
    let config_path = dir.path().join(".mochiflow/config.toml");

    let result = bin()
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "doctor",
            "config",
        ])
        .assert()
        .success();
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(!out.contains("unknown MochiFlow command"), "{out}");
}

#[test]
fn doctor_config_guides_source_repo_users_to_freeze_check() {
    let config_path = repo_root().join(".mochiflow/config.toml");

    let result = bin()
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "doctor",
            "config",
        ])
        .assert()
        .success();
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(out.contains("source repo detected"), "{out}");
    assert!(out.contains("mochiflow freeze --check"), "{out}");
    assert!(out.contains("doctor` checks project health"), "{out}");
}

/// Build a minimal mochiflow source-repo fixture under `root` and freeze it so
/// `freeze --check` reports it fresh — independent of the committed repository
/// MANIFEST. Mirrors the shape of `freeze.rs`'s private `#[cfg(test)]`
/// `setup_fixture` (which is not importable from this integration-test crate);
/// do not change that helper's visibility to share it.
fn setup_freeze_fixture(root: &Path) {
    // cli/Cargo.toml carries the version SSOT used by freeze.
    let cli_dir = root.join("cli");
    fs::create_dir_all(&cli_dir).unwrap();
    fs::write(
        cli_dir.join("Cargo.toml"),
        "[workspace]\nmembers = []\n[workspace.package]\nversion = \"1.0.0\"\n",
    )
    .unwrap();

    // engine/ with VERSION + a sample file so the manifest is non-empty.
    let engine_dir = root.join("engine");
    fs::create_dir_all(&engine_dir).unwrap();
    fs::write(engine_dir.join("VERSION"), "1.0.0\n").unwrap();
    fs::write(engine_dir.join("router.md"), "# Router\n").unwrap();

    // contracts/*.json + tests/conformance/golden/ feed the frozen-surface hash.
    let contracts_dir = root.join("contracts");
    fs::create_dir_all(&contracts_dir).unwrap();
    fs::write(contracts_dir.join("test.json"), "{}\n").unwrap();
    let golden_dir = root.join("tests/conformance/golden");
    fs::create_dir_all(&golden_dir).unwrap();
    fs::write(golden_dir.join("INDEX.md"), "# Index\n").unwrap();

    // Freeze the fixture so its derived files (engine/VERSION,
    // engine/MANIFEST.json, contracts/contracts.lock) are written fresh.
    bin()
        .args(["freeze", "--root", root.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn freeze_root_check_uses_explicit_source_repo_from_other_cwd() {
    let fixture = tempfile::tempdir().unwrap();
    setup_freeze_fixture(fixture.path());

    // Run from an unrelated cwd so resolution can only come from --root, and
    // assert against the in-test fixture rather than the committed repo MANIFEST.
    let cwd = tempfile::tempdir().unwrap();
    let result = bin()
        .current_dir(cwd.path())
        .args([
            "freeze",
            "--root",
            fixture.path().to_str().unwrap(),
            "--check",
        ])
        .assert()
        .success();
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(
        out.contains("freeze: all derived files are up to date"),
        "{out}"
    );
}

#[test]
fn freeze_root_invalid_path_fails_before_writing() {
    let root = tempfile::tempdir().unwrap();
    let result = bin()
        .args(["freeze", "--root", root.path().to_str().unwrap()])
        .assert()
        .failure()
        .code(1);
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(out.contains("not a mochiflow source repo"), "{out}");
    assert!(!root.path().join("engine/VERSION").exists());
    assert!(!root.path().join("contracts/contracts.lock").exists());
}

#[test]
fn freeze_without_root_keeps_cwd_upward_resolution() {
    let fixture = tempfile::tempdir().unwrap();
    setup_freeze_fixture(fixture.path());

    // From a nested subdirectory of the fixture, with no --root, upward cwd
    // resolution must walk up to the fixture root (not the committed repo).
    let subdir = fixture.path().join("nested/work/dir");
    fs::create_dir_all(&subdir).unwrap();
    let result = bin()
        .current_dir(&subdir)
        .args(["freeze", "--check"])
        .assert()
        .success();
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(
        out.contains("freeze: all derived files are up to date"),
        "{out}"
    );
}

#[test]
fn docs_explain_doctor_freeze_boundaries_and_root_usage() {
    let root = repo_root();
    let docs_versioning = fs::read_to_string(root.join("docs/versioning.md")).unwrap();
    assert!(
        docs_versioning.contains("## Source repo coherence"),
        "{docs_versioning}"
    );
    assert!(
        docs_versioning.contains("It does not replace"),
        "{docs_versioning}"
    );
    assert!(
        docs_versioning.contains("source-repo derived-file"),
        "{docs_versioning}"
    );
    assert!(
        docs_versioning.contains("mochiflow freeze --root <source-repo> --check"),
        "{docs_versioning}"
    );

    let contract_versioning = fs::read_to_string(root.join("contracts/VERSIONING.md")).unwrap();
    assert!(
        contract_versioning.contains("mochiflow freeze --root <source-repo>"),
        "{contract_versioning}"
    );
    assert!(
        contract_versioning.contains("It is separate from `mochiflow doctor`"),
        "{contract_versioning}"
    );
}

/// Engine docs list only Rust CLI commands; onboard is an AI engine command.
#[test]
fn engine_readme_does_not_list_onboard_as_cli_command() {
    let root = repo_root();
    let engine_readme = fs::read_to_string(root.join("engine/README.md")).unwrap();
    assert!(
        !engine_readme.contains("`mochiflow onboard`"),
        "{engine_readme}"
    );
}

#[test]
fn onboard_instructions_do_not_force_adapter_generation_by_default() {
    let text = fs::read_to_string(repo_root().join("engine/commands/onboard.md")).unwrap();
    assert!(
        !text.contains("Run adapter generate --force"),
        "onboard must not force-overwrite adapter files by default:\n{text}"
    );
    assert!(
        text.contains("mochiflow adapter generate` without"),
        "onboard should instruct safe adapter generation first:\n{text}"
    );
}

#[test]
fn engine_manifest_subcommand_is_removed() {
    let dir = tempfile::tempdir().unwrap();
    let engine_dir = dir.path().join("engine");
    copy_dir_all(&repo_root().join("engine"), &engine_dir);

    bin()
        .args([
            "engine",
            "manifest",
            "--engine-dir",
            engine_dir.to_str().unwrap(),
        ])
        .assert()
        .failure();
}

/// Removed flag --minimal is rejected by clap.
#[test]
fn init_rejects_removed_flags() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args([
            "init",
            "--minimal",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .code(2);
    bin()
        .args([
            "init",
            "--language",
            "ja",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .code(2);
}

/// --dry-run previews without writing anything.
#[test]
fn init_dry_run_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let result = bin()
        .args([
            "init",
            "--dry-run",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(out.contains("Detected:"), "{out}");
    assert!(out.contains("Created/Updated:"), "{out}");
    assert!(out.contains("Status:"), "{out}");
    assert!(
        out.contains("paste the setup prompt below into your AI agent"),
        "{out}"
    );
    assert!(out.contains("(dry-run) would"), "{out}");
    assert!(!dir.path().join(".mochiflow/config.toml").exists());
    assert!(!dir.path().join(".mochiflow/.gitignore").exists());
}

#[test]
fn init_dry_run_json_writes_nothing_and_marks_dry_run() {
    let dir = tempfile::tempdir().unwrap();
    let result = bin()
        .args([
            "init",
            "--dry-run",
            "--json",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["dry_run"].as_bool(), Some(true), "{stdout}");
    assert_eq!(json["status"].as_str(), Some("needs_ai_review"), "{stdout}");
    assert_eq!(json["exit_code"].as_i64(), Some(0), "{stdout}");
    assert!(!dir.path().join(".mochiflow/config.toml").exists());
    assert!(!dir.path().join(".mochiflow/.gitignore").exists());
}

/// AC-04: init writes {install_dir}/.gitignore with local/runtime rules, and
/// never touches the project's top-level .gitignore.
#[test]
fn init_writes_install_gitignore() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();
    let gi = dir.path().join(".mochiflow/.gitignore");
    assert!(gi.exists(), "{} should exist", gi.display());
    let body = fs::read_to_string(&gi).unwrap();
    assert!(body.contains("state/"), "got:\n{body}");
    assert!(body.contains("INDEX.md"), "got:\n{body}");
    assert!(body.contains("constitution.local.md"), "got:\n{body}");
    assert!(body.contains("instructions.local/"), "got:\n{body}");
    assert!(!body.contains("instructions/\n"), "got:\n{body}");
    assert!(!body.contains("engine/"), "got:\n{body}");
    assert!(
        !dir.path().join(".gitignore").exists(),
        "top-level .gitignore must not be created"
    );
}

/// AC-12: a state-changing command regenerates the gitignored INDEX.md without
/// tracking or staging it. After `mochiflow index`, INDEX.md exists on disk but
/// git neither tracks it nor reports it (the install .gitignore excludes it).
#[test]
fn index_is_untracked_after_state_changing_command() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(root)
        .status()
        .unwrap();
    bin()
        .args(["init", "--target", root.to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();
    let config = root.join(".mochiflow/config.toml");

    // A state-changing command regenerates the board file.
    bin()
        .args(["--config", config.to_str().unwrap(), "index"])
        .assert()
        .success();
    assert!(
        root.join(".mochiflow/INDEX.md").exists(),
        "index must write INDEX.md"
    );

    // git must not track it (gitignored), and `git add -A` must not stage it.
    std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(root)
        .status()
        .unwrap();
    let tracked = std::process::Command::new("git")
        .args(["ls-files", ".mochiflow/INDEX.md"])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&tracked.stdout).trim().is_empty(),
        "INDEX.md must never be tracked/staged"
    );
    let status = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .output()
        .unwrap();
    let status_out = String::from_utf8_lossy(&status.stdout);
    assert!(
        !status_out.contains("INDEX.md"),
        "INDEX.md must be gitignored (not shown by git status):\n{status_out}"
    );
}

/// AC-05: an existing {install_dir}/.gitignore keeps custom content without
/// --force while guaranteeing the local instruction rule, and is regenerated
/// with --force.
#[test]
fn init_gitignore_preserve_then_force() {
    let dir = tempfile::tempdir().unwrap();
    let mf = dir.path().join(".mochiflow");
    fs::create_dir_all(&mf).unwrap();
    let gi = mf.join(".gitignore");
    fs::write(&gi, "CUSTOM\n").unwrap();

    // without --force: custom content preserved and the local rule appended.
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();
    assert_eq!(
        fs::read_to_string(&gi).unwrap(),
        "CUSTOM\ninstructions.local/\n"
    );

    // with --force: regenerated to managed content
    bin()
        .args(["init", "--force", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();
    let body = fs::read_to_string(&gi).unwrap();
    assert!(body.contains("state/"), "got:\n{body}");
    assert!(body.contains("constitution.local.md"), "got:\n{body}");
    assert!(body.contains("instructions.local/"), "got:\n{body}");
    assert!(!body.contains("instructions/\n"), "got:\n{body}");
    assert!(!body.contains("engine/"), "got:\n{body}");
}

/// A missing config.toml exits 2 for non-init commands.
#[test]
fn config_error_exits_2() {
    bin()
        .args(["--config", "/nonexistent/config.toml", "config", "show"])
        .assert()
        .failure()
        .code(2);
}

/// AC-06: `doctor config` FAILs when {install_dir}/state/ is not gitignored,
/// and stays silent once it is ignored (init writes the rule).
#[test]
fn doctor_fails_when_state_not_gitignored() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(root)
        .status()
        .unwrap();
    bin()
        .args(["init", "--target", root.to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();
    let config = root.join(".mochiflow/config.toml");

    // init wrote {install_dir}/.gitignore (state/) → no WARN, exit 0.
    let ok = bin()
        .args(["--config", config.to_str().unwrap(), "doctor", "config"])
        .assert()
        .success();
    let out = String::from_utf8_lossy(&ok.get_output().stdout).into_owned();
    assert!(
        !out.contains("not gitignored"),
        "state should be ignored after init:\n{out}"
    );

    // Drop the ignore rule → FAIL because ship/pr delivery state must be ignored.
    fs::write(root.join(".mochiflow/.gitignore"), "").unwrap();
    let failed = bin()
        .args(["--config", config.to_str().unwrap(), "doctor", "config"])
        .assert()
        .failure()
        .code(1);
    let out2 = String::from_utf8_lossy(&failed.get_output().stdout).into_owned();
    assert!(out2.contains("FAIL"), "expected state-ignore FAIL:\n{out2}");
    assert!(
        out2.contains("not gitignored"),
        "expected state-ignore FAIL:\n{out2}"
    );
}

/// Done specs completed on the same day are ordered by completion time
/// (latest first), not by slug. `zzz` completed after `aaa` on the same day,
/// so it must appear above `aaa` even though it sorts later alphabetically.
/// A third spec without `completed` falls back to its `updated` date.
#[test]
fn index_orders_same_day_done_by_completion_time() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args([
            "init",
            "--artifact-language",
            "en",
            "--target",
            dir.path().to_str().unwrap(),
        ])
        .write_stdin("")
        .assert()
        .success();

    let done = dir.path().join(".mochiflow/specs/_done");
    let write_done = |slug: &str, completed: Option<&str>, updated: &str| {
        let d = done.join(slug);
        fs::create_dir_all(&d).unwrap();
        let mut yaml = format!(
            "version: 1\nslug: {slug}\ntitle: {slug} title\ntype: feature\nsurfaces:\n  - cli\nintegration: none\nrisk: standard\nstatus: done\nupdated: {updated}\n"
        );
        if let Some(c) = completed {
            yaml.push_str(&format!("completed: {c}\n"));
        }
        fs::write(d.join("spec.yaml"), yaml).unwrap();
        fs::write(d.join("spec.md"), format!("# {slug} title\n")).unwrap();
    };
    // Same day 2026-03-15: aaa completed early, zzz completed late.
    write_done("aaa-early", Some("2026-03-15T08:00:00Z"), "2026-03-15");
    write_done("zzz-late", Some("2026-03-15T20:00:00Z"), "2026-03-15");
    // Legacy spec without completed (same day) → falls back to updated date.
    write_done("mmm-legacy", None, "2026-03-15");

    let config = dir.path().join(".mochiflow/config.toml");
    bin()
        .args(["--config", config.to_str().unwrap(), "index"])
        .assert()
        .success();

    let index = fs::read_to_string(dir.path().join(".mochiflow/INDEX.md")).unwrap();
    let pos = |slug: &str| {
        index
            .find(slug)
            .unwrap_or_else(|| panic!("{slug} missing:\n{index}"))
    };
    assert!(
        pos("zzz-late") < pos("aaa-early"),
        "later completion must lead the day:\n{index}"
    );
    // All three grouped under the same month heading.
    assert_eq!(count_occurrences(&index, "### 2026-03"), 1, "{index}");
    // Display column shows the date, not the full timestamp.
    assert!(index.contains("| 2026-03-15 |"), "{index}");
    assert!(!index.contains("2026-03-15T20:00:00Z"), "{index}");

    // index --check is deterministic (clean immediately after generation).
    bin()
        .args(["--config", config.to_str().unwrap(), "index", "--check"])
        .assert()
        .success();
}

/// Write a minimal config (directory-rooted adr) and seed decision records.
fn setup_adr_project(dir: &Path) -> PathBuf {
    let install = dir.join(".mochiflow");
    let decisions = install.join("adr/decisions");
    fs::create_dir_all(&decisions).unwrap();
    fs::create_dir_all(install.join("adr/pitfalls")).unwrap();
    fs::write(
        install.join("config.toml"),
        "schema_version = 1\ninstall_dir = \".mochiflow\"\nspecs_dir = \".mochiflow/specs\"\nindex = \".mochiflow/INDEX.md\"\n\n[constitution]\nproject = \".mochiflow/constitution.md\"\nlocal = \".mochiflow/constitution.local.md\"\n\n[context]\nproduct = \".mochiflow/context/product.md\"\nstructure = \".mochiflow/context/structure.md\"\ntech = \".mochiflow/context/tech.md\"\n\n[adr]\ndecisions = \".mochiflow/adr/decisions\"\npitfalls = \".mochiflow/adr/pitfalls\"\n\n[surfaces.cli]\ndescription = \"cli\"\n\n[surfaces.cli.verify]\ndefault = \"echo ok\"\n",
    )
    .unwrap();
    // Active record (cli area).
    fs::write(
        decisions.join("2026-06-20-active-one.md"),
        "---\nid: 2026-06-20-active-one\ndate: 2026-06-20\narea: [cli]\nspec: feature-x\nstatus: active\nsupersedes: 2026-06-01-old-one\n---\n## 2026-06-20 — Active one keyword-alpha\n",
    )
    .unwrap();
    // Superseded record (excluded from default active set).
    fs::write(
        decisions.join("2026-06-01-old-one.md"),
        "---\nid: 2026-06-01-old-one\ndate: 2026-06-01\narea: [cli]\nstatus: superseded\nsuperseded_by: 2026-06-20-active-one\n---\n## 2026-06-01 — Old one keyword-beta\n",
    )
    .unwrap();
    install.join("config.toml")
}

#[test]
fn adr_list_returns_only_active_headers_by_default() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = setup_adr_project(dir.path());
    let out = bin()
        .args(["--config", cfg.to_str().unwrap(), "adr", "list"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(stdout.contains("2026-06-20-active-one"), "{stdout}");
    assert!(
        !stdout.contains("2026-06-01-old-one"),
        "superseded record must be excluded by default:\n{stdout}"
    );
    // Header only — no body text.
    assert!(!stdout.contains("**Decision"), "{stdout}");
}

#[test]
fn adr_list_status_all_includes_superseded() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = setup_adr_project(dir.path());
    let out = bin()
        .args([
            "--config",
            cfg.to_str().unwrap(),
            "adr",
            "list",
            "--status",
            "all",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(stdout.contains("2026-06-01-old-one"), "{stdout}");
    assert!(stdout.contains("2026-06-20-active-one"), "{stdout}");
}

#[test]
fn adr_search_defaults_to_active_set_and_widens_with_status_all() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = setup_adr_project(dir.path());
    // keyword-beta lives only in the superseded record's body.
    let default_out = bin()
        .args([
            "--config",
            cfg.to_str().unwrap(),
            "adr",
            "search",
            "keyword-beta",
        ])
        .assert()
        .success();
    let default_stdout = String::from_utf8_lossy(&default_out.get_output().stdout).into_owned();
    assert!(
        !default_stdout.contains("2026-06-01-old-one"),
        "default search must stay bounded to the active set:\n{default_stdout}"
    );

    let all_out = bin()
        .args([
            "--config",
            cfg.to_str().unwrap(),
            "adr",
            "search",
            "keyword-beta",
            "--status",
            "all",
        ])
        .assert()
        .success();
    let all_stdout = String::from_utf8_lossy(&all_out.get_output().stdout).into_owned();
    assert!(
        all_stdout.contains("2026-06-01-old-one"),
        "--status all must scan superseded bodies:\n{all_stdout}"
    );
}

#[test]
fn adr_show_returns_body_and_lineage() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = setup_adr_project(dir.path());
    let out = bin()
        .args([
            "--config",
            cfg.to_str().unwrap(),
            "adr",
            "show",
            "2026-06-20-active-one",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(
        stdout.contains("supersedes: 2026-06-01-old-one"),
        "{stdout}"
    );
    assert!(stdout.contains("## 2026-06-20 — Active one"), "{stdout}");
}

#[test]
fn adr_show_unknown_id_fails() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = setup_adr_project(dir.path());
    bin()
        .args([
            "--config",
            cfg.to_str().unwrap(),
            "adr",
            "show",
            "2099-01-01-nope",
        ])
        .assert()
        .failure();
}

#[test]
fn adr_filters_by_area_and_spec() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = setup_adr_project(dir.path());
    // No record has area=ui -> empty.
    let out = bin()
        .args([
            "--config",
            cfg.to_str().unwrap(),
            "adr",
            "list",
            "--area",
            "ui",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(stdout.contains("(no matching records)"), "{stdout}");

    let spec_out = bin()
        .args([
            "--config",
            cfg.to_str().unwrap(),
            "adr",
            "list",
            "--spec",
            "feature-x",
        ])
        .assert()
        .success();
    let spec_stdout = String::from_utf8_lossy(&spec_out.get_output().stdout).into_owned();
    assert!(
        spec_stdout.contains("2026-06-20-active-one"),
        "{spec_stdout}"
    );
}
