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
    /// Regenerate INDEX.md + state/index.json
    Index {
        /// Report drift without writing
        #[arg(long)]
        check: bool,
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
    /// Developer engine maintenance commands
    #[command(hide = true)]
    Engine {
        #[command(subcommand)]
        command: EngineCommand,
    },
}

#[derive(Subcommand)]
enum ConfigCommand {
    Show,
    Validate,
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

#[derive(Subcommand)]
enum EngineCommand {
    /// Regenerate MANIFEST.json for an engine directory
    Manifest {
        /// Engine directory to hash
        #[arg(long, default_value = "engine")]
        engine_dir: String,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum DoctorTarget {
    Config,
    Specs,
    Adapter,
    Engine,
}

impl DoctorTarget {
    fn as_str(self) -> &'static str {
        match self {
            Self::Config => "config",
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
                if json {
                    eprintln!(
                        "MochiFlow is already initialized; running join-style local setup. Use `mochiflow join` for existing projects."
                    );
                } else {
                    println!(
                        "MochiFlow is already initialized; running join-style local setup. Use `mochiflow join` for existing projects."
                    );
                }
                mochiflow_core::join::run_join(
                    &target,
                    dry_run,
                    json,
                    false,
                    &bundled_engine_version(),
                    Some(&extract_fn),
                )
            } else {
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
            mochiflow_core::pr::run_pr(
                &cfg,
                spec.as_deref(),
                title.as_deref(),
                body_file.as_deref(),
                draft,
                dry_run,
            )
        }
        Commands::Engine { command } => match command {
            EngineCommand::Manifest { engine_dir } => {
                let engine_dir = std::path::PathBuf::from(engine_dir);
                match mochiflow_core::upgrade::write_manifest_for_engine_dir(&engine_dir) {
                    Ok(()) => {
                        println!("wrote {}", engine_dir.join("MANIFEST.json").display());
                        0
                    }
                    Err(e) => {
                        println!("FAIL: could not write {}: {e}", engine_dir.display());
                        1
                    }
                }
            }
        },
    };

    std::process::exit(exit_code);
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
    println!("  decisions    : {}", cfg.decisions_path().display());
    println!("  pitfalls     : {}", cfg.pitfalls_path().display());
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
