//! Spec persistence mode detection.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::Config;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecPersistenceMode {
    Tracked,
    Local,
}

impl SpecPersistenceMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tracked => "tracked",
            Self::Local => "local",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpecPersistence {
    pub mode: SpecPersistenceMode,
    pub probe_path: PathBuf,
    pub reason: String,
}

/// Classify a spec's persistence mode from git's ignore rules.
///
/// The concrete probe is `spec.yaml`: if that durable spec artifact is ignored,
/// the spec is local-only; otherwise tracked mode applies.
pub fn classify_spec(cfg: &Config, slug: &str) -> Result<SpecPersistence, String> {
    classify_spec_dir(cfg, &cfg.specs_dir_path().join(slug))
}

pub fn classify_spec_dir(cfg: &Config, spec_dir: &Path) -> Result<SpecPersistence, String> {
    let probe = spec_dir.join("spec.yaml");
    let rel = rel_path(&cfg.repo_root, &probe);
    let output = Command::new("git")
        .args(["check-ignore", "-v", "--"])
        .arg(&rel)
        .current_dir(&cfg.repo_root)
        .output()
        .map_err(|e| format!("could not run git check-ignore: {e}"))?;

    if output.status.success() {
        let rule = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let reason = if rule.is_empty() {
            format!("{} is ignored by git", path_to_string(&rel))
        } else {
            format!("{} is ignored by git ({rule})", path_to_string(&rel))
        };
        Ok(SpecPersistence {
            mode: SpecPersistenceMode::Local,
            probe_path: rel,
            reason,
        })
    } else if output.status.code() == Some(1) {
        Ok(SpecPersistence {
            mode: SpecPersistenceMode::Tracked,
            probe_path: rel.clone(),
            reason: format!("{} is not ignored by git", path_to_string(&rel)),
        })
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if stderr.is_empty() {
            "git check-ignore failed".to_string()
        } else {
            format!("git check-ignore failed: {stderr}")
        })
    }
}

fn rel_path(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use std::fs;
    use std::path::Path;
    use std::process::Command;

    use crate::config::load_config;

    use super::{SpecPersistenceMode, classify_spec};

    fn git(repo: &Path, args: &[&str]) {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(repo)
                .status()
                .unwrap()
                .success(),
            "git {args:?}"
        );
    }

    fn write_config(repo: &Path) -> std::path::PathBuf {
        let cfg = repo.join(".mochiflow/config.toml");
        fs::create_dir_all(cfg.parent().unwrap()).unwrap();
        fs::write(
            &cfg,
            "schema_version = 1\ninstall_dir = \".mochiflow\"\nspecs_dir = \".mochiflow/specs\"\nindex = \".mochiflow/INDEX.md\"\n\n[constitution]\nproject = \".mochiflow/constitution.md\"\nlocal = \".mochiflow/constitution.local.md\"\n\n[context]\nproduct = \".mochiflow/context/product.md\"\nstructure = \".mochiflow/context/structure.md\"\ntech = \".mochiflow/context/tech.md\"\n\n[adr]\ndecisions = \".mochiflow/adr/decisions\"\npitfalls = \".mochiflow/adr/pitfalls\"\n\n[git]\nbase_branch = \"main\"\n\n[adapter]\ntool = \"agents\"\n\n[surfaces.app]\ndescription = \"app\"\n\n[surfaces.app.verify]\ndefault = \"echo ok\"\n",
        )
        .unwrap();
        cfg
    }

    #[test]
    fn tracked_when_spec_artifact_is_not_ignored() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        git(repo, &["init", "-q", "-b", "main"]);
        let cfg = load_config(&write_config(repo)).unwrap();

        let mode = classify_spec(&cfg, "sample").unwrap();

        assert_eq!(mode.mode, SpecPersistenceMode::Tracked);
        assert_eq!(
            mode.probe_path,
            Path::new(".mochiflow/specs/sample/spec.yaml")
        );
        assert!(mode.reason.contains("not ignored"), "{}", mode.reason);
    }

    #[test]
    fn local_when_mochiflow_root_is_ignored() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        git(repo, &["init", "-q", "-b", "main"]);
        fs::write(repo.join(".gitignore"), ".mochiflow/\n").unwrap();
        let cfg = load_config(&write_config(repo)).unwrap();

        let mode = classify_spec(&cfg, "sample").unwrap();

        assert_eq!(mode.mode, SpecPersistenceMode::Local);
        assert!(mode.reason.contains(".mochiflow/"), "{}", mode.reason);
    }

    #[test]
    fn local_when_specs_dir_is_ignored() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        git(repo, &["init", "-q", "-b", "main"]);
        fs::create_dir_all(repo.join(".mochiflow")).unwrap();
        fs::write(repo.join(".mochiflow/.gitignore"), "specs/\n").unwrap();
        let cfg = load_config(&write_config(repo)).unwrap();

        let mode = classify_spec(&cfg, "sample").unwrap();

        assert_eq!(mode.mode, SpecPersistenceMode::Local);
        assert!(mode.reason.contains("specs/"), "{}", mode.reason);
    }
}
