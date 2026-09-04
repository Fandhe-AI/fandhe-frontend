# Angle Slider

方位・回転角度のように「0〜359 度の環状値」を 1 個選ばせる入力です。値は常に `0..=359` の整数へ正規化され、`359` 度からの増分・`0` 度からの減分も符号付き剰余を経由せずラップアラウンドします。

`fandhe-frontend-headless-ui` の `angle_slider` mod が提供する構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを持つ unstyled 部品です。Themes 版が備える見た目（回転する thumb の CSS 表現等）は持たず、anatomy・ARIA・`data-*` のみを提供します。CSS は利用者が当てます。

Root / Label / Control / Thumb / MarkerGroup / Marker / HiddenInput / ValueText の 8 anatomy パーツで構成されます。Root/Label/Control/Thumb は `disabled`/`readonly`/`invalid` の 3 状態を共有し、それぞれ `data-disabled`/`data-readonly`/`data-invalid` を出力します。Marker は目盛り角度と現在値の大小関係から `data-state`（`under-value`/`over-value`/`at-value`）を出力します。

スタイル済みの表示例は [Angle Slider](../themes/angle-slider.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="angle-slider"][data-part="..."]` セレクタでスタイルを当てます。以下は circular な control とサムの回転、状態セレクタの最小例です。headless 層は回転角を CSS カスタムプロパティとして自動出力しないため、呼び出し側が `thumb` の `attrs` へ `("style", "--fandhe-angle: 135deg")` のように現在角度を渡す必要があります（`fandhe-frontend-pre-styled-ui` の `thumb_styled` はこれを内部で一元的に組み立てます）。

```css
[data-scope="angle-slider"][data-part="control"] {
  position: relative;
  width: 4rem;
  height: 4rem;
  border-radius: 50%;
  background: #eee;
}

[data-scope="angle-slider"][data-part="thumb"] {
  position: absolute;
  top: 0;
  left: 50%;
  width: 0.75rem;
  height: 0.75rem;
  border-radius: 50%;
  background: #333;
  /* --fandhe-angle は呼び出し側が thumb の attrs 経由で style 属性に
     設定する（例: ("style", "--fandhe-angle: 135deg")）。未設定時は
     0deg にフォールバックする。 */
  transform: rotate(var(--fandhe-angle, 0deg));
  transform-origin: 0 2rem;
}

[data-scope="angle-slider"][data-part="root"][data-readonly] {
  opacity: 0.7;
}

[data-scope="angle-slider"][data-part="root"][data-invalid] {
  outline: 2px solid crimson;
}

[data-scope="angle-slider"][data-part="marker"][data-state="at-value"] {
  background: #333;
}
```
