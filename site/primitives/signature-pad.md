# Signature Pad

`fandhe-frontend-headless-ui` の `signature_pad` mod が提供する手書き署名入力の unstyled 部品です。Root / Label / Control / Segment（`svg`）/ SegmentPath / Guide / ClearTrigger / HiddenInput の 8 anatomy パーツを持ち、canvas を一切使わず、ストローク（座標列）を SVG path 文字列へ変換する決定的な純粋関数のみで描画します。ポインタイベントの収集自体は本部品の責務外です。

`/themes/` 側の `SignaturePad`（Chakra ライクな variant/size を持つスタイル層）と異なり、CSS を一切持ちません。

スタイル済みの表示例は [Signature Pad](../themes/signature-pad.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
