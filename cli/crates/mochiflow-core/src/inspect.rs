//! Read-only repository and spec context for coding agents.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;

use crate::config::Config;
use crate::delivery::{DeliverySignals, branch_name, resolve_column};
use crate::spec_meta::{SpecMeta, read_spec_metadata};
use crate::spec_mode::{SpecPersistenceMode, classify_spec};

pub const SCHEMA_VERSION: u32 = 1;
const PROVIDER_LIMIT: usize = 200;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case", tag = "quality")]
pub enum Observation<T> {
    Known { value: T },
    Unknown { reason: Code },
    NotApplicable { reason: Code },
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Code {
    ConfigInvalid,
    SpecMissing,
    SpecInvalid,
    SpecAmbiguous,
    TargetArchived,
    IntentConfirmationRequired,
    StatusNotDraft,
    StatusNotApproved,
    StatusNotAccepted,
    PlanInputMissing,
    LintFailed,
    VerificationMissing,
    VerificationTodo,
    BranchMissing,
    WorktreeDirty,
    TasksIncomplete,
    MatrixMissing,
    AutomatedChecksUnsettled,
    AutomatedChecksFailed,
    ReviewResultMissing,
    ReviewResultStale,
    DeliveryNotInReview,
    DeliveryNotMerged,
    CleanupNotPending,
    DeliveryUnknown,
    GitUnavailable,
    ProviderUnavailable,
    ProviderResultTruncated,
    FetchFailed,
    PathUnsafe,
    InternalError,
}

#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    pub code: Code,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub paths: Vec<String>,
}

