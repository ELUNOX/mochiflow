//! CLI integration tests for the deterministic init and config-error behavior.
//!
//! Broader Rust-native conformance for read/query subcommands lives in
//! `tests/conformance.rs`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};

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

/// Deterministic init: no flags, piped stdin (non-TTY) → exit 0, scaffolds
/// config from machine detection. A bare temp dir detects nothing concrete, so
/// the verify command stays a TODO sentinel and confirm markers are attached
/// (detection-zero = low confidence), engine/VERSION exists.
#[test]
fn init_scaffolds_deterministically() {
    let dir = tempfile::tempdir().unwrap();
    bin()
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
    assert!(cfg.contains("language = \"en\""), "got:\n{cfg}");
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(out.contains("language: en (locale)"), "{out}");
    assert!(out.contains("Status:"), "{out}");
    assert!(out.contains("Needs AI review"), "{out}");
    assert!(out.contains("Paste this into your AI agent"), "{out}");
}

#[test]
fn init_yes_uses_japanese_locale_without_prompting() {
    let dir = tempfile::tempdir().unwrap();
    let result = bin()
        .args(["init", "--yes", "--target", dir.path().to_str().unwrap()])
        .env("LANG", "ja_JP.UTF-8")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .assert()
        .success();

    let cfg = fs::read_to_string(dir.path().join(".mochiflow/config.toml")).unwrap();
    assert!(cfg.contains("language = \"ja\""), "got:\n{cfg}");
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(out.contains("language: ja (locale)"), "{out}");
    assert!(out.contains("Needs AI review"), "{out}");
    assert!(
        out.contains("AI アシスタントにこの文を貼ってください"),
        "{out}"
    );
}

#[test]
fn init_reports_ready_when_existing_config_and_context_are_complete() {
    let dir = tempfile::tempdir().unwrap();
    let install = dir.path().join(".mochiflow");
    fs::create_dir_all(install.join("context")).unwrap();
    fs::write(install.join("context/product.md"), "# Product\n\nP.\n").unwrap();
    fs::write(install.join("context/structure.md"), "# Structure\n\nS.\n").unwrap();
    fs::write(install.join("context/tech.md"), "# Tech\n\nT.\n").unwrap();
    fs::write(
        install.join("config.toml"),
        "schema_version = 1\n\
         language = \"en\"\n\
         install_dir = \".mochiflow\"\n\
         specs_dir = \".mochiflow/specs\"\n\
         index = \".mochiflow/INDEX.md\"\n\n\
         [constitution]\nproject = \".mochiflow/constitution.md\"\nlocal = \".mochiflow/constitution.local.md\"\n\n\
         [context]\nproduct = \".mochiflow/context/product.md\"\nstructure = \".mochiflow/context/structure.md\"\ntech = \".mochiflow/context/tech.md\"\n\n\
         [adr]\ndecisions = \".mochiflow/adr/decisions.md\"\npitfalls = \".mochiflow/adr/pitfalls.md\"\n\n\
         [adapter]\ntools = [\"agents\"]\n\n\
         [write]\nallow = [\"src/**\"]\ndeny = [\".git/**\"]\n\n\
         [surfaces.app]\ndescription = \"primary surface\"\n\n\
         [surfaces.app.verify]\ndefault = \"cargo test\"\n",
    )
    .unwrap();

    let result = bin()
        .args(["init", "--yes", "--target", dir.path().to_str().unwrap()])
        .env("LANG", "ja_JP.UTF-8")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .assert()
        .success();
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(out.contains("language: en (existing_config)"), "{out}");
    assert!(out.contains("Ready"), "{out}");
    assert!(
        out.contains("Tell your AI agent what you want to build"),
        "{out}"
    );
    assert!(!out.contains("Paste this into your AI agent"), "{out}");
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
    let edited = original.replace("language = \"en\"", "language = \"ja\"");
    fs::write(&config_path, &edited).unwrap();

    // Second run without --force
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();

    // Config should be preserved (not overwritten)
    let after = fs::read_to_string(&config_path).unwrap();
    assert!(
        after.contains("language = \"ja\""),
        "hand-edited value should be preserved, got:\n{after}"
    );
}

