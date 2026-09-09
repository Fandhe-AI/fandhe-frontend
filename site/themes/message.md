# Message

`fandhe-frontend-pre-styled-ui` の `message` mod が提供するスタイル済み
Message 部品です。AI チャット UI の「会話 1 発言」を表現する Root /
Avatar / Header / Content / Footer / Group の 6 パーツ構成で、発言者の
役割・整列・応答待ち・送信失敗を見た目に反映します。

`role`（`user` / `assistant` / `system`）・`align`（`start` / `end`）・
`loading` / `error` は headless 層が出力する `data-role` / `data-align` /
`data-loading` / `data-error` を CSS セレクタとして参照するだけで、class
ベースの軸は持ちません。`role` ごとに `content` の背景・文字色が切り替わり
（`user` は accent、`assistant` は muted、`system` は透明 + 斜体）、
`loading` は `root` を半透明化、`error` は `content` の背景・文字色・枠線を
危険色へ切り替えます。

`group` は複数 `root` をまとめる連続発言のコンテナです。2 件目以降の
`root` は余白が詰まり、`avatar` は幅を残したまま非表示になります
（`display: none` ではなく `visibility: hidden` を使うことで先頭行との
横位置ずれを防ぎます）。応答のストリーミング通知（`aria-live`）や
応答待ちの読み上げ（`aria-busy`）はこの部品では付与しません。通知が
必要な場合は呼び出し側が自前で `aria-live` リージョンを合成してください。

バリデーション・送信処理・Markdown レンダリングといったアプリケーション
ロジックはこの部品では実装しません。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
