# Switch

`fandhe-frontend-headless-ui` の `switch` mod が提供するオン/オフ切り替えの unstyled 部品です。Root（`<label>`）/ Control / Thumb / Label / HiddenInput の 5 anatomy パーツを持ち、`crate::state::Checkable` を埋め込んだチェック状態機械（`data-state` は `"checked"`/`"unchecked"`、Checkbox/RadioGroup と共有語彙）を備えます。意味論・フォーム送信はネイティブ `<input type="checkbox" role="switch">`（`HiddenInput`）に委ねる構造です。

`/themes/` 側の `Switch`（Chakra ライクな variant/size を持つスタイル層）と異なり、CSS を一切持ちません。

スタイル済みの表示例は [Switch](../themes/switch.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
