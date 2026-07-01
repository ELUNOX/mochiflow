//! Output presenter (clig.dev-compliant) for the detection report and the
//! usage guide card.
//!
//! One report model is rendered three ways — `Pretty` (colour + Unicode glyphs),
//! `Plain` (no colour, ASCII glyphs), `Json` (machine-readable) — with colour
//! resolved from the `NO_COLOR` / `CLICOLOR` / `CLICOLOR_FORCE` env vars and the
//! stdout TTY (AC-05). Human-facing text never exposes internal key names
//! (surface / provider / base_branch — AC-06); the `Json` form is
//! for scriptability and is allowed to use field names.

use anstyle::{AnsiColor, Style};

use crate::detect::{Confidence, DetectionReport};

/// How the report is rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Pretty,
    Plain,
    Json,
}

/// Resolved colour decision (already accounts for env + TTY).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorChoice {
    Always,
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitStatus {
    Ready,
    NeedsAiReview,
    Blocked,
}

impl InitStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::NeedsAiReview => "needs_ai_review",
            Self::Blocked => "blocked",
        }
    }

    pub fn exit_code(self) -> i32 {
        match self {
            Self::Ready | Self::NeedsAiReview => 0,
            Self::Blocked => 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InitBlockedItem {
    pub target: String,
    pub candidate: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitLanguageSource {
    Flag,
    Prompt,
    Locale,
    Default,
    ExistingConfig,
    LegacyConfig,
    Docs,
}

impl InitLanguageSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Flag => "flag",
            Self::Prompt => "prompt",
            Self::Locale => "locale",
            Self::Default => "default",
            Self::ExistingConfig => "existing_config",
            Self::LegacyConfig => "legacy_config",
            Self::Docs => "docs",
        }
    }
}

pub struct InitJsonInput<'a> {
    pub report: &'a DetectionReport,
    pub target: &'a str,
    pub dry_run: bool,
    pub created_updated: &'a [String],
    pub status: InitStatus,
    pub extra_confirmation_items: &'a [String],
    pub blocked_items: &'a [InitBlockedItem],
    pub artifact_language: &'a str,
    pub conversation_language: &'a str,
    pub display_language: &'a str,
    pub artifact_language_source: InitLanguageSource,
    pub conversation_language_source: InitLanguageSource,
}

impl ColorChoice {
    /// Resolve colour per the [NO_COLOR](https://no-color.org) and
    /// [CLICOLOR / CLICOLOR_FORCE](https://bixense.com/clicolors) conventions:
    /// `CLICOLOR_FORCE` (set, non-`0`) forces colour; `NO_COLOR` (set, non-empty)
    /// or `CLICOLOR=0` disables it; otherwise colour is used only on a TTY.
    pub fn resolve(stdout_is_tty: bool) -> Self {
        let env = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
        if let Some(v) = env("CLICOLOR_FORCE")
            && v != "0"
        {
            return ColorChoice::Always;
        }
        if env("NO_COLOR").is_some() {
            return ColorChoice::Never;
        }
        if let Some(v) = env("CLICOLOR")
            && v == "0"
        {
            return ColorChoice::Never;
        }
        if stdout_is_tty {
            ColorChoice::Always
        } else {
            ColorChoice::Never
        }
    }
}

/// Per-item status: a confirmed fact vs. one needing human confirmation.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ItemStatus {
    Ok,
    Confirm,
}

impl ItemStatus {
    fn from_confidence(c: Confidence) -> Self {
        if c.needs_confirm() {
            ItemStatus::Confirm
        } else {
            ItemStatus::Ok
        }
    }

    /// Glyph for the status, ASCII in `Plain` and Unicode in `Pretty`.
    fn glyph(self, pretty: bool) -> &'static str {
        match (self, pretty) {
            (ItemStatus::Ok, true) => "✓",
            (ItemStatus::Confirm, true) => "⚠",
            (ItemStatus::Ok, false) => "[ok]",
            (ItemStatus::Confirm, false) => "[!] ",
        }
    }

    fn style(self) -> Style {
        match self {
            ItemStatus::Ok => Style::new().fg_color(Some(AnsiColor::Green.into())),
            ItemStatus::Confirm => Style::new().fg_color(Some(AnsiColor::Yellow.into())),
        }
    }
}

fn confidence_str(c: Confidence) -> &'static str {
    match c {
        Confidence::Low => "low",
        Confidence::Medium => "medium",
        Confidence::High => "high",
    }
}

