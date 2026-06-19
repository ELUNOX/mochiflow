<p align="center">
  <img src="assets/mochiflow-mark.png" alt="MochiFlow" width="120">
</p>

# MochiFlow へのコントリビュート

貢献に興味を持っていただきありがとうございます。このガイドでは、開発環境の構築、
本プロジェクト固有のルール、プルリクエストの出し方を説明します。

English version: [CONTRIBUTING.md](CONTRIBUTING.md)

参加にあたっては [行動規範](CODE_OF_CONDUCT.md) の遵守に同意したものとみなされます。

## 開発環境の構築

MochiFlow の唯一の実装は `cli/` 配下の Rust CLI です。`rust-toolchain.toml`
で固定された Rust ツールチェーン（edition 2024、Rust 1.96.0）が必要です。

```bash
# ビルド
cargo build --manifest-path cli/Cargo.toml

# テスト + コンフォーマンス一式を実行（これが正規の検証コマンド）
cargo test --manifest-path cli/Cargo.toml

# supply-chain / license チェック
cargo deny --manifest-path cli/Cargo.toml check --config cli/deny.toml
```

コンフォーマンススイート（`cli/crates/mochiflow-cli/tests/conformance.rs`）が
挙動の真実のソースです: JSONスキーマの受理/拒否、`index` の golden 一致、MANIFEST
ドリフト検出、凍結面のバージョンゲート、lint / doctor / config / adapter / upgrade
の挙動アサーション。`tests/` と `contracts/` 配下のコミット済み golden fixtures と
スキーマが裏付けます。`cargo test` が通るまで変更は完了ではありません。

## どこを編集するか

どのツリーがファイルを所有しているかを知ることが、よくある失敗の防止になります:

- **エンジンのソース（ここを編集）** — リポジトリルートの `engine/`（`commands/`・
  `reference/`・`templates/`・`agents/`・`adapters/`）。プロジェクト非依存のコアです。
  エンジンのドキュメントは **英語** で書き、プロジェクト固有値を含めません。固有値は
  `config.toml` に置きます。
- **生成される `MANIFEST.json`** — エンジンファイルを編集したら
  `mochiflow engine manifest` で manifest を再生成します（手編集せず、生成される
  ハッシュマップ）。
- **ベンダリングされたエンジンの複製（編集禁止）** — `.mochiflow/engine/` は dogfood 実行が
  使う gitignore 済みのインストールスナップショットです。リポジトリルートの `engine/` から
  `mochiflow upgrade` で同期され、**ソースではありません**。
- **生成されるアダプタ（手編集禁止）** — ツールのエントリポイント（`AGENTS.md`・`.kiro/`・
  `CLAUDE.md`・`.github/copilot-instructions.md`）は `engine/adapters/<tool>/*.tpl` から
  レンダリングされます。テンプレートを編集し、`mochiflow adapter generate` を実行します。

## コントラクトとバージョンゲート

契約面は `contracts/contracts.lock` で凍結されています。変更がスキーマ
（`contracts/*.json`）やその他のロック対象ファイルに触れる場合、**同一コミット内**で:

1. `contracts.lock` を再生成し、
2. `engine/VERSION` を更新する

必要があります。これによりバージョンゲートが正直に保たれます（スキーマ変更が無断で
バージョン未管理になることがない）。`config.toml` の `schema_version` は消費者向け
ファイル形式の破壊時のみ更新します。[contracts/VERSIONING.md](contracts/VERSIONING.md)
を参照してください。

## コミット / プルリクエストの作法

- コミットメッセージは [Conventional Commits](https://www.conventionalcommits.org/)
  を使用（`feat:`・`fix:`・`docs:`・`refactor:`・`chore:` など）。
- 変更は焦点を絞り、`git add .` ではなくファイルを明示的にステージング。
- PR を出す前に `cargo test --manifest-path cli/Cargo.toml` が通ることを確認。
- PR タイトルは簡潔に。本文で「何を・なぜ」変えたかを説明（PR テンプレートが項目を促します）。
- シークレット（`.env`・認証情報）はコミットしない。

MochiFlow 自身の開発に MochiFlow を使う場合（dogfood）、`git push` と PR 作成は
`mochiflow pr`（事前チェック・push・バックエンド解決を一手に担う単一コマンド）経由で行います。
貢献にあたり必須ではありませんが、`main` への直接 `git push` は推奨されません。フィーチャー
ブランチを切って PR を出してください。

## Issue の報告

Issue テンプレート（バグ報告 / 機能要望）を使用してください。セキュリティ脆弱性は
**公開 Issue を立てず**、[SECURITY.md](SECURITY.md) に従ってください。

## ライセンス

コントリビュートすることで、あなたの貢献がプロジェクトのライセンスと同じく
[MIT](LICENSE-MIT) と [Apache-2.0](LICENSE-APACHE) のデュアルライセンスで提供されることに
同意したものとみなされます。

---

> 本ドキュメントは英語版（[CONTRIBUTING.md](CONTRIBUTING.md)）を一次ソースとするミラーです。
> 変更するときは両言語を同時に更新してください。
