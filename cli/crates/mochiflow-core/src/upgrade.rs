//! Upgrade: replace engine/ only, preserve config/specs/living-spec.

use std::path::Path;

use crate::adapter;
use crate::config::Config;
use crate::doctor::check_engine;
use crate::manifest::read_engine_version;

/// Run upgrade command.
pub fn run_upgrade(cfg: &Config, source: &str, force: bool) -> i32 {
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
        copy_source_engine(&src, staging)
    })
}

/// Run upgrade from an embedded engine extractor.
pub fn run_upgrade_embedded<F>(cfg: &Config, source_label: &str, force: bool, extract: F) -> i32
where
    F: FnOnce(&Path) -> std::io::Result<()>,
{
    run_upgrade_with_stager(cfg, source_label, force, extract)
}

fn run_upgrade_with_stager<F>(cfg: &Config, source_label: &str, force: bool, stage_engine: F) -> i32
where
    F: FnOnce(&Path) -> std::io::Result<()>,
{
    let target_engine = cfg.engine_dir();

    // Check for local drift (dirty engine)
    if target_engine.join("MANIFEST.json").exists() {
        let drift_issues = check_engine(cfg);
        let dirty: Vec<_> = drift_issues
            .iter()
            .filter(|i| i.severity == "FAIL" && i.message.contains("MANIFEST drift"))
            .collect();
        if !dirty.is_empty() && !force {
            for i in &dirty {
                println!(
                    "DIRTY: {}",
                    i.message
                        .strip_prefix("engine MANIFEST drift: ")
                        .unwrap_or(&i.message)
                );
            }
            println!(
                "\nFAIL: engine has {} local change(s); re-run with --force to discard them.",
                dirty.len()
            );
            return 1;
        }
    }

    // Stage → swap
    let staging = target_engine
        .parent()
        .unwrap_or(Path::new("."))
        .join(".engine.upgrade");
    if staging.exists() {
        std::fs::remove_dir_all(&staging).ok();
    }
    if let Err(e) = std::fs::create_dir_all(&staging) {
        println!("FAIL: staging error: {e}");
        return 1;
    }
    if let Err(e) = stage_engine(&staging) {
        println!("FAIL: copy error: {e}");
        std::fs::remove_dir_all(&staging).ok();
        return 1;
    }
    if !staging.join("VERSION").exists() {
        println!("FAIL: source is not an engine dir (no VERSION): {source_label}");
        std::fs::remove_dir_all(&staging).ok();
        return 1;
    }
    if let (Ok(a), Ok(b)) = (staging.canonicalize(), target_engine.canonicalize())
        && a == b
    {
        println!("FAIL: source and target engine are the same path");
        std::fs::remove_dir_all(&staging).ok();
        return 1;
    }
    std::fs::remove_dir_all(&target_engine).ok();
    if let Err(e) = std::fs::rename(&staging, &target_engine) {
        println!("FAIL: rename error: {e}");
        return 1;
    }

    // Regenerate MANIFEST
    if let Err(e) = write_manifest(cfg) {
        println!("FAIL: could not write MANIFEST.json: {e}");
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

/// Write MANIFEST.json for the current engine state.
pub fn write_manifest(cfg: &Config) -> std::io::Result<()> {
    use sha2::{Digest, Sha256};
    use std::collections::BTreeMap;

    let engine_dir = cfg.engine_dir();
    let mut files = BTreeMap::new();
    for entry in walkdir_files(&engine_dir) {
        let rel = entry.strip_prefix(&engine_dir).unwrap_or(&entry);
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if rel_str.contains("__pycache__") || rel_str == "MANIFEST.json" {
            continue;
        }
        if let Ok(bytes) = std::fs::read(&entry) {
            let hash = Sha256::digest(&bytes);
            files.insert(rel_str, format!("sha256:{hash:x}"));
        }
    }
    let version = read_engine_version(&engine_dir).map_err(std::io::Error::other)?;
    let manifest = serde_json::json!({
        "version": version,
        "files": files,
    });
    let path = engine_dir.join("MANIFEST.json");
    let content = serde_json::to_string_pretty(&manifest).map_err(std::io::Error::other)? + "\n";
    std::fs::write(&path, content)
}

fn walkdir_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut result = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                result.extend(walkdir_files(&path));
            } else if path.is_file() {
                result.push(path);
            }
        }
    }
    result
}

fn copy_source_engine(src: &Path, staging: &Path) -> std::io::Result<()> {
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
