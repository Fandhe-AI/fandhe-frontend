# Table

`fandhe-frontend-pre-styled-ui` の `table` mod が提供するスタイル済み Table 部品です。

表形式データを表示する部品です。TableVariant（Line/Outline）で外枠・区切り線を、size（Xs〜Xl）でセルの padding/font-size を、striped（bool）で縞模様表示を、sticky_header（bool）で column-header（th）の position: sticky 表示を、interactive（bool、イシュー #2052）で row の hover 背景表示を切り替えます（5 軸 variant）。scroll_area（chakra Table.ScrollArea 相当）で root をスクロール枠に包み、sticky_header と組み合わせて見出し行を固定できます。column_header は scope="col" を関数側で固定し呼び出し側の偽装を除去します。呼び出し側は data-selected（行選択、accent-subtle 配色）・data-align（start/center/end、セル整列）を row/cell/column-header へ付与でき、column_header/cell は aria-sort 等の属性をそのまま通過させます（ソート・選択の生産は headless data-table 側の責務です）。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md)