/// Render one human line: `<glyph> <text>`, colourised by status when colour is on.
fn line(status: ItemStatus, text: &str, mode: OutputMode, color: ColorChoice) -> String {
    let pretty = mode == OutputMode::Pretty;
    let glyph = status.glyph(pretty);
    if color == ColorChoice::Always {
        let s = status.style();
        format!("  {}{}{} {}", s.render(), glyph, s.render_reset(), text)
    } else {
        format!("  {glyph} {text}")
    }
}

/// Render the detection report in the requested mode. Human modes frame the
/// facts without internal key names; `Json` emits a single machine-readable
/// document (one trailing newline, deterministic key order).
pub fn render_report(
    report: &DetectionReport,
    mode: OutputMode,
    color: ColorChoice,
    language: &str,
) -> String {
    if mode == OutputMode::Json {
        return render_json(report);
    }

    let mut out = String::new();
    let ja = language == "ja";
    out.push_str(if ja {
        "MochiFlow が検出した内容:\n\n"
    } else {
        "MochiFlow detected the following:\n\n"
    });

    for s in &report.surfaces {
        let status = ItemStatus::from_confidence(s.confidence);
        let text = if s.verify.starts_with("TODO:") {
            format!("{} — (no check command found; confirm one)", s.description)
        } else {
            format!("{} — runs: {}", s.description, s.verify)
        };
        out.push_str(&line(status, &text, mode, color));
        out.push('\n');
    }

    // Default branch (no internal key name).
    let branch_status = ItemStatus::from_confidence(report.git.branch_confidence);
    out.push_str(&line(
        branch_status,
        &format!("Default branch: {}", report.git.base_branch),
        mode,
        color,
    ));
    out.push('\n');

    // Detected remote provider is a fact to confirm, never auto-adopted.
    if report.git.has_known_provider() {
        out.push_str(&line(
            ItemStatus::Confirm,
            &format!(
                "Remote looks like {} — PRs stay manual unless you choose to automate",
                report.git.provider
            ),
            mode,
            color,
        ));
        out.push('\n');
    }

    out.push('\n');
    out.push_str(&render_next_step(language));
    out.push('\n');
    out
}

/// Render the human-facing `mochiflow init` completion summary. This keeps the
/// detection report available, but frames it around what happened, what still
/// needs confirmation, and the exact prompt to give the AI assistant next.
#[allow(clippy::too_many_arguments)]
pub fn render_init_summary(
    report: &DetectionReport,
    done_items: &[String],
    extra_confirmation_items: &[String],
    status: InitStatus,
    mode: OutputMode,
    color: ColorChoice,
    artifact_language: &str,
    conversation_language: &str,
    display_language: &str,
    artifact_language_source: InitLanguageSource,
    conversation_language_source: InitLanguageSource,
) -> String {
    let ja = display_language == "ja";
    let mut out = String::new();
    out.push_str("Detected:\n");
    out.push_str(&line(
        ItemStatus::Ok,
        &format!(
            "artifact language: {} ({})",
            artifact_language,
            artifact_language_source.as_str()
        ),
        mode,
        color,
    ));
    out.push('\n');
    out.push_str(&line(
        ItemStatus::Ok,
        &format!(
            "conversation language: {} ({})",
            conversation_language,
            conversation_language_source.as_str()
        ),
        mode,
        color,
    ));
    out.push('\n');
    for s in &report.surfaces {
        let status = ItemStatus::from_confidence(s.confidence);
        let text = if s.verify.starts_with("TODO:") {
            format!("{} — no verification command found", s.description)
        } else {
            format!("{} — runs: {}", s.description, s.verify)
        };
        out.push_str(&line(status, &text, mode, color));
        out.push('\n');
    }
    out.push_str(if ja {
        "作成/更新:\n"
    } else {
        "Created/Updated:\n"
    });
    for item in done_items {
        out.push_str(&line(ItemStatus::Ok, item, mode, color));
        out.push('\n');
    }

    out.push('\n');
    out.push_str("Status:\n");
    let status_line = match (status, ja) {
        (InitStatus::Ready, true) => "Ready — そのまま使えます",
        (InitStatus::Ready, false) => "Ready — MochiFlow is ready to use",
        (InitStatus::NeedsAiReview, true) => {
            "Next — 下の文を AI アシスタントに貼って初期設定を完了してください"
        }
        (InitStatus::NeedsAiReview, false) => {
            "Next — paste the setup prompt below into your AI agent"
        }
        (InitStatus::Blocked, true) => "Blocked — 構造化 adapter の手動統合が必要です",
        (InitStatus::Blocked, false) => "Blocked — structured adapter files need manual merge",
    };
    out.push_str(&line(
        if status == InitStatus::Ready {
            ItemStatus::Ok
        } else {
            ItemStatus::Confirm
        },
        status_line,
        mode,
        color,
    ));
    out.push('\n');

    out.push('\n');
    out.push_str(if ja {
        "確認が必要:\n"
    } else {
        "Needs review:\n"
    });
    let mut confirm_items = confirmation_items(report, display_language);
    confirm_items.extend(extra_confirmation_items.iter().cloned());
    if confirm_items.is_empty() {
        out.push_str(&line(
            ItemStatus::Ok,
            if ja {
                "確認が必要な項目はありません"
            } else {
                "No remaining confirmation items"
            },
            mode,
            color,
        ));
        out.push('\n');
    } else {
        for item in confirm_items {
            out.push_str(&line(ItemStatus::Confirm, &item, mode, color));
            out.push('\n');
        }
        if status == InitStatus::NeedsAiReview {
            out.push_str(&line(
                ItemStatus::Confirm,
                if ja {
                    "`# mochiflow: confirm` と TODO は検出が不確かな値を確認するための質問で、エラーではありません"
                } else {
                    "`# mochiflow: confirm` markers and TODOs are setup questions for uncertain detected values, not errors"
                },
                mode,
                color,
            ));
            out.push('\n');
        }
    }

    out.push('\n');
    out.push_str("Next:\n");
    out.push_str(&match status {
        InitStatus::Ready => render_ready_next(display_language),
        InitStatus::NeedsAiReview => render_ai_review_prompt(display_language),
        InitStatus::Blocked => render_blocked_next(display_language),
    });
    out.push('\n');
    out
}

