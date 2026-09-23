# List

`fandhe-frontend-wireframe-ui` の箇条書きリストプレースホルダーです。項目を
縦に積んだ箇条書き（先頭マーカー + テキストの繰り返し）の配置イメージだけを
示す、非インタラクティブな部品です。API は `list(items, ordered)` の 2 引数
（詳細は下の引数表を参照）で、props 構造体は導入していません。

視覚的な参照元は特定の 1 部品ではなく、[blocks.pm](https://www.blocks.pm/)
に対応部品がない wireframe-ui 独自追加部品です（詳細は
`docs/design/wireframe-ui-architecture.md` §8）。

本部品は表示専用のため `<ul>`/`<ol>`/`<li>` は出力せず、リンク遷移・キーボード
操作も持ちません。実際に操作可能なリストが必要な場合は
[List（Themes）](../themes/list.md) を検討してください。

## 原案差分メモ

- **独自設計**: blocks.pm に対応部品がないため、
  `docs/design/wireframe-ui-architecture.md` §6 の汎用変換規約から独立設計
  しました。
- **`items: Vec<Node>`（`&[Node]` ではない）**: `crate::grid`/`crate::frame`/
  `crate::stack` と同じく、子ツリー全体の不要な `clone()` を避けるため所有権
  を受け取ります。
- **`Size` 引数を持たない**: `crate::stack` と同じ理由で、任意 `Node` を
  子として受け取る容器のルートに `fw-wire-size-*` を付けると
  `--fw-wire-font-size`/`--fw-wire-control-size` が子孫へ意図せず継承され、
  呼び出し側が `Vec<Node>` に何を渡すか関知しない容器の責務を越えて子部品の
  寸法まで暗黙に変えてしまうためです。マーカー寸法は継承フォントに合わせて
  `em` 基準にしています。
- **`ordered: bool` は部品固有の修飾 class**: `fw-wire-list-ordered` として
  表現し、`props.rs` へ共通型を追加していません（`crate::avatar` の
  `circle: bool` と同じ判断）。
- **`<ul>`/`<ol>`/`<li>` は出力しない**: 同文書 §7（非対話制約）と
  `crate::stepper`/`crate::breadcrumbs` の先例に従い、`div`/`span` だけで
  レイアウトを表します。
- **マーカーは CSS 擬似要素のみで描く**: DOM へマーカーノードを出力しません。
  箇条書きは `::before` の小円、番号付きは CSS カウンタ
  （`counter-reset`/`counter-increment`/`content: counter(...)`）で描きます
  （`crate::breadcrumbs` の区切り記号・`crate::stepper` の連結線と同じ先例）。
- **入れ子リストへ外側の番号・マーカーは漏れない**: 項目セレクタを子結合子
  `>` で書き、カウンタはルート単位でリセットしています。
- **行頭アイコンはスロット化しない**: 行頭にアイコンを置きたい場合は呼び出し
  側が項目 `Node` 自体（`rich_text` や `icon::*` + `text` の組み合わせ）で
  表現してください。マーカー差し替え用の `Option<Node>` スロットは持ちません
  （利用実績が出た時点で別イシューとして検討する対象です）。
- **表示状態を持たない**: `data-*` は出力しません（`crate::stack`/
  `crate::annotation` と同じ）。
- **件数の上限を設けない**: `crate::breadcrumbs` の先例と同じく、件数は
  呼び出し側が決めます。空の `Vec` でも panic せず項目 0 件のルートだけを
  出力します。
- **配色はグレースケール**: 黒塗り二値ではなく `--fw-wire-ink`（本文）・
  `--fw-wire-ink-muted`（マーカー・番号）のトークンを使います。
