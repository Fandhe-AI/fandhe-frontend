# Editable

クリック（または表示切替）でテキストを「表示専用」から「編集可能」へ切り替える inline 編集部品です。preview/edit の 2 モードを anatomy 全体で切り替え、確定・キャンセルの 2 操作をトリガーとして提供します。

`fandhe-frontend-headless-ui` の `editable` mod が提供する構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを持つ unstyled 部品です。Themes 版が備える見た目は持たず、anatomy・`data-*` のみを提供します（本部品はネイティブ要素の標準操作以外に固有の ARIA を持ちません）。CSS は利用者が当てます。

スタイル済みの表示例は [Editable](../themes/editable.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
