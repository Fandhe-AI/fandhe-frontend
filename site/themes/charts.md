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
持ちません。マウス追従ツールチップ・icon 合成は #2128 系のスコープ外
です。

イシュー #2131 で、ツールチップの `tooltip-indicator`（色見本）に
`TooltipIndicator`（`Dot`〔既定〕/`Line`/`Dashed`/`None`）の 4 種を追加し、
`tooltip::layer_with`/`layer_from_entries_with` で選べるようにしました
（既定 `Dot` は既存 `layer`/`layer_from_entries` とバイト一致）。あわせて
hover 強調（active な点の拡張・非 active 点の減光）の CSS も追加しました。
祖先（`tooltip::frame`/8 個の chart 部品それぞれの `root`）が
`data-has-active` を持つときに `--fandhe-chart-inactive-opacity` を宣言し、
`data-index` を持つ各データ点がそれを継承して減光、`data-active` を追加で
持つ点のみフル不透明 + 拡大へ戻ります（`tooltip::frame_with` の
`FrameProps { has_active: true }` で明示できます）。**実際のマウス操作で
`data-has-active` を付け外しする wasm-full 側の配線は本イシューの時点では
未実装**であり（#2130 は `data-active`/`hidden` の付け外しのみ実装）、
現状はこの減光 CSS を静的な Demo 以外の経路で観測できません（後続イシュー
への引き継ぎ）。ツールチップ DOM・hit-area は #2129、マウス追従の JS 配線は
#2130 のスコープです。

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
