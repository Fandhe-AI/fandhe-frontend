# Action Bar

複数選択（チェックボックス等）に対する一括操作を提示する画面下部固定の操作バーです。`fandhe-frontend-headless-ui` の `action_bar` mod が構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを提供する unstyled 部品であり、Themes 版と異なりスタイル（CSS）は一切持ちません。開閉状態は Disclosure を埋め込んだ状態機械が管理しますが、「選択操作から開閉状態を決定する」判断自体は呼び出し側アプリケーションの責務です。

スタイル済みの表示例は [Action Bar](../themes/action-bar.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
