# Pin Input

`fandhe-frontend-headless-ui` の `pin_input` mod が提供する PIN/OTP 桁入力の unstyled 部品です。Root / Label / Control / Input（桁ごと）/ HiddenInput の 5 anatomy パーツと、桁ごとの値・フォーカス位置・complete 判定を担う独自の値状態機械を持ちます。`data-complete` は全桁充足時のみの存在属性として一元管理されます。

`/themes/` 側の `PinInput`（Chakra ライクな variant/size を持つスタイル層）と異なり、CSS を一切持ちません。

スタイル済みの表示例は [Pin Input](../themes/pin-input.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
