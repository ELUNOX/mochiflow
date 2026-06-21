//! mochiflow conformance suite (Rust-native).
//!
//! Replaces the former Python conformance runner. Validates that the
//! implementation conforms to the frozen contracts:
//!   - schema: contracts/*.schema.json accept positive / reject negative fixtures
//!   - golden: `index` output matches tests/conformance/golden (timestamp-normalized)
//!   - drift: `doctor engine` passes clean and detects MANIFEST drift
//!   - version-gate: contracts.lock hash matches frozen surfaces (or VERSION bumped)
//!   - behavioral: lint/doctor/config logic pinned by property assertions
//!
//! The committed golden fixtures and JSON schemas are the source of truth; there
//! is no second (Python) implementation to compare against.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};

/// Repo root, derived from this crate's manifest dir (cli/crates/mochiflow-cli).
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("repo root is three levels above the crate manifest dir")
        .to_path_buf()
}

fn contracts_dir() -> PathBuf {
    repo_root().join("contracts")
}

fn schema_fixtures_dir() -> PathBuf {
    repo_root().join("tests/conformance/fixtures/schema")
}

fn read_json(path: &Path) -> serde_json::Value {
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn read_repo_file(path: &str) -> String {
    let path = repo_root().join(path);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display())) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, out);
        } else if path.is_file() {
            out.push(path);
        }
    }
}

fn load_schema(name: &str) -> jsonschema::Validator {
    let schema = read_json(&contracts_dir().join(name));
    jsonschema::validator_for(&schema).unwrap_or_else(|e| panic!("compile schema {name}: {e}"))
}

fn load_fixture(name: &str) -> serde_json::Value {
    read_json(&schema_fixtures_dir().join(name))
}

// --- (a) Schema validation: positive accepted, negative rejected --------------

#[test]
fn schema_manifest_accepts_real_manifest() {
    let validator = load_schema("manifest.schema.json");
    let manifest = read_json(&repo_root().join("engine/MANIFEST.json"));
    assert!(
        validator.is_valid(&manifest),
        "engine/MANIFEST.json must be accepted by manifest.schema.json"
    );
}

#[test]
fn schema_spec_accepts_good() {
    let v = load_schema("spec.schema.json");
    assert!(v.is_valid(&load_fixture("spec_good.json")));
}

#[test]
fn schema_spec_rejects_bad_type() {
    let v = load_schema("spec.schema.json");
    assert!(
        !v.is_valid(&load_fixture("spec_bad_type.json")),
        "spec.yaml with an invalid `type` must be rejected"
    );
}

#[test]
fn schema_spec_rejects_missing_required() {
    let v = load_schema("spec.schema.json");
    assert!(
        !v.is_valid(&load_fixture("spec_missing_required.json")),
        "spec.yaml missing required fields must be rejected"
    );
}

#[test]
fn schema_config_accepts_good() {
    let v = load_schema("config.schema.json");
    assert!(v.is_valid(&load_fixture("config_good.json")));
}

#[test]
fn schema_config_i18n_rules() {
    let v = load_schema("config.schema.json");
    let mut good = load_fixture("config_good.json");

    good["i18n"]["artifact_language"] = serde_json::Value::String("pt-BR".to_string());
    good["i18n"]["conversation_language"] = serde_json::Value::String("auto".to_string());
    assert!(v.is_valid(&good));

    let mut artifact_auto = good.clone();
    artifact_auto["i18n"]["artifact_language"] = serde_json::Value::String("auto".to_string());
    assert!(!v.is_valid(&artifact_auto));

    let mut bad_conversation = good.clone();
    bad_conversation["i18n"]["conversation_language"] =
        serde_json::Value::String("../ja".to_string());
    assert!(!v.is_valid(&bad_conversation));

    let mut partial = good;
    partial["i18n"]
        .as_object_mut()
        .unwrap()
        .remove("conversation_language");
    assert!(!v.is_valid(&partial));
}

#[test]
fn schema_config_rejects_bad_schema_version() {
    let v = load_schema("config.schema.json");
    assert!(
        !v.is_valid(&load_fixture("config_bad_schema_version.json")),
        "config.toml with an out-of-range schema_version must be rejected"
    );
}

#[test]
fn schema_config_accepts_adapter_tools_array() {
    let v = load_schema("config.schema.json");
    assert!(v.is_valid(&load_fixture("config_adapter_tools.json")));
}

#[test]
fn schema_config_rejects_adapter_without_tool_or_tools() {
    let v = load_schema("config.schema.json");
    assert!(
        !v.is_valid(&load_fixture("config_adapter_empty.json")),
        "config.toml [adapter] with neither tool nor tools must be rejected"
    );
}

#[test]
fn schema_pr_request_accepts_good() {
    let v = load_schema("pr-request.schema.json");
    assert!(v.is_valid(&load_fixture("pr_good.json")));
}

#[test]
fn schema_pr_request_rejects_missing_head() {
    let v = load_schema("pr-request.schema.json");
    assert!(
        !v.is_valid(&load_fixture("pr_missing_head.json")),
        "pr-request missing required `head` must be rejected"
    );
}

// --- (b) Golden equivalence: `index` output == committed golden ---------------

/// Recursively copy a directory tree (fixtures have no symlinks).
fn copy_dir_all(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir_all(&from, &to);
        } else {
            std::fs::copy(&from, &to).unwrap();
        }
    }
}

