# Tooltip

`fandhe-frontend-pre-styled-ui` の `tooltip` mod が提供するスタイル済み Tooltip 部品です。
hover / focus で開く吹き出しヒントで、WAI-ARIA tooltip パターンに従い trigger
は `aria-describedby` で `content` と関連付けます（`aria-expanded`/
`aria-controls` は使いません）。`content` 側が `role="tooltip"` を持ちます。

> [!IMPORTANT]
> Demo はトリガー起点のオーバーレイ部品を「開いた状態」で固定掲示しています。
> 本来の配置（画面全体を覆う・トリガー直下に重なる）ではページ内の他セクションと
> 重なるため、掲示専用 CSS（`assets/pre-styled-ui.css` の `.pre-styled-showcase`
> スコープ）でページの流れの中へ収めています。実アプリケーションでの overlay 配置は
> pre-styled-ui の recipe CSS がそのまま担います。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