fn confirmation_items(report: &DetectionReport, language: &str) -> Vec<String> {
    let ja = language == "ja";
    let mut items = Vec::new();

    for s in &report.surfaces {
        if s.confidence.needs_confirm() {
            if s.verify.starts_with("TODO:") {
                items.push(if ja {
                    format!("{} の検証コマンドを決める", s.description)
                } else {
                    format!("Choose a verification command for {}", s.description)
                });
            } else {
                items.push(if ja {
                    format!("{} の検証コマンドを確認: {}", s.description, s.verify)
                } else {
                    format!(
                        "Confirm verification command for {}: {}",
                        s.description, s.verify
                    )
                });
            }
        }
    }

    if report.git.branch_confidence.needs_confirm() {
        items.push(if ja {
            format!("既定ブランチを確認: {}", report.git.base_branch)
        } else {
            format!("Confirm default branch: {}", report.git.base_branch)
        });
    }

    if report.git.has_known_provider() {
        items.push(if ja {
            format!(
                "リモートは {} のようです。PR は手動のままにするか、自動化するか確認",
                report.git.provider
            )
        } else {
            format!(
                "Remote looks like {}; confirm manual PR handoff or automation",
                report.git.provider
            )
        });
    }

    items
}

/// The single next-step trigger line (AC-04).
pub fn render_next_step(language: &str) -> String {
    if language == "ja" {
        "次の一手: 問題なければ mochiflow init を実行してください。".to_string()
    } else {
        "Next: run mochiflow init if this setup looks right.".to_string()
    }
}

/// Copy-paste prompt shown after init. This is intentionally more specific than
/// `render_next_step` because it is the first-run setup prompt from CLI to AI.
pub fn render_ai_review_prompt(language: &str) -> String {
    if language == "ja" {
        "AI アシスタントにこの文を貼ってください:\n\n\
         MochiFlow の初期設定を完成させてください。.mochiflow/config.toml を読み、\
         \"# mochiflow: confirm\" と TODO は検出が不確かな値を確認するための質問として扱い、\
         私と確認しながら解決してください。\
         .mochiflow/context/product.md、.mochiflow/context/structure.md、\
         .mochiflow/context/tech.md をコードから埋め、\
         必要なら surfaces / verify / git 設定を調整し、\
         adapter を再生成して、最後に mochiflow doctor を通してください。"
            .to_string()
    } else {
        "Paste this into your AI agent:\n\n\
         Complete MochiFlow setup for this repository. Read .mochiflow/config.toml, \
         treat \"# mochiflow: confirm\" markers and TODOs as setup questions for uncertain \
         detected values, resolve them with me, fill \
         .mochiflow/context/product.md, .mochiflow/context/structure.md, and \
         .mochiflow/context/tech.md from the code, \
         adjust surfaces / verify / git settings if needed, regenerate adapters, \
         and finish with mochiflow doctor."
            .to_string()
    }
}

