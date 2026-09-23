# Card basic

`fandhe-frontend-wireframe-ui` のカード型データ表示プレースホルダーです。先頭の
視覚要素（アバター等）スロット・主テキストと補足テキストの 2 段・末尾の補助
アイコンスロットを 1 枚の枠線カードにまとめる、非インタラクティブな部品です。
API は `card_basic(primary, secondary, leading, trailing, size)` の 5 引数
（詳細は下の引数表を参照）で、props 構造体は導入していません。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) ですが、ライセンス上の
理由によりスクリーンショットは掲載していません（詳細は
`docs/design/wireframe-ui-architecture.md` §2）。参照したい場合は外部リンク先を
直接確認してください。

本部品は表示専用のプレースホルダーです。実際に操作可能なカード部品が
必要な場合は [Card（Themes）](../themes/card.md) を検討してください。

## 原案差分メモ

- **API は独自設計**: Issue 本文は blocks.pm の Figma プロパティ（右アイコンの
  bool・asset の swap・子要素のネスト props）を Rust 引数へ変換する方針を
  記していましたが、`docs/design/wireframe-ui-architecture.md` §2・§7（blocks.pm
  の外観・anatomy・プロパティ構成を閲覧・転記して構造的に一致させない、
  PR #2670 の codex P1 指摘を受けた追記）を優先し、API は §6・§11.4 の汎用
  規約から独自設計しました。
- **右アイコンは `Option<Node>` スロットへ統一**: 右アイコンの bool・asset
  swap は `avatar`/`nav_item`/`tag`/`link` と同じ `Option<Node>` アイコン
  スロット規約（§11.4）へ統一しました。`None` のときはスロット要素自体を
  出力しません。
- **`secondary` は `Option<&str>`**: `nav_item` の `counter` と同じ扱いで、
  `None` のときは要素自体を出力しません。
- **`avatar` は内蔵しない**: `card_basic` の中から `crate::avatar` を直接
  呼ぶことはしません。部品同士の合成は呼び出し側の選択とし、デモでは
  `Some(avatar(None, size, true))` を `leading` へ渡しています。
- **末尾指示子は既存の `icon::ellipsis` を使う**: ケバブメニュー用の新しい
  アイコン（縦 3 点等）は追加していません。デモでは `icon::ellipsis` を
  `trailing` へ渡しています。
- **`menu: bool` 案は不採用**: `toast` の `dismissible: bool`（`icon::x`
  固定）と同じ発想で「末尾アイコンを固定するフラグ 1 個」案も検討しましたが、
  §11.4 のスロット形式（`avatar`/`nav_item`/`tag`/`link` と同じ規約）が
  本クレートの標準であるため採りませんでした。
- **表示状態の軸を持たない**: `progress`/`spinner` と同じく表示専用の
  data display 部品であり、`Active`/`Disabled` は持ちません。
- **配色はグレースケール**: 黒塗り二値ではなく `--fw-wire-paper`（背景）・
  `--fw-wire-ink`（主テキスト）・`--fw-wire-ink-muted`（補足テキスト）・
  `--fw-wire-line`（枠線）のトークンを使います。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、上記の外部リンクのみで代替します。
