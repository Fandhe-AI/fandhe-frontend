# Slider

`fandhe-frontend-headless-ui` の `slider` mod が提供する単一値・連続量スライダーの unstyled 部品です。Root / Label / Control / Track / Range / Thumb / HiddenInput / ValueText の 8 anatomy パーツを持ち、`value` を常に `step` 単位へスナップしてから `[min, max]` へ clamp します（`max`/`min` へは常に到達可能）。`data-state` は持ちません（連続量のため離散的な区分がない）。

`/themes/` 側の `Slider`（Chakra ライクな variant/size を持つスタイル層）と異なり、CSS を一切持ちません。

スタイル済みの表示例は [Slider](../themes/slider.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
