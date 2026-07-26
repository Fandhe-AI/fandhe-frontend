# Color Picker

HSV 色相環 + アルファ選択のポップオーバー型入力です。色領域・チャンネルスライダーの見た目は CSS グラデーションと thumb 位置のみで表現し、`canvas`/`web-sys` には依存しません（値の算出は決定的な純粋関数が担います）。

`fandhe-frontend-headless-ui` の `color_picker` mod が提供する構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを持つ unstyled 部品です。Themes 版が備える見た目（実際のグラデーション CSS 等）は持たず、anatomy・ARIA・`data-*` のみを提供します。CSS は利用者が当てます。

スタイル済みの表示例は [Color Picker](../themes/color-picker.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
