use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Shell, generate};
use include_dir::{Dir, include_dir};

/// Embedded engine directory (compiled into the binary for AC-03 self-contained use).
static EMBEDDED_ENGINE: Dir = include_dir!("$CARGO_MANIFEST_DIR/../../../engine");

#[derive(Parser)]
#[command(name = "mochiflow", version, about = "mochiflow CLI")]
struct Cli {
    /// Path to config.toml (default: derived from engine location)
    #[arg(long)]
    config: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inspect / validate config.toml
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Read-only ADR record store: list / show / search / lint
    Adr {
        #[command(subcommand)]
        command: AdrCommand,
    },
    /// Regenerate INDEX.md + state/index.json
    Index {
        /// Report drift without writing
        #[arg(long)]
        check: bool,
    },
    /// Render the live delivery board (read-only): Backlog / Active / Ready / In Review / Done
    Status {
        /// Fetch origin before computing derived delivery state
        #[arg(long)]
        fetch: bool,
    },
    /// Lint specs
    Lint {
        /// Single spec slug
        #[arg(long)]
        spec: Option<String>,
    },
    /// Quality gate: config / specs / adapter / engine
    Doctor {
        /// Emit a single JSON document on stdout
        #[arg(long)]
        json: bool,
        target: Option<DoctorTarget>,
    },
    /// Generate tool adapters
    Adapter {
        #[command(subcommand)]
        command: AdapterCommand,
    },
    /// Replace engine from the bundled engine or an explicit source (preserve config/specs)
    Upgrade {
        /// Path to source engine (developer/dogfood override; default uses bundled engine)
        #[arg(long)]
        source: Option<String>,
        /// Discard local engine changes
        #[arg(long)]
        force: bool,
    },
    /// Check whether a spec can enter build
    Ready {
        /// Spec slug or directory
        spec: String,
    },
    /// Settle the accept close-out: set accepted, stage the spec + fold, commit
    Accept {
        /// Spec slug (default: infer from current feature branch)
        slug: Option<String>,
        /// Preview without verification, mutation, staging, or commit
        #[arg(long)]
        dry_run: bool,
    },
    /// Inspect backlog seeds
    Backlog {
        #[command(subcommand)]
        command: BacklogCommand,
    },
    /// Bootstrap mochiflow into a project
    Init {
        /// Target repo root
        #[arg(long, default_value = ".")]
        target: String,
        /// AI coding tool adapter(s). Repeatable and comma-separated, e.g.
        /// `--adapter kiro --adapter claude-code` or `--adapter codex,kiro`.
        #[arg(long)]
        adapter: Vec<String>,
        /// Durable artifact language (BCP 47-style tag, e.g. en, ja, pt-BR)
        #[arg(long)]
        artifact_language: Option<String>,
        /// User-facing conversation language (`auto` or a BCP 47-style tag)
        #[arg(long)]
        conversation_language: Option<String>,
        /// Overwrite existing config
        #[arg(long)]
        force: bool,
        /// Preview without writing
        #[arg(long)]
        dry_run: bool,
        /// Accept defaults and do not prompt
        #[arg(long)]
        yes: bool,
        /// Emit the detection result as a single JSON document on stdout
        /// (all progress logs go to stderr).
        #[arg(long)]
        json: bool,
    },
    /// Set up local generated state for an existing MochiFlow project
    Join {
        /// Target repo root
        #[arg(long, default_value = ".")]
        target: String,
        /// Preview without writing
        #[arg(long)]
        dry_run: bool,
        /// Emit a single JSON document on stdout
        #[arg(long)]
        json: bool,
        /// Discard local engine changes
        #[arg(long)]
        force: bool,
    },
    /// Detach MochiFlow from this project while preserving project knowledge
    Detach {
        /// Target repo root
        #[arg(long, default_value = ".")]
        target: String,
        /// Remove all MochiFlow project data, including specs/ADR/context
        #[arg(long)]
        purge: bool,
        /// Preview without writing
        #[arg(long)]
        dry_run: bool,
        /// Emit a single JSON document on stdout
        #[arg(long)]
        json: bool,
        /// Exact confirmation phrase required for --purge in non-interactive use
        #[arg(long)]
        confirm: Option<String>,
    },
    /// Print the usage-vocabulary card (the four verbs + delivery approval gates)
    Guide,
    /// Generate shell completion scripts
    Completions { shell: Shell },
    /// Create a PR (pre-flight + push + resolved backend, or manual handoff)
    Pr {
        /// Spec slug or request directory (holds pr-description.md / pr-request.json)
        #[arg(long)]
        spec: Option<String>,
        /// PR title
        #[arg(long)]
        title: Option<String>,
        /// Path to a file containing the PR body
        #[arg(long)]
        body_file: Option<String>,
        /// Open the PR as a draft
        #[arg(long)]
        draft: bool,
        /// Preview without writing/pushing/dispatching
        #[arg(long)]
        dry_run: bool,
    },
    /// Regenerate derived version/integrity files (engine/VERSION, MANIFEST.json, contracts.lock)
    Freeze {
        /// MochiFlow source repo root (default: walk up from current directory)
        #[arg(long)]
        root: Option<String>,
        /// Report staleness without writing
        #[arg(long)]
        check: bool,
    },
}

