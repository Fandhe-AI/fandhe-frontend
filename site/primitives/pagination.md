# Pagination

総件数からページ番号列（省略記号を含む）を決定的に導出するページ送りです。`fandhe-frontend-headless-ui` の `pagination` mod が構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを提供する unstyled 部品であり、Themes 版と異なりスタイル（CSS）は一切持ちません。クリックで dispatch する Button モードと、`href` 遷移の Link モードの両方に対応します。

スタイル済みの表示例は [Pagination](../themes/pagination.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
