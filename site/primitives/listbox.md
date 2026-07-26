# Listbox

単一/複数選択の一覧から選ばせる、常時展開のリスト部品です（ポップオーバー型の [Combobox](combobox.md)/Select と異なり、`content` パーツ自身がフォーカスを受けます）。`multiple` の有無で `aria-multiselectable`・単一/複数選択のセマンティクスを切り替えます。

`fandhe-frontend-headless-ui` の `listbox` mod が提供する構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを持つ unstyled 部品です。Themes 版が備える見た目は持たず、anatomy・ARIA・`data-*` のみを提供します。CSS は利用者が当てます。

スタイル済みの表示例は [Listbox](../themes/listbox.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
