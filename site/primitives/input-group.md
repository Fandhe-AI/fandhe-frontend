# Input Group

入力欄の前後にテキスト・アイコン・ボタンなどの addon を配置する複合パターンです。`root` / `addon` / `text` / `button` の 4 パーツで構成され、実際の `<input>`/`<textarea>` は [Field](field.md) の `input`/`textarea` を呼び出し側が子として合成します。

`addon` は `data-align` 属性（`inline-start` / `inline-end` / `block-start` / `block-end`）で配置位置を表現します。`inline-*` は `<input>` の前後、`block-*` は `<textarea>` の上下に addon を置く用途を想定しています。

`fandhe-frontend-headless-ui` の `input_group` mod が提供する構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを持つ unstyled 部品です。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