/// --adapter resolves the `codex` label to the `agents` ID, and --language is
/// written to config.
#[test]
fn init_respects_adapter_language_overrides() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args([
            "init",
            "--adapter",
            "codex",
            "--language",
            "ja",
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
    assert!(cfg.contains("language = \"ja\""), "got:\n{cfg}");
    let result = bin()
        .args([
            "init",
            "--adapter",
            "codex",
            "--language",
            "ja",
            "--target",
            dir.path().to_str().unwrap(),
            "--force",
        ])
        .write_stdin("")
        .assert()
        .success();
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(out.contains("language: ja (flag)"), "{out}");
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
    assert!(
        cfg.contains("tools = [\"kiro\", \"claude-code\"]"),
        "got:\n{cfg}"
    );
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

/// Existing adapter targets are never overwritten without --force. A rendered
/// candidate is written under state/ so the user can merge it manually.
#[test]
fn init_existing_adapter_target_writes_candidate_and_needs_confirmation() {
    let dir = tempfile::tempdir().unwrap();
    let agents = dir.path().join("AGENTS.md");
    fs::write(&agents, "CUSTOM AGENTS\n").unwrap();

    let result = bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .failure()
        .code(1);
    assert_eq!(fs::read_to_string(&agents).unwrap(), "CUSTOM AGENTS\n");

    let candidate = dir.path().join(".mochiflow/state/adapters/AGENTS.md");
    let candidate_body = fs::read_to_string(&candidate).unwrap();
    assert!(
        candidate_body.contains("generated by mochiflow"),
        "{candidate_body}"
    );
    assert!(
        candidate_body.contains("Load the router"),
        "{candidate_body}"
    );

    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(out.contains("Status:"), "{out}");
    assert!(out.contains("Blocked"), "{out}");
    assert!(out.contains("Needs review:"), "{out}");
    assert!(
        out.contains("AGENTS.md already exists and was not overwritten"),
        "{out}"
    );
    assert!(out.contains(".mochiflow/state/adapters/AGENTS.md"), "{out}");
    assert!(!out.contains("generated adapters: agents"), "{out}");
}

#[test]
fn init_blocked_json_exits_1_and_includes_candidate() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("AGENTS.md"), "CUSTOM AGENTS\n").unwrap();

    let result = bin()
        .args(["init", "--json", "--target", dir.path().to_str().unwrap()])
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
        Some(".mochiflow/state/adapters/AGENTS.md"),
        "{stdout}"
    );
}

/// Blocked adapter guidance follows the configured language.
#[test]
fn init_existing_adapter_target_guidance_is_language_aware() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("AGENTS.md"), "CUSTOM AGENTS\n").unwrap();

    let result = bin()
        .args([
            "init",
            "--language",
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
        out.contains("AGENTS.md は既に存在するため上書きしませんでした"),
        "{out}"
    );
    assert!(out.contains(".mochiflow/state/adapters/AGENTS.md"), "{out}");
}

/// The blocked/candidate behavior applies to every adapter target, including
/// nested Kiro paths.
#[test]
fn init_existing_nested_adapter_target_writes_nested_candidate() {
    let dir = tempfile::tempdir().unwrap();
    let existing = dir.path().join(".kiro/agents/spec-builder.json");
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
        .join(".mochiflow/state/adapters/.kiro/agents/spec-builder.json");
    let candidate_body = fs::read_to_string(&candidate).unwrap();
    assert!(
        candidate_body.contains("generated by mochiflow"),
        "{candidate_body}"
    );
    assert!(candidate_body.contains("spec-builder"), "{candidate_body}");
}

/// Blocking is driven by manifest targets, not by one special adapter file.
#[test]
fn init_existing_adapter_targets_write_candidates_for_all_manifest_paths() {
    let dir = tempfile::tempdir().unwrap();
    for rel in [
        "AGENTS.md",
        "CLAUDE.md",
        ".github/copilot-instructions.md",
        ".kiro/agents/spec-builder.json",
    ] {
        let target = dir.path().join(rel);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(target, format!("CUSTOM {rel}\n")).unwrap();
    }

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

    for rel in [
        "AGENTS.md",
        "CLAUDE.md",
        ".github/copilot-instructions.md",
        ".kiro/agents/spec-builder.json",
    ] {
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

/// Normal adapter generation reports the candidate path when manual merge is required.
#[test]
fn adapter_generate_blocked_output_includes_candidate_path() {
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
        .failure();

    assert_eq!(
        fs::read_to_string(dir.path().join("AGENTS.md")).unwrap(),
        "CUSTOM AGENTS\n"
    );
    let candidate = dir.path().join(".mochiflow/state/adapters/AGENTS.md");
    assert!(candidate.exists());

    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(out.contains("BLOCKED: AGENTS.md"), "{out}");
    assert!(
        out.contains("candidate: .mochiflow/state/adapters/AGENTS.md"),
        "{out}"
    );
    assert!(
        out.contains("merge manually or use --force to replace"),
        "{out}"
    );
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
    let builder = dir.path().join(".kiro/agents/spec-builder.json");
    let reviewer = dir
        .path()
        .join(".kiro/agents/spec-independent-reviewer.json");

    let generated = fs::read_to_string(&builder).unwrap();
    assert!(generated.contains("\"model\": \"auto\""), "{generated}");

    set_json_field(
        &builder,
        "model",
        serde_json::Value::String("custom-builder-model".into()),
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

    let builder_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&builder).unwrap()).unwrap();
    let reviewer_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&reviewer).unwrap()).unwrap();
    assert_eq!(builder_json["model"].as_str(), Some("custom-builder-model"));
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
    let builder = dir.path().join(".kiro/agents/spec-builder.json");
    set_json_field(&builder, "custom", serde_json::Value::Bool(true));

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
        out.contains("DRIFT: .kiro/agents/spec-builder.json"),
        "{out}"
    );
}

