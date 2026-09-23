# Stepper

`fandhe-frontend-wireframe-ui` の複数ステップ進捗表示プレースホルダーです。
ステップ名のスライスと現在ステップの index から、番号付きステップの
横並び進捗表示だけを示す非インタラクティブな部品です。API は
`stepper(steps, active, size)` の 3 引数（詳細は下の引数表を参照）で、
props 構造体は導入していません。

Stepper は blocks.pm に対応する部品を持たない、wireframe-ui 独自追加の
部品です（詳細は `docs/design/wireframe-ui-architecture.md` §8「独自追加
部品」を参照）。近い視覚的な参照が欲しい場合は
[blocks.pm](https://www.blocks.pm/) を直接確認してください。

各ステップは `div`/`span` のみで出力し、`<ol>`/`<li>`/`<button>`/
`<a href>` のいずれも実装しない表示専用プレースホルダーです。実際に
操作可能なステップ表示が必要な場合は
[Steps（Themes）](../themes/steps.md) /
[Steps（Primitives）](../primitives/steps.md) を検討してください。

## 原案差分メモ

- **独自追加部品**: blocks.pm に対応部品が存在しないため、
  `docs/design/wireframe-ui-architecture.md` §6 の汎用変換規約から独立に
  設計しました。API は「ステップ名のスライス + 現在 index + `Size`」の
  最小構成のみで、blocks.pm の Figma プロパティを参照・書き写していません。
- **状態写像は 3 分岐**: `i < active` は完了（`data-complete`）、
  `i == active` は現在（既存共通型 `props::Active` を再利用した
  `data-active`）、`i > active` は未着手（属性なし）です。完了状態に
  専用の型を新設せず、`fandhe_frontend_core::attr_if` を部品ローカルで
  直接呼んでいます（headless-ui の `steps`/`questionnaire` が使う
  「complete」語彙に合わせた命名）。
- **`active` の範囲外はエラーにしない**: `active >= steps.len()`
  （空スライスを含む）は「全ステップ完了・現在ステップなし」として
  扱います。`clamp` も `panic` もしません（呼び出し側が全ステップ完了を
  表現したいケースをそのまま許容する設計）。
- **番号は常に表示（チェックアイコンへの差し替えは不採用）**: 完了
  ステップも `checkbox` のようにグリフへ差し替えず、常に `i + 1` の番号を
  表示します。アイコン差し替えは装飾用の `aria-hidden` 付与を要し、非対話
  テストの単純さを崩すため見送りました。
- **横並びのみ**: `props::Orientation` は受け付けません。縦並び表示は
  対象外としています（必要であれば別イシューとして起票を検討）。
- **連結線はノードを持たない**: ステップ間の連結線は CSS の `::before`
  疑似要素のみで描き、余分なノードを出しません。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、上記の外部リンクのみで代替します。
- **配色はグレースケール**: `--fw-wire-ink`/`--fw-wire-ink-muted`/
  `--fw-wire-fill` トークンのみを使い、`ColorPalette` には依存しません。