/// Normalize the volatile `> updated: ...` timestamp line to a fixed placeholder,
/// matching the golden fixture (Python's TIMESTAMP_RE replacement, expressed in
/// Rust without a regex dependency).
fn normalize_timestamp(text: &str) -> String {
    let mut out: String = text
        .lines()
        .map(|line| {
            if line.starts_with("> updated: ") {
                "> updated: {{TIMESTAMP}}".to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    if text.ends_with('\n') {
        out.push('\n');
    }
    out
}

#[test]
fn golden_index_matches() {
    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path().join("sample-project");
    copy_dir_all(
        &repo_root().join("tests/conformance/fixtures/sample-project"),
        &project,
    );
    let config = project.join(".mochiflow/config.toml");

    assert_cmd::Command::cargo_bin("mochiflow")
        .unwrap()
        .args(["--config", config.to_str().unwrap(), "index"])
        .assert()
        .success();

    let index_path = project.join(".mochiflow/INDEX.md");
    let actual = normalize_timestamp(&std::fs::read_to_string(&index_path).unwrap());
    let expected =
        std::fs::read_to_string(repo_root().join("tests/conformance/golden/INDEX.md")).unwrap();

    assert_eq!(
        actual, expected,
        "`index` output must match tests/conformance/golden/INDEX.md (timestamp-normalized)"
    );
}

#[test]
fn index_check_passes_clean_and_fails_stale() {
    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path().join("sample-project");
    copy_dir_all(
        &repo_root().join("tests/conformance/fixtures/sample-project"),
        &project,
    );
    let config = project.join(".mochiflow/config.toml");

    assert_cmd::Command::cargo_bin("mochiflow")
        .unwrap()
        .args(["--config", config.to_str().unwrap(), "index"])
        .assert()
        .success();
    assert_cmd::Command::cargo_bin("mochiflow")
        .unwrap()
        .args(["--config", config.to_str().unwrap(), "index", "--check"])
        .assert()
        .success();

    std::fs::write(project.join(".mochiflow/INDEX.md"), "# stale\n").unwrap();
    let out = assert_cmd::Command::cargo_bin("mochiflow")
        .unwrap()
        .args(["--config", config.to_str().unwrap(), "index", "--check"])
        .assert()
        .failure();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(stdout.contains("INDEX.md is stale"), "{stdout}");
}

// --- (c) MANIFEST drift detection --------------------------------------------

/// Minimal config that loads and points engine_dir at <root>/.mochiflow/engine.
fn write_drift_config(install: &Path) {
    std::fs::write(
        install.join("config.toml"),
        "schema_version = 1\n\
         install_dir = \".mochiflow\"\n\
         specs_dir = \".mochiflow/specs\"\n\
         index = \".mochiflow/INDEX.md\"\n\n\
         [constitution]\n\
         project = \".mochiflow/constitution.md\"\n\
         local = \".mochiflow/constitution.local.md\"\n\n\
         [context]\n\
         product = \".mochiflow/context/product.md\"\n\
         structure = \".mochiflow/context/structure.md\"\n\
         tech = \".mochiflow/context/tech.md\"\n\n\
         [adr]\n\
         decisions = \".mochiflow/adr/decisions.md\"\n\
         pitfalls = \".mochiflow/adr/pitfalls.md\"\n",
    )
    .unwrap();
}

#[test]
fn drift_doctor_passes_clean_then_detects_edit() {
    let tmp = tempfile::tempdir().unwrap();
    let install = tmp.path().join(".mochiflow");
    std::fs::create_dir_all(&install).unwrap();
    copy_dir_all(&repo_root().join("engine"), &install.join("engine"));
    write_drift_config(&install);
    let config = install.join("config.toml");

    // Clean materialization: doctor engine passes (real MANIFEST matches files).
    assert_cmd::Command::cargo_bin("mochiflow")
        .unwrap()
        .args(["--config", config.to_str().unwrap(), "doctor", "engine"])
        .assert()
        .success();

    // Mutate an engine file → doctor must fail and report MANIFEST drift.
    std::fs::write(install.join("engine/VERSION"), "99.99.99\n").unwrap();
    let out = assert_cmd::Command::cargo_bin("mochiflow")
        .unwrap()
        .args(["--config", config.to_str().unwrap(), "doctor", "engine"])
        .assert()
        .failure();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(
        stdout.contains("MANIFEST drift"),
        "doctor engine must report MANIFEST drift after an engine edit; got:\n{stdout}"
    );
}

#[test]
fn doctor_engine_detects_manifest_version_mismatch() {
    let tmp = tempfile::tempdir().unwrap();
    let install = tmp.path().join(".mochiflow");
    std::fs::create_dir_all(&install).unwrap();
    copy_dir_all(&repo_root().join("engine"), &install.join("engine"));
    write_drift_config(&install);
    let config = install.join("config.toml");

    let manifest_path = install.join("engine/MANIFEST.json");
    let mut manifest = read_json(&manifest_path);
    manifest["version"] = serde_json::Value::String("0.0.0".to_string());
    std::fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).unwrap() + "\n",
    )
    .unwrap();

    let out = assert_cmd::Command::cargo_bin("mochiflow")
        .unwrap()
        .args(["--config", config.to_str().unwrap(), "doctor", "engine"])
        .assert()
        .failure();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(
        stdout.contains("MANIFEST version mismatch"),
        "doctor engine must report MANIFEST/VERSION mismatch; got:\n{stdout}"
    );
}

#[test]
fn doctor_engine_warns_when_dogfood_source_engine_differs() {
    let tmp = tempfile::tempdir().unwrap();
    let install = tmp.path().join(".mochiflow");
    std::fs::create_dir_all(&install).unwrap();
    copy_dir_all(&repo_root().join("engine"), &install.join("engine"));
    write_drift_config(&install);
    let source_engine = tmp.path().join("engine");
    std::fs::create_dir_all(&source_engine).unwrap();
    std::fs::write(source_engine.join("VERSION"), "99.99.99\n").unwrap();

    let out = assert_cmd::Command::cargo_bin("mochiflow")
        .unwrap()
        .args([
            "--config",
            install.join("config.toml").to_str().unwrap(),
            "doctor",
            "engine",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(
        stdout.contains("source engine is 99.99.99") && stdout.contains("installed engine is"),
        "dogfood source/vendored mismatch should warn; got:\n{stdout}"
    );
}

// --- (d) Version gate: frozen-surface hash vs contracts.lock ------------------

/// Compute the frozen-surface hash: sha256 over sorted `contracts/*.json` bytes,
/// then sorted `golden/**` file bytes.
///
/// The frozen surface is intentionally limited to the consumer-facing contracts
/// — JSON schemas and golden output. The `engine/MANIFEST.json` files map
/// (every engine file's content hash) is deliberately NOT part of this hash:
/// engine prose edits are not a compatibility change, and including them forced
/// a lock regeneration on every doc tweak. Engine-file integrity is covered
/// separately by `doctor engine` MANIFEST drift detection, which is an integrity
/// check, not a versioning gate.
fn compute_contracts_hash() -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();

    let contracts = contracts_dir();
    let mut schema_files: Vec<PathBuf> = std::fs::read_dir(&contracts)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json") && p.is_file())
        .collect();
    schema_files.sort();
    for f in &schema_files {
        hasher.update(std::fs::read(f).unwrap());
    }

    let golden = repo_root().join("tests/conformance/golden");
    let mut golden_files: Vec<PathBuf> = Vec::new();
    fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                collect(&p, out);
            } else if p.is_file() {
                out.push(p);
            }
        }
    }
    collect(&golden, &mut golden_files);
    golden_files.sort();
    for f in &golden_files {
        hasher.update(std::fs::read(f).unwrap());
    }

    format!("{:x}", hasher.finalize())
}

#[test]
fn version_gate_consistent() {
    let lock = read_json(&contracts_dir().join("contracts.lock"));
    let lock_hash = lock["hash"].as_str().unwrap();
    let lock_version = lock["version"].as_str().unwrap();
    let current = compute_contracts_hash();
    let engine_version = std::fs::read_to_string(repo_root().join("engine/VERSION"))
        .unwrap()
        .trim()
        .to_string();

    // Gate (parity with the former Python check): consistent when the hash
    // matches the lock, or when a frozen surface changed AND VERSION was bumped.
    let pass = current == lock_hash || engine_version != lock_version;
    assert!(
        pass,
        "frozen contract surface changed (lock {lock_hash:.12}… → {current:.12}…) \
         but engine/VERSION ({engine_version}) was not bumped past lock.version ({lock_version})"
    );
}

