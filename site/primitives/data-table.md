# Data Table

表の操作 UI（toolbar・ソート可能な列ヘッダー・行選択・列表示切替・footer）の shadcn/ui Data Table 相当の部品です（参照軸はイシュー #2001。ark-ui には対応する component がないため headless は ark-ui 系統を維持します）。`fandhe-frontend-headless-ui` の `data_table` mod が構造・アクセシビリティ（WAI-ARIA）・表示状態（`data-*`）のみを提供する unstyled 部品で、Root / Toolbar / ColumnHeader / SortTrigger / SelectAll / SelectRow / Footer / SelectionCount の 8 anatomy パーツと、ソート方向・非表示列の**表示状態のみ**を持つ最小の状態機械 `DataTable` を持ちます。

**責務境界**

行の実際の並べ替え（比較関数・安定ソート・多列優先順位）・選択結果の保持・送信・永続化・列定義や列順の永続化・ページサイズに応じたデータ取得や総件数算出はアプリケーション側の責務であり、本部品は持ちません。部品が担うのは「現在のソート列/方向・非表示列集合に応じて表示状態（`aria-sort`/`data-*`）を切り替え、トリガーの dispatch 通知を出す」までです。行選択集合そのものも状態機械に持たせず、`select_all` / `select_row` は呼び出し側から選択状態（`checkbox::CheckedState`）を受け取るだけの表示パーツです。

**表組み要素を生成しない理由**

`table` / `empty-state` / `skeleton` は `fandhe-frontend-pre-styled-ui` 側にのみ anatomy があり、headless-ui は上層へ依存できません。そのため `data_table` は `<table>` / `<thead>` / `<tbody>` / `<tr>` を一切生成しません。セル単位の表示状態（ソート・列非表示・行選択）は `column_attrs` / `column_header_attrs` / `row_attrs` という node を作らない属性ヘルパとして公開しており、pre-styled `table::column_header` / `table::cell` / `table::row` の `attrs` へそのまま渡せます（Themes 層は後続イシュー #2127）。併せて headless 単独経路（Primitives Demo・自前 CSS 利用者）向けに `column_header`（`th`）/ `select_all`（`th`）/ `select_row`（`td`）というノード生成パーツも持ちます。

**状態モデル**

`DataTable` は `sort: Option<(String, SortDirection)>`（ソート中の列 id と方向、高々 1 列）と `hidden_columns: Vec<String>`（重複なしの非表示列 id 集合）を持ちます。行選択集合は持ちません。

ソート方向の巡回規則（`"sort"` dispatch）は、同じ列 id を指定した場合は `none → ascending → descending → none` を巡回し、別の列 id を指定した場合は常に `ascending` から開始します（別列へ切り替えた瞬間に旧列のソートは失われます）。

| `aria-sort` / `data-sort` | 意味 |
|---|---|
| `none` | ソート可能だが未ソート |
| `ascending` | 昇順ソート中 |
| `descending` | 降順ソート中 |
| `other` | カスタムの並べ替え（状態機械は自動生成せず、SSR で呼び出し側が明示指定する場合のみ） |

**トリガーと Root の表示状態**

`sort_trigger(id, attrs, children)` は `type="button"` の `button` で、`data-value` に列 id（`fandhe-frontend-wasm-full` の `MAPPING_TABLE` が読む契約）、`data-sort` に現在の方向を出力します。`root(props, attrs, children)` は `DataTableProps { loading, empty }` を受け取り、`loading` のとき `data-loading` + `aria-busy="true"`、`empty` のとき `data-empty` を出力します。

**Toolbar / Footer スロット**

`toolbar` / `footer` は純スロットです。`toolbar` へは Field の `input`（フィルタ）・`column_toggle_item`（列表示切替、Menu の checkbox item の薄いラッパ）を、`footer` へは Pagination のパーツと `selection_count`（整形済みの選択件数文字列スロット）を入れ子にする契約とします。各 scope は `data-table` scope と独立して残ります。

**アクセシビリティ**

- `column_header` / `select_all` は `th` 要素で `scope="col"` を固定出力します。
- `role="toolbar"` は意図的に不採用です（矢印キー roving focus の配線義務が生じるため。shadcn/ui も素の flex div を使います）。
- キーボード操作は Tab / Shift+Tab（フォーカス移動）・Enter / Space（`sort_trigger` の押下）のみで、矢印キー等の独自ハンドリングは持ちません。`toolbar` / `footer` に入れ子にした Field / Menu / Checkbox / Pagination のキー操作はそれぞれの部品のものを継承します。

`sort-trigger` の click から dispatch（`"sort"`）への実際の DOM 配線・列表示切替・行選択の全選択 indeterminate 表示・ページングの `data-disabled` は `fandhe-frontend-wasm-full` 側の後続イシュー（#2126）のスコープです。

**参考サイトとの差分**

shadcn/ui の `Data Table`（`@tanstack/react-table` ベース）は `ColumnDef` / `getCoreRowModel` / `getSortedRowModel` / `getFilteredRowModel` / `getPaginationRowModel` 等のテーブルエンジンと行選択の状態管理（`rowSelection` state）を内包しますが、本部品はこれらを一切持ちません。並べ替え・フィルタ・ページングの実処理はアプリケーションが Rust コードで書いて結果（表示済みの行・現在のソート状態）を渡す契約です。列定義配列からの一括描画も非採用で、パーツを個別に呼ぶ組み立て方は他の headless モジュールと同型です。

列非表示の実現手段として `col` / `colgroup` 要素（`visibility: collapse`）はブラウザ差があるため不採用とし、セル単位の `hidden` 属性で代替します。

`fandhe-frontend-pre-styled-ui` に対応するスタイル済み部品があります。Themes 版は [Data Table](../themes/data-table.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

```css
[data-scope="data-table"][data-part="column-header"][data-sort="ascending"]::after {
  content: " ▲";
}
[data-scope="data-table"][data-part="column-header"][data-sort="descending"]::after {
  content: " ▼";
}
[data-scope="data-table"][data-part="column-header"][hidden] {
  display: none;
}
[data-scope="data-table"][data-part="root"][data-loading] {
  opacity: 0.6;
  pointer-events: none;
}
[data-scope="data-table"][data-part="select-row"][data-state="checked"] {
  background: #eff6ff;
}
```
