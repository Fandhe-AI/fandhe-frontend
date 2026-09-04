# Image Cropper

矩形の切り抜き範囲（crop rect）を選択させる部品です。表示画像上に選択枠と方位別ハンドルを重ね、選択範囲の x/y/width/height を値として保持します。canvas による実際のピクセル切り出し（画像処理）は本部品のスコープ外で、利用者側の責務です。

`fandhe-frontend-headless-ui` の `image_cropper` mod が提供する構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを持つ unstyled 部品です。Themes 版が備える見た目は持たず、anatomy・ARIA・`data-*` のみを提供します。CSS は利用者が当てます。

参照実装（ark-ui/zag.js `image-cropper` machine）との突合により、キーボード操作の受け口は handle ではなく selection（focusable な `role="slider"`、`aria-roledescription="2d slider"`）へ集約されています。handle は `role="presentation"` + `aria-hidden="true"`（非 focusable）で、`data-position`（旧 `data-handle-position` から改名）のみを出力します。`ImageCropperProps { disabled, dragging }` に応じて root/viewport/selection/handle へ `data-disabled`、root/selection/grid へ `data-dragging` を出力します。`grid` に `GridAxis::Horizontal`/`Vertical` を渡すと `data-axis` を出力します（省略時は単一コンテナ）。`action_for_key(key, modifiers)` はキーボード操作（Arrow = 移動、Alt+Arrow = SE ハンドル基準のリサイズ、Shift/Ctrl(Cmd) で step 拡大）の対応表を純粋関数として提供しますが、実際の DOM keydown 配線は `fandhe-frontend-wasm-full` 側の後続責務です。

スタイル済みの表示例は [Image Cropper](../themes/image-cropper.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)

**自前 CSS の最小例**

Themes 版を使わず本部品を直接使う場合、`[data-scope="image-cropper"][data-part="..."]` セレクタでスタイルを当てます。以下は選択枠・ハンドル・selection のフォーカスリングの最小例です。headless 層は選択範囲の位置・寸法を CSS カスタムプロパティとして自動出力しないため、呼び出し側が `ImageCropper::x_percent()` 等から `selection` の `attrs` へ `("style", "--crop-x: 20%; ...")` のように現在値を渡す必要があります（`fandhe-frontend-pre-styled-ui` の `selection` はこれを内部で一元的に組み立てます）。

```css
[data-scope="image-cropper"][data-part="selection"] {
  position: absolute;
  /* --crop-x/-y/-w/-h は呼び出し側が selection の attrs 経由で style
     属性に設定する（例: ImageCropper::x_percent() 等から組み立てた
     "--crop-x: 20%; --crop-y: 10%; --crop-w: 60%; --crop-h: 50%"）。 */
  left: var(--crop-x, 0%);
  top: var(--crop-y, 0%);
  width: var(--crop-w, 100%);
  height: var(--crop-h, 100%);
  border: 1px solid #fff;
  box-shadow: 0 0 0 9999px rgba(0, 0, 0, 0.5);
}

[data-scope="image-cropper"][data-part="selection"]:focus-visible {
  outline: 2px solid #4a90d9;
  outline-offset: 2px;
}

[data-scope="image-cropper"][data-part="handle"] {
  position: absolute;
  width: 0.75rem;
  height: 0.75rem;
  background: #fff;
  border: 1px solid rgba(0, 0, 0, 0.25);
}

[data-scope="image-cropper"][data-part="handle"][data-position="se"] {
  top: 100%;
  left: 100%;
  cursor: nwse-resize;
}

[data-scope="image-cropper"][data-part="root"][data-disabled] {
  opacity: 0.5;
}

[data-scope="image-cropper"][data-part="grid"][data-axis="horizontal"] {
  /* 横方向のグリッド線のみを描く場合に使う。 */
}
```