#[test]
fn version_gate_hash_matches_committed_lock() {
    // Byte-parity guard: the computed frozen-surface hash must equal the
    // committed contracts.lock hash. The frozen surface is schemas + golden only
    // (see compute_contracts_hash); regenerate the lock when either changes.
    let lock = read_json(&contracts_dir().join("contracts.lock"));
    assert_eq!(
        compute_contracts_hash(),
        lock["hash"].as_str().unwrap(),
        "computed frozen-surface hash must equal contracts.lock; regenerate the lock if a frozen surface changed"
    );
}

// --- (e) Engine prose/template drift guards ----------------------------------

#[test]
fn router_merged_event_is_cleanup_only() {
    let router = read_repo_file("engine/router.md");
    let merged_line = router
        .lines()
        .find(|line| line.contains("Event patterns `{slug} merged`"))
        .expect("router merged-event routing line exists");

    assert!(
        merged_line.contains("post-merge local cleanup only"),
        "merged-event routing must resume cleanup only; got: {merged_line}"
    );
    assert!(
        merged_line.contains("{specs_dir}/_done/{slug}/"),
        "merged-event routing must resolve archived specs first; got: {merged_line}"
    );
    assert!(
        merged_line.contains("Fold/archive already happened"),
        "merged-event routing must state fold/archive already happened before PR; got: {merged_line}"
    );
    assert!(
        !merged_line.contains("fold → archive") && !merged_line.contains("fold -> archive"),
        "merged-event routing must not instruct fold/archive after merge; got: {merged_line}"
    );
}

#[test]
fn branch_placeholders_use_prefix_slug() {
    let qa = read_repo_file("engine/templates/delivery/qa-instructions.md");
    let git = read_repo_file("engine/reference/git.md");

    assert!(
        qa.contains("**branch**: `{prefix}/{slug}`"),
        "QA template must display the real branch placeholder"
    );
    assert!(
        !qa.contains("{type}/{slug}"),
        "QA template must not use the unmapped branch placeholder"
    );
    assert!(
        git.contains("git branch -d {prefix}/{slug}"),
        "post-merge cleanup must delete the real branch placeholder"
    );
    assert!(
        !git.contains("git branch -d {type}/{slug}"),
        "post-merge cleanup must not delete the unmapped branch placeholder"
    );
}

#[test]
fn auto_commit_gate_is_verification_not_reviewer() {
    let git = read_repo_file("engine/reference/git.md");

    assert!(
        git.contains("Commit only after verification PASS"),
        "auto-commit rules must keep verification as the commit gate"
    );
    assert!(
        git.contains("not a pre-commit gate"),
        "auto-commit rules must state reviewer verdict gates completion, not commits"
    );
    assert!(
        !git.contains("reviewer PASS when `risk.md` requires it"),
        "auto-commit rules must not require reviewer PASS before each commit"
    );
}

#[test]
fn ac_matrix_pending_human_is_canonical_provisional_token() {
    let workflow = read_repo_file("engine/reference/workflow.md");
    let build = read_repo_file("engine/commands/build.md");
    let language = read_repo_file("engine/reference/language.md");
    let qa = read_repo_file("engine/templates/delivery/qa-instructions.md");

    assert!(workflow.contains("`PENDING_HUMAN`"), "{workflow}");
    assert!(
        workflow.contains("not done-eligible"),
        "workflow must mark provisional results as not done-eligible"
    );
    assert!(
        build.contains("`PENDING_HUMAN`") && !build.contains("\"pending human verification\""),
        "build must use the canonical provisional token"
    );
    assert!(
        language.contains("`PENDING_HUMAN`"),
        "language reference must preserve the provisional token"
    );
    assert!(
        qa.contains("Human confirmed` is worksheet prose only") && qa.contains("`人間確認済み`"),
        "QA worksheet must map prose confirmation to the Matrix token"
    );
}

#[test]
fn ad_hoc_review_is_report_only() {
    let review = read_repo_file("engine/commands/review.md");
    let risk = read_repo_file("engine/reference/risk.md");

    assert!(
        review.contains("Reports findings only")
            && review.contains("Do not fix inline as part of ad-hoc review"),
        "ad-hoc review command must be report-only"
    );
    assert!(
        risk.contains("For mandatory risk-cadence review during `build`")
            && risk.contains("For ad-hoc review, do not fix findings inline"),
        "risk reference must separate build review fixes from ad-hoc review reporting"
    );
    assert!(
        !review.contains("fix inline and re-run") && !risk.contains("fix inline and re-run"),
        "review docs must not auto-fix from ad-hoc review"
    );
}

#[test]
fn no_pr_fast_path_skips_pr_gate_but_still_ships() {
    let workflow = read_repo_file("engine/reference/workflow.md");
    let git = read_repo_file("engine/reference/git.md");
    let ship = read_repo_file("engine/commands/ship.md");
    let build = read_repo_file("engine/commands/build.md");

    assert!(
        workflow.contains("skips")
            && workflow.contains("**approve-PR**")
            && workflow.contains("still runs `ship`"),
        "workflow must describe the no-PR gate exception"
    );
    assert!(
        git.contains("no-PR skips PR creation and the approve-PR gate")
            && git.contains("still runs `ship` acceptance"),
        "git reference must keep no-PR tied to ship acceptance"
    );
    assert!(
        ship.contains("On the explicit no-PR fast path, skip this PR section")
            && ship.contains("same close-out commit"),
        "ship must skip PR work only after close-out"
    );
    assert!(
        build.contains("no-PR fast path branch choice") && !build.contains("no-PR fast commit"),
        "build must not imply no-PR completes at build commit"
    );
}

#[test]
fn workflow_gate_2_uses_mochiflow_pr() {
    let workflow = read_repo_file("engine/reference/workflow.md");
    let gate_2 = workflow
        .lines()
        .find(|line| line.contains("**approve-PR**"))
        .expect("workflow approve-PR gate line exists");

    assert!(
        gate_2.contains("before `mochiflow pr` runs"),
        "gate 2 must point to mochiflow pr; got: {gate_2}"
    );
    assert!(
        !gate_2.contains("[git].pr_command"),
        "gate 2 must not point to deprecated [git].pr_command; got: {gate_2}"
    );
}

#[test]
fn readme_lists_public_cli_commands() {
    let readme = read_repo_file("README.md");
    let commands = [
        "init",
        "join",
        "detach",
        "guide",
        "config",
        "lint",
        "doctor",
        "adapter",
        "index",
        "ready",
        "backlog",
        "upgrade",
        "pr",
        "completions",
    ];

    for command in commands {
        let needle = format!("`mochiflow {command}`");
        assert!(
            readme.contains(&needle),
            "README.md must list public CLI command {needle}"
        );
    }
}

#[test]
fn micro_template_has_no_ac_verification_matrix() {
    let template = read_repo_file("engine/templates/spec/spec.micro.md");

    assert!(
        !template.contains("AC Verification Matrix"),
        "micro spec template must not include the build/ship-owned AC Verification Matrix"
    );
    assert!(
        !template.contains("UNVERIFIED"),
        "UNVERIFIED is not part of the AC result enum and must not appear in the micro template"
    );
}

