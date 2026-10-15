# Field

1 個のフォームコントロール（`input`/`textarea`/`select`）を `label`・`helper_text`・`error_text` と一貫して結び付ける入力ラッパーです。`disabled`/`invalid`/`required`/`readonly` の各フラグから `for`/`id`/`aria-describedby`/`aria-invalid` を決定的に合成します。値の検証処理（バリデーションロジック）自体は利用者側の通常の Rust コードが担い、本部品はその結果（`invalid`/エラーメッセージ）を受け取って構造・ARIA へ反映するだけです。

ark-ui 準拠の 8 パーツに加え、イシュー #2185 で `group`（複数 `root` の縦積みコンテナ）・`content`/`title`（`<label for>` を結び付けにくい場面向けの見出し + 補助テキスト列）・`separator`（内部パーツ `separator-line`/`separator-content` から成るテキスト付き区切り線）の 6 パーツを純追加しました（shadcn/ui `FieldGroup`/`FieldContent`/`FieldTitle`/`FieldSeparator` 相当）。`title` は `for`/`id` を自動導出しないため、呼び出し側が `title` の `id` を対応するコントロールの `aria-labelledby` へ渡して結び付けます。

`fandhe-frontend-headless-ui` の `field` mod が提供する構造・アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）のみを持つ unstyled 部品です。`fandhe-frontend-pre-styled-ui` に対応するスタイル済み部品があります。Themes 版は [Field](../themes/field.md) を参照してください。

**型付き autocomplete**

`input_with_autocomplete` に `AutocompleteToken::new(AutocompleteField::Username).with_webauthn()` を渡すと、passkey ログインのユーザー名欄として `username webauthn` を出力します。`email` も指定できます（`email webauthn` は文法上は有効ですが、通常は使いません）。`extra_attrs` の `autocomplete` は大文字小文字を区別せず除去され、型付きの値が 1 個だけ出力されます。

`AutocompleteToken` は固定語彙で、任意の文字列を受け付けません。属性値に意図しない文字列が混ざる経路を型で閉じるためです。既存の `input` と `extra_attrs` の生文字列経路は互換のために残っていますが、新規のコードでは型付きの経路を使ってください。パスワード欄側の指定は [Password Input](./password-input.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
