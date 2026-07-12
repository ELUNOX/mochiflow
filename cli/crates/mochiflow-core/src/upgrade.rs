//! Upgrade: replace engine/ only, preserve config/specs/living-spec.

use std::path::Path;

use crate::adapter;
use crate::config::Config;
use crate::doctor::check_engine;
use crate::manifest::read_engine_version;

#[derive(Debug)]
pub enum EngineInstallError {
    Validation(String),
    Dirty {
        entries: Vec<String>,
    },
    Staging(std::io::Error),
    Copy(std::io::Error),
    MissingVersion {
        source_label: String,
    },
    SamePath,
    Rename(std::io::Error),
    Manifest(std::io::Error),
    Rollback {
        install: std::io::Error,
        rollback: std::io::Error,
        backup: std::path::PathBuf,
    },
    Cleanup {
        error: std::io::Error,
        backup: std::path::PathBuf,
    },
}

impl EngineInstallError {
    pub fn report_lines(&self) -> Vec<String> {
        match self {
            Self::Validation(error) => vec![format!("FAIL: {error}")],
            Self::Dirty { entries } => {
                let mut lines: Vec<String> = entries
                    .iter()
                    .map(|entry| format!("DIRTY: {entry}"))
                    .collect();
                lines.push(format!(
                    "\nFAIL: engine has {} local change(s); re-run with --force to discard them.",
                    entries.len()
                ));
                lines
            }
            Self::Staging(e) => vec![format!("FAIL: staging error: {e}")],
            Self::Copy(e) => vec![format!("FAIL: copy error: {e}")],
            Self::MissingVersion { source_label } => {
                vec![format!(
                    "FAIL: source is not an engine dir (no VERSION): {source_label}"
                )]
            }
            Self::SamePath => vec!["FAIL: source and target engine are the same path".into()],
            Self::Rename(e) => vec![format!("FAIL: rename error: {e}")],
            Self::Manifest(e) => vec![format!("FAIL: could not write MANIFEST.json: {e}")],
            Self::Rollback {
                install,
                rollback,
                backup,
            } => vec![format!(
                "FAIL: engine install failed ({install}) and rollback failed ({rollback}); recover backup from {}",
                backup.display()
            )],
            Self::Cleanup { error, backup } => vec![format!(
                "FAIL: engine installed but backup cleanup failed ({error}); remove preserved backup after inspection: {}",
                backup.display()
            )],
        }
    }
}

/// Run upgrade command.
pub fn run_upgrade(cfg: &Config, source: &str, force: bool) -> i32 {
    if let Err(error) = cfg.checked_engine_dir() {
        println!("FAIL: {error}");
        return 1;
    }
    let mut src = std::path::PathBuf::from(source);
    if !src.is_absolute() {
        src = std::env::current_dir().unwrap_or_default().join(&src);
    }
    if src.join("engine").is_dir() {
        src = src.join("engine");
    }
    if let (Ok(a), Ok(b)) = (src.canonicalize(), cfg.engine_dir().canonicalize())
        && a == b
    {
        println!("FAIL: source and target engine are the same path");
        return 1;
    }
    let label = src.display().to_string();
    run_upgrade_with_stager(cfg, &label, force, |staging| {
        stage_source_engine(&src, staging)
    })
}

/// Run upgrade from an embedded engine extractor.
pub fn run_upgrade_embedded<F>(cfg: &Config, source_label: &str, force: bool, extract: F) -> i32
where
    F: FnOnce(&Path) -> std::io::Result<()>,
{
    if let Err(error) = cfg.checked_engine_dir() {
        println!("FAIL: {error}");
        return 1;
    }
    run_upgrade_with_stager(cfg, source_label, force, extract)
}

fn run_upgrade_with_stager<F>(cfg: &Config, source_label: &str, force: bool, stage_engine: F) -> i32
where
    F: FnOnce(&Path) -> std::io::Result<()>,
{
    if let Err(e) = install_engine_staged(cfg, source_label, force, stage_engine) {
        for line in e.report_lines() {
            println!("{line}");
        }
        return 1;
    }
    println!("upgraded engine <- {source_label}");

    let adapter_result = adapter::generate(cfg, false, false);
    for f in &adapter_result.wrote {
        println!("wrote: {f}");
    }
    for blocked in &adapter_result.blocked {
        println!(
            "BLOCKED: {} (candidate: {}; merge manually or use --force to replace)",
            blocked.target, blocked.candidate
        );
    }
    for error in &adapter_result.errors {
        println!("FAIL: {error}");
    }
    if adapter_result.blocked.is_empty() && adapter_result.errors.is_empty() {
        println!("run: mochiflow doctor");
        0
    } else {
        println!(
            "engine upgraded; adapter merge required ({} blocked, {} failed)",
            adapter_result.blocked.len(),
            adapter_result.errors.len()
        );
        1
    }
}