#[test]
fn language_reference_uses_current_ac_results() {
    let language = read_repo_file("engine/reference/language.md");

    for stale in ["UNVERIFIED", "HUMAN_CONFIRMED"] {
        assert!(
            !language.contains(stale),
            "language reference must not preserve stale AC result token {stale}"
        );
    }
    for current in [
        "`PASS`",
        "`人間確認済み`",
        "`対象外（<reason>）`",
        "`FAIL`",
        "`PENDING_HUMAN`",
    ] {
        assert!(
            language.contains(current),
            "language reference must list current AC result value {current}"
        );
    }
}

#[test]
fn templates_do_not_use_fixed_ios_surface() {
    for path in [
        "engine/templates/spec/spec.yaml",
        "engine/templates/backlog/seed.md",
    ] {
        let template = read_repo_file(path);
        assert!(
            !template.contains("surface: ios"),
            "{path} must not hard-code surface: ios"
        );
        assert!(
            !template.contains("- ios"),
            "{path} must not hard-code an ios list item"
        );
        assert!(
            template.contains("{surface}"),
            "{path} must expose a surface placeholder"
        );
    }
}

#[test]
fn design_template_optional_residue_guard() {
    let plan = read_repo_file("engine/commands/plan.md");
    let design = read_repo_file("engine/templates/spec/design.md");

    assert!(
        plan.contains("delete optional sections at creation time"),
        "plan must require optional design sections to be removed at creation time"
    );
    assert!(
        design.matches("Delete this heading").count() >= 4,
        "design template optional sections must instruct agents to delete inapplicable headings"
    );
}

#[test]
fn engine_templates_are_english_source() {
    let templates_dir = repo_root().join("engine/templates");
    let mut files = Vec::new();
    collect_files(&templates_dir, &mut files);

    for path in files {
        let mut text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        for token in ["人間確認済み", "対象外"] {
            text = text.replace(token, "");
        }
        let has_japanese = text.chars().any(|c| {
            ('\u{3040}'..='\u{30ff}').contains(&c) || ('\u{4e00}'..='\u{9fff}').contains(&c)
        });
        assert!(
            !has_japanese,
            "engine source templates must stay English-only: {}",
            path.strip_prefix(repo_root()).unwrap_or(&path).display()
        );
    }
}

#[test]
fn english_template_headings_are_present() {
    let spec = read_repo_file("engine/templates/spec/spec.md");
    let design = read_repo_file("engine/templates/spec/design.md");
    let tasks = read_repo_file("engine/templates/spec/tasks.md");
    let qa = read_repo_file("engine/templates/delivery/qa-instructions.md");

    for heading in [
        "## Background and Design Rationale",
        "## User Story",
        "## Scope",
        "## Acceptance Criteria (EARS)",
        "## QA Scenarios",
        "## Completion Conditions",
    ] {
        assert!(spec.contains(heading), "spec template missing {heading}");
    }
    for heading in [
        "## Design Decisions",
        "## Architecture",
        "## Data Model / Interfaces",
        "## Error Handling",
        "## Test Strategy",
        "## Integration Log",
    ] {
        assert!(
            design.contains(heading),
            "design template missing {heading}"
        );
    }
    for heading in [
        "Implementation Summary:",
        "Critical Stop Conditions:",
        "Covers AC:",
        "Planned Change Areas:",
        "Additional Stop Conditions:",
    ] {
        assert!(tasks.contains(heading), "tasks template missing {heading}");
    }
    for heading in [
        "# QA Instructions:",
        "## Legend",
        "## Preparation",
        "## Scenario List",
        "## Detailed Steps",
        "## Report Formats",
    ] {
        assert!(qa.contains(heading), "QA template missing {heading}");
    }
}

#[test]
fn engine_references_do_not_use_removed_japanese_template_headings() {
    let engine_dir = repo_root().join("engine");
    let mut files = Vec::new();
    collect_files(&engine_dir, &mut files);
    let removed = ["背景と設計判断", "設計判断", "統合ログ", "対応 AC"];

    for path in files {
        let rel = path
            .strip_prefix(repo_root())
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if rel == "engine/MANIFEST.json" {
            continue;
        }
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        for token in removed {
            assert!(
                !text.contains(token),
                "{rel} must not reference removed Japanese template heading token {token}"
            );
        }
    }
}

// --- (f) Behavioral / property tests: lint rules pinned without golden --------

