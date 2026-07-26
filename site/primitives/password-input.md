# Password Input

`fandhe-frontend-headless-ui` の `password_input` mod が提供する表示切替トリガー付きパスワード入力の unstyled 部品です。Root / Label / Control / Input / VisibilityTrigger / Indicator の 6 anatomy パーツで構成され、表示切替（`visible`）状態機械のみを持ちます。パスワード値そのものは一切保持・出力しません（`input` は `value` 引数自体を持たない設計です）。

`/themes/` 側の `PasswordInput`（Chakra ライクな variant/size を持つスタイル層）と異なり、CSS を一切持ちません。

スタイル済みの表示例は [Password Input](../themes/password-input.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
