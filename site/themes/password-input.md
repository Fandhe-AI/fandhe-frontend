# Password Input

`fandhe-frontend-pre-styled-ui` の `password_input` mod が提供するスタイル済み Password Input 部品です。

`password_input` mod からも `input_with_autocomplete` と `AutocompleteToken` / `AutocompleteField` を使えるため、pre-styled-ui のパスだけで `PasswordAutocomplete::CurrentPassword.with_webauthn()` を渡して `current-password webauthn` を指定できます。出力は固定語彙の静的リテラルに限られ、任意の文字列は受け付けません。passkey での使いどころと、ユーザー名欄との組み合わせは [Password Input（Primitives）](../primitives/password-input.md) を参照してください。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
