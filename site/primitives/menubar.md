# Menubar

複数の Menu を水平（または垂直）に並べたアプリケーションメニューバーです。`fandhe-frontend-headless-ui` の `menubar` mod が構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを提供する unstyled 部品であり、Themes 版と異なりスタイル（CSS）は一切持ちません。開いている Menu を跨いだ左右移動の状態遷移 API（`MenubarAction`）を持ちますが、実 DOM のキー配線は wasm ランタイム層の責務です。

スタイル済みの表示例は [Menubar](../themes/menubar.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
