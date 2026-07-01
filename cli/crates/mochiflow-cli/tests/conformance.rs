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
use std::process::Command as Proc;

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

fn frontmatter(body: &str) -> Option<&str> {
    let body = body.strip_prefix("---\n")?;
    let end = body.find("\n---")?;
    Some(&body[..end])
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

fn engine_markdown_files() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    collect_files(&repo_root().join("engine"), &mut paths);
    paths
        .into_iter()
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("md"))
        .collect()
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
/// — JSON schemas and golden output. Delegates to the single implementation in
/// `mochiflow_core::freeze::compute_contracts_hash`.
fn compute_contracts_hash() -> String {
    mochiflow_core::freeze::compute_contracts_hash(&repo_root()).unwrap()
}

#[test]
fn version_gate_consistent() {
    let root = repo_root();
    let lock = read_json(&contracts_dir().join("contracts.lock"));
    let lock_hash = lock["hash"].as_str().unwrap();
    let lock_version = lock["version"].as_str().unwrap();
    let current = compute_contracts_hash();
    let workspace_version = mochiflow_core::freeze::read_workspace_version(&root).unwrap();
    let engine_version = std::fs::read_to_string(root.join("engine/VERSION"))
        .unwrap()
        .trim()
        .to_string();

    // Single rule: all three versions must agree AND hash must match.
    assert_eq!(
        current, lock_hash,
        "frozen-surface hash mismatch (lock {lock_hash:.12}… ≠ computed {current:.12}…); run `mochiflow freeze`"
    );
    assert_eq!(
        lock_version, workspace_version,
        "contracts.lock version ({lock_version}) ≠ workspace version ({workspace_version}); run `mochiflow freeze`"
    );
    assert_eq!(
        engine_version, workspace_version,
        "engine/VERSION ({engine_version}) ≠ workspace version ({workspace_version}); run `mochiflow freeze`"
    );
}

#[test]
fn version_gate_hash_matches_committed_lock() {
    let lock = read_json(&contracts_dir().join("contracts.lock"));
    assert_eq!(
        compute_contracts_hash(),
        lock["hash"].as_str().unwrap(),
        "computed frozen-surface hash must equal contracts.lock; run `mochiflow freeze` if a frozen surface changed"
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
        merged_line.contains("commands/close.md"),
        "merged-event routing must route to close; got: {merged_line}"
    );
    assert!(
        merged_line.contains("flat `{specs_dir}/{slug}/`")
            && merged_line.contains("never moved to `_done/`"),
        "merged-event routing must resolve the flat spec, not a _done archive; got: {merged_line}"
    );
    assert!(
        merged_line.contains("`merged` is derived"),
        "merged-event routing must state merged is derived; got: {merged_line}"
    );
    assert!(
        !merged_line.contains("fold → archive") && !merged_line.contains("fold -> archive"),
        "merged-event routing must not instruct fold/archive after merge; got: {merged_line}"
    );
}

#[test]
fn router_plan_requires_existing_draft_spec() {
    let router = read_repo_file("engine/router.md");

    assert!(
        router.contains("Exception: `{slug} discuss` resolves against a seed"),
        "router must retain the backlog discuss exception"
    );
    assert!(
        router.contains("`{slug} plan` requires an existing active spec directory")
            && router.contains("backlog files are raw seeds, not plan-ready handoffs"),
        "router must require an existing draft spec directory for plan"
    );
    assert!(
        router.contains("{specs_dir}/_backlog/{slug}.md")
            && router.contains("guide back to `{slug} discuss`"),
        "router must route raw backlog seeds back to discuss"
    );
    assert!(
        router.contains("Event patterns `{slug} merged`")
            && router.contains("flat `{specs_dir}/{slug}/`"),
        "router must resolve the merged-event against the flat spec"
    );
}

#[test]
fn router_defines_lazy_load_contract_without_second_card() {
    let router = read_repo_file("engine/router.md");

    assert!(
        router.contains("`router.md` is the only standing router artifact")
            && router.contains("lazy-load catalog")
            && router.contains("matching `commands/{verb}.md`")
            && router.contains("frontmatter `references`"),
        "router must define the standing-vs-lazy loading contract"
    );
    assert!(
        !repo_root().join("engine/router.card.md").exists(),
        "v1 must not introduce a second router card artifact"
    );
    for path in engine_markdown_files() {
        let body = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        if let Some(frontmatter) = frontmatter(&body) {
            assert!(
                !frontmatter.contains(".md#"),
                "frontmatter references must stay file-level, not section anchors: {}",
                path.display()
            );
        }
    }
    assert!(
        router.contains("load the store `INDEX.md` first")
            && router.contains("relevant active records"),
        "router must keep ADR loading bounded by index and relevant active records"
    );
}

#[test]
fn adapters_separate_standing_inputs_from_load_on_demand() {
    for path in [
        "engine/adapters/agents/AGENTS.md.tpl",
        "engine/adapters/claude-code/CLAUDE.md.tpl",
        "engine/adapters/copilot/copilot-instructions.md.tpl",
    ] {
        let body = read_repo_file(path);
        assert!(body.contains("### Standing inputs"), "{path}");
        assert!(body.contains("### Load on demand"), "{path}");
        assert!(body.contains("### Artifact roles"), "{path}");
        assert!(
            body.find("### Standing inputs") < body.find("### Load on demand"),
            "{path} must present standing inputs before lazy-loaded files"
        );
        assert!(
            body.find("### Load on demand") < body.find("### Artifact roles"),
            "{path} must keep artifact-role descriptions outside load-on-demand files"
        );
        assert!(
            body.contains("commands/{discuss,plan,build,open,update,close}.md")
                && body.contains(
                    "reference/{workflow,risk,authoring,git,language,engineering-standards}.md"
                ),
            "{path} must keep command/reference files in the load-on-demand section"
        );
    }

    let kiro = read_repo_file("engine/adapters/kiro/steering/mochiflow.md.tpl");
    assert!(kiro.contains("## Always loaded"), "{kiro}");
    assert!(kiro.contains("### Load on demand"), "{kiro}");
    assert!(kiro.contains("### Artifact roles"), "{kiro}");
    assert!(
        kiro.find("### Load on demand") < kiro.find("### Artifact roles"),
        "Kiro must keep artifact-role descriptions outside load-on-demand files"
    );
    assert!(
        !kiro.contains("#[[file:{{engine}}/commands")
            && !kiro.contains("#[[file:{{engine}}/reference"),
        "Kiro file references must stay limited to standing inputs:\n{kiro}"
    );
}

#[test]
fn router_preserves_named_routing_branches() {
    let router = read_repo_file("engine/router.md");

    for required in [
        "On the retired explicit command `mochiflow-patch`",
        "On any other explicit command (`mochiflow-<verb>`) match",
        "A natural-language trigger",
        "Exception: `{slug} discuss` resolves against a seed",
        "`{slug} plan` requires an existing active spec directory",
        "concrete small-edit requests",
        "as plan intent hints",
        "ad-hoc review (user-triggered via `レビューして` / `mochiflow-review`",
        "Feedback patterns `{slug} feedback`",
        "Event patterns `{slug} merged`",
    ] {
        assert!(
            router.contains(required),
            "router must preserve routing branch: {required}"
        );
    }
}

