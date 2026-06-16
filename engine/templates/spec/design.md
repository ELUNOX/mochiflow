# {タイトル} — 設計

> `design.md` は risk≥elevated / integration≠none / surfaces>1 の時に作る。
> 判断と契約のみを書く。具象 class/struct 定義は書かない（実装時に source を読む）。

## 設計判断

- 判断 / 理由

## アーキテクチャ

- 

## データモデル / インターフェース（signature レベル）

- 

## エラーハンドリング

- 

## テスト戦略

- 

## Workstreams

<!-- 複数 workstream / cross-surface のときのみ -->

| Workstream | Surface | 責務 | 依存 | 検証 |
| --- | --- | --- | --- | --- |
|  |  |  |  |  |

## Integration Contract

<!-- integration ≠ none のときのみ -->

- Contract owner / Request / Response / Error / Auth / Compatibility / Failure handling / Verification

## Review Results

<!-- risk≥elevated の mandatory reviewer run を記録。Reviewer mode: delegated | inline / Verdict: pass | pass-with-comments | fail -->

## 統合ログ

<!-- build 中に append-only。design からの seam 乖離・所有権境界・dead code 扱い・次セッション申し送り。
     plan で確定済みの内容や commit log で足りる情報は書かない。 -->
