# Visually Hidden

視覚的には隠すが支援技術（スクリーンリーダー）には読ませ続けるテキストコンテナです。`fandhe-frontend-headless-ui` の `visually_hidden` mod は Root（`span`）の 1 パーツのみで構成され、装飾要素の `aria-hidden="true"` 固定付与パターンとは逆に `aria-hidden` を意図的に付与しません。Themes 版と異なり clip 手法の CSS を持たず、構造の出力のみを担います。

スタイル済みの表示例は [Visually Hidden](../themes/visually-hidden.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
