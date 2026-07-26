# Number Input

`fandhe-frontend-headless-ui` の `number_input` mod が提供する数値入力の unstyled 部品です。Root / Label / Control / Input / IncrementTrigger / DecrementTrigger の 6 anatomy パーツで構成され、`min`/`max`/`step` に沿った値の正規化・境界到達時のトリガー無効化（`data-disabled`）を担います。連続量のため離散的な `data-state` は持ちません。

`/themes/` 側の `Input`（Chakra ライクな variant/size を持つスタイル層）と異なり、CSS を一切持たず、増減操作の実 DOM 配線もクライアントランタイム側の責務です。

スタイル済みの表示例は [Number Input](../themes/number-input.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
