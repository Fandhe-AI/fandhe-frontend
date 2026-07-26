# Tags Input

`fandhe-frontend-headless-ui` の `tags_input` mod が提供するタグ配列入力の unstyled 部品です。Root / Label / Control / Input / Item / ItemPreview / ItemText / ItemInput / ItemDeleteTrigger / ClearTrigger / HiddenInput / LiveRegion の 12 anatomy パーツを持ち、タグ文字列の可変長リスト + 重複拒否 + 上限 + 編集中インデックスを表現する値状態機械を備えます。重複・カンマ含有・空文字列のタグはすべての入口で拒否されます。LiveRegion はタグ数の変化を `aria-live="polite"` で支援技術へ通知します（テキスト更新の実配線は wasm-full の後続責務）。

`/themes/` 側の `TagsInput`（Chakra ライクな variant/size を持つスタイル層）と異なり、CSS を一切持ちません。

スタイル済みの表示例は [Tags Input](../themes/tags-input.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
