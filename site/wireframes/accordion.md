# Accordion

`fandhe-frontend-wireframe-ui` のアコーディオンのプレースホルダーです。見出し +
開閉キャレット + 本文からなる項目を縦に並べます。API は `accordion(items, size)`
の 2 引数で、`items` は「見出しテキスト・本文スロット・展開済みか」の組の列
（`Vec<(&str, Node, bool)>`）です（詳細は下の引数表を参照）。

accordion は blocks.pm カタログに対応部品を持たない wireframe-ui 独自追加部品です
（詳細は `docs/design/wireframe-ui-architecture.md` §7）。

本部品は表示専用のため、実際に開閉するアコーディオンとしては出力しません。実際に
操作できるアコーディオンが必要な場合は [Accordion（Themes）](../themes/accordion.md)・
[Accordion（Primitives）](../primitives/accordion.md) を検討してください。

## 原案差分メモ

- **独自追加部品**: blocks.pm カタログに対応部品を持たないため、外観・プロパティ
  構成は `docs/design/wireframe-ui-architecture.md` §6 の汎用変換規約から独立に
  設計しました。
- **項目は `Vec` の所有で受ける**: 借用スライス（`&[(&str, Node, bool)]`）ではなく
  `Vec<(&str, Node, bool)>` の所有で受けます。`Node` を子として所有で受ける
  既存部品（`stack`/`grid`/`frame`/`question`）の先例に揃えた設計で、借用スライス
  だと項目ごとに `Node::clone()` が必要になり `coding-rust.md`「不要な `clone()`
  を避ける」に反するためです。
- **展開状態は `props::Active` を再利用**: 新しい専用型（`Expanded` 等）は追加せず、
  既存の `props::Active` を項目単位で再利用し `data-active` を付与します
  （switch/radio/tabs の先例と同じ判断）。`aria-expanded` は非対話制約
  （§7）により出力しません。
- **折りたたみ項目の本文は出力しない**: `expanded == false` の項目は渡された
  本文 `Node` を出力せず捨てます。`hidden` 属性や CSS で隠す方式は使わず、
  静的 SSR プレースホルダーとして隠しコンテンツを持たない決定的な出力にして
  います。
- **`<details>`/`<summary>` は使わない**: ブラウザ標準で開閉できる対話要素の
  ため、ルート・項目・見出し行はすべて `div` で組みます（§7 の非対話制約）。
  `<button>`/`<a>`/`<input>`/`role`/`aria-*`（アイコン基盤の装飾用
  `aria-hidden` を除く）/`tabindex`/`style`/`on*` も一切出力しません。
- **キャレットの向き**: 見出し行の末尾に固定パートとしてキャレットを置き、
  展開時は `icon::caret_up`、折りたたみ時は `icon::caret_down` を使います
  （select の末尾ドロップダウン指示子と同じ扱い）。
- **責務外の軸**: `Bold`/`Primary`/`Disabled`/`Orientation`・先頭アイコン
  スロットは持ちません。