#[test]
fn branch_placeholders_use_prefix_slug() {
    let git = read_repo_file("engine/reference/git.md");

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
fn pr_feedback_routes_to_update_without_restore() {
    let update = read_repo_file("engine/commands/update.md");
    let build = read_repo_file("engine/commands/build.md");
    let router = read_repo_file("engine/router.md");

    assert!(
        update.contains("do not move it")
            && update.contains("do not revert")
            && update.contains("bounded inline PR-feedback fix")
            && update.contains("Build's eligibility gate")
            && update.contains("flat"),
        "update must apply bounded inline feedback fixes without moving or reverting the flat spec"
    );
    assert!(
        router.contains("applies bounded inline fixes")
            && !router.contains("delegates the code change through `build`"),
        "router PR feedback must describe bounded inline update fixes, not build delegation"
    );
    assert!(
        !build.contains("when build resumes from `commands/update.md`")
            && !build.contains("update resumes"),
        "build.md must not keep an update-resume path now that update fixes are inline"
    );
    assert!(
        router.contains("commands/update.md") && !router.contains("commands/ship.md"),
        "router PR feedback must route to update, not the removed ship command"
    );
}

#[test]
fn engine_open_update_close_defined_no_ship_verb() {
    for cmd in ["open", "update", "close"] {
        let doc = read_repo_file(&format!("engine/commands/{cmd}.md"));
        assert!(
            doc.contains(&format!("mochiflow-{cmd}")) && doc.contains("triggers:"),
            "{cmd}.md must define its explicit command and triggers"
        );
    }
    let router = read_repo_file("engine/router.md");
    assert!(
        router.contains("commands/open.md")
            && router.contains("commands/update.md")
            && router.contains("commands/close.md"),
        "router must reference open/update/close"
    );
    assert!(
        !router.contains("commands/ship.md"),
        "router must not reference the removed ship command"
    );
    assert!(
        !repo_root().join("engine/commands/ship.md").exists(),
        "engine/commands/ship.md must be deleted"
    );
}

#[test]
fn build_ends_at_approved_without_pr_or_move() {
    let build = read_repo_file("engine/commands/build.md");
    assert!(
        build.contains("Build ends at `status: approved`")
            && build.contains("create a PR, or move the spec"),
        "build completion card must end at approved with no PR/terminal/move"
    );
    assert!(
        build.contains("Create the PR** (`open`") && !build.contains("mochiflow-ship"),
        "build must hand off to open, not ship"
    );
}

#[test]
fn open_orders_acceptance_fold_accept_pr_gate() {
    let open = read_repo_file("engine/commands/open.md");
    assert!(
        open.contains("never created before the") && open.contains("approve-PR gate (f)"),
        "open must state the PR is never created before the approve-PR gate"
    );
    for marker in [
        "### (a) Acceptance",
        "### (b) Finalize the fold",
        "### (c) Context refresh commit (optional)",
        "### (d) Accept close-out commit",
        "### (e) Generate PR title/body",
        "### (f) Approve-PR gate",
        "### (g) Push and create the PR",
    ] {
        assert!(open.contains(marker), "open.md must document step {marker}");
    }
    assert!(
        open.contains("owns authoring the fold (not `accept`)"),
        "open (not accept) owns authoring the fold"
    );
}

#[test]
fn close_is_local_hygiene_only() {
    let close = read_repo_file("engine/commands/close.md");
    assert!(
        close.contains("local hygiene") && close.contains("writes nothing to the base branch"),
        "close must be local hygiene only with no base write"
    );
    assert!(
        close.contains("`merged` is derived"),
        "close must state merged is derived, not persisted"
    );
}

#[test]
fn delivery_guidance_is_conversational_and_language_aware() {
    let open = read_repo_file("engine/commands/open.md");
    let router = read_repo_file("engine/router.md");
    let close = read_repo_file("engine/commands/close.md");
    let language = read_repo_file("engine/reference/language.md");

    // open.md: PR-created conversational handoff with URL-when-available and a
    // URL-less manual-handoff path, kept out of the PR body. (Single-line
    // substrings only, to stay robust against prose re-wrapping.)
    assert!(
        open.contains("PR-created conversational handoff")
            && open.contains("merge the PR in the provider UI")
            && open.contains("report that it merged so post-merge local cleanup can run"),
        "open.md must require a conversational merge-then-report next action after PR creation"
    );
    assert!(
        open.contains("Include the PR URL when one is available")
            && open.contains("on a URL-less manual"),
        "open.md must require the PR URL when available and handle the URL-less handoff path"
    );
    assert!(
        open.contains("never written into the PR body"),
        "open.md must keep the post-merge next action out of the PR body"
    );

    // router.md: contextual bare merge-report routing with disambiguation and a
    // no-candidate fall-through.
    assert!(
        router.contains("## Merge Report Routing"),
        "router must define contextual merge-report routing"
    );
    assert!(
        router.contains("intent examples, not fixed trigger strings"),
        "router merge-report examples must be intents, not fixed trigger strings"
    );
    assert!(
        router.contains("route to `commands/close.md`")
            && router.contains("ask exactly one disambiguation question"),
        "router must route a single candidate to close and disambiguate multiple candidates"
    );
    assert!(
        router.contains("no in-review or cleanup-pending candidate → do not route to cleanup")
            && router.contains("through to normal routing"),
        "router must fall through to normal routing when no candidate exists"
    );

    // close.md: conversational start + completion as post-merge local cleanup.
    assert!(
        close.contains("At close start") && close.contains("At close completion"),
        "close must describe conversational start and completion"
    );
    assert!(
        close.contains("the local feature branch and temporary delivery files were cleaned"),
        "close completion must report local branch and delivery-file cleanup"
    );

    // language.md: delivery next-action ownership (conversation vs artifact).
    assert!(
        language.contains("## Delivery Next Actions"),
        "language reference must own delivery next-action language policy"
    );
    assert!(
        language.contains("local cleanup pending")
            && language.contains("`[i18n].artifact_language` deterministically"),
        "language reference must cover the cleanup hint and the auto CLI fallback"
    );
    assert!(
        language.contains("into the PR body")
            && language.contains("intent examples, not fixed trigger strings"),
        "language reference must keep next actions out of the PR body and examples non-canonical"
    );
}

#[test]
fn discuss_branches_from_origin_with_stale_base_guard() {
    let discuss = read_repo_file("engine/commands/discuss.md");
    let git = read_repo_file("engine/reference/git.md");
    assert!(
        discuss.contains("from `origin/{[git].base_branch}`")
            && discuss.contains("never from a stale local base")
            && discuss.contains("is behind `origin/{[git].base_branch}`"),
        "discuss must branch from origin and warn on a stale local base"
    );
    assert!(
        git.contains("warns when the local base branch is behind")
            && git.contains("stale local base"),
        "git reference must document the stale-base guard"
    );
}

#[test]
fn open_ships_context_refresh_in_pr_before_accept() {
    // Change B: an open-detected coarse structural shift runs refresh-context
    // on the feature branch under human confirmation and ships the regenerated
    // context inside the PR as a separate docs(context) commit placed before the
    // accept close-out commit. Post-merge refresh is only the at/after-merge
    // fallback. No engine doc (body or frontmatter) may present post-merge
    // refresh as the primary path for open-detected staleness.
    let open = read_repo_file("engine/commands/open.md");
    let git = read_repo_file("engine/reference/git.md");
    let refresh = read_repo_file("engine/commands/refresh-context.md");
    let router = read_repo_file("engine/router.md");

    // open.md: in-branch refresh, separate docs(context) commit before accept.
    assert!(
        open.contains(
            "optional `docs(context)` commit → accept close-out → PR title/body → approve-PR →"
        ),
        "open.md (a)-(g) sequence must place the docs(context) commit before accept close-out"
    );
    assert!(
        open.contains("### (c) Context refresh commit (optional)"),
        "open.md must have a dedicated context-refresh commit step before accept"
    );
    assert!(
        open.contains(
            "and creates a separate `docs(context): ...` commit on the feature branch with"
        ),
        "open.md must commit the regenerated context as a separate docs(context) commit"
    );
    assert!(
        open.contains(
            "**after** the fold/context-check and **before** the accept close-out commit"
        ),
        "open.md must pin the docs(context) commit position before the accept close-out"
    );
    // Negative existence: the old post-merge-primary wording is gone.
    assert!(
        !open.contains("post-merge `refresh-context` follow-up after PR creation or")
            && !open.contains("Do **not** run or trigger `refresh-context` before the")
            && !open.contains("would dirty the tree before PR pre-flight"),
        "open.md must not present a post-merge refresh as the primary path"
    );

    // refresh-context.md: in-branch-before-PR is primary; no-auto-commit kept.
    assert!(
        refresh
            .contains("point) and runs this **on the feature branch, before the PR**, under human"),
        "refresh-context.md 'When it runs' must make the in-branch-before-PR path primary"
    );
    assert!(
        refresh.contains("This is the primary open-triggered"),
        "refresh-context.md must name the open-triggered in-branch path as primary"
    );
    assert!(
        refresh.contains(
            "detects a coarse structural shift, it runs this on the feature branch under"
        ),
        "refresh-context.md frontmatter description must describe the in-branch-before-PR path"
    );
    assert!(
        refresh.contains("Refresh does not auto-commit"),
        "refresh-context.md must keep its no-auto-commit contract (commit is open's)"
    );
    // Negative existence: old post-merge-primary wording removed (incl. frontmatter).
    assert!(
        !refresh.contains("trigger this during close-out")
            && !refresh.contains("after PR creation / merge")
            && !refresh.contains("as separate follow-up"),
        "refresh-context.md must not present a post-merge refresh as the primary path"
    );

    // git.md: in-PR primary path, preceding docs(context) commit, post-merge fallback.
    assert!(
        git.contains("human confirmation and ship the regenerated context **inside the PR** as a"),
        "git.md fold guidance must ship the context refresh inside the PR"
    );
    assert!(
        git.contains("before the PR is the primary path — never a post-merge base-branch edit."),
        "git.md must make the in-branch refresh primary and post-merge the fallback"
    );
    assert!(
        git.contains(
            "shift, it makes a separate `docs(context): ...` commit (regenerated `[context]`"
        ),
        "git.md auto-commit section must document the preceding docs(context) spec-lane commit"
    );
    assert!(
        git.contains("fold/context-check and **before** this close-out commit"),
        "git.md must position the docs(context) commit before the single accept close-out commit"
    );
    // Negative existence: old post-merge-primary wording removed.
    assert!(
        !git.contains("Context refresh is separate work after PR creation / merge"),
        "git.md must not present a post-merge refresh as the primary path"
    );

    // router.md: open summary mentions the optional docs(context) commit before accept.
    assert!(
        router.contains("optional `docs(context)` commit (regenerated `[context]`, before accept)"),
        "router.md open summary must mention the optional docs(context) commit before accept"
    );
}

#[test]
fn plan_offers_pre_approval_review_before_confirm_for_elevated() {
    // Change A: for risk >= elevated, the plan readiness card offers a
    // pre-approval Review (in the reviewer's plan-quality mode) before the
    // confirm-plan (approve-to-build) action; the standard-risk order is
    // unchanged (confirm as today, review only post-approval at step 10).
    let plan = read_repo_file("engine/commands/plan.md");

    assert!(
        plan.contains("When `risk >= elevated`: present **Review**"),
        "plan.md must offer Review in the readiness card for risk >= elevated"
    );
    assert!(
        plan.contains("**before** **Confirm the plan**"),
        "plan.md must order pre-approval Review before the confirm-plan action for risk >= elevated"
    );
    assert!(
        plan.contains("reviewer's plan-quality mode"),
        "plan.md pre-approval review must use the reviewer's plan-quality mode"
    );
    assert!(
        plan.contains("leave `spec.yaml` `status: draft`, make no plan commit"),
        "plan.md pre-approval review fail must leave status draft with no plan commit"
    );

    // Standard risk keeps the prior approve-then-optional-review order.
    assert!(
        plan.contains("When `risk = standard`: present **Confirm the plan** as today"),
        "plan.md must keep the standard-risk approve-then-review order unchanged"
    );
    assert!(
        plan.contains("**Review** (`risk = standard` only)"),
        "plan.md step 10 must keep post-approval Review for standard risk only"
    );

    // Positional guard: the elevated pre-approval Review offer precedes the
    // step-10 post-approval next-step card.
    let elevated_review = plan
        .find("When `risk >= elevated`: present **Review**")
        .expect("elevated pre-approval review offer present");
    let post_approval_card = plan
        .find("After the approved consistency check passes and the plan commit is created")
        .expect("step-10 post-approval card present");
    assert!(
        elevated_review < post_approval_card,
        "the pre-approval review offer must precede the post-approval next-step card"
    );
}

#[test]
fn accept_guidance_uses_cli_and_stages_spec_and_adr() {
    let open = read_repo_file("engine/commands/open.md");
    let git = read_repo_file("engine/reference/git.md");
    let workflow = read_repo_file("engine/reference/workflow.md");

    assert!(
        open.contains("mochiflow accept {slug}")
            && open.contains("git add {specs_dir}/{slug} {adr_record_paths...}")
            && open.contains("Never stage"),
        "open guidance must use the accept CLI close-out and never stage INDEX"
    );
    assert!(
        git.contains("Use `mochiflow accept {slug}`")
            && git.contains("git add {specs_dir}/{slug} {adr_record_paths...}")
            && git.contains("git diff --cached --name-status -z")
            && git.contains("never stage `INDEX.md`"),
        "git reference must document the flat accept staging and INDEX exclusion"
    );
    assert!(
        !git.contains("git mv {specs_dir}/{slug}/ {specs_dir}/_done/{slug}/` so both"),
        "git reference must not require staging a _done move"
    );
    assert!(
        workflow.contains("`mochiflow accept {slug}`") && workflow.contains("mechanical close-out"),
        "workflow must name the accept CLI as the accepted close-out mechanism"
    );
    assert!(
        !workflow.contains("there is no CLI transition command"),
        "workflow must not claim accepted has no CLI transition command"
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
fn build_commit_cadence_is_task_based_not_risk_based() {
    let risk = read_repo_file("engine/reference/risk.md");
    let build = read_repo_file("engine/commands/build.md");
    let git = read_repo_file("engine/reference/git.md");
    let concepts = read_repo_file("docs/concepts.md");

    assert!(
        risk.contains("Build commit cadence is task-based")
            && risk.contains("not by this risk table"),
        "risk reference must explicitly remove commit cadence from risk consequences"
    );
    assert!(
        !risk.contains("commit granularity")
            && !risk.contains("per logical step")
            && !risk.contains("| `standard` | none (AC Matrix only) | not written | 1 commit |"),
        "risk table must not define commit granularity by risk"
    );
    assert!(
        build.contains("the unit is one currently open task")
            && build.contains("taskless / micro specs")
            && build.contains("Normal build commits do not combine multiple task completions")
            && build.contains("docs(spec): record build verification")
            && build.contains("no `Task:` trailer"),
        "build command must define task-based commit units"
    );
    assert!(
        !build.contains("standard = one commit")
            && !build.contains("elevated = per logical step")
            && !build.contains("critical = per task"),
        "build command must not derive commit units from risk"
    );
    assert!(
        git.contains("Normal build commits complete one task")
            && git.contains("Multiple `Task:` lines are kept for compatibility"),
        "git reference must keep Task trailers while making one-task commits normal"
    );
    assert!(
        !concepts
            .contains("Riskier changes can require stricter review cadence and commit granularity"),
        "user docs must not say risk controls commit granularity"
    );
}

#[test]
fn spec_templates_require_done_eligible_matrix_results() {
    for path in [
        "engine/templates/spec/spec.md",
        "engine/templates/spec/spec.standard.md",
    ] {
        let template = read_repo_file(path);
        assert!(
            template.contains("done-eligible result token"),
            "{path} must describe the current Matrix completion rule"
        );
        assert!(
            !template.contains("with no `FAIL` result"),
            "{path} must not imply FAIL is the only non-final Matrix state"
        );
        assert!(
            template.contains("## Verification Plan / AC Matrix"),
            "{path} must keep the canonical Matrix heading in spec.md"
        );
        assert!(
            !template.contains("at the end of tasks.md"),
            "{path} must not offer tasks.md as a Matrix location"
        );
    }
    let tasks = read_repo_file("engine/templates/spec/tasks.md");
    assert!(
        tasks.contains("Task rows reference AC IDs; the AC Matrix belongs in `spec.md`."),
        "tasks.md template must point Matrix authors back to spec.md"
    );
    assert!(
        !tasks.contains("## Verification Plan / AC Matrix"),
        "tasks.md template must not include the canonical Matrix heading"
    );
    assert!(
        !tasks.contains("| AC | Scope | Verification method |"),
        "tasks.md template must not include a Matrix table"
    );
    assert!(
        !tasks.contains("AC Verification Matrix here"),
        "tasks.md template must not ask plan to create the Matrix there"
    );
    assert!(
        !tasks.contains("| AC-01 | {surface} | automated |"),
        "tasks.md template must not include a Matrix example row"
    );
    let git = read_repo_file("engine/reference/git.md");
    assert!(
        !git.contains("right after verification"),
        "no-PR close-out must be tied to ship acceptance, not raw verification"
    );
}

#[test]
fn discuss_persists_pitch_draft_spec() {
    let discuss = read_repo_file("engine/commands/discuss.md");
    let workflow = read_repo_file("engine/reference/workflow.md");
    let plan = read_repo_file("engine/commands/plan.md");
    let pitch = read_repo_file("engine/templates/spec/pitch.md");

    assert!(
        discuss.contains("{specs_dir}/{slug}/spec.yaml (status: draft)")
            && discuss.contains("{specs_dir}/{slug}/pitch.md")
            && discuss.contains("templates/spec/pitch.md")
            && discuss.contains("mochiflow lint --spec {slug}")
            && discuss.contains("docs(spec):"),
        "discuss must persist pitch + draft spec and commit them"
    );
    assert!(
        plan.contains("{specs_dir}/{slug}/pitch.md exists")
            && plan.contains("Read `{specs_dir}/{slug}/spec.yaml`")
            && plan.contains("`{specs_dir}/{slug}/pitch.md`")
            && plan.contains("raw `{specs_dir}/_backlog/{slug}.md`"),
        "plan must use pitch.md instead of ready-for-plan handoffs"
    );
    assert!(
        workflow.contains("Raw seed: `maturity: seed`")
            && workflow.contains("raw ideas only")
            && workflow.contains("{specs_dir}/{slug}/pitch.md")
            && !workflow.contains("Ready-for-plan handoff:"),
        "workflow must document seed-only backlog and pitch lifecycle"
    );
    for heading in [
        "## Problem",
        "## Appetite",
        "## Solution",
        "## Rabbit Holes",
        "## No-gos",
        "## Alternatives Considered",
        "## Open Questions",
    ] {
        assert!(pitch.contains(heading), "pitch template missing {heading}");
    }
}

#[test]
fn ac_matrix_pending_human_is_canonical_provisional_token() {
    let workflow = read_repo_file("engine/reference/workflow.md");
    let build = read_repo_file("engine/commands/build.md");
    let language = read_repo_file("engine/reference/language.md");
    let open = read_repo_file("engine/commands/open.md");

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
        open.contains("`CONFIRMED`"),
        "open round-trip protocol must map human confirmation to the canonical Matrix token"
    );
}

#[test]
fn behavioral_kiro_retires_spec_worker_agent_and_self_heals() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = materialize_full(tmp.path());
    let worker = tmp.path().join(".kiro/agents/spec-worker.json");
    let reviewer = tmp
        .path()
        .join(".kiro/agents/spec-independent-reviewer.json");

    std::fs::create_dir_all(worker.parent().unwrap()).unwrap();
    std::fs::write(
        &worker,
        "{\n  \"_generated\": \"generated by mochiflow adapter=kiro version=1.1.3\",\n  \"name\": \"spec-worker\"\n}\n",
    )
    .unwrap();

    let (code, out) = run_cli(&cfg, &["adapter", "generate"]);
    assert_eq!(code, 0, "{out}");
    assert!(
        reviewer.exists(),
        "the reviewer agent must still be generated"
    );
    assert!(
        !worker.exists(),
        "markered retired spec-worker.json must be self-healed away"
    );

    let (code, out) = run_cli(&cfg, &["adapter", "generate", "--check"]);
    assert_eq!(
        code, 0,
        "retired worker generation must be deterministic: {out}"
    );

    std::fs::write(&worker, "{\"name\":\"custom-worker\"}\n").unwrap();
    let (code, out) = run_cli(&cfg, &["adapter", "generate"]);
    assert_eq!(code, 0, "{out}");
    assert!(
        worker.exists(),
        "markerless spec-worker.json must be preserved as user-owned"
    );
}

#[test]
fn inline_rework_lifecycle_and_adapter_lifecycle_are_specified() {
    let git = read_repo_file("engine/reference/git.md");
    let open = read_repo_file("engine/commands/open.md");
    let update = read_repo_file("engine/commands/update.md");

    assert!(
        git.contains("PR-feedback fixes as bounded inline code changes"),
        "git.md must define update fixes as bounded inline code changes"
    );
    assert!(
        git.contains("no** `Task:` trailer") && git.contains("`Spec: {slug}` trailer"),
        "git.md must keep the rework commit convention without Task trailers"
    );
    assert!(
        open.contains("bounded inline code fix")
            && open.contains("do not re-run\n     build's phase-entry gate")
            && open.contains("no `Task:` trailer"),
        "open.md must define QA-FAIL rework as bounded inline fix"
    );
    assert!(
        update.contains("bounded inline PR-feedback fix")
            && update.contains("already `accepted`")
            && update.contains("never\n   reverted to `approved`"),
        "update.md must preserve accepted state and avoid build phase restart"
    );
    for body in [git.as_str(), open.as_str(), update.as_str()] {
        assert!(!body.contains("unit_kind"), "unit_kind must be retired");
        assert!(
            !body.contains("build worker mechanism"),
            "worker mechanism wording must be retired"
        );
    }

    for tpl in [
        "engine/adapters/kiro/steering/mochiflow.md.tpl",
        "engine/adapters/agents/AGENTS.md.tpl",
        "engine/adapters/claude-code/CLAUDE.md.tpl",
        "engine/adapters/copilot/copilot-instructions.md.tpl",
    ] {
        let body = read_repo_file(tpl);
        assert!(
            body.contains("draft → approved → accepted"),
            "{tpl} must use the draft -> approved -> accepted lifecycle"
        );
        assert!(
            !body.contains("draft → approved → done"),
            "{tpl} must not keep the stale draft -> approved -> done lifecycle"
        );
    }
}

#[test]
fn session_recoverability_is_authoring_rule_not_lint() {
    let plan = read_repo_file("engine/commands/plan.md");
    let authoring = read_repo_file("engine/reference/authoring.md");
    let reviewer = read_repo_file("engine/agents/independent-reviewer.md");

    assert!(
        authoring.contains("## Session-recoverability"),
        "authoring.md must define session-recoverability"
    );
    assert!(
        authoring
            .contains("`spec.md`, `design.md`, the task row, committed code, and git trailers"),
        "authoring.md must state the durable recoverability source set"
    );
    assert!(
        authoring.contains("more than one task's `Files`") && authoring.contains("`Done`"),
        "authoring.md must require shared-file Done to document shared-state handling"
    );
    assert!(
        authoring.contains("not a\nnew deterministic lint")
            || authoring.contains("not a new deterministic lint"),
        "authoring.md must state recoverability is not a new lint"
    );
    assert!(
        plan.contains("session-recoverable")
            && plan.contains("reference/authoring.md ## Session-recoverability"),
        "plan.md must reference the session-recoverability authoring rule"
    );
    assert!(
        reviewer.contains("session-recoverability"),
        "reviewer plan-quality mode must judge session-recoverability"
    );
    for body in [plan.as_str(), authoring.as_str(), reviewer.as_str()] {
        assert!(
            !body.contains("worker-recover"),
            "worker-recoverability wording must be gone"
        );
    }
}

#[test]
fn build_is_inline_and_review_transport_is_reviewer_only() {
    let build = read_repo_file("engine/commands/build.md");
    let router = read_repo_file("engine/router.md");
    let risk = read_repo_file("engine/reference/risk.md");

    assert!(
        build.contains("Implement an approved spec inline")
            && build.contains("Run the task loop inline"),
        "build.md must describe inline implementation"
    );
    assert!(
        build.contains("execution: inline") && build.contains("agents/independent-reviewer.md"),
        "build frontmatter must be inline and delegate only review"
    );
    assert!(
        !build.contains("agents/worker.md")
            && !build.contains("orchestrator")
            && !build.contains("compact report"),
        "build.md must not retain worker/orchestrator contract"
    );
    assert!(
        router.contains("judgment and implementation stay single-threaded; review may delegate"),
        "router principle 5 must state the new invariant"
    );
    let build_row = router
        .lines()
        .find(|l| l.starts_with("| build |"))
        .expect("router Verb Delegation build row exists");
    assert!(
        build_row.contains("inline") && !build_row.contains("worker"),
        "build row must be inline and worker-free: {build_row}"
    );
    assert!(
        risk.contains("independent-reviewer transport")
            && risk.contains("applies only to the read-only\n`agents/independent-reviewer.md`")
            && risk.contains("Build implementation itself is inline"),
        "risk.md transport must be reviewer-only"
    );
    assert!(
        !risk.contains("agents/worker.md")
            && !risk.contains("compact report")
            && !risk.contains("shared delegation transport"),
        "risk transport must not mention worker/shared build delegation"
    );
    assert!(
        risk.contains("| `elevated` | independent-reviewer once, after all tasks | optional |")
            && risk.contains("| `critical` | independent-reviewer after **each** task | required, appended per task |"),
        "the risk-cadence table must stay unchanged"
    );
}

#[test]
fn worker_role_and_template_are_retired() {
    let manifest = read_repo_file("engine/adapters/kiro/manifest.toml");
    let repo = repo_root();

    assert!(
        !repo.join("engine/agents/worker.md").exists(),
        "worker role doc must be removed"
    );
    assert!(
        !repo
            .join("engine/adapters/kiro/agents/spec-worker.json.tpl")
            .exists(),
        "kiro worker template must be removed"
    );
    assert!(
        !manifest.contains("spec-worker.json"),
        "kiro source manifest must not generate spec-worker.json"
    );
}

#[test]
fn kiro_docs_and_router_do_not_reference_retired_workers() {
    for path in ["README.md", "docs/configuration.md", "engine/router.md"] {
        let body = read_repo_file(path);
        assert!(
            !body.contains("build-worker") && !body.contains("write-capable"),
            "{path} must not describe retired Kiro worker agents"
        );
        assert!(
            !body.contains("`spec-builder` agent"),
            "{path} must not reference the retired Kiro spec-builder agent"
        );
    }

    for path in ["README.md", "docs/configuration.md"] {
        let body = read_repo_file(path);
        assert!(
            body.contains("read-only reviewer"),
            "{path} must describe Kiro's remaining generated reviewer"
        );
    }
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
fn workflow_todo_verify_is_not_runnable() {
    let workflow = read_repo_file("engine/reference/workflow.md");
    assert!(
        workflow.contains("`TODO:` placeholder is not yet runnable")
            && workflow.contains("define\nits command before building that surface"),
        "workflow must say TODO verification is not runnable before build"
    );
}

#[test]
fn no_pr_fast_path_skips_pr_gate_but_still_accepts() {
    let workflow = read_repo_file("engine/reference/workflow.md");
    let git = read_repo_file("engine/reference/git.md");
    let build = read_repo_file("engine/commands/build.md");

    assert!(
        workflow.contains("skips")
            && workflow.contains("**approve-PR**")
            && workflow.contains("still runs `accept`"),
        "workflow must describe the no-PR gate exception with accept"
    );
    assert!(
        git.contains("no-PR skips PR creation and the approve-PR gate")
            && git.contains("still runs `accept`"),
        "git reference must keep no-PR tied to accept"
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
        "status",
        "ready",
        "accept",
        "backlog",
        "upgrade",
        "freeze",
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
    assert!(
        readme.contains("verify a done-eligible Matrix")
            && readme.contains("linked ADR fold records"),
        "README.md must describe accept as a strict mechanical close-out"
    );
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
        "`CONFIRMED`",
        "`N/A: <reason>`",
        "`FAIL`",
        "`PENDING_HUMAN`",
    ] {
        assert!(
            language.contains(current),
            "language reference must list current AC result value {current}"
        );
    }
    // Deprecated aliases must remain present (for backward-compatibility documentation)
    for deprecated in ["`人間確認済み`", "`対象外（<reason>）`"] {
        assert!(
            language.contains(deprecated),
            "language reference must still mention deprecated alias {deprecated}"
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
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
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
        "## Defaults",
        "## Tasks",
        "- [ ] T-001 [AC-01]",
        "Depends on:",
        "Files:",
        "Done:",
        "Stop:",
    ] {
        assert!(tasks.contains(heading), "tasks template missing {heading}");
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

    // A `done` status is valid only for archived specs under `_done/`; place such
    // fixtures there so they exercise the archived-spec lint path (lint resolves
    // `--spec s` against both the active and the `_done/` location).
    let spec_dir = if spec_yaml.contains("status: done") {
        install.join("specs").join("_done").join("s")
    } else {
        install.join("specs").join("s")
    };
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

fn run_lint_case_with_optional_files(
    spec_yaml: &str,
    spec_md: Option<&str>,
    pitch_md: Option<&str>,
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

    // A `done` status is valid only for archived specs under `_done/`; route such
    // fixtures there so they exercise the archived-spec lint path.
    let spec_dir = if spec_yaml.contains("status: done") {
        install.join("specs").join("_done").join("s")
    } else {
        install.join("specs").join("s")
    };
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(spec_dir.join("spec.yaml"), spec_yaml).unwrap();
    if let Some(pitch) = pitch_md {
        std::fs::write(spec_dir.join("pitch.md"), pitch).unwrap();
    }
    if let Some(md) = spec_md {
        std::fs::write(spec_dir.join("spec.md"), md).unwrap();
    }
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

fn run_lint_case_with_dirty_file(
    spec_yaml: &str,
    spec_md: &str,
    tasks_md: &str,
    dirty_file: &str,
) -> (i32, String) {
    run_lint_case_with_dirty_file_state(
        spec_yaml,
        spec_md,
        tasks_md,
        dirty_file,
        DirtyFileState::Untracked,
    )
}

enum DirtyFileState {
    Untracked,
    Deleted,
}

fn run_lint_case_with_dirty_file_state(
    spec_yaml: &str,
    spec_md: &str,
    tasks_md: &str,
    dirty_file: &str,
    dirty_state: DirtyFileState,
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
    std::fs::write(spec_dir.join("tasks.md"), tasks_md).unwrap();

    let git = std::process::Command::new("git")
        .arg("init")
        .current_dir(tmp.path())
        .output()
        .unwrap();
    assert!(git.status.success(), "git init failed");

    let dirty_path = tmp.path().join(dirty_file);
    std::fs::create_dir_all(dirty_path.parent().unwrap()).unwrap();
    std::fs::write(&dirty_path, "dirty\n").unwrap();

    if matches!(dirty_state, DirtyFileState::Deleted) {
        let add = std::process::Command::new("git")
            .args(["add", dirty_file])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        assert!(add.status.success(), "git add failed");
        let commit = std::process::Command::new("git")
            .args([
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.com",
                "commit",
                "-m",
                "track dirty fixture",
            ])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        assert!(commit.status.success(), "git commit failed");
        std::fs::remove_file(dirty_path).unwrap();
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

/// Materialize a spec at the ACTIVE `specs/s` location (never `_done/`) and lint
/// it. Used to prove an active spec cannot carry `status: done`.
fn run_lint_case_active(spec_yaml: &str, spec_md: &str) -> (i32, String) {
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

    let out = assert_cmd::Command::cargo_bin("mochiflow")
        .unwrap()
        .args([
            "--config",
            install.join("config.toml").to_str().unwrap(),
            "lint",
            "--spec",
            install.join("specs").join("s").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

#[test]
fn lint_passes_archived_done_spec() {
    // Positive: a complete `done` spec under `_done/` lints clean (read-only
    // legacy compatibility). run_lint_case routes `status: done` to `_done/`.
    let yaml =
        GOOD_YAML.replace("status: approved", "status: done") + "completed: 2026-06-21T22:16:03Z\n";
    let (code, out) = run_lint_case(&yaml, DONE_MATRIX_MD, None, None);
    assert_eq!(code, 0, "archived done spec must lint clean: {out}");
}

#[test]
fn lint_rejects_done_on_active_spec() {
    // Negative: `status: done` on an active (non-`_done/`) spec is rejected.
    let yaml = GOOD_YAML.replace("status: approved", "status: done");
    let (code, out) = run_lint_case_active(&yaml, DONE_MATRIX_MD);
    assert_eq!(code, 1, "{out}");
    assert!(
        out.contains("status: done is reserved for archived specs under _done/"),
        "{out}"
    );
}

#[test]
fn lint_passes_valid_approved_spec() {
    let md = "# S\n\n## Requirements / Acceptance Criteria\n\n| AC | Type | Priority | Requirement | Verification |\n| --- | --- | --- | --- | --- |\n| AC-01 | functional | Must | THE SYSTEM SHALL do the thing. | automated |\n\n## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | UNVERIFIED |\n";
    let (code, _out) = run_lint_case(GOOD_YAML, md, None, None);
    assert_eq!(code, 0, "a well-formed approved spec must lint clean");
}

#[test]
fn lint_passes_pitch_only_draft_spec() {
    let yaml = "version: 1\nslug: s\ntitle: S\ntype: feature\nsurfaces:\n  - app\nintegration: workflow\nrisk: elevated\nstatus: draft\n";
    let pitch = "# S\n\n## Problem\n\nx\n\n## Appetite\n\nx\n\n## Solution\n\nx\n\n## Rabbit Holes\n\n- x\n\n## No-gos\n\n- x\n\n## Alternatives Considered\n\n- x\n\n## Open Questions\n\n- None.\n";
    let (code, out) = run_lint_case_with_optional_files(yaml, None, Some(pitch), None, None);
    assert_eq!(code, 0, "pitch-only draft must lint clean: {out}");
}

#[test]
fn lint_draft_requires_pitch() {
    let yaml = GOOD_YAML.replace("status: approved", "status: draft");
    let (code, out) = run_lint_case_with_optional_files(&yaml, None, None, None, None);
    assert_eq!(code, 1, "{out}");
    assert!(
        out.contains("pitch.md is required for draft status"),
        "{out}"
    );
}

#[test]
fn lint_expanded_draft_enforces_required_design() {
    let yaml = "version: 1\nslug: s\ntitle: S\ntype: feature\nsurfaces:\n  - app\nintegration: workflow\nrisk: elevated\nstatus: draft\n";
    let pitch = "# S\n\n## Problem\n\nx\n";
    let md = "# S\n\n## Acceptance Criteria\n\n- AC-01: THE SYSTEM SHALL x.\n";
    let (code, out) = run_lint_case_with_optional_files(yaml, Some(md), Some(pitch), None, None);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("design.md is required"), "{out}");
}

#[test]
fn lint_approved_requires_spec_md_even_with_pitch() {
    let pitch = "# S\n\n## Problem\n\nx\n";
    let (code, out) = run_lint_case_with_optional_files(GOOD_YAML, None, Some(pitch), None, None);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("spec.md is required"), "{out}");
}

const DONE_MATRIX_MD: &str = "# S\n\n## Acceptance Criteria\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
     ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | PASS |\n";

#[test]
fn lint_warns_when_done_spec_missing_completed() {
    let yaml = GOOD_YAML.replace("status: approved", "status: done");
    let (code, out) = run_lint_case(&yaml, DONE_MATRIX_MD, None, None);
    assert_eq!(code, 0, "missing completed is a WARN, not a FAIL: {out}");
    assert!(out.contains("`completed` timestamp is missing"), "{out}");
}

#[test]
fn lint_passes_done_spec_with_valid_completed() {
    let yaml =
        GOOD_YAML.replace("status: approved", "status: done") + "completed: 2026-06-21T22:16:03Z\n";
    let (code, out) = run_lint_case(&yaml, DONE_MATRIX_MD, None, None);
    assert_eq!(code, 0, "{out}");
    assert!(!out.contains("`completed` timestamp is missing"), "{out}");
}

#[test]
fn lint_fails_when_completed_is_malformed() {
    let yaml = GOOD_YAML.replace("status: approved", "status: done") + "completed: yesterday\n";
    let (code, out) = run_lint_case(&yaml, DONE_MATRIX_MD, None, None);
    assert_eq!(code, 1, "{out}");
    assert!(
        out.contains("must be an ISO 8601 date or timestamp"),
        "{out}"
    );
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
fn lint_done_ignores_matrix_heading_inside_comment() {
    let yaml = GOOD_YAML.replace("status: approved", "status: done");
    let md = "# S\n\n## 受入基準\n\n- AC-01: THE SYSTEM SHALL do the thing.\n\n\
              <!-- After implementation, append ## AC Verification Matrix here. -->\n";
    let (code, out) = run_lint_case(&yaml, md, None, None);
    assert_eq!(code, 1);
    assert!(out.contains("AC Verification Matrix is missing"), "{out}");
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
fn lint_done_passes_with_ascii_canonical_tokens() {
    let yaml = GOOD_YAML.replace("status: approved", "status: done");
    let md = "# S\n\n## 受入基準\n\n\
              - AC-01: THE SYSTEM SHALL x.\n\
              - AC-02: WHEN y, THE SYSTEM SHALL z.\n\
              - AC-03: WHERE q, THE SYSTEM SHALL r.\n\n\
              ## AC Verification Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | PASS |\n| AC-02 | CONFIRMED |\n| AC-03 | N/A: not relevant for CLI |\n";
    let (code, out) = run_lint_case(&yaml, md, None, None);
    assert_eq!(code, 0, "{out}");
}

#[test]
fn lint_error_message_shows_ascii_tokens_as_primary() {
    let yaml = GOOD_YAML.replace("status: approved", "status: done");
    let md = "# S\n\n## 受入基準\n\n\
              - AC-01: THE SYSTEM SHALL x.\n\n\
              ## AC Verification Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | BOGUS |\n";
    let (code, out) = run_lint_case(&yaml, md, None, None);
    assert_ne!(code, 0);
    assert!(
        out.contains("CONFIRMED") && out.contains("N/A: <reason>") && out.contains("also accepted"),
        "lint error must show ASCII tokens as primary and note deprecated aliases: {out}"
    );
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
fn lint_accepts_compound_task_ac_references() {
    let yaml = GOOD_YAML.replace("status: approved", "status: done");
    let md = "# S\n\n## Acceptance Criteria\n\n- AC-01: THE SYSTEM SHALL x.\n- AC-02: WHEN y, THE SYSTEM SHALL z.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | PASS |\n| AC-02 | PASS |\n";
    let tasks = "# Tasks\n\n- [x] T-001 [AC-01, AC-02] Do x and z\n  - Depends on: none\n  - Files:\n    - `src/x.rs`\n  - Done:\n    - [ ] Verification passed\n  - Stop:\n    - stop\n";
    let (code, out) = run_lint_case(&yaml, md, None, Some(tasks));
    assert_eq!(code, 0, "{out}");
}

#[test]
fn lint_rejects_unknown_ac_in_compound_task_reference() {
    let md = "# S\n\n## Acceptance Criteria\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | UNVERIFIED |\n";
    let tasks = "# Tasks\n\n- [ ] T-001 [AC-01, AC-02] Do x\n  - Depends on: none\n  - Files:\n    - `src/x.rs`\n  - Done:\n    - [ ] Verification passed\n  - Stop:\n    - stop\n";
    let (code, out) = run_lint_case(GOOD_YAML, md, None, Some(tasks));
    assert_eq!(code, 1);
    assert!(
        out.contains("tasks reference AC IDs not in spec.md: AC-02"),
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
fn lint_approved_warns_when_dirty_task_file_is_unchecked() {
    let md = "# S\n\n## Acceptance Criteria\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | UNVERIFIED |\n";
    let tasks = "# Tasks\n\n- [ ] T-001 [AC-01] Do x\n  - Depends on: none\n  - Files:\n    - `src/x.rs`\n  - Done:\n    - [ ] Verification passed\n  - Stop:\n    - stop\n";
    let (code, out) = run_lint_case_with_dirty_file(GOOD_YAML, md, tasks, "src/x.rs");
    assert_eq!(code, 0, "approved drift is a WARN, not a FAIL: {out}");
    assert!(
        out.contains("WARN:")
            && out.contains("task T-001 has modified Files entries and is not checked: src/x.rs"),
        "{out}"
    );
}

#[test]
fn lint_approved_does_not_warn_when_dirty_task_file_is_checked() {
    let md = "# S\n\n## Acceptance Criteria\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | UNVERIFIED |\n";
    let tasks = "# Tasks\n\n- [x] T-001 [AC-01] Do x\n  - Depends on: none\n  - Files:\n    - `src/x.rs`\n  - Done:\n    - [ ] Verification passed\n  - Stop:\n    - stop\n";
    let (code, out) = run_lint_case_with_dirty_file(GOOD_YAML, md, tasks, "src/x.rs");
    assert_eq!(code, 0, "{out}");
    assert!(
        !out.contains("modified Files entries and is not checked"),
        "{out}"
    );
}

#[test]
fn lint_approved_does_not_warn_when_deleted_task_file_is_marked_deleted() {
    let md = "# S\n\n## Acceptance Criteria\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | UNVERIFIED |\n";
    let tasks = "# Tasks\n\n- [ ] T-001 [AC-01] Do x\n  - Depends on: none\n  - Files:\n    - deleted: `src/x.rs`\n  - Done:\n    - [ ] Verification passed\n  - Stop:\n    - stop\n";
    let (code, out) = run_lint_case_with_dirty_file_state(
        GOOD_YAML,
        md,
        tasks,
        "src/x.rs",
        DirtyFileState::Deleted,
    );
    assert_eq!(code, 0, "{out}");
    assert!(
        !out.contains("modified Files entries and is not checked"),
        "{out}"
    );
}

#[test]
fn lint_approved_warns_when_deleted_marker_file_is_not_deleted() {
    let md = "# S\n\n## Acceptance Criteria\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | UNVERIFIED |\n";
    let tasks = "# Tasks\n\n- [ ] T-001 [AC-01] Do x\n  - Depends on: none\n  - Files:\n    - deleted: `src/x.rs`\n  - Done:\n    - [ ] Verification passed\n  - Stop:\n    - stop\n";
    let (code, out) = run_lint_case_with_dirty_file_state(
        GOOD_YAML,
        md,
        tasks,
        "src/x.rs",
        DirtyFileState::Untracked,
    );
    assert_eq!(code, 0, "approved drift is a WARN, not a FAIL: {out}");
    assert!(
        out.contains("WARN:")
            && out.contains("task T-001 has modified Files entries and is not checked: src/x.rs"),
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
fn lint_fails_on_template_residue_classes() {
    for (residue, expected) in [
        ("As {user}, I want x.\n", "unreplaced `{...}` placeholder"),
        ("<!-- Create this from the template. -->\n", "HTML comment"),
        (
            "| QA | Scope | Steps | Expected result |\n| --- | --- | --- | --- |\n| QA-01 | app | ... | ... |\n",
            "example-only table row",
        ),
        ("TBD\n", "bare `TBD`"),
    ] {
        let md = format!(
            "# S\n\n## Background\n\n{residue}\n## Acceptance Criteria\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
             ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | UNVERIFIED |\n"
        );
        let (code, out) = run_lint_case(GOOD_YAML, &md, None, None);
        assert_eq!(code, 1, "{residue}: {out}");
        assert!(out.contains(expected), "{residue}: {out}");
    }
}

#[test]
fn lint_ignores_template_like_text_in_code() {
    let md = "# S\n\n## Background\n\n\
              Inline code is allowed: `{user}` and `TBD`.\n\n\
              ```json\n{\"value\":\"{placeholder}\", \"status\":\"TBD\"}\n```\n\n\
              | AC | Result |\n| --- | --- |\n| `AC-00` | `...` |\n\n\
              ## Acceptance Criteria\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | UNVERIFIED |\n";
    let (code, out) = run_lint_case(GOOD_YAML, md, None, None);
    assert_eq!(code, 0, "{out}");
    assert!(!out.contains("template residue remains"), "{out}");
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
fn lint_accepts_multiline_ears_ac_blocks() {
    let md = "# S\n\n## Requirements / Acceptance Criteria\n\n\
              - AC-01: IF x happens,\n  THEN THE SYSTEM SHALL y.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | UNVERIFIED |\n";
    let (code, out) = run_lint_case(GOOD_YAML, md, None, None);
    assert_eq!(code, 0, "{out}");
    assert!(!out.contains("AC without EARS keyword"), "{out}");
}

#[test]
fn lint_multiline_ears_blocks_stop_at_next_ac() {
    let md = "# S\n\n## Requirements / Acceptance Criteria\n\n\
              - AC-01: x happens.\n\
              - AC-02: THE SYSTEM SHALL y.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | UNVERIFIED |\n| AC-02 | UNVERIFIED |\n";
    let (code, out) = run_lint_case(GOOD_YAML, md, None, None);
    assert_eq!(code, 0);
    assert!(
        out.contains("AC without EARS keyword") && out.contains("AC-01") && !out.contains("AC-02"),
        "{out}"
    );
}

#[test]
fn lint_multiline_ears_blocks_stop_at_next_heading() {
    let md = "# S\n\n## Requirements / Acceptance Criteria\n\n\
              - AC-01: x happens.\n\n\
              ## Notes\n\nTHE SYSTEM SHALL not count here.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | UNVERIFIED |\n";
    let (code, out) = run_lint_case(GOOD_YAML, md, None, None);
    assert_eq!(code, 0);
    assert!(
        out.contains("AC without EARS keyword") && out.contains("AC-01"),
        "{out}"
    );
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
fn lint_done_elevated_fails_with_only_fail_reviewer_verdict() {
    let yaml = GOOD_YAML
        .replace("status: approved", "status: done")
        .replace("risk: standard", "risk: elevated");
    let md = "# S\n\n## 受入基準\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
              ## AC Verification Matrix\n\n| AC | 結果 |\n| --- | --- |\n| AC-01 | PASS |\n";
    let design = "# design\n\n## Review Results\n\nReviewer mode: inline\nVerdict: fail\n";
    let (code, out) = run_lint_case(&yaml, md, Some(design), None);
    assert_eq!(code, 1);
    assert!(
        out.contains("latest Review Results reviewer Verdict is fail"),
        "{out}"
    );
    assert!(
        !out.contains("reviewer verdict (pass/pass-with-comments) is not recorded"),
        "{out}"
    );
}

#[test]
fn lint_done_elevated_uses_latest_reviewer_verdict() {
    let yaml = GOOD_YAML
        .replace("status: approved", "status: done")
        .replace("risk: standard", "risk: elevated");
    let md = "# S\n\n## 受入基準\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | PASS |\n";
    let fail_then_pass = "# design\n\n## Review Results\n\nReviewer mode: delegated\nVerdict: fail\n\nReviewer mode: delegated\nVerdict: pass\n";
    let (code, out) = run_lint_case(&yaml, md, Some(fail_then_pass), None);
    assert_eq!(code, 0, "{out}");

    let pass_then_fail = "# design\n\n## Review Results\n\nReviewer mode: delegated\nVerdict: pass\n\nReviewer mode: delegated\nVerdict: fail\n";
    let (code, out) = run_lint_case(&yaml, md, Some(pass_then_fail), None);
    assert_eq!(code, 1, "{out}");
    assert!(
        out.contains("latest Review Results reviewer Verdict is fail"),
        "{out}"
    );
}

#[test]
fn lint_done_critical_requires_passing_verdict_per_task() {
    let yaml = GOOD_YAML
        .replace("status: approved", "status: done")
        .replace("risk: standard", "risk: critical");
    let md = "# S\n\n## 受入基準\n\n- AC-01: THE SYSTEM SHALL x.\n- AC-02: THE SYSTEM SHALL y.\n";
    let tasks_one_verdict = "# Tasks\n\n## Task 1\n\nCovers AC: AC-01\n\n## Task 2\n\nCovers AC: AC-02\n\n\
                             ## AC Verification Matrix\n\n| AC | 結果 |\n| --- | --- |\n| AC-01 | PASS |\n| AC-02 | PASS |\n";
    let design_one = "# design\n\n## Review Results\n\nReviewer mode: inline\nVerdict: pass\n";
    let (code, out) = run_lint_case(&yaml, md, Some(design_one), Some(tasks_one_verdict));
    assert_eq!(code, 1);
    assert!(
        out.contains("critical risk requires at least 2 passing reviewer verdict"),
        "{out}"
    );

    let design_two = "# design\n\n## Review Results\n\nReviewer mode: inline\nVerdict: pass\n\nReviewer mode: inline\nVerdict: pass-with-comments\n";
    let (code, out) = run_lint_case(&yaml, md, Some(design_two), Some(tasks_one_verdict));
    assert_eq!(code, 0, "{out}");
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
        out.contains("status must be one of: draft, approved, accepted, done"),
        "{out}"
    );
}

const ACCEPTED_MATRIX_MD: &str = "# S\n\n## Acceptance Criteria\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
     ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | PASS |\n";

#[test]
fn lint_passes_accepted_spec() {
    let yaml = GOOD_YAML.replace("status: approved", "status: accepted");
    let (code, out) = run_lint_case(&yaml, ACCEPTED_MATRIX_MD, None, None);
    assert_eq!(
        code, 0,
        "a well-formed accepted spec must lint clean: {out}"
    );
}

#[test]
fn lint_accepted_does_not_warn_on_missing_completed() {
    // `accepted` never writes `completed`; the legacy completed WARN stays scoped
    // to `done` reads only and must not fire on an accepted spec.
    let yaml = GOOD_YAML.replace("status: approved", "status: accepted");
    let (code, out) = run_lint_case(&yaml, ACCEPTED_MATRIX_MD, None, None);
    assert_eq!(code, 0, "{out}");
    assert!(!out.contains("`completed` timestamp is missing"), "{out}");
}

#[test]
fn lint_accepted_fails_when_task_is_unchecked() {
    let yaml = GOOD_YAML.replace("status: approved", "status: accepted");
    let md = "# S\n\n## Acceptance Criteria\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | PASS |\n";
    let tasks = "# Tasks\n\n- [ ] T-001 [AC-01] Do x\n  - Depends on: none\n  - Files:\n    - `src/x.rs`\n  - Done:\n    - [ ] Verification passed\n  - Stop:\n    - stop\n";
    let (code, out) = run_lint_case(&yaml, md, None, Some(tasks));
    assert_eq!(code, 1, "{out}");
    assert!(
        out.contains("status is accepted but task T-001 is not checked"),
        "{out}"
    );
}

#[test]
fn lint_accepted_fails_when_ac_untasked() {
    let yaml = GOOD_YAML.replace("status: approved", "status: accepted");
    let md = "# S\n\n## Acceptance Criteria\n\n- AC-01: THE SYSTEM SHALL x.\n- AC-02: WHEN y, THE SYSTEM SHALL z.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | PASS |\n| AC-02 | PASS |\n";
    let tasks = "# Tasks\n\n- [x] T-001 [AC-01] Do x\n  - Depends on: none\n  - Files:\n    - `src/x.rs`\n  - Done:\n    - [ ] Verification passed\n  - Stop:\n    - stop\n";
    let (code, out) = run_lint_case(&yaml, md, None, Some(tasks));
    assert_eq!(code, 1, "{out}");
    assert!(
        out.contains("AC not covered by any task Covers AC: AC-02"),
        "{out}"
    );
}

#[test]
fn lint_accepted_requires_reviewer_verdict_when_elevated() {
    let yaml = "version: 1\nslug: s\ntitle: S\ntype: feature\nsurfaces:\n  - app\nintegration: none\nrisk: elevated\nstatus: accepted\n";
    let md = "# S\n\n## Acceptance Criteria\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | PASS |\n";
    let design = "# D\n\n## Review Results\n\n- Pending.\n";
    let (code, out) = run_lint_case(yaml, md, Some(design), None);
    assert_eq!(code, 1, "{out}");
    assert!(
        out.contains("reviewer verdict (pass/pass-with-comments) is not recorded"),
        "{out}"
    );
}

#[test]
fn lint_accepts_elevated_accepted_with_reviewer_verdict() {
    let yaml = "version: 1\nslug: s\ntitle: S\ntype: feature\nsurfaces:\n  - app\nintegration: none\nrisk: elevated\nstatus: accepted\n";
    let md = "# S\n\n## Acceptance Criteria\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
              ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | PASS |\n";
    let design = "# D\n\n## Review Results\n\n- Reviewer mode: delegated\n- Verdict: pass\n";
    let (code, out) = run_lint_case(yaml, md, Some(design), None);
    assert_eq!(code, 0, "{out}");
}

#[test]
fn lint_accepted_fails_with_non_final_matrix_results() {
    let yaml = GOOD_YAML.replace("status: approved", "status: accepted");
    for result in ["UNVERIFIED", "PENDING_HUMAN", "FAIL", ""] {
        let md = format!(
            "# S\n\n## Acceptance Criteria\n\n- AC-01: THE SYSTEM SHALL x.\n\n\
             ## Verification Plan / AC Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | {result} |\n"
        );
        let (code, out) = run_lint_case(&yaml, &md, None, None);
        assert_eq!(code, 1, "{result}: {out}");
        if result.is_empty() {
            assert!(out.contains("invalid result `<empty>`"), "{out}");
        } else {
            assert!(out.contains(result), "{out}");
        }
    }
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
        "---\nslug: sample-seed\ntitle: Sample Seed\nmaturity: seed\nsource: conversation\ncreated: 2026-03-10\nupdated: 2026-03-10\n---\n\n## Signal\n\nAn idea.\n\n## Why It Matters\n\nIt could help.\n\n## Evidence\n\n- Observed somewhere.\n\n## Open Questions\n\n- What scope?\n",
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
    run_cli_in_dir(config, Path::new("."), args)
}

fn run_cli_in_dir(config: &Path, cwd: &Path, args: &[&str]) -> (i32, String) {
    let mut full = vec!["--config", config.to_str().unwrap()];
    full.extend_from_slice(args);
    let out = assert_cmd::Command::cargo_bin("mochiflow")
        .unwrap()
        .current_dir(cwd)
        .args(&full)
        .output()
        .unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

fn run_cli_capture(config: &Path, cwd: &Path, args: &[&str]) -> (i32, String, String) {
    let mut full = vec!["--config", config.to_str().unwrap()];
    full.extend_from_slice(args);
    let out = assert_cmd::Command::cargo_bin("mochiflow")
        .unwrap()
        .current_dir(cwd)
        .args(&full)
        .output()
        .unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn git_ok(dir: &Path, args: &[&str]) {
    let status = Proc::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?} failed in {}", dir.display());
}

fn git_out(dir: &Path, args: &[&str]) -> String {
    let out = Proc::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?} failed in {}",
        dir.display()
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn materialize_ship_repo(root: &Path, slug: &str, specs_dir: &str) -> (PathBuf, PathBuf) {
    let bare = root.join("origin.git");
    std::fs::create_dir_all(&bare).unwrap();
    git_ok(&bare, &["init", "--bare", "-q"]);

    let repo = root.join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    git_ok(&repo, &["init", "-q", "-b", "main"]);
    git_ok(&repo, &["config", "user.email", "t@example.com"]);
    git_ok(&repo, &["config", "user.name", "Test"]);

    let install = repo.join(".mochiflow");
    let specs = repo.join(specs_dir);
    std::fs::create_dir_all(specs.join(slug)).unwrap();
    std::fs::create_dir_all(install.join("context")).unwrap();
    std::fs::create_dir_all(install.join("adr/decisions")).unwrap();
    std::fs::create_dir_all(install.join("adr/pitfalls")).unwrap();
    std::fs::write(
        repo.join(".gitignore"),
        ".mochiflow/state/\n.mochiflow/INDEX.md\n.mochiflow/adr/**/INDEX.md\n",
    )
    .unwrap();
    for name in ["constitution.md", "constitution.local.md"] {
        std::fs::write(install.join(name), "# c\n").unwrap();
    }
    for name in ["product.md", "structure.md", "tech.md"] {
        std::fs::write(install.join("context").join(name), "# c\n").unwrap();
    }
    std::fs::write(
        install.join("config.toml"),
        format!(
            "schema_version = 1\n\
             install_dir = \".mochiflow\"\n\
             specs_dir = \"{specs_dir}\"\n\
             index = \".mochiflow/INDEX.md\"\n\n\
             [i18n]\nartifact_language = \"en\"\nconversation_language = \"auto\"\n\n\
             [constitution]\nproject = \".mochiflow/constitution.md\"\nlocal = \".mochiflow/constitution.local.md\"\n\n\
             [context]\nproduct = \".mochiflow/context/product.md\"\nstructure = \".mochiflow/context/structure.md\"\ntech = \".mochiflow/context/tech.md\"\n\n\
             [adr]\ndecisions = \".mochiflow/adr/decisions\"\npitfalls = \".mochiflow/adr/pitfalls\"\n\n\
             [git]\nprovider = \"none\"\nbase_branch = \"main\"\n\n\
             [adapter]\ntool = \"agents\"\n\n\
             [surfaces.app]\ndescription = \"app\"\n\n[surfaces.app.verify]\ndefault = \"echo test-pass\"\n"
        ),
    )
    .unwrap();
    write_active_ship_spec(&specs.join(slug), slug);
    git_ok(&repo, &["add", "."]);
    git_ok(&repo, &["commit", "-q", "-m", "init"]);
    git_ok(&repo, &["remote", "add", "origin", bare.to_str().unwrap()]);
    git_ok(&repo, &["push", "-q", "-u", "origin", "main"]);
    git_ok(&repo, &["checkout", "-q", "-b", &format!("fix/{slug}")]);
    (install.join("config.toml"), repo)
}

fn write_active_ship_spec(spec_dir: &Path, slug: &str) {
    std::fs::create_dir_all(spec_dir).unwrap();
    std::fs::write(
        spec_dir.join("spec.yaml"),
        format!(
            "version: 1\nslug: {slug}\ntitle: Ship Fixture\ntype: fix\nsurfaces:\n  - app\nintegration: none\nrisk: standard\nstatus: approved\nupdated: \"2026-06-27\"\n"
        ),
    )
    .unwrap();
    std::fs::write(spec_dir.join("pitch.md"), "# Pitch\n").unwrap();
    std::fs::write(
        spec_dir.join("spec.md"),
        "# Ship Fixture\n\n## Acceptance Criteria (EARS)\n\n- AC-01: WHEN shipping, THE SYSTEM SHALL finish safely.\n\n## Verification Plan / AC Matrix\n\n| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |\n| --- | --- | --- | --- | --- | --- | --- | --- |\n| AC-01 | app | automated | final verify | fixture | PASS | behavioral fixture assertion |  |\n",
    )
    .unwrap();
}

fn write_ship_matrix_row(
    repo: &Path,
    slug: &str,
    scope: &str,
    method: &str,
    result: &str,
    evidence: &str,
) {
    let spec_md = repo.join(format!(".mochiflow/specs/{slug}/spec.md"));
    let text = std::fs::read_to_string(&spec_md).unwrap();
    let replacement = format!(
        "| AC-01 | {scope} | {method} | final verify | fixture | {result} | {evidence} |  |"
    );
    let lines: Vec<_> = text
        .lines()
        .map(|line| {
            if line.starts_with("| AC-01 |") {
                replacement.clone()
            } else {
                line.to_string()
            }
        })
        .collect();
    std::fs::write(&spec_md, lines.join("\n") + "\n").unwrap();
}

fn write_decision_record(repo: &Path, id: &str, status: &str, spec: Option<&str>, links: &str) {
    let spec_line = spec
        .map(|slug| format!("spec: {slug}\n"))
        .unwrap_or_default();
    std::fs::write(
        repo.join(format!(".mochiflow/adr/decisions/{id}.md")),
        format!(
            "---\n\
             id: {id}\n\
             date: 2026-06-27\n\
             area: [app]\n\
             {spec_line}\
             status: {status}\n\
             {links}\
             ---\n\
             ## {id}\n"
        ),
    )
    .unwrap();
}

#[test]
fn behavioral_accept_commits_flat_spec_with_safe_paths() {
    let tmp = tempfile::tempdir().unwrap();
    let slug = "accept-fixture";
    let (cfg, repo) = materialize_ship_repo(tmp.path(), slug, ".mochiflow/specs");

    let (code, out, err) = run_cli_capture(&cfg, &repo, &["accept"]);
    assert_eq!(code, 0, "stdout:\n{out}\nstderr:\n{err}");
    // The spec stays flat: no `_done/` move.
    assert!(repo.join(format!(".mochiflow/specs/{slug}")).exists());
    assert!(!repo.join(format!(".mochiflow/specs/_done/{slug}")).exists());

    let yaml =
        std::fs::read_to_string(repo.join(format!(".mochiflow/specs/{slug}/spec.yaml"))).unwrap();
    assert!(yaml.contains("status: \"accepted\""), "{yaml}");
    assert!(
        !yaml.contains("status: \"done\"") && !yaml.contains("completed:"),
        "accept must not write done or completed: {yaml}"
    );
    let spec =
        std::fs::read_to_string(repo.join(format!(".mochiflow/specs/{slug}/spec.md"))).unwrap();
    assert!(spec.contains("| AC-01 | app | automated"));
    assert!(spec.contains(
        "| PASS | behavioral fixture assertion<br>final verification: `echo test-pass` |"
    ));

    let message = git_out(&repo, &["show", "-s", "--format=%B", "HEAD"]);
    assert!(
        message.starts_with("fix: complete delivery record"),
        "{message}"
    );
    assert!(message.contains(&format!("Spec: {slug}")), "{message}");
    let name_status = git_out(
        &repo,
        &["diff-tree", "--no-commit-id", "--name-status", "-r", "HEAD"],
    );
    assert!(
        name_status.contains(&format!(".mochiflow/specs/{slug}/spec.yaml")),
        "{name_status}"
    );
    // Neither the gitignored board nor its state file is ever staged/committed,
    // and no _done move is staged.
    assert!(!name_status.contains("INDEX.md"), "{name_status}");
    assert!(
        !name_status.contains(".mochiflow/state/index.json"),
        "{name_status}"
    );
    assert!(
        !name_status.contains("_done/"),
        "accept must not stage any _done move: {name_status}"
    );
    let status = git_out(&repo, &["status", "--short"]);
    assert_eq!(status, "", "{status}");
}

#[test]
fn behavioral_accept_does_not_duplicate_final_verification_evidence() {
    let tmp = tempfile::tempdir().unwrap();
    let slug = "duplicate-final-evidence-fixture";
    let (cfg, repo) = materialize_ship_repo(tmp.path(), slug, ".mochiflow/specs");
    write_ship_matrix_row(
        &repo,
        slug,
        "app",
        "automated",
        "PASS",
        "behavioral fixture assertion<br>final verification: `echo test-pass`",
    );

    let (code, out, err) = run_cli_capture(&cfg, &repo, &["accept", slug]);
    assert_eq!(code, 0, "stdout:\n{out}\nstderr:\n{err}");
    let spec =
        std::fs::read_to_string(repo.join(format!(".mochiflow/specs/{slug}/spec.md"))).unwrap();
    assert_eq!(
        spec.matches("final verification: `echo test-pass`").count(),
        1,
        "{spec}"
    );
}

#[test]
fn behavioral_accept_does_not_update_confirmed_or_not_applicable_rows() {
    for (slug, result, evidence) in [
        (
            "confirmed-row-fixture",
            "CONFIRMED",
            "human confirmed on simulator",
        ),
        ("na-row-fixture", "N/A: CLI only", "not applicable evidence"),
    ] {
        let tmp = tempfile::tempdir().unwrap();
        let (cfg, repo) = materialize_ship_repo(tmp.path(), slug, ".mochiflow/specs");
        write_ship_matrix_row(&repo, slug, "app", "automated", result, evidence);

        let (code, out, err) = run_cli_capture(&cfg, &repo, &["accept", slug]);
        assert_eq!(code, 0, "{slug} stdout:\n{out}\nstderr:\n{err}");
        let spec =
            std::fs::read_to_string(repo.join(format!(".mochiflow/specs/{slug}/spec.md"))).unwrap();
        assert!(
            spec.contains(&format!("| {result} | {evidence} |")),
            "{spec}"
        );
        assert!(!spec.contains("final verification:"), "{spec}");
    }
}

#[test]
fn behavioral_accept_commits_only_target_adr_records_and_reciprocals() {
    let tmp = tempfile::tempdir().unwrap();
    let slug = "adr-fold-fixture";
    let (cfg, repo) = materialize_ship_repo(tmp.path(), slug, ".mochiflow/specs");

    write_decision_record(&repo, "2026-06-27-unrelated", "active", Some("other"), "");
    git_ok(
        &repo,
        &["add", ".mochiflow/adr/decisions/2026-06-27-unrelated.md"],
    );
    git_ok(&repo, &["commit", "-q", "-m", "test: unrelated adr"]);

    write_decision_record(
        &repo,
        "2026-06-27-target",
        "active",
        Some(slug),
        "supersedes: 2026-06-27-old\n",
    );
    write_decision_record(
        &repo,
        "2026-06-27-old",
        "superseded",
        None,
        "superseded_by: 2026-06-27-target\n",
    );
    std::fs::write(
        repo.join(".mochiflow/adr/decisions/INDEX.md"),
        "# generated\n",
    )
    .unwrap();

    let (code, out, err) = run_cli_capture(&cfg, &repo, &["accept", slug]);
    assert_eq!(code, 0, "stdout:\n{out}\nstderr:\n{err}");
    let name_status = git_out(
        &repo,
        &["diff-tree", "--no-commit-id", "--name-status", "-r", "HEAD"],
    );
    assert!(
        name_status.contains(".mochiflow/adr/decisions/2026-06-27-target.md"),
        "{name_status}"
    );
    assert!(
        name_status.contains(".mochiflow/adr/decisions/2026-06-27-old.md"),
        "{name_status}"
    );
    assert!(
        !name_status.contains(".mochiflow/adr/decisions/2026-06-27-unrelated.md"),
        "{name_status}"
    );
    assert!(!name_status.contains("INDEX.md"), "{name_status}");
}

#[test]
fn behavioral_accept_rejects_dirty_unlinked_adr_before_mutation() {
    let tmp = tempfile::tempdir().unwrap();
    let slug = "dirty-adr-fixture";
    let (cfg, repo) = materialize_ship_repo(tmp.path(), slug, ".mochiflow/specs");
    write_decision_record(&repo, "2026-06-27-other", "active", Some("other"), "");
    let before_yaml =
        std::fs::read_to_string(repo.join(format!(".mochiflow/specs/{slug}/spec.yaml"))).unwrap();

    let (code, out, err) = run_cli_capture(&cfg, &repo, &["accept", slug]);
    assert_eq!(code, 1, "stdout:\n{out}\nstderr:\n{err}");
    assert!(
        err.contains(".mochiflow/adr/decisions/2026-06-27-other.md"),
        "{err}"
    );
    let after_yaml =
        std::fs::read_to_string(repo.join(format!(".mochiflow/specs/{slug}/spec.yaml"))).unwrap();
    assert_eq!(after_yaml, before_yaml);
}

#[test]
fn behavioral_accept_rejects_dirty_adr_without_spec_before_mutation() {
    let tmp = tempfile::tempdir().unwrap();
    let slug = "adr-no-spec-fixture";
    let (cfg, repo) = materialize_ship_repo(tmp.path(), slug, ".mochiflow/specs");
    write_decision_record(&repo, "2026-06-27-no-spec", "active", None, "");
    let before_yaml =
        std::fs::read_to_string(repo.join(format!(".mochiflow/specs/{slug}/spec.yaml"))).unwrap();

    let (code, out, err) = run_cli_capture(&cfg, &repo, &["accept", slug]);
    assert_eq!(code, 1, "stdout:\n{out}\nstderr:\n{err}");
    assert!(
        err.contains(".mochiflow/adr/decisions/2026-06-27-no-spec.md"),
        "{err}"
    );
    let after_yaml =
        std::fs::read_to_string(repo.join(format!(".mochiflow/specs/{slug}/spec.yaml"))).unwrap();
    assert_eq!(after_yaml, before_yaml);
}

#[test]
fn behavioral_accept_rejects_dirty_unparseable_adr_before_mutation() {
    let tmp = tempfile::tempdir().unwrap();
    let slug = "bad-adr-fixture";
    let (cfg, repo) = materialize_ship_repo(tmp.path(), slug, ".mochiflow/specs");
    std::fs::write(
        repo.join(".mochiflow/adr/decisions/2026-06-27-bad.md"),
        "missing front matter\n",
    )
    .unwrap();
    let before_yaml =
        std::fs::read_to_string(repo.join(format!(".mochiflow/specs/{slug}/spec.yaml"))).unwrap();

    let (code, out, err) = run_cli_capture(&cfg, &repo, &["accept", slug]);
    assert_eq!(code, 1, "stdout:\n{out}\nstderr:\n{err}");
    assert!(
        err.contains("ADR record cannot be parsed")
            && err.contains(".mochiflow/adr/decisions/2026-06-27-bad.md"),
        "{err}"
    );
    let after_yaml =
        std::fs::read_to_string(repo.join(format!(".mochiflow/specs/{slug}/spec.yaml"))).unwrap();
    assert_eq!(after_yaml, before_yaml);
}

#[test]
fn behavioral_accept_rejects_legacy_adr_file_config_before_mutation() {
    let tmp = tempfile::tempdir().unwrap();
    let slug = "legacy-adr-config-fixture";
    let (cfg, repo) = materialize_ship_repo(tmp.path(), slug, ".mochiflow/specs");
    std::fs::write(repo.join(".mochiflow/adr/decisions.md"), "# legacy\n").unwrap();
    std::fs::write(repo.join(".mochiflow/adr/pitfalls.md"), "# legacy\n").unwrap();
    let config = std::fs::read_to_string(&cfg).unwrap().replace(
        "decisions = \".mochiflow/adr/decisions\"\npitfalls = \".mochiflow/adr/pitfalls\"",
        "decisions = \".mochiflow/adr/decisions.md\"\npitfalls = \".mochiflow/adr/pitfalls.md\"",
    );
    std::fs::write(&cfg, config).unwrap();
    git_ok(&repo, &["add", ".mochiflow/config.toml", ".mochiflow/adr"]);
    git_ok(&repo, &["commit", "-q", "-m", "test: legacy adr config"]);
    let before_yaml =
        std::fs::read_to_string(repo.join(format!(".mochiflow/specs/{slug}/spec.yaml"))).unwrap();

    let (code, out, err) = run_cli_capture(&cfg, &repo, &["accept", slug]);
    assert_eq!(code, 1, "stdout:\n{out}\nstderr:\n{err}");
    assert!(err.contains("must resolve to a record directory"), "{err}");
    let after_yaml =
        std::fs::read_to_string(repo.join(format!(".mochiflow/specs/{slug}/spec.yaml"))).unwrap();
    assert_eq!(after_yaml, before_yaml);
}

#[test]
fn behavioral_accept_does_not_modify_legacy_done_specs() {
    let tmp = tempfile::tempdir().unwrap();
    let slug = "legacy-active-fixture";
    let legacy_slug = "legacy-done-fixture";
    let (cfg, repo) = materialize_ship_repo(tmp.path(), slug, ".mochiflow/specs");
    write_done_spec(&repo.join(".mochiflow/specs"), legacy_slug);
    let legacy_dir = repo.join(format!(".mochiflow/specs/_done/{legacy_slug}"));
    let legacy_yaml = legacy_dir.join("spec.yaml");
    let legacy_spec = legacy_dir.join("spec.md");
    let before_yaml = std::fs::read(&legacy_yaml).unwrap();
    let before_spec = std::fs::read(&legacy_spec).unwrap();
    assert!(
        !String::from_utf8_lossy(&before_yaml).contains("completed:"),
        "fixture must model legacy _done metadata without completed"
    );
    git_ok(
        &repo,
        &[
            "add",
            legacy_yaml.to_str().unwrap(),
            legacy_spec.to_str().unwrap(),
        ],
    );
    git_ok(&repo, &["commit", "-q", "-m", "test: legacy done fixture"]);

    let (code, out, err) = run_cli_capture(&cfg, &repo, &["accept", slug]);
    assert_eq!(code, 0, "stdout:\n{out}\nstderr:\n{err}");
    assert_eq!(std::fs::read(&legacy_yaml).unwrap(), before_yaml);
    assert_eq!(std::fs::read(&legacy_spec).unwrap(), before_spec);
    let name_status = git_out(
        &repo,
        &["diff-tree", "--no-commit-id", "--name-status", "-r", "HEAD"],
    );
    assert!(
        !name_status.contains(&format!(".mochiflow/specs/_done/{legacy_slug}/")),
        "{name_status}"
    );
}

#[test]
fn behavioral_accept_dry_run_does_not_mutate_or_stage() {
    let tmp = tempfile::tempdir().unwrap();
    let slug = "dry-run-fixture";
    let (cfg, repo) = materialize_ship_repo(tmp.path(), slug, ".mochiflow/specs");

    let (code, out, err) = run_cli_capture(&cfg, &repo, &["accept", slug, "--dry-run"]);
    assert_eq!(code, 0, "stdout:\n{out}\nstderr:\n{err}");
    assert!(out.contains("dry-run: no verification"), "{out}");
    assert!(repo.join(format!(".mochiflow/specs/{slug}")).exists());
    assert!(!repo.join(format!(".mochiflow/specs/_done/{slug}")).exists());
    assert!(!repo.join(".mochiflow/INDEX.md").exists());
    let status = git_out(&repo, &["status", "--short"]);
    assert_eq!(status, "", "{status}");
}

#[test]
fn behavioral_accept_dry_run_reports_unverified_without_mutation() {
    let tmp = tempfile::tempdir().unwrap();
    let slug = "dry-run-unverified-fixture";
    let (cfg, repo) = materialize_ship_repo(tmp.path(), slug, ".mochiflow/specs");
    write_ship_matrix_row(
        &repo,
        slug,
        "app",
        "automated",
        "UNVERIFIED",
        "behavioral fixture assertion",
    );
    let spec_md = repo.join(format!(".mochiflow/specs/{slug}/spec.md"));
    let spec_yaml = repo.join(format!(".mochiflow/specs/{slug}/spec.yaml"));
    let before_spec = std::fs::read_to_string(&spec_md).unwrap();
    let before_yaml = std::fs::read_to_string(&spec_yaml).unwrap();

    let (code, out, err) = run_cli_capture(&cfg, &repo, &["accept", slug, "--dry-run"]);
    assert_eq!(code, 1, "stdout:\n{out}\nstderr:\n{err}");
    assert!(out.contains("UNVERIFIED"), "{out}");
    assert_eq!(std::fs::read_to_string(&spec_md).unwrap(), before_spec);
    assert_eq!(std::fs::read_to_string(&spec_yaml).unwrap(), before_yaml);
    let staged = git_out(&repo, &["diff", "--cached", "--name-only"]);
    assert_eq!(staged, "", "{staged}");
}

#[test]
fn behavioral_accept_rejects_unrelated_staged_path_with_spaces() {
    let tmp = tempfile::tempdir().unwrap();
    let slug = "dirty-fixture";
    let (cfg, repo) = materialize_ship_repo(tmp.path(), slug, ".mochiflow/specs");
    std::fs::write(repo.join("unrelated file.txt"), "x\n").unwrap();
    git_ok(&repo, &["add", "unrelated file.txt"]);

    let (code, out, err) = run_cli_capture(&cfg, &repo, &["accept", slug]);
    assert_eq!(code, 1, "stdout:\n{out}\nstderr:\n{err}");
    assert!(err.contains("unrelated file.txt"), "{err}");
    assert!(repo.join(format!(".mochiflow/specs/{slug}")).exists());
    assert!(!repo.join(format!(".mochiflow/specs/_done/{slug}")).exists());
}

#[test]
fn behavioral_accept_rejects_unverified_with_evidence_before_mutation() {
    let tmp = tempfile::tempdir().unwrap();
    let slug = "unverified-evidence-fixture";
    let (cfg, repo) = materialize_ship_repo(tmp.path(), slug, ".mochiflow/specs");
    let config = std::fs::read_to_string(&cfg).unwrap().replace(
        "default = \"echo test-pass\"",
        "default = \"touch verification-ran\"",
    );
    std::fs::write(&cfg, config).unwrap();
    git_ok(&repo, &["add", ".mochiflow/config.toml"]);
    git_ok(&repo, &["commit", "-q", "-m", "test: observable verify"]);
    write_ship_matrix_row(
        &repo,
        slug,
        "app",
        "automated",
        "UNVERIFIED",
        "behavioral fixture assertion",
    );
    let spec_md = repo.join(format!(".mochiflow/specs/{slug}/spec.md"));
    let spec_yaml = repo.join(format!(".mochiflow/specs/{slug}/spec.yaml"));
    let before_spec = std::fs::read_to_string(&spec_md).unwrap();
    let before_yaml = std::fs::read_to_string(&spec_yaml).unwrap();
    let before_head = git_out(&repo, &["rev-parse", "HEAD"]);

    let (code, out, err) = run_cli_capture(&cfg, &repo, &["accept", slug]);
    assert_eq!(code, 1, "stdout:\n{out}\nstderr:\n{err}");
    assert!(err.contains("UNVERIFIED"), "{err}");
    assert_eq!(std::fs::read_to_string(&spec_md).unwrap(), before_spec);
    assert_eq!(std::fs::read_to_string(&spec_yaml).unwrap(), before_yaml);
    assert_eq!(git_out(&repo, &["rev-parse", "HEAD"]), before_head);
    assert!(!repo.join("verification-ran").exists());
}

#[test]
fn behavioral_accept_stops_before_mutation_when_verification_cannot_pass() {
    let tmp = tempfile::tempdir().unwrap();
    let slug = "verify-fail-fixture";
    let (cfg, repo) = materialize_ship_repo(tmp.path(), slug, ".mochiflow/specs");
    let config = std::fs::read_to_string(&cfg)
        .unwrap()
        .replace("default = \"echo test-pass\"", "default = \"false\"");
    std::fs::write(&cfg, config).unwrap();
    git_ok(&repo, &["add", ".mochiflow/config.toml"]);
    git_ok(
        &repo,
        &["commit", "-q", "-m", "test: failing verify fixture"],
    );

    let (code, out, err) = run_cli_capture(&cfg, &repo, &["accept", slug]);
    assert_eq!(code, 1, "stdout:\n{out}\nstderr:\n{err}");
    assert!(err.contains("verification failed"), "{err}");
    assert!(repo.join(format!(".mochiflow/specs/{slug}")).exists());
    assert!(!repo.join(format!(".mochiflow/specs/_done/{slug}")).exists());

    let config = std::fs::read_to_string(&cfg).unwrap().replace(
        "default = \"false\"",
        "default = \"TODO: define test command\"",
    );
    std::fs::write(&cfg, config).unwrap();
    git_ok(&repo, &["add", ".mochiflow/config.toml"]);
    git_ok(&repo, &["commit", "-q", "-m", "test: todo verify fixture"]);
    let (code, out, err) = run_cli_capture(&cfg, &repo, &["accept", slug]);
    assert_eq!(code, 1, "stdout:\n{out}\nstderr:\n{err}");
    assert!(err.contains("not runnable"), "{err}");
    assert!(repo.join(format!(".mochiflow/specs/{slug}")).exists());

    let config = std::fs::read_to_string(&cfg)
        .unwrap()
        .replace("default = \"TODO: define test command\"\n", "");
    std::fs::write(&cfg, config).unwrap();
    git_ok(&repo, &["add", ".mochiflow/config.toml"]);
    git_ok(
        &repo,
        &["commit", "-q", "-m", "test: missing verify fixture"],
    );
    let (code, out, err) = run_cli_capture(&cfg, &repo, &["accept", slug]);
    assert_eq!(code, 1, "stdout:\n{out}\nstderr:\n{err}");
    assert!(err.contains("has no verify profile"), "{err}");
    assert!(repo.join(format!(".mochiflow/specs/{slug}")).exists());
}

#[test]
fn behavioral_ship_stops_before_mutation_for_pending_human_matrix_row() {
    let tmp = tempfile::tempdir().unwrap();
    let slug = "pending-human-fixture";
    let (cfg, repo) = materialize_ship_repo(tmp.path(), slug, ".mochiflow/specs");
    write_ship_matrix_row(&repo, slug, "app", "automated", "PENDING_HUMAN", "needs QA");

    let (code, out, err) = run_cli_capture(&cfg, &repo, &["accept", slug]);
    assert_eq!(code, 1, "stdout:\n{out}\nstderr:\n{err}");
    assert!(err.contains("PENDING_HUMAN"), "{err}");
    assert!(repo.join(format!(".mochiflow/specs/{slug}")).exists());
    assert!(!repo.join(format!(".mochiflow/specs/_done/{slug}")).exists());
}

#[test]
fn behavioral_ship_stops_before_mutation_for_generic_only_matrix_evidence() {
    let tmp = tempfile::tempdir().unwrap();
    let slug = "generic-evidence-fixture";
    let (cfg, repo) = materialize_ship_repo(tmp.path(), slug, ".mochiflow/specs");
    write_ship_matrix_row(&repo, slug, "app", "automated", "UNVERIFIED", "");

    let (code, out, err) = run_cli_capture(&cfg, &repo, &["accept", slug]);
    assert_eq!(code, 1, "stdout:\n{out}\nstderr:\n{err}");
    assert!(err.contains("UNVERIFIED"), "{err}");
    assert!(repo.join(format!(".mochiflow/specs/{slug}")).exists());
    assert!(!repo.join(format!(".mochiflow/specs/_done/{slug}")).exists());
}

#[test]
fn behavioral_ship_stops_before_mutation_for_non_automated_unverified_row() {
    let tmp = tempfile::tempdir().unwrap();
    let slug = "manual-unverified-fixture";
    let (cfg, repo) = materialize_ship_repo(tmp.path(), slug, ".mochiflow/specs");
    write_ship_matrix_row(&repo, slug, "app", "human", "UNVERIFIED", "manual evidence");

    let (code, out, err) = run_cli_capture(&cfg, &repo, &["accept", slug]);
    assert_eq!(code, 1, "stdout:\n{out}\nstderr:\n{err}");
    assert!(
        err.contains("cannot be completed by final verification"),
        "{err}"
    );
    assert!(repo.join(format!(".mochiflow/specs/{slug}")).exists());
    assert!(!repo.join(format!(".mochiflow/specs/_done/{slug}")).exists());
}

#[test]
fn behavioral_ship_honors_non_default_specs_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let slug = "custom-specs-fixture";
    let (cfg, repo) = materialize_ship_repo(tmp.path(), slug, ".workflow/specs");

    let (code, out, err) = run_cli_capture(&cfg, &repo, &["accept", slug]);
    assert_eq!(code, 0, "stdout:\n{out}\nstderr:\n{err}");
    // Flat: the spec stays at its active path under the configured specs_dir.
    assert!(repo.join(format!(".workflow/specs/{slug}")).exists());
    assert!(!repo.join(format!(".workflow/specs/_done/{slug}")).exists());
    let name_status = git_out(
        &repo,
        &["diff-tree", "--no-commit-id", "--name-status", "-r", "HEAD"],
    );
    assert!(
        name_status.contains(&format!(".workflow/specs/{slug}/spec.yaml")),
        "{name_status}"
    );
    assert!(!name_status.contains("_done/"), "{name_status}");
}

#[test]
fn behavioral_ship_reports_both_or_missing_lifecycle_states_without_mutation() {
    let tmp = tempfile::tempdir().unwrap();
    let slug = "state-fixture";
    let (cfg, repo) = materialize_ship_repo(tmp.path(), slug, ".mochiflow/specs");
    let active = repo.join(format!(".mochiflow/specs/{slug}"));
    let done = repo.join(format!(".mochiflow/specs/_done/{slug}"));
    write_active_ship_spec(&done, slug);

    let (code, out, err) = run_cli_capture(&cfg, &repo, &["accept", slug]);
    assert_eq!(code, 1, "stdout:\n{out}\nstderr:\n{err}");
    assert!(err.contains("both active and archived"), "{err}");
    std::fs::remove_dir_all(&active).unwrap();
    std::fs::remove_dir_all(&done).unwrap();
    let (code, out, err) = run_cli_capture(&cfg, &repo, &["accept", slug]);
    assert_eq!(code, 1, "stdout:\n{out}\nstderr:\n{err}");
    assert!(err.contains("was not found"), "{err}");
}

#[test]
fn behavioral_pr_slug_guard_requires_committed_ship_closeout() {
    let tmp = tempfile::tempdir().unwrap();
    let slug = "pr-guard-fixture";
    let (cfg, repo) = materialize_ship_repo(tmp.path(), slug, ".mochiflow/specs");

    // An active (not-yet-accepted) spec is rejected by the pr pre-flight.
    let (code, _out, err) = run_cli_capture(&cfg, &repo, &["pr", "--spec", slug, "--title", "Add"]);
    assert_eq!(code, 3, "{err}");
    assert!(err.contains("accept"), "{err}");

    // After accept (flat, accepted, committed with a Spec: trailer), the
    // pre-flight passes and pr reaches manual handoff (provider = none).
    let (code, out, err) = run_cli_capture(&cfg, &repo, &["accept", slug]);
    assert_eq!(code, 0, "stdout:\n{out}\nstderr:\n{err}");
    let (code, out, err) = run_cli_capture(&cfg, &repo, &["pr", "--spec", slug, "--title", "Add"]);
    assert_eq!(code, 10, "stdout:\n{out}\nstderr:\n{err}");
}

#[test]
fn behavioral_pr_path_like_spec_preserves_request_dir_behavior() {
    let tmp = tempfile::tempdir().unwrap();
    let slug = "path-like-pr-fixture";
    let (cfg, repo) = materialize_ship_repo(tmp.path(), slug, ".mochiflow/specs");
    let request_dir = repo.join("request-dir");
    std::fs::create_dir_all(&request_dir).unwrap();

    let (code, out, err) = run_cli_capture(
        &cfg,
        &repo,
        &["pr", "--spec", "./request-dir", "--title", "Add"],
    );
    assert_eq!(code, 10, "stdout:\n{out}\nstderr:\n{err}");
    assert!(!err.contains("still active"), "{err}");
}

fn write_done_spec(specs_dir: &Path, slug: &str) {
    let spec_dir = specs_dir.join("_done").join(slug);
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(
        spec_dir.join("spec.yaml"),
        format!(
            "version: 1\nslug: {slug}\ntitle: Archived Spec\ntype: feature\nsurfaces:\n  - app\nintegration: none\nrisk: standard\nstatus: done\n"
        ),
    )
    .unwrap();
    std::fs::write(
        spec_dir.join("spec.md"),
        "# Archived Spec\n\n## 受入基準\n\n- AC-01: THE SYSTEM SHALL do the thing.\n\n## AC Verification Matrix\n\n| AC | Result |\n| --- | --- |\n| AC-01 | PASS |\n",
    )
    .unwrap();
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
fn behavioral_ready_fails_when_spec_verify_is_todo() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = materialize_full(tmp.path());
    let config = std::fs::read_to_string(&cfg).unwrap().replace(
        "default = \"echo test-pass\"",
        "default = \"TODO: define test command\"",
    );
    std::fs::write(&cfg, config).unwrap();

    let (code, out) = run_cli(&cfg, &["ready", "sample-spec"]);
    assert_eq!(code, 1);
    assert!(out.contains("FAIL"), "{out}");
    assert!(
        out.contains("verification command for surface `app`"),
        "{out}"
    );
    assert!(out.contains("TODO: define test command"), "{out}");
    assert!(!out.contains("READY"), "{out}");
}

#[test]
fn behavioral_ready_fails_when_spec_verify_default_is_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = materialize_full(tmp.path());
    let config = std::fs::read_to_string(&cfg)
        .unwrap()
        .replace("default = \"echo test-pass\"\n", "");
    std::fs::write(&cfg, config).unwrap();

    let (code, out) = run_cli(&cfg, &["ready", "sample-spec"]);
    assert_eq!(code, 1);
    assert!(out.contains("FAIL"), "{out}");
    assert!(
        out.contains("verification command for surface `app`"),
        "{out}"
    );
    assert!(out.contains("has no verify profile: default"), "{out}");
    assert!(!out.contains("READY"), "{out}");
}

#[test]
fn behavioral_ready_fails_when_any_spec_surface_verify_is_not_runnable() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = materialize_full(tmp.path());
    let mut config = std::fs::read_to_string(&cfg).unwrap();
    config.push_str(
        "\n[surfaces.api]\n\
         description = \"api\"\n\n\
         [surfaces.api.verify]\n\
         default = \"TODO: define test command\"\n",
    );
    std::fs::write(&cfg, config).unwrap();

    let spec_dir = cfg.parent().unwrap().join("specs").join("sample-spec");
    std::fs::write(
        spec_dir.join("spec.yaml"),
        "version: 1\nslug: sample-spec\ntitle: Sample Spec\ntype: feature\nsurfaces:\n  - app\n  - api\nintegration: none\nrisk: standard\nstatus: approved\n",
    )
    .unwrap();
    std::fs::write(
        spec_dir.join("design.md"),
        "# Design\n\n## Integration Log\n\n- Multi-surface verification guard.\n",
    )
    .unwrap();

    let (code, out) = run_cli(&cfg, &["ready", "sample-spec"]);
    assert_eq!(code, 1);
    assert!(out.contains("FAIL"), "{out}");
    assert!(
        out.contains("verification command for surface `api`"),
        "{out}"
    );
    assert!(!out.contains("READY"), "{out}");
}

#[test]
fn behavioral_ready_ignores_todo_verify_on_unused_surface() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = materialize_full(tmp.path());
    let mut config = std::fs::read_to_string(&cfg).unwrap();
    config.push_str(
        "\n[surfaces.unused]\n\
         description = \"unused\"\n\n\
         [surfaces.unused.verify]\n\
         default = \"TODO: define test command\"\n",
    );
    std::fs::write(&cfg, config).unwrap();

    let (code, out) = run_cli(&cfg, &["ready", "sample-spec"]);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("READY") && out.contains("sample-spec"),
        "{out}"
    );
}

#[test]
fn behavioral_lint_spec_missing_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = materialize_full(tmp.path());

    let (code, out) = run_cli(&cfg, &["lint", "--spec", "missing-spec"]);
    assert_eq!(code, 1);
    assert!(out.contains("spec not found: missing-spec"), "{out}");
}

#[test]
fn behavioral_lint_archived_spec_by_slug_passes() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = materialize_full(tmp.path());
    write_done_spec(&tmp.path().join(".mochiflow/specs"), "archived-spec");

    let (code, out) = run_cli(&cfg, &["lint", "--spec", "archived-spec"]);
    assert_eq!(code, 0, "{out}");
}

#[test]
fn behavioral_lint_spec_slug_ambiguity_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = materialize_full(tmp.path());
    write_done_spec(&tmp.path().join(".mochiflow/specs"), "sample-spec");

    let (code, out) = run_cli(&cfg, &["lint", "--spec", "sample-spec"]);
    assert_eq!(code, 1);
    assert!(
        out.contains("spec target is ambiguous: sample-spec"),
        "{out}"
    );
    assert!(out.contains(".mochiflow/specs/sample-spec"), "{out}");
    assert!(out.contains(".mochiflow/specs/_done/sample-spec"), "{out}");
}

#[test]
fn behavioral_lint_spec_path_to_done_passes() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = materialize_full(tmp.path());
    write_done_spec(&tmp.path().join(".mochiflow/specs"), "archived-spec");

    let (code, out) = run_cli_in_dir(
        &cfg,
        tmp.path(),
        &["lint", "--spec", ".mochiflow/specs/_done/archived-spec"],
    );
    assert_eq!(code, 0, "{out}");

    let (code, out) = run_cli_in_dir(
        &cfg,
        tmp.path(),
        &[
            "lint",
            "--spec",
            ".mochiflow/specs/_done/archived-spec/spec.yaml",
        ],
    );
    assert_eq!(code, 0, "{out}");
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

    // A malformed seed (bad maturity, missing headings) must fail validation.
    let backlog = tmp.path().join(".mochiflow/specs/_backlog");
    std::fs::write(
        backlog.join("broken-seed.md"),
        "---\nslug: broken-seed\ntitle: Broken\nmaturity: triaged\nsource: conversation\ncreated: 2026-03-10\nupdated: 2026-03-10\n---\n\n## Signal\n\nx\n",
    )
    .unwrap();
    let (code, out) = run_cli(&cfg, &["backlog", "validate", "broken-seed"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("maturity must be seed"), "{out}");

    std::fs::write(
        backlog.join("legacy-handoff.md"),
        "---\nslug: legacy-handoff\ntitle: Legacy Handoff\nmaturity: ready-for-plan\nsource: conversation\nsource_phase: discuss\ncreated: 2026-03-10\nupdated: 2026-03-10\n---\n\n## Decision Summary\n\nx\n",
    )
    .unwrap();
    let (code, out) = run_cli(&cfg, &["backlog", "validate", "legacy-handoff"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("maturity must be seed"), "{out}");
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
    assert!(out.contains("wrote: .kiro/steering/mochiflow.md"), "{out}");

    let manifest = read_json(&tmp.path().join(".mochiflow/engine/MANIFEST.json"));
    let installed_version =
        std::fs::read_to_string(tmp.path().join(".mochiflow/engine/VERSION")).unwrap();
    assert_eq!(manifest["version"].as_str(), Some(installed_version.trim()));
    assert!(tmp.path().join(".kiro/steering/mochiflow.md").exists());
    assert!(
        tmp.path()
            .join(".kiro/agents/spec-independent-reviewer.json")
            .exists()
    );
    assert!(!tmp.path().join(".kiro/agents/spec-builder.json").exists());

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
    let target = tmp
        .path()
        .join(".kiro/agents/spec-independent-reviewer.json");
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    std::fs::write(&target, "{\"custom\": true}\n").unwrap();

    let (code, out) = run_cli(&cfg, &["upgrade"]);
    assert_eq!(code, 1, "blocked adapter merge should be non-zero");
    assert!(out.contains("upgraded engine <- bundled engine"), "{out}");
    assert!(
        out.contains("BLOCKED: .kiro/agents/spec-independent-reviewer.json"),
        "{out}"
    );
    assert!(
        out.contains("engine upgraded; adapter merge required"),
        "{out}"
    );
    assert!(
        tmp.path()
            .join(".mochiflow/state/adapters/.kiro/agents/spec-independent-reviewer.json")
            .exists()
    );
}

#[test]
fn behavioral_kiro_self_heal_and_full_file_steering() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = materialize_full(tmp.path());
    let root = tmp.path();

    // Seed deprecated outputs and an unmanaged user file before regeneration.
    let marker = "<!-- generated by mochiflow adapter=kiro version=1.1.0 -->\nold\n";
    let builder = root.join(".kiro/agents/spec-builder.json");
    std::fs::create_dir_all(builder.parent().unwrap()).unwrap();
    std::fs::write(&builder, marker).unwrap();
    let hook = root.join(".kiro/hooks/generate-project-index.kiro.hook");
    std::fs::create_dir_all(hook.parent().unwrap()).unwrap();
    std::fs::write(&hook, marker).unwrap();
    let steering_dir = root.join(".kiro/steering");
    std::fs::create_dir_all(&steering_dir).unwrap();
    // listed deprecated path but marker-stripped (user-edited): must be preserved
    std::fs::write(steering_dir.join("spec.md"), "hand edited, no marker\n").unwrap();
    // unlisted user file: must be untouched and unreported
    std::fs::write(steering_dir.join("release.md"), "release notes\n").unwrap();
    // markerless pre-existing mochiflow.md: must be overwritten whole
    std::fs::write(steering_dir.join("mochiflow.md"), "stale hand-written\n").unwrap();

    let (code, out) = run_cli(&cfg, &["adapter", "generate"]);
    assert_eq!(code, 0, "{out}");
    // markered deprecated files removed and reported (incl. legacy hook)
    assert!(
        out.contains("removed: .kiro/agents/spec-builder.json"),
        "{out}"
    );
    assert!(
        out.contains("removed: .kiro/hooks/generate-project-index.kiro.hook"),
        "{out}"
    );
    assert!(!builder.exists());
    assert!(!hook.exists());
    // markerless listed file preserved + reported; still on disk
    assert!(out.contains("preserved: .kiro/steering/spec.md"), "{out}");
    assert!(steering_dir.join("spec.md").exists());
    // unlisted file untouched and never reported
    assert!(steering_dir.join("release.md").exists());
    assert!(!out.contains("release.md"), "{out}");
    // the two managed outputs generated
    assert!(steering_dir.join("mochiflow.md").exists());
    assert!(
        root.join(".kiro/agents/spec-independent-reviewer.json")
            .exists()
    );
    // markerless mochiflow.md overwritten whole: frontmatter at the very top
    let steering = std::fs::read_to_string(steering_dir.join("mochiflow.md")).unwrap();
    assert!(
        steering.starts_with("---\ninclusion: always\n---"),
        "frontmatter must be on line 1:\n{steering}"
    );
    assert!(!steering.contains("stale hand-written"), "{steering}");
    assert!(
        steering.contains("#[[file:") && steering.contains("/router.md]]"),
        "router pointer missing:\n{steering}"
    );

    // No drift after regeneration: markerless preserved/unlisted files do not drift.
    let (code, out) = run_cli(&cfg, &["adapter", "generate", "--check"]);
    assert_eq!(code, 0, "regenerated layout must be drift-free: {out}");
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

/// Minimal config rooted at `install`'s parent, with directory-rooted adr.
fn write_adr_config(install: &Path) -> PathBuf {
    std::fs::create_dir_all(install).unwrap();
    let cfg = install.join("config.toml");
    std::fs::write(
        &cfg,
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
         decisions = \".mochiflow/adr/decisions\"\n\
         pitfalls = \".mochiflow/adr/pitfalls\"\n\n\
         [surfaces.cli]\n\
         description = \"cli\"\n\n\
         [surfaces.cli.verify]\n\
         default = \"echo ok\"\n",
    )
    .unwrap();
    cfg
}

fn write_decision(install: &Path, name: &str, front: &str, body: &str) {
    let dir = install.join("adr/decisions");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(name), format!("---\n{front}---\n{body}")).unwrap();
}

#[test]
fn doctor_adr_gates_on_dangling_reference() {
    let tmp = tempfile::tempdir().unwrap();
    let install = tmp.path().join(".mochiflow");
    let cfg = write_adr_config(&install);
    write_decision(
        &install,
        "2026-06-20-x.md",
        "id: 2026-06-20-x\ndate: 2026-06-20\narea: [cli]\nstatus: active\nsupersedes: ghost\n",
        "## 2026-06-20 — X\n",
    );
    let (code, out) = run_cli(&cfg, &["doctor", "adr"]);
    assert_eq!(code, 1, "dangling supersedes must gate doctor:\n{out}");
    assert!(out.contains("FAIL"), "{out}");
}

#[test]
fn doctor_adr_warns_on_orphan_but_passes() {
    let tmp = tempfile::tempdir().unwrap();
    let install = tmp.path().join(".mochiflow");
    let cfg = write_adr_config(&install);
    // Deprecated record with no successor link and nothing referencing it:
    // an orphan, which is a non-blocking warning.
    write_decision(
        &install,
        "2026-06-10-dep.md",
        "id: 2026-06-10-dep\ndate: 2026-06-10\narea: [cli]\nstatus: deprecated\n",
        "## 2026-06-10 — Dep\n",
    );
    let (code, out) = run_cli(&cfg, &["doctor", "adr"]);
    assert_eq!(code, 0, "orphan must not gate doctor:\n{out}");
    assert!(out.contains("WARN"), "{out}");
}

#[test]
fn doctor_adr_regenerates_stale_index_instead_of_failing() {
    let tmp = tempfile::tempdir().unwrap();
    let install = tmp.path().join(".mochiflow");
    let cfg = write_adr_config(&install);
    write_decision(
        &install,
        "2026-06-20-x.md",
        "id: 2026-06-20-x\ndate: 2026-06-20\narea: cli\nstatus: active\n",
        "## 2026-06-20 — X\n",
    );
    let index = install.join("adr/decisions/INDEX.md");
    assert!(!index.exists(), "index absent before doctor");
    let (code, out) = run_cli(&cfg, &["doctor", "adr"]);
    assert_eq!(
        code, 0,
        "absent index must be regenerated, not gated:\n{out}"
    );
    assert!(index.exists(), "doctor must regenerate the ADR INDEX.md");
    assert!(
        std::fs::read_to_string(&index)
            .unwrap()
            .contains("| 2026-06-20 |")
    );
}

#[test]
fn doctor_adr_gates_on_unknown_area() {
    let tmp = tempfile::tempdir().unwrap();
    let install = tmp.path().join(".mochiflow");
    let cfg = write_adr_config(&install); // surface = cli
    write_decision(
        &install,
        "2026-06-20-bad-area.md",
        "id: 2026-06-20-bad-area\ndate: 2026-06-20\narea: [nope]\nstatus: active\n",
        "## 2026-06-20 — Bad area\n",
    );
    let (code, out) = run_cli(&cfg, &["adr", "lint"]);
    assert_eq!(code, 1, "unknown area must gate adr lint:\n{out}");
    assert!(out.contains("unknown area"), "{out}");
}