pub fn render_ready_next(language: &str) -> String {
    if language == "ja" {
        "AI アシスタントに実装したい内容を伝えてください。".to_string()
    } else {
        "Tell your AI agent what you want to build.".to_string()
    }
}

pub fn render_blocked_next(language: &str) -> String {
    if language == "ja" {
        "構造化 adapter の candidate ファイルを確認して手動で統合するか、置き換える場合は --force で再実行してください。".to_string()
    } else {
        "Review the structured adapter candidate files and merge them manually, or re-run with --force to replace existing files.".to_string()
    }
}

pub fn render_init_json(input: InitJsonInput<'_>) -> String {
    let review_items = {
        let mut items = confirmation_items(input.report, input.display_language);
        items.extend(input.extra_confirmation_items.iter().cloned());
        items
    };
    let blocked_json: Vec<serde_json::Value> = input
        .blocked_items
        .iter()
        .map(|item| {
            serde_json::json!({
                "target": item.target,
                "candidate": item.candidate,
                "message": item.message,
            })
        })
        .collect();
    let review_required = input.status == InitStatus::NeedsAiReview;
    let next_message = match input.status {
        InitStatus::Ready => render_ready_next(input.display_language),
        InitStatus::NeedsAiReview => render_ai_review_prompt(input.display_language),
        InitStatus::Blocked => render_blocked_next(input.display_language),
    };
    let next_kind = match input.status {
        InitStatus::Ready => "start_building",
        InitStatus::NeedsAiReview => "ai_review",
        InitStatus::Blocked => "merge_adapter",
    };

    let doc = serde_json::json!({
        "schema_version": 1,
        "status": input.status.as_str(),
        "exit_code": input.status.exit_code(),
        "target": input.target,
        "dry_run": input.dry_run,
        "i18n": {
            "artifact_language": input.artifact_language,
            "artifact_language_source": input.artifact_language_source.as_str(),
            "conversation_language": input.conversation_language,
            "conversation_language_source": input.conversation_language_source.as_str(),
            "display_language": input.display_language,
        },
        "detected": detected_json(input.report),
        "created_updated": input.created_updated,
        "review": {
            "required": review_required,
            "items": review_items,
            "prompt": if review_required {
                serde_json::Value::String(render_ai_review_prompt(input.display_language))
            } else {
                serde_json::Value::Null
            },
        },
        "blocked": {
            "required": input.status == InitStatus::Blocked,
            "items": blocked_json,
        },
        "next": {
            "kind": next_kind,
            "message": next_message,
        },
    });
    let mut s = serde_json::to_string_pretty(&doc).unwrap_or_else(|_| "{}".to_string());
    s.push('\n');
    s
}

fn detected_json(report: &DetectionReport) -> serde_json::Value {
    let surfaces: Vec<serde_json::Value> = report
        .surfaces
        .iter()
        .map(|s| {
            serde_json::json!({
                "name": s.name,
                "description": s.description,
                "verify": s.verify,
                "confidence": confidence_str(s.confidence),
                "confirm": s.confidence.needs_confirm(),
            })
        })
        .collect();

    serde_json::json!({
        "surfaces": surfaces,
        "git": {
            "provider": "none",
            "detected_provider": if report.git.has_known_provider() {
                serde_json::Value::String(report.git.provider.clone())
            } else {
                serde_json::Value::Null
            },
            "base_branch": report.git.base_branch,
            "branch_confidence": confidence_str(report.git.branch_confidence),
        },
    })
}

