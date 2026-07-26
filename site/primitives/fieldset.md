# Fieldset

複数の [Field](field.md) をグループ化するネイティブ `<fieldset>`/`<legend>` コンテナです。`<fieldset disabled>` の HTML 標準挙動（内側の全コントロールが自動的に無効化される）を前提に、グループ全体の `disabled`/`invalid` を各 Field へ整合させます。

`fandhe-frontend-headless-ui` の `fieldset` mod が提供する構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを持つ unstyled 部品です。対応する `/themes/` ページは現時点で存在しません（本部品は headless-ui 側でのみ提供されます）。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
