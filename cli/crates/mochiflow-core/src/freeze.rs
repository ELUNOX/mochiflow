//! freeze: version SSOT enforcement and derived-file regeneration.
//!
//! `cli/Cargo.toml` `[workspace.package].version` is the single source of truth.
//! This module provides pure builders that compute desired state without writing.

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Resolve the mochiflow repo root by walking up from `cwd` to find an ancestor
/// containing both `cli/Cargo.toml` (with `[workspace.package].version`) and
/// `engine/VERSION`.
pub fn resolve_repo_root(cwd: &Path) -> Result<PathBuf, String> {
    let mut dir = cwd.to_path_buf();
    loop {
        if dir.join("cli/Cargo.toml").is_file() && dir.join("engine/VERSION").is_file() {
            return Ok(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    Err("FAIL: not a mochiflow source repo (no cli/Cargo.toml + engine/ found)".into())
}

/// Validate an explicit mochiflow repo root without walking to ancestors.
pub fn validate_repo_root(root: &Path) -> Result<PathBuf, String> {
    if root.join("cli/Cargo.toml").is_file() && root.join("engine/VERSION").is_file() {
        return Ok(root.to_path_buf());
    }
    Err("FAIL: not a mochiflow source repo (no cli/Cargo.toml + engine/ found)".into())
}

/// Read `[workspace.package].version` from `cli/Cargo.toml` at the given repo root.
pub fn read_workspace_version(repo_root: &Path) -> Result<String, String> {
    let path = repo_root.join("cli/Cargo.toml");
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let doc: toml::Value =
        toml::from_str(&text).map_err(|e| format!("cannot parse {}: {e}", path.display()))?;
    doc.get("workspace")
        .and_then(|w| w.get("package"))
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("{}: missing [workspace.package].version", path.display()))
}

/// Compute the frozen-surface hash: sha256 over sorted `contracts/*.json` then
/// sorted `tests/conformance/golden/**`.
pub fn compute_contracts_hash(repo_root: &Path) -> Result<String, String> {
    let contracts = repo_root.join("contracts");
    if !contracts.is_dir() {
        return Err(format!(
            "missing contracts directory: {}",
            contracts.display()
        ));
    }
    let golden = repo_root.join("tests/conformance/golden");
    if !golden.is_dir() {
        return Err(format!("missing golden directory: {}", golden.display()));
    }

    let mut hasher = Sha256::new();

    let mut schema_files: Vec<PathBuf> = std::fs::read_dir(&contracts)
        .map_err(|e| format!("read contracts: {e}"))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json") && p.is_file())
        .collect();
    schema_files.sort();
    for f in &schema_files {
        let bytes = std::fs::read(f).map_err(|e| format!("read {}: {e}", f.display()))?;
        hasher.update(&bytes);
    }

    let mut golden_files: Vec<PathBuf> = Vec::new();
    collect_files_recursive(&golden, &mut golden_files);
    golden_files.sort();
    for f in &golden_files {
        let bytes = std::fs::read(f).map_err(|e| format!("read {}: {e}", f.display()))?;
        hasher.update(&bytes);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Build `MANIFEST.json` content for an engine directory with an explicit version.
/// Pure builder: no I/O writes. Returns serialized JSON.
pub fn build_manifest(engine_dir: &Path, version: &str) -> Result<String, String> {
    build_manifest_with_overrides(engine_dir, version, &BTreeMap::new())
}

fn build_manifest_with_overrides(
    engine_dir: &Path,
    version: &str,
    overrides: &BTreeMap<String, Vec<u8>>,
) -> Result<String, String> {
    let mut files = BTreeMap::new();
    let mut all_files = Vec::new();
    collect_files_recursive(engine_dir, &mut all_files);
    all_files.sort();
    for entry in all_files {
        let rel = entry.strip_prefix(engine_dir).unwrap_or(&entry);
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if rel_str.contains("__pycache__") || rel_str == "MANIFEST.json" {
            continue;
        }
        let bytes = match overrides.get(&rel_str) {
            Some(bytes) => bytes.clone(),
            None => std::fs::read(&entry).map_err(|e| format!("read {}: {e}", entry.display()))?,
        };
        let hash = Sha256::digest(&bytes);
        files.insert(rel_str, format!("sha256:{hash:x}"));
    }
    let manifest = serde_json::json!({
        "version": version,
        "files": files,
    });
    serde_json::to_string_pretty(&manifest)
        .map(|s| s + "\n")
        .map_err(|e| format!("serialize manifest: {e}"))
}

fn collect_files_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(&path, out);
        } else if path.is_file() {
            out.push(path);
        }
    }
}

/// Derived file staleness report entry.
#[derive(Debug)]
pub struct StaleEntry {
    pub path: PathBuf,
    pub reason: String,
}

/// Result of a freeze operation.
#[derive(Debug)]
pub struct FreezeReport {
    pub written: Vec<PathBuf>,
    pub stale: Vec<StaleEntry>,
}

/// Run freeze: compute desired state of all derived files, then either write
/// (check=false) or report staleness (check=true).
pub fn freeze(repo_root: &Path, check: bool) -> Result<FreezeReport, String> {
    let version = read_workspace_version(repo_root)?;
    let hash = compute_contracts_hash(repo_root)?;

    // Desired content for each derived file
    let engine_dir = repo_root.join("engine");
    let version_path = engine_dir.join("VERSION");
    let manifest_path = engine_dir.join("MANIFEST.json");
    let lock_path = repo_root.join("contracts/contracts.lock");

    let desired_version = format!("{version}\n");
    let desired_manifest = build_manifest_with_overrides(
        &engine_dir,
        &version,
        &BTreeMap::from([("VERSION".to_string(), desired_version.as_bytes().to_vec())]),
    )?;
    let desired_lock = format!("{{\"version\": \"{version}\", \"hash\": \"{hash}\"}}\n");

    let targets: Vec<(PathBuf, String)> = vec![
        (version_path, desired_version),
        (manifest_path, desired_manifest),
        (lock_path, desired_lock),
    ];

    let mut report = FreezeReport {
        written: Vec::new(),
        stale: Vec::new(),
    };

    for (path, desired) in targets {
        let current = std::fs::read_to_string(&path).unwrap_or_default();
        if current != desired {
            if check {
                report.stale.push(StaleEntry {
                    path,
                    reason: "content differs from computed state".into(),
                });
            } else {
                std::fs::write(&path, &desired)
                    .map_err(|e| format!("write {}: {e}", path.display()))?;
                report.written.push(path);
            }
        }
    }

    Ok(report)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn real_repo_root() -> PathBuf {
        // mochiflow-core manifest is at cli/crates/mochiflow-core; repo root is 3 up.
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("repo root three levels above mochiflow-core")
            .to_path_buf()
    }

    #[test]
    fn read_workspace_version_parses_real_cargo_toml() {
        let version = read_workspace_version(&real_repo_root()).unwrap();
        assert!(
            version.contains('.'),
            "workspace version should be semver: {version}"
        );
    }

    #[test]
    fn compute_contracts_hash_matches_committed_lock() {
        let root = real_repo_root();
        let hash = compute_contracts_hash(&root).unwrap();
        let lock_text = std::fs::read_to_string(root.join("contracts/contracts.lock")).unwrap();
        let lock: serde_json::Value = serde_json::from_str(&lock_text).unwrap();
        assert_eq!(hash, lock["hash"].as_str().unwrap());
    }

    #[test]
    fn resolve_repo_root_from_subdir() {
        let root = real_repo_root();
        let subdir = root.join("cli/crates/mochiflow-core");
        let resolved = resolve_repo_root(&subdir).unwrap();
        assert_eq!(
            resolved.canonicalize().unwrap(),
            root.canonicalize().unwrap()
        );
    }

    #[test]
    fn resolve_repo_root_fails_for_non_repo() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(resolve_repo_root(tmp.path()).is_err());
    }

    #[test]
    fn validate_repo_root_does_not_walk_to_parent() {
        let root = real_repo_root();
        let subdir = root.join("cli/crates/mochiflow-core");
        assert!(validate_repo_root(&subdir).is_err());
        assert_eq!(
            validate_repo_root(&root).unwrap().canonicalize().unwrap(),
            root.canonicalize().unwrap()
        );
    }

    #[test]
    fn build_manifest_is_pure_and_matches_format() {
        let root = real_repo_root();
        let engine_dir = root.join("engine");
        let version = read_workspace_version(&root).unwrap();
        let content = build_manifest(&engine_dir, &version).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["version"].as_str().unwrap(), version);
        assert!(!parsed["files"].as_object().unwrap().is_empty());
    }

    #[test]
    fn freeze_write_idempotent_in_fixture() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        // Build minimal fixture
        setup_fixture(root, "1.0.0");

        // First freeze writes
        let r1 = freeze(root, false).unwrap();
        assert!(!r1.written.is_empty());

        // Second freeze is idempotent
        let r2 = freeze(root, false).unwrap();
        assert!(r2.written.is_empty(), "second freeze should write nothing");

        // Check mode passes
        let r3 = freeze(root, true).unwrap();
        assert!(r3.stale.is_empty());
    }

    #[test]
    fn freeze_check_detects_staleness() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        setup_fixture(root, "1.0.0");

        freeze(root, false).unwrap();

        // Hand-edit engine/VERSION to introduce drift
        std::fs::write(root.join("engine/VERSION"), "9.9.9\n").unwrap();

        let report = freeze(root, true).unwrap();
        assert!(!report.stale.is_empty());
        // Check mode doesn't write
        let v = std::fs::read_to_string(root.join("engine/VERSION")).unwrap();
        assert_eq!(v.trim(), "9.9.9");
    }

    #[test]
    fn freeze_version_triple_mismatch_fails_gate() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        setup_fixture(root, "2.0.0");

        freeze(root, false).unwrap();

        // Tamper version in Cargo.toml to create mismatch
        let cargo_path = root.join("cli/Cargo.toml");
        std::fs::write(
            &cargo_path,
            "[workspace]\nmembers = []\n[workspace.package]\nversion = \"3.0.0\"\n",
        )
        .unwrap();

        let report = freeze(root, true).unwrap();
        assert!(
            !report.stale.is_empty(),
            "mismatched version triple must be detected as stale"
        );
    }

    #[test]
    fn freeze_write_rehashes_manifest_after_workspace_version_bump() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        setup_fixture(root, "1.0.0");

        freeze(root, false).unwrap();
        std::fs::write(
            root.join("cli/Cargo.toml"),
            "[workspace]\nmembers = []\n[workspace.package]\nversion = \"1.1.0\"\n",
        )
        .unwrap();

        freeze(root, false).unwrap();
        let report = freeze(root, true).unwrap();
        assert!(
            report.stale.is_empty(),
            "freeze must converge after a single version-bump write: {report:?}"
        );

        let version_bytes = std::fs::read(root.join("engine/VERSION")).unwrap();
        let expected = format!("sha256:{:x}", Sha256::digest(&version_bytes));
        let manifest_text = std::fs::read_to_string(root.join("engine/MANIFEST.json")).unwrap();
        let manifest: serde_json::Value = serde_json::from_str(&manifest_text).unwrap();
        assert_eq!(
            manifest["files"]["VERSION"].as_str(),
            Some(expected.as_str())
        );
    }

    fn setup_fixture(root: &Path, version: &str) {
        // cli/Cargo.toml
        let cli_dir = root.join("cli");
        std::fs::create_dir_all(&cli_dir).unwrap();
        std::fs::write(
            cli_dir.join("Cargo.toml"),
            format!("[workspace]\nmembers = []\n[workspace.package]\nversion = \"{version}\"\n"),
        )
        .unwrap();

        // engine/VERSION + a sample file
        let engine_dir = root.join("engine");
        std::fs::create_dir_all(&engine_dir).unwrap();
        std::fs::write(engine_dir.join("VERSION"), format!("{version}\n")).unwrap();
        std::fs::write(engine_dir.join("router.md"), "# Router\n").unwrap();

        // contracts/*.json
        let contracts_dir = root.join("contracts");
        std::fs::create_dir_all(&contracts_dir).unwrap();
        std::fs::write(contracts_dir.join("test.json"), "{}\n").unwrap();

        // tests/conformance/golden/
        let golden_dir = root.join("tests/conformance/golden");
        std::fs::create_dir_all(&golden_dir).unwrap();
        std::fs::write(golden_dir.join("INDEX.md"), "# Index\n").unwrap();
    }
}