/// Materialize a minimal project (config + memory + specs) and write a single
/// spec under slug `s`. Returns (exit_code, stdout) of `lint --spec s`.
fn run_lint_case(
    spec_yaml: &str,
    spec_md: &str,
    design_md: Option<&str>,
    tasks_md: Option<&str>,
) -> (i32, String) {
    let tmp = tempfile::tempdir().unwrap();
    let install = tmp.path().join(".mochiflow");
    let memory = install.join("memory");
    std::fs::create_dir_all(&memory).unwrap();
    for name in ["architecture.md", "pitfalls.md"] {
        std::fs::write(memory.join(name), "# m\n").unwrap();
    }
    let context = install.join("context");
    std::fs::create_dir_all(&context).unwrap();
    for name in ["product.md", "structure.md", "tech.md"] {
        std::fs::write(context.join(name), "# c\n").unwrap();
    }
    std::fs::write(
        install.join("config.toml"),
        "schema_version = 1\n\
         install_dir = \".mochiflow\"\n\
         specs_dir = \".mochiflow/specs\"\n\
         index = \".mochiflow/INDEX.md\"\n\n\
         [constitution]\n\
         project = \".mochiflow/constitution.md\"\n\
         local = \".mochiflow/constitution.local.md\"\n\n\
         [context]\n\
         product = \".mochiflow/context/product.md\"\n\
         structure = \".mochiflow/context/structure.md\"\n\
         tech = \".mochiflow/context/tech.md\"\n\n\
         [adr]\n\
         decisions = \".mochiflow/adr/decisions.md\"\n\
         pitfalls = \".mochiflow/adr/pitfalls.md\"\n\n\
         [surfaces.app]\n\
         description = \"app\"\n\n\
         [surfaces.app.verify]\n\
         default = \"echo ok\"\n",
    )
    .unwrap();

    let spec_dir = install.join("specs").join("s");
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(spec_dir.join("spec.yaml"), spec_yaml).unwrap();
    std::fs::write(spec_dir.join("spec.md"), spec_md).unwrap();
    if let Some(d) = design_md {
        std::fs::write(spec_dir.join("design.md"), d).unwrap();
    }
    if let Some(t) = tasks_md {
        std::fs::write(spec_dir.join("tasks.md"), t).unwrap();
    }

    let out = assert_cmd::Command::cargo_bin("mochiflow")
        .unwrap()
        .args([
            "--config",
            install.join("config.toml").to_str().unwrap(),
            "lint",
            "--spec",
            "s",
        ])
        .output()
        .unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

/// Valid approved spec (standard risk, single surface) → lint passes.
const GOOD_YAML: &str = "version: 1\nslug: s\ntitle: S\ntype: feature\nsurfaces:\n  - app\nintegration: none\nrisk: standard\nstatus: approved\n";

#[test]
fn lint_passes_valid_approved_spec() {
    let md = "# S\n\n## Requirements / Acceptance Criteria\n\n| AC | Type | Priority | Requirement | Verification |\n| --- | --- | --- | --- | --- |\n| AC-01 | functional | Must | THE SYSTEM SHALL do the thing. | automated |\n\n## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | UNVERIFIED |\n";
    let (code, _out) = run_lint_case(GOOD_YAML, md, None, None);
    assert_eq!(code, 0, "a well-formed approved spec must lint clean");
}

#[test]
fn lint_passes_english_template_headings_and_covers_ac() {
    let yaml = GOOD_YAML.replace("status: approved", "status: done");
    let md = "# S\n\n## Acceptance Criteria (EARS)\n\n- AC-01: THE SYSTEM SHALL do the thing.\n";
    let tasks = "# Tasks\n\n## Task 1\n\nCovers AC: AC-01\n\n\
                 ## AC Verification Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | PASS |\n";
    let (code, out) = run_lint_case(&yaml, md, None, Some(tasks));
    assert_eq!(code, 0, "{out}");
}

#[test]
fn lint_done_fails_when_matrix_missing() {
    let yaml = GOOD_YAML.replace("status: approved", "status: done");
    let md = "# S\n\n## 受入基準\n\n- AC-01: THE SYSTEM SHALL do the thing.\n";
    let (code, out) = run_lint_case(&yaml, md, None, None);
    assert_eq!(code, 1);
    assert!(out.contains("AC Matrix is missing"), "{out}");
}

#[test]
fn lint_done_fails_when_matrix_contains_fail() {
    let yaml = GOOD_YAML.replace("status: approved", "status: done");
    let md = "# S\n\n## 受入基準\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | FAIL |\n";
    let (code, out) = run_lint_case(&yaml, md, None, None);
    assert_eq!(code, 1);
    assert!(out.contains("invalid result `FAIL`"), "{out}");
}

#[test]
fn lint_done_passes_with_canonical_final_matrix_results() {
    let yaml = GOOD_YAML.replace("status: approved", "status: done");
    let md = "# S\n\n## 受入基準\n\n\
              - AC-01: THE SYSTEM SHALL x.\n\
              - AC-02: WHEN y, THE SYSTEM SHALL z.\n\
              - AC-03: WHERE q, THE SYSTEM SHALL r.\n\n\
              ## AC Verification Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | PASS |\n| AC-02 | 人間確認済み |\n| AC-03 | 対象外（not relevant for CLI） |\n";
    let (code, out) = run_lint_case(&yaml, md, None, None);
    assert_eq!(code, 0, "{out}");
}

#[test]
fn lint_done_fails_when_matrix_contains_pending_or_unknown_result() {
    let yaml = GOOD_YAML.replace("status: approved", "status: done");
    for result in [
        "PENDING_HUMAN",
        "pending human verification",
        "Human confirmed",
    ] {
        let md = format!(
            "# S\n\n## 受入基準\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
             ## AC Verification Matrix\n\n| AC | 結果 |\n| --- | --- |\n| AC-01 | {result} |\n"
        );
        let (code, out) = run_lint_case(&yaml, &md, None, None);
        assert_eq!(code, 1, "{result}: {out}");
        assert!(out.contains(&format!("invalid result `{result}`")), "{out}");
    }
}

#[test]
fn lint_done_fails_when_matrix_result_is_empty() {
    let yaml = GOOD_YAML.replace("status: approved", "status: done");
    let md = "# S\n\n## 受入基準\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
              ## AC Verification Matrix\n\n| AC | 結果 |\n| --- | --- |\n| AC-01 |  |\n";
    let (code, out) = run_lint_case(&yaml, md, None, None);
    assert_eq!(code, 1);
    assert!(out.contains("invalid result `<empty>`"), "{out}");
}

#[test]
fn lint_done_fails_when_not_applicable_reason_is_placeholder_or_empty() {
    let yaml = GOOD_YAML.replace("status: approved", "status: done");
    for result in ["対象外（）", "対象外（理由）"] {
        let md = format!(
            "# S\n\n## 受入基準\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
             ## AC Verification Matrix\n\n| AC | 結果 |\n| --- | --- |\n| AC-01 | {result} |\n"
        );
        let (code, out) = run_lint_case(&yaml, &md, None, None);
        assert_eq!(code, 1, "{result}: {out}");
        assert!(out.contains(&format!("invalid result `{result}`")), "{out}");
    }
}

#[test]
fn lint_done_fails_when_matrix_contains_pending_human() {
    let yaml = GOOD_YAML.replace("status: approved", "status: done");
    for result in ["UNVERIFIED", "PENDING_HUMAN"] {
        let md = format!(
            "# S\n\n## Acceptance Criteria\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
             ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | {result} |\n"
        );
        let (code, out) = run_lint_case(&yaml, &md, None, None);
        assert_eq!(code, 1, "{result}: {out}");
        assert!(out.contains(result), "{result}: {out}");
    }
}

#[test]
fn lint_fails_when_matrix_result_is_not_canonical() {
    let md = "# S\n\n## Acceptance Criteria\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | 人間確認待ち |\n";
    let (code, out) = run_lint_case(GOOD_YAML, md, None, None);
    assert_eq!(code, 1);
    assert!(out.contains("must be one of UNVERIFIED"), "{out}");
}

#[test]
fn lint_fails_when_matrix_na_has_no_reason() {
    let yaml = GOOD_YAML.replace("status: approved", "status: done");
    let md = "# S\n\n## Acceptance Criteria\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | N/A |\n";
    let (code, out) = run_lint_case(&yaml, md, None, None);
    assert_eq!(code, 1);
    assert!(out.contains("N/A: <reason>"), "{out}");
}

#[test]
fn lint_done_fails_when_ac_not_in_matrix() {
    let yaml = GOOD_YAML.replace("status: approved", "status: done");
    let md = "# S\n\n## 受入基準\n\n- AC-01: THE SYSTEM SHALL x.\n- AC-02: WHEN y, THE SYSTEM SHALL z.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | PASS |\n";
    let (code, out) = run_lint_case(&yaml, md, None, None);
    assert_eq!(code, 1);
    assert!(out.contains("AC not present in AC Matrix: AC-02"), "{out}");
}

#[test]
fn lint_approved_fails_when_ac_not_in_matrix() {
    let md = "# S\n\n## Requirements / Acceptance Criteria\n\n- AC-01: THE SYSTEM SHALL x.\n- AC-02: WHEN y, THE SYSTEM SHALL z.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | UNVERIFIED |\n";
    let (code, out) = run_lint_case(GOOD_YAML, md, None, None);
    assert_eq!(code, 1);
    assert!(out.contains("AC not present in AC Matrix: AC-02"), "{out}");
}

#[test]
fn lint_fails_when_tasks_do_not_cover_all_acs() {
    let yaml = GOOD_YAML.replace("status: approved", "status: done");
    let md = "# S\n\n## 受入基準\n\n- AC-01: THE SYSTEM SHALL x.\n- AC-02: WHEN y, THE SYSTEM SHALL z.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | PASS |\n| AC-02 | PASS |\n";
    let tasks = "# Tasks\n\n- [x] T-001 [AC-01] Do x\n  - Depends on: none\n  - Files:\n    - `src/x.rs`\n  - Done:\n    - [ ] Verification passed\n  - Stop:\n    - stop\n";
    let (code, out) = run_lint_case(&yaml, md, None, Some(tasks));
    assert_eq!(code, 1);
    assert!(
        out.contains("AC not covered by any task Covers AC: AC-02"),
        "{out}"
    );
}

#[test]
fn lint_accepts_english_acceptance_and_task_headings() {
    let yaml = GOOD_YAML.replace("status: approved", "status: done");
    let md = "# S\n\n## Acceptance Criteria (EARS)\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | PASS |\n";
    let tasks = "# Tasks\n\n- [x] T-001 [AC-01] Do x\n  - Depends on: none\n  - Files:\n    - `src/x.rs`\n  - Done:\n    - [ ] Verification passed\n    - [ ] AC Matrix updated\n  - Stop:\n    - stop\n";
    let (code, out) = run_lint_case(&yaml, md, None, Some(tasks));
    assert_eq!(code, 0, "{out}");
}

#[test]
fn lint_accepts_legacy_matrix_in_tasks_when_spec_has_none() {
    let yaml = GOOD_YAML.replace("status: approved", "status: done");
    let md = "# S\n\n## Acceptance Criteria\n\n- AC-01: THE SYSTEM SHALL x.\n";
    let tasks = "# Tasks\n\n- [x] T-001 [AC-01] Do x\n  - Depends on: none\n  - Files:\n    - `src/x.rs`\n  - Done:\n    - [ ] Verification passed\n  - Stop:\n    - stop\n\n\
                 ## AC Verification Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | PASS |\n";
    let (code, out) = run_lint_case(&yaml, md, None, Some(tasks));
    assert_eq!(code, 0, "{out}");
}

#[test]
fn lint_done_fails_when_task_is_unchecked() {
    let yaml = GOOD_YAML.replace("status: approved", "status: done");
    let md = "# S\n\n## Acceptance Criteria\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | PASS |\n";
    let tasks = "# Tasks\n\n- [ ] T-001 [AC-01] Do x\n  - Depends on: none\n  - Files:\n    - `src/x.rs`\n  - Done:\n    - [ ] Verification passed\n  - Stop:\n    - stop\n";
    let (code, out) = run_lint_case(&yaml, md, None, Some(tasks));
    assert_eq!(code, 1);
    assert!(
        out.contains("status is done but task T-001 is not checked"),
        "{out}"
    );
}

#[test]
fn lint_approved_fails_with_needs_clarification() {
    let md = "# S\n\n## Requirements / Acceptance Criteria\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
              ## Open Questions\n\n- [NEEDS-CLARIFICATION: decide x]\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | UNVERIFIED |\n";
    let (code, out) = run_lint_case(GOOD_YAML, md, None, None);
    assert_eq!(code, 1);
    assert!(out.contains("[NEEDS-CLARIFICATION] remains"), "{out}");
}

#[test]
fn lint_requirements_acceptance_heading_participates_in_ears_check() {
    let md = "# S\n\n## Requirements / Acceptance Criteria\n\n- AC-01: x happens.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | UNVERIFIED |\n";
    let (code, out) = run_lint_case(GOOD_YAML, md, None, None);
    assert_eq!(code, 0);
    assert!(out.contains("AC without EARS keyword"), "{out}");
}

#[test]
fn lint_contract_integration_accepts_integration_contract_section() {
    let yaml = GOOD_YAML.replace("integration: none", "integration: contract");
    let md = "# S\n\n## Requirements / Acceptance Criteria\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | UNVERIFIED |\n";
    let design = "# S — Design\n\n## Integration Contract\n\n| Contract | Input / Request | Output / Response | Errors | Compatibility |\n| --- | --- | --- | --- | --- |\n| c | i | o | e | compatible |\n";
    let (code, out) = run_lint_case(&yaml, md, Some(design), None);
    assert_eq!(code, 0, "{out}");
}

#[test]
fn lint_fails_when_tasks_are_not_executable_checklist() {
    let tasks = "# Tasks\n\n## Task 1\n\nCovers AC: AC-01\n";
    let md = "# S\n\n## Acceptance Criteria\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | UNVERIFIED |\n";
    let (code, out) = run_lint_case(GOOD_YAML, md, None, Some(tasks));
    assert_eq!(code, 1);
    assert!(out.contains("top-level T-### checkbox tasks"), "{out}");
}

#[test]
fn lint_fails_when_task_missing_required_blocks() {
    let tasks = "# Tasks\n\n- [ ] T-001 [AC-01] Do x\n  - Depends on: none\n";
    let md = "# S\n\n## Acceptance Criteria\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | UNVERIFIED |\n";
    let (code, out) = run_lint_case(GOOD_YAML, md, None, Some(tasks));
    assert_eq!(code, 1);
    assert!(out.contains("missing Files:"), "{out}");
    assert!(out.contains("missing Done:"), "{out}");
    assert!(out.contains("missing Stop:"), "{out}");
}

#[test]
fn lint_done_elevated_fails_without_reviewer_verdict() {
    let yaml = GOOD_YAML
        .replace("status: approved", "status: done")
        .replace("risk: standard", "risk: elevated");
    // elevated → design.md required; provide it so only the verdict check fails.
    let md = "# S\n\n## 受入基準\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | PASS |\n";
    let design = "# design\n\n## 設計判断\n\n- ok\n";
    let (code, out) = run_lint_case(&yaml, md, Some(design), None);
    assert_eq!(code, 1);
    assert!(out.contains("reviewer verdict"), "{out}");
}

#[test]
fn lint_done_elevated_passes_with_reviewer_verdict() {
    let yaml = GOOD_YAML
        .replace("status: approved", "status: done")
        .replace("risk: standard", "risk: elevated");
    let md = "# S\n\n## 受入基準\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | PASS |\n";
    let design = "# design\n\n## 設計判断\n\n- ok\n\n## Review Results\n\nReviewer mode: inline\nVerdict: pass\n";
    let (code, _out) = run_lint_case(&yaml, md, Some(design), None);
    assert_eq!(code, 0);
}

#[test]
fn lint_done_elevated_ignores_reviewer_verdict_outside_design() {
    let yaml = GOOD_YAML
        .replace("status: approved", "status: done")
        .replace("risk: standard", "risk: elevated");
    let md = "# S\n\n## 受入基準\n\n- AC-01: THE SYSTEM SHALL x.\n";
    let design = "# design\n\n## 設計判断\n\n- ok\n\n## Review Results\n\n";
    let tasks = "# tasks\n\n- [x] T-001 [AC-01] Do x\n  - Depends on: none\n  - Files:\n    - `src/x.rs`\n  - Done:\n    - [ ] Verification passed\n  - Stop:\n    - stop\n\nVerdict: pass\n\n\
                 ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | PASS |\n";
    let (code, out) = run_lint_case(&yaml, md, Some(design), Some(tasks));
    assert_eq!(code, 1);
    assert!(out.contains("reviewer verdict"), "{out}");
}

#[test]
fn lint_rejects_unsupported_style_and_test_types() {
    for spec_type in ["style", "test"] {
        let yaml = GOOD_YAML.replace("type: feature", &format!("type: {spec_type}"));
        let md = "# S\n\n## 受入基準\n\n- AC-01: THE SYSTEM SHALL x.\n";
        let (code, out) = run_lint_case(&yaml, md, None, None);
        assert_eq!(code, 1, "{spec_type} should be rejected");
        assert!(
            out.contains("type must be one of: feature, fix, refactor, docs, chore"),
            "{out}"
        );
    }
}

#[test]
fn lint_fails_on_markdown_frontmatter() {
    let md = "---\ntitle: nope\n---\n\n# S\n\n## 受入基準\n\n- AC-01: THE SYSTEM SHALL x.\n";
    let (code, out) = run_lint_case(GOOD_YAML, md, None, None);
    assert_eq!(code, 1);
    assert!(out.contains("frontmatter is not allowed"), "{out}");
}

#[test]
fn lint_fails_on_invalid_status() {
    let yaml = GOOD_YAML.replace("status: approved", "status: bogus");
    let md = "# S\n\n## 受入基準\n\n- AC-01: THE SYSTEM SHALL x.\n";
    let (code, out) = run_lint_case(&yaml, md, None, None);
    assert_eq!(code, 1);
    assert!(
        out.contains("status must be one of: draft, approved, done"),
        "{out}"
    );
}

#[test]
fn lint_fails_on_spec_yaml_missing_required_keys() {
    // Negative fixture (AC-06): spec.yaml missing required keys is rejected.
    let yaml = "version: 1\nslug: s\ntitle: S\n";
    let md = "# S\n\n## 受入基準\n\n- AC-01: THE SYSTEM SHALL x.\n";
    let (code, out) = run_lint_case(yaml, md, None, None);
    assert_eq!(code, 1);
    assert!(out.contains("missing required keys"), "{out}");
}

// --- (f) Subcommand behavioral contract (re-expresses former Rust↔Python -------
//         parity as Rust-vs-contract assertions on a materialized project) -----

/// Materialize a self-contained project: real engine (with its MANIFEST), a
/// config (adapter=kiro), a sample spec, a backlog seed, and memory files.
/// Returns the config.toml path.
fn materialize_full(root: &Path) -> PathBuf {
    let install = root.join(".mochiflow");
    let engine = install.join("engine");
    copy_dir_all(&repo_root().join("engine"), &engine);

    std::fs::create_dir_all(install.join("memory")).unwrap();
    for name in ["architecture.md", "pitfalls.md"] {
        std::fs::write(install.join("memory").join(name), "# m\n").unwrap();
    }
    std::fs::create_dir_all(install.join("context")).unwrap();
    for name in ["product.md", "structure.md", "tech.md"] {
        std::fs::write(install.join("context").join(name), "# c\n").unwrap();
    }

    std::fs::write(
        install.join("config.toml"),
        "schema_version = 1\n\
         install_dir = \".mochiflow\"\n\
         specs_dir = \".mochiflow/specs\"\n\
         index = \".mochiflow/INDEX.md\"\n\n\
         [i18n]\n\
         artifact_language = \"en\"\n\
         conversation_language = \"auto\"\n\n\
         [constitution]\n\
         project = \".mochiflow/constitution.md\"\n\
         local = \".mochiflow/constitution.local.md\"\n\n\
         [context]\n\
         product = \".mochiflow/context/product.md\"\n\
         structure = \".mochiflow/context/structure.md\"\n\
         tech = \".mochiflow/context/tech.md\"\n\n\
         [adr]\n\
         decisions = \".mochiflow/adr/decisions.md\"\n\
         pitfalls = \".mochiflow/adr/pitfalls.md\"\n\n\
         [git]\n\
         provider = \"github\"\n\
         base_branch = \"main\"\n\
         pr_command = \"gh pr create --fill\"\n\n\
         [adapter]\n\
         tool = \"kiro\"\n\n\
         [write]\n\
         allow = [\"src/**\"]\n\
         deny = [\".git/**\"]\n\n\
         [surfaces.app]\n\
         description = \"primary surface\"\n\n\
         [surfaces.app.verify]\n\
         default = \"echo test-pass\"\n",
    )
    .unwrap();

    let specs = install.join("specs");
    std::fs::create_dir_all(specs.join("_done")).unwrap();
    let backlog = specs.join("_backlog");
    std::fs::create_dir_all(&backlog).unwrap();
    std::fs::write(
        backlog.join("sample-seed.md"),
        "---\nslug: sample-seed\ntitle: Sample Seed\nmaturity: seed\nsource: conversation\ncreated: 2026-03-10\n---\n\n## Signal\n\nAn idea.\n",
    )
    .unwrap();

    let spec_dir = specs.join("sample-spec");
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(
        spec_dir.join("spec.yaml"),
        "version: 1\nslug: sample-spec\ntitle: Sample Spec\ntype: feature\nsurfaces:\n  - app\nintegration: none\nrisk: standard\nstatus: approved\n",
    )
    .unwrap();
    std::fs::write(
        spec_dir.join("spec.md"),
        "# Sample Spec\n\n## 受入基準\n\n- AC-01: THE SYSTEM SHALL do the thing.\n\n## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | UNVERIFIED |\n",
    )
    .unwrap();

    install.join("config.toml")
}

fn run_cli(config: &Path, args: &[&str]) -> (i32, String) {
    let mut full = vec!["--config", config.to_str().unwrap()];
    full.extend_from_slice(args);
    let out = assert_cmd::Command::cargo_bin("mochiflow")
        .unwrap()
        .args(&full)
        .output()
        .unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

#[test]
fn behavioral_config_show_and_validate() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = materialize_full(tmp.path());

    let (code, out) = run_cli(&cfg, &["config", "show"]);
    assert_eq!(code, 0);
    assert!(
        out.contains("installed_engine_version :")
            && out.contains("bundled_engine_version")
            && out.contains("surfaces"),
        "{out}"
    );
    assert!(!out.contains("\nengine_version :"), "{out}");

    let (code, out) = run_cli(&cfg, &["config", "validate"]);
    assert_eq!(code, 0, "valid config must validate clean: {out}");
}

#[test]
fn behavioral_lint_and_ready() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = materialize_full(tmp.path());

    let (code, _out) = run_cli(&cfg, &["lint"]);
    assert_eq!(code, 0);

    let (code, out) = run_cli(&cfg, &["ready", "sample-spec"]);
    assert_eq!(code, 0);
    assert!(
        out.contains("READY") && out.contains("sample-spec"),
        "{out}"
    );
}

