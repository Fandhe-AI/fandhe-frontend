# Fieldset

複数の [Field](field.md) をグループ化するネイティブ `<fieldset>`/`<legend>` コンテナです。`<fieldset disabled>` の HTML 標準挙動（内側の全コントロールが自動的に無効化される）を前提に、`disabled` を各 Field へ OR 伝播し、`invalid` はグループ単位でのみ表現します。

`fandhe-frontend-headless-ui` の `fieldset` mod が提供する構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを持つ unstyled 部品です。`fandhe-frontend-pre-styled-ui` に対応するスタイル済み部品があります。Themes 版は [Fieldset](../themes/fieldset.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
