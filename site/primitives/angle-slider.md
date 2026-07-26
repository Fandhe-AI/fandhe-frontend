# Angle Slider

方位・回転角度のように「0〜359 度の環状値」を 1 個選ばせる入力です。値は常に `0..=359` の整数へ正規化され、`359` 度からの増分・`0` 度からの減分も符号付き剰余を経由せずラップアラウンドします。

`fandhe-frontend-headless-ui` の `angle_slider` mod が提供する構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを持つ unstyled 部品です。Themes 版が備える見た目（回転する thumb の CSS 表現等）は持たず、anatomy・ARIA・`data-*` のみを提供します。CSS は利用者が当てます。

スタイル済みの表示例は [Angle Slider](../themes/angle-slider.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