#[test]
fn behavioral_backlog() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = materialize_full(tmp.path());

    let (code, out) = run_cli(&cfg, &["backlog", "list"]);
    assert_eq!(code, 0);
    assert!(out.contains("sample-seed"), "{out}");

    let (code, out) = run_cli(&cfg, &["backlog", "show", "sample-seed"]);
    assert_eq!(code, 0);
    assert!(out.contains("Sample Seed"), "{out}");

    let (code, _out) = run_cli(&cfg, &["backlog", "validate", "sample-seed"]);
    assert_eq!(code, 0);
}

#[test]
fn behavioral_doctor_config_and_engine() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = materialize_full(tmp.path());

    let (code, _out) = run_cli(&cfg, &["doctor", "config"]);
    assert_eq!(code, 0);

    // Engine materialized with its real MANIFEST → no drift.
    let (code, _out) = run_cli(&cfg, &["doctor", "engine"]);
    assert_eq!(code, 0);
}

#[test]
fn behavioral_doctor_warns_when_index_is_stale() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = materialize_full(tmp.path());

    let (code, out) = run_cli(&cfg, &["doctor", "config"]);
    assert_eq!(code, 0);
    assert!(out.contains("INDEX.md is stale"), "{out}");
}

#[test]
fn behavioral_adapter_generate_then_check_clean_and_doctor() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = materialize_full(tmp.path());

    // Generate adapter outputs, then --check must report zero drift
    // (deterministic generation).
    let (code, _out) = run_cli(&cfg, &["adapter", "generate"]);
    assert_eq!(code, 0);
    let (code, out) = run_cli(&cfg, &["adapter", "generate", "--check"]);
    assert_eq!(code, 0, "adapter generate must be deterministic: {out}");

    // With adapters generated, full doctor (config/specs/adapter/engine) is clean.
    let (code, out) = run_cli(&cfg, &["doctor"]);
    assert_eq!(
        code, 0,
        "full doctor must pass on a complete project: {out}"
    );
}

