---
slug: "commit-lifecycle-unification"
title: "Unify commit timing across discuss/plan/build/ship on a single branch"
surface: "cli"
type_hint: "refactor"
maturity: "ready-for-plan"
source: "conversation"
source_phase: "discuss"
created: "2026-06-22"
updated: "2026-06-23"
---

# Unify commit timing across discuss/plan/build/ship on a single branch

## Decision Summary

全4フェーズ（discuss/plan/build/ship）のコミットを同一フィーチャーブランチ
`{prefix}/{slug}` 上に統一する。ブランチ作成は discuss agreement 時。
pitch.md を新規アーティファクトとして導入し、discuss の永続的成果物とする。
ready-for-plan handoff 形式は廃止し、pitch.md がその役割を担う。

## Decisions

- **ブランチ作成タイミング**: discuss agreement 時にブランチ作成 + switch。
  build が作成していた現行ルールを変更。合意に至らない途中セッション切れは
  raw seed で再開可能なため、途中コミットは不要。

- **pitch.md 導入**: discuss の成果物として `{specs_dir}/{slug}/pitch.md` を
  新規導入。7セクション構造: Problem / Appetite / Solution / Rabbit Holes /
  No-gos / Alternatives Considered / Open Questions。Shape Up の pitch + RFC の
  alternatives/unresolved questions を融合した設計。ライフサイクル全体で
  immutable に残り `_done/{slug}/` にアーカイブされる。

- **ready-for-plan handoff 廃止**: `_backlog/{slug}.md (maturity: ready-for-plan)`
  形式を廃止。pitch.md が plan への入力ソースとなる。
  `templates/backlog/discuss-handoff.md` は廃止。

- **`status: draft` の所有者変更**: plan → discuss に変更。discuss が
  `spec.yaml (status: draft)` を作成する。plan は既存の spec.yaml を読み、
  承認時に `status: approved` に変更する。

- **discuss 時点の spec.yaml 最小フィールド**: slug, title, status, type,
  created, updated。type はブランチ prefix 解決に必要。risk / surfaces /
  module は plan で追加。

- **`_backlog/{slug}.md` 削除タイミング**: discuss コミットで同時削除（seed →
  spec dir への昇格を原子的に行う）。seed が存在しない場合はスキップ。

- **plan の動作変更**: pitch.md を immutable として読み、spec.md の
  `## Background and Design Rationale` に情報を引き込む。承認後に1回コミット
  （spec.md + design.md + tasks.md + spec.yaml approved）。

- **build の簡素化**: ブランチ「作成」→「存在確認 + switch」に変更。ブランチが
  存在しなければエラー停止。spec ファイル初回ステージング責務は消滅。

- **ship**: 変更なし。pitch.md は spec dir の一部として既存 archive ロジックに
  自然に乗る。

- **コミット type**: discuss/plan のコミットは `docs` type（Conventional Commits
  準拠）。`Spec:` トレーラー付き。`discuss` / `plan` を独自 type にしない
  （外部ツール互換性のため）。

- **lint 変更**: `draft` は pitch.md のみ必須（spec.md 不要）。`approved` 以上は
  spec.md 必須。

- **develop-branch-workflow はやらない**: 全コミットは PR 経由。main/develop への
  直接 push は行わない。

## Assumptions

- discuss は探索的であり、合意に至らないケースがある。途中コミットの仕組みは
  不要（raw seed による再開で十分）。
- 複数 spec の並行 plan は `git switch` で対応する。頻発する場合は
  `parallel-spec-context-switch` seed で別途対応。
- pitch.md の情報は spec.md に要約として引き込まれるため、pitch.md を読まなくても
  spec.md だけで実装に必要な情報は得られる。

## Open Questions

None — ready for plan.

## Change Impact

- **エンジンドキュメント変更**: `commands/discuss.md`（forbidden_writes 解除、
  ブランチ作成・コミット手順追加）、`commands/plan.md`（pitch.md 読み取り、
  handoff 削除ステップ除去、コミットステップ追加）、`commands/build.md`（ブランチ
  作成ステップをswitch に変更）、`reference/git.md`（コミットライフサイクル表
  追加）、`reference/workflow.md`（Verbs and state 表: draft set by を discuss
  に変更、Backlog seeds セクション更新）。
- **テンプレート変更**: `templates/spec/pitch.md` 新規作成、
  `templates/backlog/discuss-handoff.md` 廃止。
- **CLI lint ロジック変更**: `status: draft` + pitch.md only を valid とする条件
  追加。
- **CLI コード変更なし**: ブランチ作成・コミットは git 操作でありエンジンドキュメント
  のガイダンスで制御される。

## Evidence

- 現行 `git.md ## Branch`: ブランチ作成は build step 2 で
  `origin/{base_branch}` から作成と明記。
- 現行 `discuss.md`: `forbidden_writes` に `{specs_dir}/{slug}/**` が含まれる。
- 現行 `workflow.md ## Verbs and state`: draft は plan が set。
- 現行 `git.md ## Ship close-out commit`: 既にフィーチャーブランチ上で close-out。
- `_backlog/develop-branch-workflow.md`: 直 push モデルの提案あり → 本 spec では
  採用しない判断。
- Shape Up (basecamp.com/shapeup/1.5-chapter-06): pitch の5要素
  (Problem/Appetite/Solution/Rabbit Holes/No-gos)。
- Rust RFC template: Summary/Motivation/Detailed design/Drawbacks/Alternatives/
  Unresolved questions の構造。
