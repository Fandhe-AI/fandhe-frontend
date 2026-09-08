# Charts（共通 API）

`fandhe-frontend-pre-styled-ui` の `charts` mod が提供するスタイル済み Charts（共通 API） 部品です。

系列ごとに `label`/`color`/`icon`（shadcn/ui `ChartConfig` 相当、イシュー #2077）を
`Series::with_label`/`with_color`/`with_icon` で個別に設定できます。凡例・
line/area/bar/radar の各消費者は `ChartData::series_color_var(index)` を
経由して同じ色を共有します。`--fandhe-color-chart-1`〜`-6`（6 段階）は
shadcn/ui の 5 段階を包含するため据え置いています。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md)
