# Number Input

`fandhe-frontend-headless-ui` の `number_input` mod が提供する数値入力の unstyled 部品です。Root / Label / Control / Input / IncrementTrigger / DecrementTrigger / ValueText の 7 anatomy パーツで構成され、`min`/`max`/`step` に沿った値の正規化・境界到達時のトリガー無効化（`data-disabled`）を担います。連続量のため離散的な `data-state` は持ちません。

`NumberInputFlags`（`disabled`/`readonly`/`required`/`invalid`）は Root / Label / Control / Input / ValueText の全パーツで共通に使い、`data-readonly` は Root/Control/ValueText へ、`data-required` は Label のみへ反映されます（ark-ui/zag.js の number-input machine に倣った差分）。Control は `role="group"` を持ち、`disabled`/`invalid` に応じて `aria-disabled`/`aria-invalid` を出力します。dispatch は `"increment"`/`"decrement"`/`"set"`/`"clear"` に加えて `"home"`（値を `min` へ）/`"end"`（値を `max` へ）に対応します。参考サイトが持つ Scrubber パーツ（Pointer Lock API 前提）は非採用です。

`/themes/` 側の `Input`（Chakra ライクな variant/size を持つスタイル層）と異なり、CSS を一切持たず、増減操作の実 DOM 配線もクライアントランタイム側の責務です。

スタイル済みの表示例は [Number Input](../themes/number-input.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