#[derive(Subcommand)]
enum ConfigCommand {
    Show,
    Validate,
}

/// Which ADR store to operate on.
#[derive(Clone, Copy, ValueEnum)]
enum AdrKindArg {
    Decisions,
    Pitfalls,
}

impl AdrKindArg {
    fn to_kind(self) -> mochiflow_core::adr::AdrKind {
        match self {
            AdrKindArg::Decisions => mochiflow_core::adr::AdrKind::Decisions,
            AdrKindArg::Pitfalls => mochiflow_core::adr::AdrKind::Pitfalls,
        }
    }
}

#[derive(Subcommand)]
enum AdrCommand {
    /// List active record headers (filterable)
    List {
        #[arg(long)]
        kind: Option<AdrKindArg>,
        #[arg(long)]
        area: Option<String>,
        /// Status filter; `all` widens to full history (default: active set)
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        spec: Option<String>,
    },
    /// Show a record's full body and supersession lineage
    Show {
        /// Record id (e.g. 2026-06-22-version-ssot)
        id: String,
        #[arg(long)]
        kind: Option<AdrKindArg>,
    },
    /// Search record headers by keyword over the default-active set
    Search {
        /// Search term (case-insensitive)
        term: String,
        #[arg(long)]
        kind: Option<AdrKindArg>,
        #[arg(long)]
        area: Option<String>,
        /// Status filter; `all` widens to full history (default: active set)
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        spec: Option<String>,
    },
    /// Deterministic structural lint of the ADR record stores
    Lint {
        /// Limit to one store (default: both)
        #[arg(long)]
        kind: Option<AdrKindArg>,
    },
}

#[derive(Subcommand)]
enum AdapterCommand {
    Generate {
        /// Overwrite non-generated target files
        #[arg(long)]
        force: bool,
        /// Report drift without writing
        #[arg(long)]
        check: bool,
    },
}

#[derive(Subcommand)]
enum BacklogCommand {
    List,
    Show { slug: String },
    Validate { slug: String },
}

#[derive(Clone, Copy, ValueEnum)]
enum DoctorTarget {
    Config,
    Adr,
    Specs,
    Adapter,
    Engine,
}