#[test]
fn adapter_generate_fails_when_candidate_parent_cannot_be_created() {
    let dir = tempfile::tempdir().unwrap();
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();
    fs::write(dir.path().join("AGENTS.md"), "CUSTOM AGENTS\n").unwrap();
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
        fs::read_to_string(dir.path().join("AGENTS.md")).unwrap(),
        "CUSTOM AGENTS\n"
    );
    let out = String::from_utf8_lossy(&result.get_output().stdout).into_owned();
    assert!(out.contains("FAIL:"), "{out}");
    assert!(out.contains(".mochiflow/state/adapters"), "{out}");
    assert!(
        out.contains("Summary: 0 written, 0 blocked, 1 failed"),
        "{out}"
    );
}

#[test]
fn adapter_generate_errors_make_init_fail_when_candidate_parent_cannot_be_created() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("AGENTS.md"), "CUSTOM AGENTS\n").unwrap();
    let install = dir.path().join(".mochiflow");
    fs::create_dir_all(&install).unwrap();
    fs::write(install.join("state"), "not a directory\n").unwrap();

    let result = bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .failure();

    assert_eq!(
        fs::read_to_string(dir.path().join("AGENTS.md")).unwrap(),
        "CUSTOM AGENTS\n"
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

/// Public docs list only Rust CLI commands; onboard is an AI engine command.
#[test]
fn readmes_do_not_list_onboard_as_cli_command() {
    let root = repo_root();
    for rel in ["README.md", "README.ja.md"] {
        let text = fs::read_to_string(root.join(rel)).unwrap();
        assert!(!text.contains("init | onboard |"), "{rel}:\n{text}");
    }

    let engine_readme = fs::read_to_string(root.join("engine/README.md")).unwrap();
    assert!(
        !engine_readme.contains("`mochiflow onboard`"),
        "{engine_readme}"
    );
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
    assert!(out.contains("Needs AI review"), "{out}");
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

/// AC-04: init writes {install_dir}/.gitignore with engine/ and state/, and
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
    assert!(body.contains("engine/"), "got:\n{body}");
    assert!(body.contains("state/"), "got:\n{body}");
    assert!(
        !dir.path().join(".gitignore").exists(),
        "top-level .gitignore must not be created"
    );
}

/// AC-05: an existing {install_dir}/.gitignore is preserved without --force and
/// regenerated with --force.
#[test]
fn init_gitignore_preserve_then_force() {
    let dir = tempfile::tempdir().unwrap();
    let mf = dir.path().join(".mochiflow");
    fs::create_dir_all(&mf).unwrap();
    let gi = mf.join(".gitignore");
    fs::write(&gi, "CUSTOM\n").unwrap();

    // without --force: preserved
    bin()
        .args(["init", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();
    assert_eq!(fs::read_to_string(&gi).unwrap(), "CUSTOM\n");

    // with --force: regenerated to managed content
    bin()
        .args(["init", "--force", "--target", dir.path().to_str().unwrap()])
        .write_stdin("")
        .assert()
        .success();
    let body = fs::read_to_string(&gi).unwrap();
    assert!(
        body.contains("engine/") && body.contains("state/"),
        "got:\n{body}"
    );
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

/// AC-06: `doctor config` WARNs (exit 0) when {install_dir}/state/ is not
/// gitignored, and stays silent once it is ignored (init writes the rule).
#[test]
fn doctor_warns_when_state_not_gitignored() {
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

    // Drop the ignore rule → WARN, still exit 0 (WARN is not a failure).
    fs::write(root.join(".mochiflow/.gitignore"), "").unwrap();
    let warned = bin()
        .args(["--config", config.to_str().unwrap(), "doctor", "config"])
        .assert()
        .success();
    let out2 = String::from_utf8_lossy(&warned.get_output().stdout).into_owned();
    assert!(
        out2.contains("not gitignored"),
        "expected state-ignore WARN:\n{out2}"
    );
}
