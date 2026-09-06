# Field

1 個のフォームコントロール（`input`/`textarea`/`select`）を `label`・`helper_text`・`error_text` と一貫して結び付ける入力ラッパーです。`disabled`/`invalid`/`required`/`readonly` の各フラグから `for`/`id`/`aria-describedby`/`aria-invalid` を決定的に合成します。値の検証処理（バリデーションロジック）自体は利用者側の通常の Rust コードが担い、本部品はその結果（`invalid`/エラーメッセージ）を受け取って構造・ARIA へ反映するだけです。

`fandhe-frontend-headless-ui` の `field` mod が提供する構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを持つ unstyled 部品です。`fandhe-frontend-pre-styled-ui` に対応するスタイル済み部品があります。Themes 版は [Field](../themes/field.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
