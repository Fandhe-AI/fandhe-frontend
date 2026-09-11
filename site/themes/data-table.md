# Data Table

`fandhe-frontend-pre-styled-ui` の `data_table` mod が提供するスタイル済み
Data Table 部品です。shadcn/ui `Data Table` 相当の表操作 UI で、Root /
Toolbar / Column Header / Sort Trigger / Select All / Select Row /
Footer / Selection Count の 8 パーツ構成です。

見た目 variant（`ColorPalette` / `Size` 等の軸）は持ちません。
`data-loading` / `data-empty` / `data-sort` / `data-state` / `data-hidden`
は headless 層が出力する `data-*` を CSS セレクタとして参照するだけで、
class ベースの軸を持ちません。

表本体（`<table>` / `<thead>` / `<tbody>` / `<tr>`）は新規に作らず、既存の
`table` mod をそのまま使います。列ヘッダー・セル・行の表示状態
（ソート・列非表示・行選択）は `column_attrs` / `column_header_attrs` /
`row_attrs`（node を作らない属性ヘルパ）を `table::column_header` /
`table::cell` / `table::row` の attrs へ渡すことで合成します。本部品自身の
`column_header` / `sort_trigger`（`th` / `button` ノードを実際に生成する
パーツ）は自前で表を組み立てる利用者向けであり、`table` mod と組み合わせる
Themes 推奨経路の Demo には現れません。`select_all` / `select_row` は
`<table>` セル階層に直接載る `th` / `td` であるため、Themes 経路でも
そのまま使います。

`sort_trigger` は現在のソート方向に応じて `▲` / `▼` / `↕` のアイコンを
`::after` で表示します。選択行の背景は `table` の `row` パーツが持つ
`data-selected` 規則をそのまま利用し、本部品は独自の行背景規則を持ちません。

行の並べ替え・フィルタ・ページング処理そのもの、選択結果の保持・送信・
永続化、列定義・列順の永続化はこの部品では実装しません（アプリケーション
側が Rust コードで処理し、結果を表示状態として渡す設計です）。
`fandhe-frontend-wasm-full` 側の DOM 配線（sort-trigger click →
dispatch・列表示切替・ページング等）も別イシューで担当します。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
