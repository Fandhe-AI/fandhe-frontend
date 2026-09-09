# Charts（共通 API）

`fandhe-frontend-pre-styled-ui` の `charts` mod が提供するスタイル済み Charts（共通 API） 部品です。

系列ごとに `label`/`color`/`icon`（shadcn/ui `ChartConfig` 相当、イシュー #2077）を
`Series::with_label`/`with_color`/`with_icon` で個別に設定できます。凡例・
line/area/bar/radar の各消費者は `ChartData::series_color_var(index)` を
経由して同じ色を共有します。`--fandhe-color-chart-1`〜`-6`（6 段階）は
shadcn/ui の 5 段階を包含するため据え置いています。

イシュー #2086 で shadcn/ui Charts（tooltip、registry 9 バリアント）と突合し、
凡例（`legend::legend`）へ `hide_marker`（`hideIcon` 相当）・`align`（水平
揃え、既定は左寄せのまま `Center`/`End` を opt-in）・`marker`（マーカー
形状、既定は円形のまま角丸四角 `Square` を opt-in）を、ツールチップへ
複数行本文 `tooltip::datum_label_lines`（見出し省略可・整形済み値・footer）
を、それぞれ静的バリアントとして純追加しました。凡例をチャート上部に置く
配置はノード合成順序（`legend()` を先に並べる）で表現し、専用の CSS 軸は
持ちません。マウス追従ツールチップ・indicator・icon 合成・ツールチップ
DOM・hit-area は #2128 系（#2129〜#2131）、凡例の系列トグルは #2132 の
スコープです。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md)
