# Modal

`fandhe-frontend-wireframe-ui` のモーダルダイアログ風プレースホルダーです。
「中央に置かれたダイアログ枠（タイトル・本文・アクション行）」の配置
イメージだけを示す非インタラクティブな部品です。API は
`modal(title, body, actions, size)` の 4 引数（詳細は下の引数表を参照）で、
props 構造体は導入していません。`body`/`actions` は既存の wireframe-ui
部品（`paragraph`/`text`/`button` 等）の戻り値をそのまま渡す `Node` スロット
です。

blocks.pm には対応部品がなく、`fandhe-frontend-wireframe-ui` 独自追加の
部品です（詳細は
[`docs/design/wireframe-ui-architecture.md`](https://github.com/Fandhe-AI/fandhe-frontend/blob/main/docs/design/wireframe-ui-architecture.md)
§8）。参考として [blocks.pm](https://www.blocks.pm/) のリンクのみ掲載し、
画像は掲載していません。

開閉状態・フォーカストラップ・Esc 操作などの対話は一切実装しない表示専用
プレースホルダーです。実際に操作可能なダイアログが必要な場合は
[Dialog（Themes）](../themes/dialog.md) /
[Dialog（Primitives）](../primitives/dialog.md) を検討してください。

## 原案差分メモ

- **API は独自設計**: `docs/design/wireframe-ui-architecture.md` §8 の
  「blocks.pm 対応部品なし・独自追加」に従い、引数構成は既存部品（`frame`・
  `question`）と同じ「`Node` スロット + `Size`」の汎用変換規約から独立に
  設計しました。
- **`actions` は `Vec<Node>`（借用ではなく所有渡し）**: core のノード木 API・
  他の wireframe-ui 部品と同じく `&[Node]` ではなく `Vec<Node>` を採用して
  います。`&[Node]` だと呼び出し側で `.to_vec()` の不要な `clone()` が
  必要になり `.claude/rules/coding-rust.md`「不要な clone() を避け、借用を
  優先」に反するためです。
- **中央配置は `position: fixed` にしない**: docs ページ全体を覆う
  `position: fixed`/`absolute`/`z-index` は他のデモ枠のレイアウトを破壊
  するため使いません。ルート `.fw-wire-modal` は in-flow のブロック
  （`display: grid; place-items: center;`）とし、その内側のパネルを
  中央寄せしています。
- **`<dialog>`・`role`・`aria-modal` 等は出力しない**: 非対話制約
  （`docs/design/wireframe-ui-architecture.md` §7）により、`<dialog>`/
  `role`（`dialog` 含む）/`aria-*`（`aria-modal` 含む）/`tabindex`/`style`/
  `<button>`/`<form>`/`<input>`/`<a href>` は一切出力しません。
- **タイトルは見出し要素にしない**: `h1`〜`h6` ではなく `div` で出力し、
  docs ページの目次・検索インデックスへ見出しとして混入しないようにして
  います。見た目は太字（`font-weight: 600`）で表現します。
- **閉じるボタン・開閉状態の表示軸は持たない**: `Active`/`Disabled` 等の
  表示状態軸を追加せず、パネルの中身は常に「開いた状態」の 1 枚絵として
  表示します。閉じる「×」アイコンスロットの追加是非は本イシューのスコープ
  外とし、必要が見えた時点で別イシューとして検討します。
- **配色はグレースケール**: `--fw-wire-ink`/`--fw-wire-ink-muted`/
  `--fw-wire-paper`/`--fw-wire-fill-subtle`/`--fw-wire-line` トークンのみを
  使い、`ColorPalette` には依存しません。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、上記の外部リンクのみで代替します。
- **`size` はパネル最大幅にのみ効き、共有 `fw-wire-size-*` は使わない**:
  レビュー指摘（codex-review・Cursor Bugbot、イシュー #2645）を受けて
  是正しました。当初はルートへ共有 `fw-wire-size-*` class を付与しており、
  `--fw-wire-font-size` がパネル・タイトルや `body`/`actions` スロットへ
  暗黙に継承され、「パネル最大幅にのみ効く」という公開契約と実装が
  食い違っていました。`crate::frame`（`fw-wire-frame-padding-*`、
  イシュー #2609）・`crate::stack`（`fw-wire-stack-gap-*`）が同種の問題を
  先に回避した前例に合わせ、Modal 専用の修飾 class
  `fw-wire-modal-max-width-<段階>` を新設して `.fw-wire-modal-panel` から
  `font-size: var(--fw-wire-font-size, ...)` を削除しました。