impl DoctorTarget {
    fn as_str(self) -> &'static str {
        match self {
            Self::Config => "config",
            Self::Adr => "adr",
            Self::Specs => "specs",
            Self::Adapter => "adapter",
            Self::Engine => "engine",
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let exit_code = match cli.command {
        Commands::Config { command } => match command {
            ConfigCommand::Show => {
                let cfg = load_cfg(cli.config.as_deref())?;
                cmd_config_show(&cfg, &bundled_engine_version());
                0
            }
            ConfigCommand::Validate => {
                let cfg = load_cfg(cli.config.as_deref())?;
                let issues = mochiflow_core::doctor::validate_config(&cfg);
                for i in &issues {
                    println!("{}: {}", i.severity, i.message);
                }
                let fails = issues.iter().filter(|i| i.severity == "FAIL").count();
                let warns = issues.iter().filter(|i| i.severity == "WARN").count();
                println!("\nSummary: {fails} fail, {warns} warn");
                if fails > 0 { 1 } else { 0 }
            }
        },
        Commands::Adr { command } => match command {
            AdrCommand::List {
                kind,
                area,
                status,
                spec,
            } => {
                let cfg = load_cfg(cli.config.as_deref())?;
                let query = mochiflow_core::adr::AdrQuery {
                    kind: kind.map(AdrKindArg::to_kind),
                    area,
                    status,
                    spec,
                };
                mochiflow_core::adr::run_adr_list(&cfg, &query)
            }
            AdrCommand::Show { id, kind } => {
                let cfg = load_cfg(cli.config.as_deref())?;
                mochiflow_core::adr::run_adr_show(&cfg, &id, kind.map(AdrKindArg::to_kind))
            }
            AdrCommand::Search {
                term,
                kind,
                area,
                status,
                spec,
            } => {
                let cfg = load_cfg(cli.config.as_deref())?;
                let query = mochiflow_core::adr::AdrQuery {
                    kind: kind.map(AdrKindArg::to_kind),
                    area,
                    status,
                    spec,
                };
                mochiflow_core::adr::run_adr_search(&cfg, &term, &query)
            }
            AdrCommand::Lint { kind } => {
                let cfg = load_cfg(cli.config.as_deref())?;
                mochiflow_core::adr::run_adr_lint(&cfg, kind.map(AdrKindArg::to_kind))
            }
        },
        Commands::Index { check } => {
            let cfg = load_cfg(cli.config.as_deref())?;
            if check {
                mochiflow_core::index::check_index(&cfg)
            } else {
                match mochiflow_core::index::generate_index(&cfg) {
                    Ok(()) => 0,
                    Err(e) => {
                        println!("FAIL: could not write index files: {e}");
                        1
                    }
                }
            }
        }
        Commands::Lint { spec } => {
            let cfg = load_cfg(cli.config.as_deref())?;
            mochiflow_core::lint::run_lint(&cfg, spec.as_deref(), false)
        }
        Commands::Status { fetch } => {
            let cfg = load_cfg(cli.config.as_deref())?;
            mochiflow_core::status::run_status(&cfg, fetch)
        }
        Commands::Doctor { json, target } => {
            let cfg = load_cfg(cli.config.as_deref())?;
            let bundled = bundled_engine_version();
            if json {
                mochiflow_core::doctor::run_doctor_json_with_bundled(
                    &cfg,
                    target.map(DoctorTarget::as_str),
                    Some(&bundled),
                )
            } else {
                mochiflow_core::doctor::run_doctor_with_bundled(
                    &cfg,
                    target.map(DoctorTarget::as_str),
                    false,
                    Some(&bundled),
                )
            }
        }
        Commands::Adapter { command } => match command {
            AdapterCommand::Generate { force, check } => {
                let cfg = load_cfg(cli.config.as_deref())?;
                mochiflow_core::adapter::cmd_adapter_generate(&cfg, check, force)
            }
        },
        Commands::Upgrade { source, force } => {
            let cfg = load_cfg(cli.config.as_deref())?;
            if let Some(source) = source {
                mochiflow_core::upgrade::run_upgrade(&cfg, &source, force)
            } else {
                let extract_fn = |target_dir: &std::path::Path| -> std::io::Result<()> {
                    std::fs::create_dir_all(target_dir)?;
                    EMBEDDED_ENGINE.extract(target_dir)
                };
                mochiflow_core::upgrade::run_upgrade_embedded(
                    &cfg,
                    "bundled engine",
                    force,
                    extract_fn,
                )
            }
        }
        Commands::Ready { spec } => {
            let cfg = load_cfg(cli.config.as_deref())?;
            cmd_ready(&cfg, &spec)
        }
        Commands::Accept { slug, dry_run } => {
            let cfg = load_cfg(cli.config.as_deref())?;
            let code = mochiflow_core::accept::run_accept(&cfg, slug.as_deref(), dry_run);
            // Regenerate the gitignored board to reflect the new accepted/Ready
            // state. Skipped on dry-run; never staged or committed.
            if !dry_run && code == 0 {
                refresh_board_after_state_change(&cfg);
            }
            code
        }
        Commands::Backlog { command } => {
            let cfg = load_cfg(cli.config.as_deref())?;
            match command {
                BacklogCommand::List => mochiflow_core::backlog::list_seeds(&cfg),
                BacklogCommand::Show { slug } => mochiflow_core::backlog::show_seed(&cfg, &slug),
                BacklogCommand::Validate { slug } => {
                    mochiflow_core::backlog::validate_seed(&cfg, &slug)
                }
            }
        }
        Commands::Init {
            target,
            adapter,
            artifact_language,
            conversation_language,
            force,
            dry_run,
            yes,
            json,
        } => {
            let extract_fn = |target_dir: &std::path::Path| -> std::io::Result<()> {
                std::fs::create_dir_all(target_dir)?;
                EMBEDDED_ENGINE.extract(target_dir)
            };
            if !force
                && std::path::PathBuf::from(&target)
                    .join(".mochiflow")
                    .join("config.toml")
                    .exists()
            {
                let existing_config = std::path::PathBuf::from(&target)
                    .join(".mochiflow")
                    .join("config.toml");
                if !adapter.is_empty()
                    && let Ok(cfg) = mochiflow_core::config::load_config(&existing_config)
                {
                    let message = format!(
                        "Ignoring --adapter {} because existing config is kept; configured adapters: {}. Use --force to rewrite config.",
                        adapter.join(", "),
                        cfg.adapter_tools().join(", ")
                    );
                    if json {
                        eprintln!("{message}");
                    } else {
                        println!("{message}");
                    }
                }
            }
            mochiflow_core::init::run_init(
                &target,
                &adapter,
                artifact_language.as_deref(),
                conversation_language.as_deref(),
                force,
                dry_run,
                json,
                yes,
                Some(&extract_fn),
            )
        }
        Commands::Join {
            target,
            dry_run,
            json,
            force,
        } => {
            let extract_fn = |target_dir: &std::path::Path| -> std::io::Result<()> {
                std::fs::create_dir_all(target_dir)?;
                EMBEDDED_ENGINE.extract(target_dir)
            };
            mochiflow_core::join::run_join(
                &target,
                dry_run,
                json,
                force,
                &bundled_engine_version(),
                Some(&extract_fn),
            )
        }
        Commands::Detach {
            target,
            purge,
            dry_run,
            json,
            confirm,
        } => {
            let cfg = load_detach_cfg(cli.config.as_deref(), &target, purge, dry_run, json)?;
            mochiflow_core::detach::run_detach(&cfg, purge, dry_run, json, confirm.as_deref())
        }
        Commands::Guide => {
            // The card is usable before setup is complete; resolve the language from
            // config when present, otherwise default to English (never exit).
            let path = match cli.config.as_deref() {
                Some(p) => std::path::PathBuf::from(p),
                None => std::env::current_dir()?
                    .join(".mochiflow")
                    .join("config.toml"),
            };
            let language = mochiflow_core::config::load_config(&path)
                .map(|c| c.conversation_output_language().to_string())
                .unwrap_or_else(|_| "en".to_string());
            print!("{}", mochiflow_core::present::render_guide(&language));
            0
        }
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "mochiflow", &mut std::io::stdout());
            0
        }
        Commands::Pr {
            spec,
            title,
            body_file,
            draft,
            dry_run,
        } => {
            let cfg = load_cfg(cli.config.as_deref())?;
            let code = mochiflow_core::pr::run_pr(
                &cfg,
                spec.as_deref(),
                title.as_deref(),
                body_file.as_deref(),
                draft,
                dry_run,
            );
            // Regenerate the gitignored board to reflect the new delivery state
            // (e.g. In Review). Skipped on dry-run; never staged or committed.
            if !dry_run
                && (code == mochiflow_core::pr::EXIT_OK || code == mochiflow_core::pr::EXIT_MANUAL)
            {
                refresh_board_after_state_change(&cfg);
            }
            code
        }
        Commands::Freeze { root, check } => {
            let cwd = std::env::current_dir()?;
            let repo_root = if let Some(root) = root {
                match mochiflow_core::freeze::validate_repo_root(std::path::Path::new(&root)) {
                    Ok(r) => r,
                    Err(e) => {
                        println!("{e}");
                        std::process::exit(1);
                    }
                }
            } else {
                match mochiflow_core::freeze::resolve_repo_root(&cwd) {
                    Ok(r) => r,
                    Err(e) => {
                        println!("{e}");
                        std::process::exit(1);
                    }
                }
            };
            match mochiflow_core::freeze::freeze(&repo_root, check) {
                Ok(report) => {
                    if check {
                        if report.stale.is_empty() {
                            println!("freeze: all derived files are up to date");
                            0
                        } else {
                            for entry in &report.stale {
                                println!("STALE: {}", entry.path.display());
                            }
                            1
                        }
                    } else {
                        for path in &report.written {
                            println!("wrote: {}", path.display());
                        }
                        if report.written.is_empty() {
                            println!("freeze: nothing to update");
                        }
                        0
                    }
                }
                Err(e) => {
                    println!("FAIL: {e}");
                    1
                }
            }
        }
    };

    std::process::exit(exit_code);
}

