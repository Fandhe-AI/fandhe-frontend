# Card

`fandhe-frontend-pre-styled-ui` の `card` mod が提供するスタイル済み Card 部品です。

関連情報をひとまとめに表示するレイアウトコンテナです。header/body/footer/title/description の 6 パーツを組み合わせて構造化できます。純粋なレイアウト部品のため role/aria-* は付与しません。

variant（elevated/outline/subtle）に加え、size（xs〜xl、既定 md）で padding・角丸・title の文字サイズが連動して変化します（chakra-ui/Radix Themes の Card を参考にした調整）。header/footer の区切り線は参照サイトに合わせて廃止し、padding のみで段を分けます。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md)
