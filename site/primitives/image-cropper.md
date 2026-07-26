# Image Cropper

矩形の切り抜き範囲（crop rect）を選択させる部品です。表示画像上に選択枠と方位別ハンドルを重ね、選択範囲の x/y/width/height を値として保持します。canvas による実際のピクセル切り出し（画像処理）は本部品のスコープ外で、利用者側の責務です。

`fandhe-frontend-headless-ui` の `image_cropper` mod が提供する構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを持つ unstyled 部品です。Themes 版が備える見た目は持たず、anatomy・ARIA・`data-*` のみを提供します。CSS は利用者が当てます。

スタイル済みの表示例は [Image Cropper](../themes/image-cropper.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
