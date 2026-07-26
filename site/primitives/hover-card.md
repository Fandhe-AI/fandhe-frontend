# Hover Card

リンク先プレビュー等、hover / focus で開閉するオーバーレイです。
`fandhe-frontend-headless-ui` の `hover_card` mod は Root / Trigger /
Positioner / Content / Arrow / ArrowTip の 6 anatomy パーツを提供します。
trigger はリンク先プレビュー用途の `a` 要素であり、`HoverCardDelays`
（`open_ms`/`close_ms`、ark-ui 既定の 600/300 ms）を `data-open-delay`/
`data-close-delay` として決定的に出力します。WAI-ARIA に hover card 専用
パターンは存在しないため、`aria-expanded`/`aria-controls`/`aria-haspopup`
及び固定 `role` を一切付与しません（Tooltip との違い）。

スタイル済みの表示例は [Hover Card](../themes/hover-card.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
