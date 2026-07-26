# Menubar

`fandhe-frontend-pre-styled-ui` の `menubar` mod が提供するスタイル済み Menubar 部品です。

複数の [Menu](menu.md) を水平（または垂直）に並べるコンテナです。`role="menubar"` を持つルートの配下に、`role="none"` の Menu ラッパーと `role="menuitem"` のトリガーが並び、roving tabindex（フォーカスが常に 1 トリガーのみに当たる状態）の状態機械を提供します。Menubar 特有の挙動として、ある Menu が開いた状態でフォーカスを隣のトリガーへ移動すると、開く Menu も一緒に移動します。既存の Menu の anatomy はそのまま再利用せず、開閉状態機械・値語彙のみを再利用します。矢印キーによる実際のフォーカス移動（DOM 配線）はクライアントランタイム（`fandhe-frontend-wasm-full`）側の後続実装です。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
