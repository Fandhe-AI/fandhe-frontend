# Card

`fandhe-frontend-pre-styled-ui` の `card` mod が提供するスタイル済み Card 部品です。

関連情報をひとまとめに表示するレイアウトコンテナです。header/body/footer/title/description/action/cover の 8 パーツを組み合わせて構造化できます（action/cover はイシュー #2046 で shadcn/ui 突合により純追加）。純粋なレイアウト部品のため role/aria-* は付与しません。

variant（elevated/outline/subtle）に加え、size（xs〜xl、既定 md）で padding・角丸・title の文字サイズが連動して変化します（chakra-ui/Radix Themes の Card を参考にした調整）。size sm 相当（shadcn/ui の `size="sm"`）はこの 5 段の size 軸で包含済みのため別軸を追加していません。header/footer の区切り線は既定では持たず（chakra-ui/Radix Themes 準拠）、padding のみで段を分けます。

header へ `data-has-action` を付けると grid 化され、`action` パーツ（shadcn/ui `CardAction` 相当）が右上へ配置されます。`cover` パーツへ `image::image` を子として渡すと、上端の角が root の角丸に沿ってクリップされる cover image 枠になります。header/footer へ `data-bordered` を付けると、shadcn/ui の Header/Footer with Border 相当の 1px 区切り線が opt-in で表示されます（既定の区切り線なしは変わりません）。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md)
