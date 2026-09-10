# Message Scroller

`fandhe-frontend-pre-styled-ui` の `message_scroller` mod が提供する
スタイル済み Message Scroller 部品です。AI チャット UI の会話ログを
収めるスクロールコンテナで、Root / Viewport / Content / Anchor /
Jump To Latest / Load More の 6 パーツ構成です。

`viewport` は `overflow-y: auto` によるネイティブスクロール（JS 不要）
で、既定で高さ `24rem`（`--fandhe-message-scroller-height` で上書き可能）
を確保します。両端は `mask-image` による既定フェードを持ち、`root` が
最下部に張り付いている（`data-stuck="bottom"`）ときは末尾フェードを
自動的に解除して最新メッセージを霞ませません。

`root` の `data-stuck`（`bottom` / `free`）・`data-has-new`（存在属性）、
`jump_to_latest` の `data-visible` / `hidden`（2 択）、`load_more` の
`data-loading` / `data-disabled`（存在属性）は headless 層が出力する
`data-*` を CSS セレクタとして参照するだけで、class ベースの軸は持ちま
せん。`data-has-new` が付いた `root` の `jump-to-latest` はアクセント色
へ強調されます。`load_more` は `loading` が `true` のとき装飾的な
spinner を children 先頭へ埋め込みます。

最下部追従・新着検知・履歴読み込み時の位置維持といったスクロール位置の
計測・実行時更新はこの部品では実装しません（`fandhe-frontend-wasm-full`
側の配線が別イシューで担当します）。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
