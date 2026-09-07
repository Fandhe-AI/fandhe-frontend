# Hover Card

`fandhe-frontend-pre-styled-ui` の `hover_card` mod が提供するスタイル済み Hover Card 部品です。
リンク先プレビュー等を目的に hover / focus で開閉するオーバーレイで、trigger は
`a` 要素として組み立てられます。WAI-ARIA に hover card 専用パターンは存在しない
ため `aria-expanded`/`aria-controls`/`aria-haspopup` は付与しません。開閉遅延
（既定 600ms/300ms）は `data-open-delay`/`data-close-delay` として決定的に
出力される SSR 設定値で、実タイマー駆動は wasm-full 側の責務です。content は
自由な `children` を受け取るため、`fandhe-frontend-pre-styled-ui` の `avatar`
mod と組み合わせてユーザープロフィールプレビュー等の合成パターンも表現できます
（下記 Examples 節「User profile preview」参照）。

> [!IMPORTANT]
> Demo はトリガー起点のオーバーレイ部品を「開いた状態」で固定掲示しています。
> 本来の配置（画面全体を覆う・トリガー直下に重なる）ではページ内の他セクションと
> 重なるため、掲示専用 CSS（`assets/pre-styled-ui.css` の `.pre-styled-showcase`
> スコープ）でページの流れの中へ収めています。実アプリケーションでの overlay 配置は
> pre-styled-ui の recipe CSS がそのまま担います。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