#[test]
fn behavioral_upgrade_from_source_engine() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = materialize_full(tmp.path());
    let source = repo_root().join("engine");

    let (code, out) = run_cli(&cfg, &["upgrade", "--source", source.to_str().unwrap()]);
    assert_eq!(
        code, 0,
        "upgrade from a valid engine source must succeed: {out}"
    );
    assert!(out.contains("upgraded engine"), "{out}");

    let manifest = read_json(&tmp.path().join(".mochiflow/engine/MANIFEST.json"));
    let installed_version =
        std::fs::read_to_string(tmp.path().join(".mochiflow/engine/VERSION")).unwrap();
    assert_eq!(
        manifest["version"].as_str(),
        Some(installed_version.trim()),
        "upgrade must write MANIFEST.version from installed engine VERSION"
    );

    // After upgrade the engine remains drift-free.
    let (code, _out) = run_cli(&cfg, &["doctor", "engine"]);
    assert_eq!(code, 0);
}

#[test]
fn behavioral_upgrade_from_bundled_engine_regenerates_adapters() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = materialize_full(tmp.path());

    let (code, out) = run_cli(&cfg, &["upgrade"]);
    assert_eq!(
        code, 0,
        "upgrade without --source must use bundled engine: {out}"
    );
    assert!(out.contains("upgraded engine <- bundled engine"), "{out}");
    assert!(out.contains("wrote: .kiro/steering/spec.md"), "{out}");

    let manifest = read_json(&tmp.path().join(".mochiflow/engine/MANIFEST.json"));
    let installed_version =
        std::fs::read_to_string(tmp.path().join(".mochiflow/engine/VERSION")).unwrap();
    assert_eq!(manifest["version"].as_str(), Some(installed_version.trim()));
    assert!(tmp.path().join(".kiro/agents/spec-builder.json").exists());

    let (code, out) = run_cli(&cfg, &["adapter", "generate", "--check"]);
    assert_eq!(code, 0, "upgrade must leave adapters deterministic: {out}");
}

