# Checkbox Group

「0 個以上の項目が同時選択される」複数選択グループです。単一選択版の Radio Group と対称の構造を持ち、ネイティブ `<input type="checkbox">`（`checkbox` mod の `hidden_input` を再利用）をラベル配下へ入れ子にすることで、クリック委譲を JS なしで成立させます。

`fandhe-frontend-headless-ui` の `checkbox_group` mod が提供する構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを持つ unstyled 部品です。Themes 版が備える見た目は持たず、anatomy・ARIA・`data-*` のみを提供します。CSS は利用者が当てます。

`CheckboxGroupProps`（disabled/readonly/invalid）を root と各項目パーツへ渡すと `data-disabled`/`data-readonly`/`data-invalid` が一律出力されます。矢印キーによる項目間移動は採用せず、各項目のネイティブ `<input type="checkbox">` が独立した Tab ストップになります（ark-ui `Checkbox.Group` と同じ設計、Radix Themes の RovingFocus とは異なります）。

スタイル済みの表示例は [Checkbox Group](../themes/checkbox-group.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