/// Static usage-vocabulary card: the spec-lane verbs (natural-language
/// trigger + explicit command) and the two delivery approval gates. Content is fixed
/// and project-independent; only the language of the framing text varies.
pub fn render_guide(language: &str) -> String {
    if language == "ja" {
        "MochiFlow の使い方\n\
         \n\
         動詞 — AI に自然文で伝えるか、明示コマンドを使う:\n\
         \u{20}\u{20}• discuss — 目的/内容を壁打ち       (mochiflow-discuss)\n\
         \u{20}\u{20}• plan    — 仕様に落とす            (mochiflow-plan)\n\
         \u{20}\u{20}• build   — 承認済み仕様を実装       (mochiflow-build)\n\
         \u{20}\u{20}• open    — 受入して PR を作る        (mochiflow-open)\n\
         \u{20}\u{20}• update  — PR フィードバック対応     (mochiflow-update)\n\
         \u{20}\u{20}• close   — マージ後の整理           (mochiflow-close)\n\
         \n\
         承認は2回あなたが行う: build 前に仕様を承認し、PR 作成前に PR を承認する。\n"
            .to_string()
    } else {
        "MochiFlow — how to drive it\n\
         \n\
         Verbs — say it naturally to your AI, or use the explicit command:\n\
         \u{20}\u{20}• discuss — talk through the why/what   (mochiflow-discuss)\n\
         \u{20}\u{20}• plan    — turn it into a spec         (mochiflow-plan)\n\
         \u{20}\u{20}• build   — implement the approved spec (mochiflow-build)\n\
         \u{20}\u{20}• open    — accept, then open the PR    (mochiflow-open)\n\
         \u{20}\u{20}• update  — address PR feedback         (mochiflow-update)\n\
         \u{20}\u{20}• close   — wrap up after merge         (mochiflow-close)\n\
         \n\
         Two approvals are yours: approve the spec before build, and the PR before it is created.\n"
            .to_string()
    }
}

