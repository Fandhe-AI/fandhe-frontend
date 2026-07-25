# Toolbar

`fandhe-frontend-pre-styled-ui` の `toolbar` mod が提供するスタイル済み Toolbar 部品です。

ボタン・リンク・セパレータ・ToggleGroup を横方向（または縦方向）にグループ化する操作バーです。`role="toolbar"` + `aria-orientation` を持つグループ化コンテナと、roving tabindex（フォーカスが常に 1 項目のみに当たる状態）の状態機械を提供します。押下状態の管理は独自実装せず、既存の [Toggle Group](toggle-group.md) の状態機械（`ToggleGroup`/`MultiToggleGroup`）をそのまま再利用します。矢印キーによる実際のフォーカス移動（DOM 配線）はクライアントランタイム（`fandhe-frontend-wasm-full`）側の後続実装です。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
