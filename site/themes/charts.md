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
DOM・hit-area は #2128 系（#2129〜#2131）のスコープです。

イシュー #2133 で、凡例の item を `<button type="button" aria-pressed>`
（`trigger` slot）へ変更し、期間切替・凡例トグルの SSR 構造を追加しました。
各チャート Props の `range: Option<&str>`（または `Option<String>`）に
値を渡すと、root へ `data-range="<v>"` を出力します（期間文字列→カテゴリ
集合の写像は定義せず、アプリ側の責務です）。`hidden_series`/
`hidden_categories` に系列名/カテゴリ index を渡すと、該当する描画要素へ
`data-hidden` を、`LegendProps::hidden_series`/`hidden_categories` に渡すと
対応する凡例 trigger へ `aria-pressed="false"` を付与します。期間切替 UI
は既存の `toggle-group`（`fandhe_frontend_headless_ui::toggle_group`）を
再利用してください。例:

```rust
use fandhe_frontend_pre_styled_ui::charts::legend::{legend, LegendProps};
use fandhe_frontend_pre_styled_ui::line_chart::{line_chart, LineChartProps};

let mut chart_props = LineChartProps::new(&data, "monthly visits");
chart_props.range = Some("90d");
chart_props.hidden_series = &["signups"];
let chart = line_chart(&chart_props, vec![("id", "monthly-visits")]).unwrap();

let legend_props = LegendProps {
    hidden_series: vec!["signups".to_string()],
    controls: Some("monthly-visits".to_string()),
    ..Default::default()
};
let legend_node = legend(&data, &legend_props);
```

JS 無効時は凡例ボタンを押しても表示は変わりませんが、SSR 構造自体は単独で
成立します（progressive enhancement）。実際の click 配線（`aria-pressed`
の付け外し・対応する系列の表示トグル・スケール再計算）は wasm-full 側
（#2134）が担います。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md)