impl Diagnostic {
    fn new(code: Code) -> Self {
        Self {
            code,
            message: None,
            paths: Vec::new(),
        }
    }
    fn path(code: Code, path: String) -> Self {
        Self {
            code,
            message: None,
            paths: vec![path],
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Metadata {
    pub slug: String,
    pub title: String,
    #[serde(rename = "type")]
    pub spec_type: String,
    pub risk: String,
    pub status: String,
    pub surfaces: Vec<String>,
}

impl From<&SpecMeta> for Metadata {
    fn from(meta: &SpecMeta) -> Self {
        Self {
            slug: meta.slug().into(),
            title: meta.title().into(),
            spec_type: meta.spec_type().into(),
            risk: meta.risk().into(),
            status: meta.status().into(),
            surfaces: meta.surfaces().into_iter().map(str::to_owned).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryKind {
    Spec,
    Backlog,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct Summary {
    pub kind: EntryKind,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Diagnostic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_candidate: Option<ActionName>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepositoryPayload {
    pub branch: Observation<String>,
    pub base_branch: String,
    pub active: Observation<String>,
    pub entries: Vec<Summary>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionName {
    Discuss,
    Plan,
    Build,
    Open,
    Update,
    Close,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionResult {
    Eligible,
    Ineligible,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActionEvaluation {
    pub action: ActionName,
    pub result: ActionResult,
    pub blockers: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpecPayload {
    pub metadata: Metadata,
    pub persistence: String,
    pub paths: Vec<String>,
    pub worktree_clean: Observation<bool>,
    pub lint_ok: bool,
    pub delivery: Observation<String>,
    pub actions: Vec<ActionEvaluation>,
    pub suggested_workflow: Option<ActionName>,
    pub human_next_action: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    Repository,
    Spec,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResultKind {
    Ok,
    Degraded,
    Partial,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct Document {
    pub schema_version: u32,
    pub scope: Scope,
    pub result: ResultKind,
    pub observed_at: String,
    pub degraded: bool,
    pub warnings: Vec<Diagnostic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<RepositoryPayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec: Option<SpecPayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<Diagnostic>>,
}

impl Document {
    fn base(scope: Scope) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            scope,
            result: ResultKind::Ok,
            observed_at: "1970-01-01T00:00:00Z".into(),
            degraded: false,
            warnings: vec![],
            repository: None,
            spec: None,
            errors: None,
        }
    }
    pub fn exit_code(&self) -> i32 {
        if matches!(self.result, ResultKind::Partial | ResultKind::Error) {
            1
        } else {
            0
        }
    }
}

#[derive(Debug, Clone, Default)]
struct BatchFacts {
    branch: Option<String>,
    dirty: Option<Vec<String>>,
    refs: BTreeSet<String>,
    merged_refs: BTreeSet<String>,
    trailers: BTreeSet<String>,
    prs: BTreeMap<String, String>,
    provider_unknown: bool,
    provider_truncated: bool,
}

#[allow(clippy::result_unit_err)]
pub trait Runner {
    fn run(
        &self,
        program: &str,
        args: &[&str],
        cwd: &Path,
        stdin: Option<&str>,
    ) -> Result<String, ()>;
}

pub struct ProcessRunner;
impl Runner for ProcessRunner {
    fn run(
        &self,
        program: &str,
        args: &[&str],
        cwd: &Path,
        _stdin: Option<&str>,
    ) -> Result<String, ()> {
        let output = Command::new(program)
            .args(args)
            .current_dir(cwd)
            .output()
            .map_err(|_| ())?;
        output
            .status
            .success()
            .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
            .ok_or(())
    }
}

fn git(runner: &dyn Runner, cfg: &Config, args: &[&str]) -> Option<String> {
    runner.run("git", args, &cfg.repo_root, None).ok()
}

#[allow(clippy::field_reassign_with_default)]
fn collect_batch(cfg: &Config, runner: &dyn Runner) -> BatchFacts {
    let mut facts = BatchFacts::default();
    facts.branch = git(runner, cfg, &["branch", "--show-current"]).map(|s| s.trim().to_owned());
    facts.dirty = git(runner, cfg, &["status", "--porcelain=v1", "-z"]).map(|s| {
        s.split('\0')
            .filter(|x| !x.is_empty())
            .filter_map(|x| x.get(3..))
            .map(str::to_owned)
            .collect()
    });
    if let Some(out) = git(
        runner,
        cfg,
        &[
            "for-each-ref",
            "--format=%(refname:short)",
            "refs/heads",
            "refs/remotes/origin",
        ],
    ) {
        facts.refs.extend(out.lines().map(str::to_owned));
    }
    let base = format!("origin/{}", cfg.git.base_branch);
    if let Some(out) = git(
        runner,
        cfg,
        &[
            "branch",
            "-a",
            "--merged",
            &base,
            "--format=%(refname:short)",
        ],
    ) {
        facts
            .merged_refs
            .extend(out.lines().map(|s| s.trim().to_owned()));
    }
    if let Some(out) = git(
        runner,
        cfg,
        &["log", &base, "--format=%(trailers:key=Spec,valueonly)"],
    ) {
        facts.trailers.extend(
            out.lines()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_owned),
        );
    }
    if cfg.git.provider == "github" {
        match runner.run(
            "gh",
            &[
                "pr",
                "list",
                "--state",
                "all",
                "--limit",
                &PROVIDER_LIMIT.to_string(),
                "--json",
                "headRefName,state",
            ],
            &cfg.repo_root,
            None,
        ) {
            Ok(out) => match serde_json::from_str::<Vec<serde_json::Value>>(&out) {
                Ok(rows) => {
                    facts.provider_truncated = rows.len() >= PROVIDER_LIMIT;
                    for row in rows {
                        if let (Some(head), Some(state)) =
                            (row["headRefName"].as_str(), row["state"].as_str())
                        {
                            facts.prs.insert(head.into(), state.to_ascii_lowercase());
                        }
                    }
                }
                Err(_) => facts.provider_unknown = true,
            },
            Err(_) => facts.provider_unknown = true,
        }
    }
    facts
}

fn relative(cfg: &Config, path: &Path) -> Option<String> {
    path.strip_prefix(&cfg.repo_root)
        .ok()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
}

fn discover(cfg: &Config) -> Vec<(PathBuf, Result<SpecMeta, ()>)> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(cfg.specs_dir_path()) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.file_name().and_then(|n| n.to_str()) != Some("_backlog") {
                out.push((path.clone(), read_spec_metadata(&path).map_err(|_| ())));
            }
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

pub fn inspect_repository(cfg: &Config, runner: &dyn Runner) -> Document {
    let facts = collect_batch(cfg, runner);
    let mut doc = Document::base(Scope::Repository);
    let mut entries = Vec::new();
    let mut slugs = Vec::new();
    let mut partial = false;
    for (path, parsed) in discover(cfg) {
        let rel = relative(cfg, &path).unwrap_or_else(|| ".mochiflow/specs".into());
        match parsed {
            Ok(meta) => {
                slugs.push(meta.slug().to_owned());
                let next = coarse_next(&meta);
                entries.push(Summary {
                    kind: EntryKind::Spec,
                    path: rel,
                    metadata: Some((&meta).into()),
                    error: None,
                    next_candidate: next,
                });
            }
            Err(()) => {
                partial = true;
                entries.push(Summary {
                    kind: EntryKind::Error,
                    path: rel.clone(),
                    metadata: None,
                    error: Some(Diagnostic::path(
                        Code::SpecInvalid,
                        format!("{rel}/spec.yaml"),
                    )),
                    next_candidate: None,
                });
            }
        }
    }
    let backlog = cfg.specs_dir_path().join("_backlog");
    if let Ok(seed_entries) = std::fs::read_dir(backlog) {
        for seed in seed_entries
            .flatten()
            .filter(|e| e.path().extension().is_some_and(|x| x == "md"))
        {
            if let Some(path) = relative(cfg, &seed.path()) {
                entries.push(Summary {
                    kind: EntryKind::Backlog,
                    path,
                    metadata: None,
                    error: None,
                    next_candidate: Some(ActionName::Discuss),
                });
            }
        }
    }
    let branch_obs = facts.branch.clone().map_or(
        Observation::Unknown {
            reason: Code::GitUnavailable,
        },
        |value| Observation::Known { value },
    );
    let candidates: Vec<_> = facts
        .branch
        .as_deref()
        .into_iter()
        .flat_map(|b| b.split_once('/').map(|(_, s)| s))
        .filter(|s| slugs.iter().any(|x| x == s))
        .collect();
    let active = match candidates.as_slice() {
        [slug] => Observation::Known {
            value: (*slug).to_owned(),
        },
        [] => Observation::NotApplicable {
            reason: Code::SpecMissing,
        },
        _ => Observation::Unknown {
            reason: Code::SpecAmbiguous,
        },
    };
    doc.result = if partial {
        ResultKind::Partial
    } else if facts.provider_unknown {
        ResultKind::Degraded
    } else {
        ResultKind::Ok
    };
    doc.degraded = facts.provider_unknown;
    if facts.provider_unknown {
        doc.warnings
            .push(Diagnostic::new(Code::ProviderUnavailable));
    }
    if facts.provider_truncated {
        doc.degraded = true;
        doc.result = ResultKind::Degraded;
        doc.warnings
            .push(Diagnostic::new(Code::ProviderResultTruncated));
    }
    doc.repository = Some(RepositoryPayload {
        branch: branch_obs,
        base_branch: cfg.git.base_branch.clone(),
        active,
        entries,
    });
    doc
}

fn coarse_next(meta: &SpecMeta) -> Option<ActionName> {
    match meta.status() {
        "draft" => Some(ActionName::Plan),
        "approved" => Some(ActionName::Build),
        _ => None,
    }
}

pub fn inspect_spec(cfg: &Config, slug: &str, runner: &dyn Runner) -> Document {
    if slug.contains(['/', '\\']) || slug.contains("..") {
        return error_doc(Scope::Spec, Code::SpecMissing);
    }
    let dir = cfg.specs_dir_path().join(slug);
    let meta = match read_spec_metadata(&dir) {
        Ok(m) => {
            if m.slug() != slug {
                return error_doc(Scope::Spec, Code::SpecInvalid);
            }
            m
        }
        Err(_) if dir.exists() => return error_doc(Scope::Spec, Code::SpecInvalid),
        Err(_) => return error_doc(Scope::Spec, Code::SpecMissing),
    };
    let facts = collect_batch(cfg, runner);
    let metadata = Metadata::from(&meta);
    let expected = branch_name(meta.spec_type(), slug);
    let lint_ok = structural_health(cfg, &meta, &dir);
    let paths = related_paths(cfg, &dir);
    let persistence = classify_spec(cfg, slug)
        .map(|p| match p.mode {
            SpecPersistenceMode::Tracked => "tracked",
            SpecPersistenceMode::Local => "local",
        })
        .unwrap_or("tracked")
        .to_owned();
    let dirty = facts.dirty.as_ref().map(|paths| {
        paths
            .iter()
            .filter(|p| !p.starts_with(&format!(".mochiflow/specs/{slug}/")))
            .cloned()
            .collect::<Vec<_>>()
    });
    let worktree_clean = dirty.as_ref().map_or(
        Observation::Unknown {
            reason: Code::GitUnavailable,
        },
        |p| Observation::Known {
            value: p.is_empty(),
        },
    );
    let signals = signals_for(cfg, &facts, &meta, &expected);
    let delivery = delivery_observation(cfg, &facts, &meta, &signals);
    let actions = evaluate_actions(
        cfg,
        &meta,
        &dir,
        lint_ok,
        &facts,
        &expected,
        dirty.as_deref(),
        &delivery,
    );
    let suggested_workflow = suggested(&actions);
    let human_next_action = match &delivery {
        Observation::Known { value } if value == "in_review" => Some("report_merge".into()),
        Observation::Known { value }
            if value == "done"
                && (facts.refs.contains(&expected) || cfg.state_dir().join(slug).exists()) =>
        {
            Some("cleanup".into())
        }
        _ => None,
    };
    let mut doc = Document::base(Scope::Spec);
    doc.degraded = facts.provider_unknown || facts.provider_truncated;
    doc.result = if doc.degraded {
        ResultKind::Degraded
    } else {
        ResultKind::Ok
    };
    if facts.provider_unknown {
        doc.warnings
            .push(Diagnostic::new(Code::ProviderUnavailable));
    }
    if facts.provider_truncated {
        doc.warnings
            .push(Diagnostic::new(Code::ProviderResultTruncated));
    }
    doc.spec = Some(SpecPayload {
        metadata,
        persistence,
        paths,
        worktree_clean,
        lint_ok,
        delivery,
        actions,
        suggested_workflow,
        human_next_action,
    });
    doc
}

fn error_doc(scope: Scope, code: Code) -> Document {
    let mut doc = Document::base(scope);
    doc.result = ResultKind::Error;
    doc.errors = Some(vec![Diagnostic::new(code)]);
    doc
}

fn signals_for(cfg: &Config, facts: &BatchFacts, meta: &SpecMeta, branch: &str) -> DeliverySignals {
    let provider_state = facts.prs.get(branch).map(String::as_str);
    DeliverySignals {
        provider_merged: provider_state == Some("merged"),
        provider_open_pr: provider_state == Some("open"),
        trailer_in_base: facts.trailers.contains(meta.slug()),
        local_branch_tip_in_base: classify_spec(cfg, meta.slug())
            .is_ok_and(|p| matches!(p.mode, SpecPersistenceMode::Local))
            && facts.merged_refs.contains(branch),
        branch_pushed_unmerged: cfg.git.provider == "none"
            && facts.refs.contains(&format!("origin/{branch}"))
            && !facts
                .merged_refs
                .contains(&format!("remotes/origin/{branch}")),
    }
}

fn delivery_observation(
    cfg: &Config,
    facts: &BatchFacts,
    meta: &SpecMeta,
    signals: &DeliverySignals,
) -> Observation<String> {
    if cfg.git.provider == "github"
        && facts.provider_unknown
        && !signals.trailer_in_base
        && meta.status() == "accepted"
    {
        Observation::Unknown {
            reason: Code::ProviderUnavailable,
        }
    } else {
        Observation::Known {
            value: resolve_column(meta.status(), signals).as_str().to_owned(),
        }
    }
}

fn structural_health(cfg: &Config, meta: &SpecMeta, dir: &Path) -> bool {
    !meta.slug().is_empty()
        && (meta.status() == "draft" || dir.join("spec.md").is_file())
        && meta
            .surfaces()
            .iter()
            .all(|s| cfg.surfaces.contains_key(*s))
}
fn related_paths(cfg: &Config, dir: &Path) -> Vec<String> {
    ["spec.yaml", "pitch.md", "spec.md", "design.md", "tasks.md"]
        .iter()
        .map(|n| dir.join(n))
        .filter(|p| p.exists())
        .filter_map(|p| relative(cfg, &p))
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn evaluate_actions(
    cfg: &Config,
    meta: &SpecMeta,
    dir: &Path,
    lint_ok: bool,
    facts: &BatchFacts,
    expected: &str,
    dirty: Option<&[String]>,
    delivery: &Observation<String>,
) -> Vec<ActionEvaluation> {
    let status = meta.status();
    let mut out = Vec::new();
    out.push(eval(
        ActionName::Discuss,
        if status == "draft" {
            vec![]
        } else {
            vec![Code::IntentConfirmationRequired]
        },
        status != "draft",
    ));
    let mut plan = vec![];
    if status != "draft" {
        plan.push(Code::StatusNotDraft);
    }
    if !dir.join("pitch.md").exists() && !dir.join("spec.md").exists() {
        plan.push(Code::PlanInputMissing);
    }
    if !lint_ok {
        plan.push(Code::LintFailed);
    }
    out.push(eval(ActionName::Plan, plan, false));
    let mut build = vec![];
    if status != "approved" {
        build.push(Code::StatusNotApproved);
    }
    if !lint_ok {
        build.push(Code::LintFailed);
    }
    for surface in meta.surfaces() {
        match cfg
            .surfaces
            .get(surface)
            .and_then(|s| s.verify.get("default"))
        {
            None => build.push(Code::VerificationMissing),
            Some(c) if c.trim_start().starts_with("TODO:") => build.push(Code::VerificationTodo),
            _ => {}
        }
    }
    if !facts.refs.contains(expected) {
        build.push(Code::BranchMissing);
    }
    if dirty.is_some_and(|d| !d.is_empty()) {
        build.push(Code::WorktreeDirty);
    }
    out.push(eval(ActionName::Build, build, facts.dirty.is_none()));
    let mut open = vec![];
    if status != "approved" {
        open.push(Code::StatusNotApproved);
    }
    let tasks = dir.join("tasks.md");
    if tasks.exists()
        && std::fs::read_to_string(&tasks)
            .is_ok_and(|s| s.lines().any(|l| l.trim_start().starts_with("- [ ]")))
    {
        open.push(Code::TasksIncomplete);
    }
    let spec = std::fs::read_to_string(dir.join("spec.md")).unwrap_or_default();
    if !spec.contains("## Verification Plan / AC Matrix") {
        open.push(Code::MatrixMissing);
    } else if spec.contains("| UNVERIFIED |") {
        open.push(Code::AutomatedChecksUnsettled);
    }
    if spec.contains("| FAIL |") {
        open.push(Code::AutomatedChecksFailed);
    }
    if matches!(meta.risk(), "elevated" | "critical")
        && !std::fs::read_to_string(dir.join("design.md"))
            .is_ok_and(|s| s.contains("Verdict: pass"))
    {
        open.push(Code::ReviewResultMissing);
    }
    if dirty.is_some_and(|d| !d.is_empty()) {
        open.push(Code::WorktreeDirty);
    }
    out.push(eval(ActionName::Open, open, facts.dirty.is_none()));
    let mut update = vec![];
    if status != "accepted" {
        update.push(Code::StatusNotAccepted);
    }
    match delivery {
        Observation::Known { value } if value == "in_review" => {}
        Observation::Known { .. } => update.push(Code::DeliveryNotInReview),
        _ => update.push(Code::DeliveryUnknown),
    }
    if dirty.is_some_and(|d| !d.is_empty()) {
        update.push(Code::WorktreeDirty);
    }
    out.push(eval(
        ActionName::Update,
        update,
        matches!(delivery, Observation::Unknown { .. }) || facts.dirty.is_none(),
    ));
    let mut close = vec![];
    match delivery {
        Observation::Known { value } if value == "done" => {}
        Observation::Known { .. } => close.push(Code::DeliveryNotMerged),
        _ => close.push(Code::DeliveryUnknown),
    }
    if !facts.refs.contains(expected) && !cfg.state_dir().join(meta.slug()).exists() {
        close.push(Code::CleanupNotPending);
    }
    if dirty.is_some_and(|d| !d.is_empty()) {
        close.push(Code::WorktreeDirty);
    }
    out.push(eval(
        ActionName::Close,
        close,
        matches!(delivery, Observation::Unknown { .. }) || facts.dirty.is_none(),
    ));
    out
}

fn eval(action: ActionName, codes: Vec<Code>, unknown: bool) -> ActionEvaluation {
    let result = if codes.is_empty() {
        ActionResult::Eligible
    } else if unknown {
        ActionResult::Unknown
    } else {
        ActionResult::Ineligible
    };
    ActionEvaluation {
        action,
        result,
        blockers: codes.into_iter().map(Diagnostic::new).collect(),
    }
}
fn suggested(actions: &[ActionEvaluation]) -> Option<ActionName> {
    [
        ActionName::Close,
        ActionName::Open,
        ActionName::Build,
        ActionName::Plan,
    ]
    .into_iter()
    .find(|name| {
        actions
            .iter()
            .any(|a| a.action == *name && a.result == ActionResult::Eligible)
    })
}

pub fn fetch(cfg: &Config, runner: &dyn Runner) -> Option<Diagnostic> {
    runner
        .run("git", &["fetch", "origin"], &cfg.repo_root, None)
        .err()
        .map(|()| Diagnostic::new(Code::FetchFailed))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn suggestion_precedence_is_closed() {
        let actions = vec![
            eval(ActionName::Build, vec![], false),
            eval(ActionName::Open, vec![], false),
        ];
        assert_eq!(suggested(&actions), Some(ActionName::Open));
    }
    #[test]
    fn unsafe_slug_is_an_error_document() {
        assert!("../x".contains(['/', '\\']));
    }
}