/// Build the machine-readable JSON document. Field names are intentional here
/// (scriptability), unlike the human modes.
fn render_json(report: &DetectionReport) -> String {
    let surfaces: Vec<serde_json::Value> = report
        .surfaces
        .iter()
        .map(|s| {
            serde_json::json!({
                "name": s.name,
                "description": s.description,
                "verify": s.verify,
                "confidence": confidence_str(s.confidence),
                "confirm": s.confidence.needs_confirm(),
            })
        })
        .collect();

    let doc = serde_json::json!({
        "surfaces": surfaces,
        "git": {
            // provider written to config is always "none" (manual PR default);
            // detected_provider records the fact for confirmation only.
            "provider": "none",
            "detected_provider": if report.git.has_known_provider() {
                serde_json::Value::String(report.git.provider.clone())
            } else {
                serde_json::Value::Null
            },
            "base_branch": report.git.base_branch,
            "branch_confidence": confidence_str(report.git.branch_confidence),
        },
        "needs_confirm": report.needs_any_confirm(),
    });

    let mut s = serde_json::to_string_pretty(&doc).unwrap_or_else(|_| "{}".to_string());
    s.push('\n');
    s
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::detect::{DetectedGit, DetectedSurface};

    fn report() -> DetectionReport {
        DetectionReport {
            surfaces: vec![DetectedSurface {
                name: "cli".into(),
                description: "Rust crate / workspace".into(),
                verify: "cargo test".into(),
                confidence: Confidence::High,
            }],
            git: DetectedGit {
                provider: "github".into(),
                base_branch: "main".into(),
                branch_confidence: Confidence::High,
            },
        }
    }

    #[test]
    fn color_resolution_respects_env_and_tty() {
        // Pure resolution by TTY is exercised here; env-var cases are covered by
        // the CLI integration tests (process-level env is global).
        // On a non-TTY with no env, colour is off.
        // (We cannot safely mutate process env in parallel unit tests.)
        assert_eq!(ColorChoice::resolve(false), ColorChoice::Never);
    }

    #[test]
    fn plain_uses_ascii_glyphs_and_no_ansi() {
        let out = render_report(&report(), OutputMode::Plain, ColorChoice::Never, "en");
        assert!(out.contains("[ok]"), "{out}");
        assert!(
            !out.contains('\u{1b}'),
            "plain must contain no ANSI escapes:\n{out}"
        );
        // confirm item for the detected provider
        assert!(out.contains("[!]"), "{out}");
    }

    #[test]
    fn pretty_with_color_emits_ansi() {
        let out = render_report(&report(), OutputMode::Pretty, ColorChoice::Always, "en");
        assert!(
            out.contains('\u{1b}'),
            "pretty+color must contain ANSI escapes"
        );
    }

    #[test]
    fn human_output_hides_internal_key_names() {
        // AC-06: no surface/provider/base_branch jargon — in both
        // a normal report and the empty-project fallback (description differs).
        let mut empty = report();
        empty.surfaces = vec![DetectedSurface {
            name: "app".into(),
            description: "main project".into(),
            verify: "TODO: define test command".into(),
            confidence: Confidence::Low,
        }];
        empty.git.provider = "none".into();
        for r in [report(), empty] {
            let out = render_report(&r, OutputMode::Plain, ColorChoice::Never, "en");
            for jargon in ["surface", "provider", "base_branch"] {
                assert!(
                    !out.to_lowercase().contains(jargon),
                    "human output leaked internal key '{jargon}':\n{out}"
                );
            }
        }
    }

    #[test]
    fn json_is_single_document_with_provider_none() {
        let out = render_report(&report(), OutputMode::Json, ColorChoice::Never, "en");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["git"]["provider"], "none");
        assert_eq!(v["git"]["detected_provider"], "github");
        assert_eq!(v["needs_confirm"], true);
        assert!(out.ends_with('\n'));
    }

    #[test]
    fn next_step_is_language_aware() {
        assert!(render_next_step("ja").contains("mochiflow init"));
        assert!(render_next_step("en").contains("mochiflow init"));
    }

    #[test]
    fn init_summary_has_status_and_copy_paste_prompt() {
        let out = render_init_summary(
            &report(),
            &["wrote .mochiflow/config.toml".into()],
            &[".kiro/agents/spec-change-reviewer.json needs manual merge".into()],
            InitStatus::NeedsAiReview,
            OutputMode::Plain,
            ColorChoice::Never,
            "en",
            "auto",
            "en",
            InitLanguageSource::Flag,
            InitLanguageSource::Default,
        );
        assert!(out.contains("Detected:"), "{out}");
        assert!(out.contains("artifact language: en (flag)"), "{out}");
        assert!(
            out.contains("conversation language: auto (default)"),
            "{out}"
        );
        assert!(out.contains("Created/Updated:"), "{out}");
        assert!(out.contains("Status:"), "{out}");
        assert!(
            out.contains("paste the setup prompt below into your AI agent"),
            "{out}"
        );
        assert!(out.contains("Needs review:"), "{out}");
        assert!(
            out.contains(".kiro/agents/spec-change-reviewer.json"),
            "{out}"
        );
        assert!(out.contains("not errors"), "{out}");
        assert!(out.contains("Paste this into your AI agent"), "{out}");
        assert!(out.contains(".mochiflow/config.toml"), "{out}");
    }

    #[test]
    fn init_summary_is_language_aware() {
        let out = render_init_summary(
            &report(),
            &[".mochiflow/config.toml を作成".into()],
            &[".kiro/agents/spec-builder.json は手動統合が必要です".into()],
            InitStatus::Blocked,
            OutputMode::Plain,
            ColorChoice::Never,
            "ja",
            "auto",
            "ja",
            InitLanguageSource::ExistingConfig,
            InitLanguageSource::Default,
        );
        assert!(
            out.contains("artifact language: ja (existing_config)"),
            "{out}"
        );
        assert!(
            out.contains("conversation language: auto (default)"),
            "{out}"
        );
        assert!(out.contains("作成/更新:"), "{out}");
        assert!(out.contains("Status:"), "{out}");
        assert!(out.contains("Blocked"), "{out}");
        assert!(out.contains("確認が必要:"), "{out}");
        assert!(out.contains("手動統合が必要"), "{out}");
        assert!(out.contains("手動で統合"), "{out}");
    }

    #[test]
    fn guide_card_is_static_with_verbs_commands_and_gates() {
        for lang in ["en", "ja"] {
            let card = render_guide(lang);
            for cmd in [
                "mochiflow-discuss",
                "mochiflow-plan",
                "mochiflow-build",
                "mochiflow-open",
                "mochiflow-update",
                "mochiflow-close",
            ] {
                assert!(card.contains(cmd), "{lang} card missing {cmd}:\n{card}");
            }
            assert!(
                !card.contains("mochiflow-ship"),
                "{lang} card must not reference the removed ship verb:\n{card}"
            );
        }
        // language framing differs, commands are identical (static content)
        assert!(render_guide("ja").contains("使い方"));
        assert!(render_guide("en").contains("how to drive it"));
        // an unknown language falls back to the English card
        assert_eq!(render_guide("fr"), render_guide("en"));
    }
}
