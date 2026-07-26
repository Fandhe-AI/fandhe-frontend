# Segment Group

`fandhe-frontend-headless-ui` の `segment_group` mod が提供する segmented control（セグメントコントロール）の unstyled 部品です。Root / Indicator / Item / ItemText / ItemControl / ItemHiddenInput の 6 anatomy パーツを持ちますが、状態機械・dispatch・hydration は `radio_group` へ全委譲しており独自の状態機械は持ちません（WAI-ARIA 上 segmented control は radio パターンそのものであるため）。`Indicator` は SSR 決定的な位置表現（CSS カスタムプロパティ 2 種）のみを提供します。

`/themes/` 側の `SegmentGroup`（Chakra ライクな variant/size を持つスタイル層）と異なり、CSS を一切持ちません。

スタイル済みの表示例は [Segment Group](../themes/segment-group.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