/// Install an engine by staging the full source first, then swapping it into
/// `{install_dir}/engine`. This owns engine integrity only; callers own any
/// adapter regeneration and user-facing success output.
pub fn install_engine_staged<F>(
    cfg: &Config,
    source_label: &str,
    force: bool,
    stage_engine: F,
) -> Result<(), EngineInstallError>
where
    F: FnOnce(&Path) -> std::io::Result<()>,
{
    cfg.checked_engine_dir()
        .map_err(|error| EngineInstallError::Validation(error.to_string()))?;
    let target_engine = cfg.engine_dir();

    if target_engine.exists() {
        let dirty: Vec<String> = check_engine(cfg)
            .into_iter()
            .filter(|i| i.severity == "FAIL")
            .map(|i| {
                i.message
                    .strip_prefix("engine MANIFEST drift: ")
                    .unwrap_or(&i.message)
                    .to_string()
            })
            .collect();
        if !dirty.is_empty() && !force {
            return Err(EngineInstallError::Dirty { entries: dirty });
        }
    }

    let parent = target_engine.parent().unwrap_or(Path::new("."));
    let staging = unique_engine_temp_path(parent, ".engine.upgrade");
    std::fs::create_dir_all(&staging).map_err(EngineInstallError::Staging)?;

    if let Err(e) = stage_engine(&staging) {
        std::fs::remove_dir_all(&staging).ok();
        return Err(EngineInstallError::Copy(e));
    }
    if !staging.join("VERSION").exists() {
        std::fs::remove_dir_all(&staging).ok();
        return Err(EngineInstallError::MissingVersion {
            source_label: source_label.to_string(),
        });
    }
    if let (Ok(a), Ok(b)) = (staging.canonicalize(), target_engine.canonicalize())
        && a == b
    {
        std::fs::remove_dir_all(&staging).ok();
        return Err(EngineInstallError::SamePath);
    }

    if let Err(e) = write_manifest_for_engine_dir(&staging) {
        std::fs::remove_dir_all(&staging).ok();
        return Err(EngineInstallError::Manifest(e));
    }

    let backup = unique_engine_temp_path(parent, ".engine.backup");
    if target_engine.exists() {
        std::fs::rename(&target_engine, &backup).map_err(EngineInstallError::Rename)?;
    }
    if let Err(e) = std::fs::rename(&staging, &target_engine) {
        if backup.exists()
            && let Err(rollback) = std::fs::rename(&backup, &target_engine)
        {
            return Err(EngineInstallError::Rollback {
                install: e,
                rollback,
                backup,
            });
        }
        return Err(EngineInstallError::Rename(e));
    }
    if backup.exists()
        && let Err(error) = std::fs::remove_dir_all(&backup)
    {
        return Err(EngineInstallError::Cleanup { error, backup });
    }

    Ok(())
}

fn unique_engine_temp_path(parent: &Path, prefix: &str) -> std::path::PathBuf {
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    for attempt in 0..1000_u32 {
        let candidate = parent.join(format!("{prefix}-{pid}-{nanos}-{attempt}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    parent.join(format!("{prefix}-{pid}-{nanos}-fallback"))
}

/// Write MANIFEST.json for the current engine state.
pub fn write_manifest(cfg: &Config) -> std::io::Result<()> {
    write_manifest_for_engine_dir(&cfg.engine_dir())
}

/// Write MANIFEST.json for an explicit engine directory.
pub fn write_manifest_for_engine_dir(engine_dir: &Path) -> std::io::Result<()> {
    let version = read_engine_version(engine_dir).map_err(std::io::Error::other)?;
    let content =
        crate::freeze::build_manifest(engine_dir, &version).map_err(std::io::Error::other)?;
    std::fs::write(engine_dir.join("MANIFEST.json"), content)
}

pub(crate) fn stage_source_engine(src: &Path, staging: &Path) -> std::io::Result<()> {
    let mut src = src.to_path_buf();
    if src.join("engine").is_dir() {
        src = src.join("engine");
    }
    if !src.join("VERSION").exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "source is not an engine dir (no VERSION): {}",
                src.display()
            ),
        ));
    }
    copy_dir_all(&src, staging)
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let target = dst.join(entry.file_name());
        if entry.path().is_dir() {
            let name = entry.file_name();
            if name.to_string_lossy() == "__pycache__" {
                continue;
            }
            copy_dir_all(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