/// Regenerate the gitignored board (`INDEX.md` + state index) after a
/// state-changing CLI command. Best-effort: failures are ignored, and the file
/// is never staged or committed. Callers gate this on a successful, non-dry-run
/// invocation so the board reflects the new delivery state.
fn refresh_board_after_state_change(cfg: &mochiflow_core::config::Config) {
    let _ = mochiflow_core::index::generate_index_quiet(cfg);
}

fn bundled_engine_version() -> String {
    EMBEDDED_ENGINE
        .get_file("VERSION")
        .and_then(|f| f.contents_utf8())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn cmd_config_show(cfg: &mochiflow_core::config::Config, bundled_engine_version: &str) {
    println!("schema_version : {}", cfg.schema_version);
    println!(
        "installed_engine_version : {}",
        mochiflow_core::manifest::engine_version_label(&cfg.engine_dir())
    );
    println!("bundled_engine_version   : {bundled_engine_version}");
    println!("i18n           :");
    println!("  artifact     : {}", cfg.i18n.artifact_language);
    println!("  conversation : {}", cfg.i18n.conversation_language);
    println!("repo_root      : {}", cfg.repo_root.display());
    println!("install_dir    : {}", cfg.install_dir_path().display());
    println!("engine_dir     : {}", cfg.engine_dir().display());
    println!("specs_dir      : {}", cfg.specs_dir_path().display());
    println!("index          : {}", cfg.index_path().display());
    println!("constitution   :");
    println!("  project      : {}", cfg.constitution_path().display());
    println!(
        "  local        : {}",
        cfg.constitution_local_path().display()
    );
    println!("context(refresh):");
    println!("  product      : {}", cfg.product_path().display());
    println!("  structure    : {}", cfg.structure_path().display());
    println!("  tech         : {}", cfg.tech_path().display());
    println!("adr (fold)     :");
    println!("  decisions    : {}", cfg.decisions_dir().display());
    println!("  pitfalls     : {}", cfg.pitfalls_dir().display());
    println!(
        "git            : {} base={}",
        cfg.git.provider, cfg.git.base_branch
    );
    println!("  pr_command   : {}", cfg.git.pr_command);
    println!("adapter        : {}", cfg.primary_tool());
    println!("surfaces       :");
    for name in cfg.surface_names() {
        let surface = &cfg.surfaces[&name];
        println!("  {}: {}", name, surface.description);
        for (profile, cmd) in &surface.verify {
            println!("    {}: {}", profile, cmd);
        }
    }
}

fn cmd_ready(cfg: &mochiflow_core::config::Config, spec_arg: &str) -> i32 {
    use std::path::PathBuf;

    let spec_dir = {
        let p = PathBuf::from(spec_arg);
        if p.is_file() {
            p.parent().unwrap_or(&p).to_path_buf()
        } else if p.is_absolute() || p.exists() {
            p
        } else {
            cfg.specs_dir_path().join(spec_arg)
        }
    };

    if mochiflow_core::lint::run_lint(
        cfg,
        Some(
            spec_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(spec_arg),
        ),
        false,
    ) != 0
    {
        return 1;
    }
    match mochiflow_core::spec_meta::read_spec_metadata(&spec_dir) {
        Ok(meta) => {
            if meta.status() != "approved" {
                println!(
                    "FAIL: {}: status must be approved to enter build, got {}",
                    meta.path.display(),
                    meta.status()
                );
                return 1;
            }
            for surface in meta.surfaces() {
                let command = match cfg.verify_command(surface, "default", None) {
                    Ok(command) => command,
                    Err(e) => {
                        println!(
                            "FAIL: {}: verification command for surface `{surface}` is not runnable: {e}",
                            meta.path.display()
                        );
                        return 1;
                    }
                };
                if command.trim_start().starts_with("TODO:") {
                    println!(
                        "FAIL: {}: verification command for surface `{surface}` is not runnable: {command}",
                        meta.path.display()
                    );
                    return 1;
                }
            }
            println!(
                "READY: {} ({}, risk={})",
                meta.slug(),
                meta.spec_type(),
                meta.risk()
            );
            0
        }
        Err(e) => {
            println!("FAIL: {}: {e}", spec_dir.join("spec.yaml").display());
            1
        }
    }
}

fn load_cfg(config_path: Option<&str>) -> Result<mochiflow_core::config::Config> {
    let path = match config_path {
        Some(p) => std::path::PathBuf::from(p),
        None => {
            let cwd = std::env::current_dir()?;
            cwd.join(".mochiflow").join("config.toml")
        }
    };
    match mochiflow_core::config::load_config(&path) {
        Ok(cfg) => Ok(cfg),
        Err(e) => {
            println!("FAIL: {e}");
            std::process::exit(2);
        }
    }
}

fn load_detach_cfg(
    config_path: Option<&str>,
    target: &str,
    purge: bool,
    dry_run: bool,
    json: bool,
) -> Result<mochiflow_core::config::Config> {
    let path = match config_path {
        Some(p) => std::path::PathBuf::from(p),
        None => std::path::PathBuf::from(target)
            .join(".mochiflow")
            .join("config.toml"),
    };
    match mochiflow_core::config::load_config(&path) {
        Ok(cfg) => Ok(cfg),
        Err(e) => {
            if json {
                let doc = serde_json::json!({
                    "mode": if purge { "purge" } else { "detach" },
                    "dry_run": dry_run,
                    "removed": [],
                    "updated": [],
                    "kept": [],
                    "skipped": [],
                    "errors": [format!("{e}")],
                    "exit_code": 2,
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&doc).unwrap_or_else(|_| "{}".into())
                );
            } else {
                println!("FAIL: {e}");
            }
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn doctor_terminal_command_allowlist_matches_clap_subcommands() {
        let actual: BTreeSet<_> = Cli::command()
            .get_subcommands()
            .map(|command| command.get_name().to_string())
            .filter(|name| name != "help")
            .collect();
        let allowed: BTreeSet<_> = mochiflow_core::doctor::terminal_cli_command_references()
            .iter()
            .map(|command| (*command).to_string())
            .collect();

        assert_eq!(allowed, actual);
        assert!(mochiflow_core::doctor::workflow_command_references().contains(&"discuss"));
    }
}
