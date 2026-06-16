# PR Description Template

PR description は外部 reviewer 向け。spec ファイル / 内部 ID（slug、AC-01 など）/
mochiflow 用語（risk: standard など）を含めない。日本語で書く。

```markdown
## 概要

{この変更の目的を 1〜3 行。なぜ必要か}

## 変更

- {変更の論理的単位で箇条書き。何が・どう変わったか}

## テスト

- {実行した検証 command と結果}
- {人間確認した場合の概要を 1 行}

## リスク

- {自然な日本語で。可逆性、schema/contract 影響、影響範囲を 1〜2 行}
```
