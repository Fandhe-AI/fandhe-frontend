# Rating Group

`fandhe-frontend-headless-ui` の `rating_group` mod が提供する星評価の unstyled 部品です。Root / Label / Control / Item / HiddenInput の 5 anatomy パーツを持ち、`1..=count` の数値評価値（未評価は `None`）と hover プレビューを表現する状態機械を備えます。`readonly` フラグにより表示専用の平均評価等も安全に描画できます。

`/themes/` 側の `RatingGroup`（Chakra ライクな variant/size を持つスタイル層）と異なり、CSS を一切持ちません。

スタイル済みの表示例は [Rating Group](../themes/rating-group.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
