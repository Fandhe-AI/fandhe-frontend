# Icon

`fandhe-frontend-wireframe-ui` のアイコン単体プレースホルダーです。SVG ライン
アートアイコン基盤（`icon` モジュール、イシュー #2606）が提供するグリフを
1 つだけ、装飾用途として表示する非インタラクティブな部品です。API は
`icon(glyph, size)` の 2 引数（詳細は下の引数表を参照）で、props 構造体は
導入していません。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) の Icon 部品（Figma
プロパティ: Size, Icon(swap)）ですが、ライセンス上の理由によりスクリーン
ショットは掲載していません（詳細は `docs/design/wireframe-ui-architecture.md`
§2）。参照したい場合は外部リンク先を直接確認してください。

本部品は表示専用のプレースホルダーです。ラベル付き・対話可能なアイコンが
必要な場合は [Icon（Themes）](../themes/icon.md) を検討してください。

## 原案差分メモ

- **API は独自設計**: blocks.pm の Figma プロパティ（Size・Icon の instance
  swap）をそのまま Rust 引数へ変換したものではありません。
  `docs/design/wireframe-ui-architecture.md` §2・§7（blocks.pm の外観・
  anatomy・プロパティ構成を閲覧・転記して構造的に一致させない）に従い、
  API は §11 の SVG アイコン基盤の規約から独自設計しました。
- **instance swap は `Node` ではなく `fn(Size) -> Node`**: 通常の
  `Option<Node>` アイコンスロット規約（§11.4）は「本文の横にアイコンを
  置くホスト部品」向けであり、差し替え対象は任意の `Node` です。対して
  `icon` 部品にとっての差し替え対象は「どのグリフか」であるため、
  `icon::IconEntry` の要素型と同じ `fn(Size) -> Node`（関数ポインタ）を
  `glyph` 引数として受け取ります。`icon(glyph: Node, size)` 形式で
  `icon(icon::search(Size::Sm), Size::Lg)` のようにサイズを 2 か所で
  指定できてしまうと黙って `Sm` のまま描画される誤用を招くため、部品側が
  `glyph(size)` を 1 回だけ呼ぶ設計でサイズ指定を 1 か所に固定しました。
- **`role`/`aria-label` は付けない**: 全部品は非インタラクティブな表示専用
  プレースホルダーという不変条件（同文書 §1/§7）に従い、ルートは
  `role`/`aria-*`/`tabindex`/`style`/`data-*` を一切持たない `<span>` です。
  ラベル付きで使いたい場合は [Icon（Themes）](../themes/icon.md) を
  案内します。
- **アイコン一覧の表示元**: `icon::ALL`（SVG ラインアートアイコン基盤の
  全種レジストリ）の一覧表示は、`docs/design/wireframe-ui-architecture.md`
  §12 D8 の決定どおり本ページの Demo が担います（基盤自体の #2606 では
  一覧を表示しませんでした）。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、上記の外部リンクのみで代替します。
- **配色はグレースケール**: 黒塗り二値ではなく `--fw-wire-ink`（線色）の
  トークンを使います。
