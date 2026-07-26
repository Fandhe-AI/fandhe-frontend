# Breadcrumb

現在地までの階層（サイト内の位置）をリンク列として示すパンくずナビゲーションです。`fandhe-frontend-headless-ui` の `breadcrumb` mod が構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを提供する unstyled 部品であり、Themes 版と異なりスタイル（CSS）は一切持ちません。末尾項目のみ非対話の `current-link`（`aria-current="page"`）として描画し、中間項目は `ellipsis` パーツで折り畳めます。

スタイル済みの表示例は [Breadcrumb](../themes/breadcrumb.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
