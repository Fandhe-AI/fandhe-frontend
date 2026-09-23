# Input

`fandhe-frontend-wireframe-ui` のテキストフィールド風プレースホルダーです。
先頭アイコン + プレースホルダー風テキストで「テキストフィールドらしさ」だけを
表現する非インタラクティブな部品です。API は `input(text, leading, size,
active, disabled)` の 5 引数（詳細は下の引数表を参照）で、props 構造体は
導入していません。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) の Input 部品ですが、
ライセンス上の理由によりスクリーンショットは掲載していません（詳細は
`docs/design/wireframe-ui-architecture.md` §2）。参照したい場合は外部リンク先を
直接確認してください。

本部品は表示専用のため実際に入力操作できるフィールドは出力しません。実際に
入力可能なフィールドが必要な場合は [Input（Themes）](../themes/input.md) を
検討してください。

## 原案差分メモ

- **API は独自設計**: `docs/design/wireframe-ui-architecture.md` §6 の
  汎用変換規約から独立設計しました。同文書 §2 の保留（イシュー #2602）に
  従い、blocks.pm の Figma プロパティ構成を参照・書き写さず、テキスト
  フィールドで広く使われる一般的な UI パターンおよび本クレート内の
  既存共通規約（`leading: Option<Node>` アイコンスロット・`Active`/
  `Disabled` 共通型）から独立に設計しました。
- **アイコンスロットで表現**: 先頭アイコンは `leading: Option<Node>` で
  受け取ります（同文書 §11.4 の `Option<Node>` アイコンスロット規約の実例）。
  専用の `Icon` bool 型や第 2 の引数は追加していません。
- **状態は共通型で表現**: `Active`/`Disabled` の `data-*`（同文書 §5「表示状態
  まで」）で表現します。専用の `InputState` 列挙は追加していません。`Active`
  の `data-*` 出力を実際に行う最初の部品です（型自体はイシュー #2605 で
  定義済みでしたが、これまで実消費者がありませんでした）。`Disabled` は
  本モジュールより先に main へマージされた `button`（イシュー #2621）が
  最初の実消費者であり、本部品は 2 例目の実消費者です。
- **テキストはプレースホルダー風の 1 種類のみ**: 実際の入力値とプレースホルダー
  文言の区別はモデル化していません。`Bold`/`Primary` は使いません（テキスト
  フィールドに強調軸を持たせる根拠がないため）。
- **`<input>` は出力しない**: ルートは `div` とし、`placeholder`/`value`/
  `role`/`aria-*`/`tabindex` は一切出力しません（同文書 §7 の非対話制約）。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、上記の外部リンクのみで代替します。
- **配色はグレースケール**: `--fw-wire-ink-muted`（既定テキスト色）を基調とし、
  Active 時は `--fw-wire-ink` の枠線 + box-shadow、Disabled 時は破線 +
  `--fw-wire-fill-subtle` の背景で表現します。
