# Skip Nav

キーボード操作時のみ視覚的に現れる「本文へスキップ」リンクです（WCAG 2.1 SC 2.4.1 Bypass Blocks）。`fandhe-frontend-headless-ui` の `skip_nav` mod は link / content の 2 anatomy パーツを提供し、`href` に任意スキームを受け付けず常に `#<id>` フラグメントのみを組み立てます。Themes 版と異なり focus 時のみ表示する CSS を持たず、構造とフォーカス移動属性のみを担います。

スタイル済みの表示例は [Skip Nav](../themes/skip-nav.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
