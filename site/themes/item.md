# Item

`fandhe-frontend-pre-styled-ui` の `item` mod が提供するスタイル済み Item
部品です。Root / Media / Content / Title / Description / Actions / Header /
Footer / Group / Separator の 10 パーツ構成で、media（アイコン・画像・
アバター）+ title/description + actions からなる汎用リスト行を表現します。

`variant`（`default` / `outline` / `muted`）・`size`（`default` / `sm`）は
headless 層が出力する `data-variant` / `data-size` を CSS セレクタとして
参照するだけで、class ベースの軸は持ちません。`root` は `href` を渡すと
`div` ではなく `a` として描画され、ポインタ・カーソル・下線解除に加えて
hover 背景と `:focus-visible` リングが付きます。`media` は
`data-variant="icon"` / `"image"` で固定サイズ・背景・角丸を切り替え、
`image` variant では子 `img` を `object-fit: cover` でトリミングします。

`group` は複数 `root` の縦並びコンテナで、`separator` を挟んで区切れます。
バリデーション・送信処理・データ整形といったアプリケーションロジックは
この部品では実装しません。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