#[test]
fn behavioral_upgrade_from_bundled_engine_respects_drift_force() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = materialize_full(tmp.path());
    let router = tmp.path().join(".mochiflow/engine/router.md");
    std::fs::write(&router, "# locally edited\n").unwrap();

    let (code, out) = run_cli(&cfg, &["upgrade"]);
    assert_eq!(code, 1, "dirty engine must block upgrade without force");
    assert!(out.contains("DIRTY: router.md"), "{out}");

    let (code, out) = run_cli(&cfg, &["upgrade", "--force"]);
    assert_eq!(code, 0, "force must replace dirty engine: {out}");
    assert!(out.contains("upgraded engine <- bundled engine"), "{out}");
}

#[test]
fn behavioral_upgrade_reports_adapter_merge_required_after_engine_update() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = materialize_full(tmp.path());
    let target = tmp.path().join(".kiro/agents/spec-builder.json");
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    std::fs::write(&target, "{\"custom\": true}\n").unwrap();

    let (code, out) = run_cli(&cfg, &["upgrade"]);
    assert_eq!(code, 1, "blocked adapter merge should be non-zero");
    assert!(out.contains("upgraded engine <- bundled engine"), "{out}");
    assert!(
        out.contains("BLOCKED: .kiro/agents/spec-builder.json"),
        "{out}"
    );
    assert!(
        out.contains("engine upgraded; adapter merge required"),
        "{out}"
    );
    assert!(
        tmp.path()
            .join(".mochiflow/state/adapters/.kiro/agents/spec-builder.json")
            .exists()
    );
}

#[test]
fn behavioral_doctor_engine_points_to_bundled_upgrade() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = materialize_full(tmp.path());
    std::fs::write(tmp.path().join(".mochiflow/engine/VERSION"), "0.0.0\n").unwrap();

    let (code, out) = run_cli(&cfg, &["doctor", "engine"]);
    assert_eq!(code, 1);
    assert!(out.contains("run `mochiflow upgrade`"), "{out}");
}

#[test]
fn behavioral_adapter_all_tools_deterministic() {
    // Pins generation determinism for ALL four adapters (agents/kiro/copilot/
    // claude-code), not just kiro — closes the gap left by dropping the former
    // Python adapter byte-parity assertions. Output *correctness* is pinned by
    // adapter::render unit tests + frozen .tpl (version-gate) + Task-8 dogfood.
    let tmp = tempfile::tempdir().unwrap();
    let cfg = materialize_full(tmp.path());
    let text = std::fs::read_to_string(&cfg).unwrap().replace(
        "tool = \"kiro\"",
        "tools = [\"agents\", \"kiro\", \"copilot\", \"claude-code\"]",
    );
    std::fs::write(&cfg, text).unwrap();

    let (code, out) = run_cli(&cfg, &["adapter", "generate"]);
    assert_eq!(code, 0, "{out}");
    let marker_version = std::fs::read_to_string(tmp.path().join(".mochiflow/engine/VERSION"))
        .unwrap()
        .trim()
        .to_string();
    let agents = std::fs::read_to_string(tmp.path().join("AGENTS.md")).unwrap();
    assert!(
        agents.contains(&format!("version={marker_version}")),
        "adapter marker must use installed engine VERSION: {agents}"
    );
    let (code, out) = run_cli(&cfg, &["adapter", "generate", "--check"]);
    assert_eq!(
        code, 0,
        "adapter generation must be deterministic for all four tools: {out}"
    );
}
