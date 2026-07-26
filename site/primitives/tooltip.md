# Tooltip

吹き出しヒントです。`fandhe-frontend-headless-ui` の `tooltip` mod は
Root / Trigger / Positioner / Content / Arrow / ArrowTip の 6 anatomy
パーツを提供します。WAI-ARIA tooltip パターンに従い、trigger は
`aria-describedby` で content（`role="tooltip"`）と関連付けます。
`aria-expanded`/`aria-controls` は使用しません（Collapsible 等の
disclosure 系との違い）。`openDelay`/`closeDelay`/`interactive`/
`closeOnEscape` はクライアントサイド実行時挙動としてスコープ外です。

スタイル済みの表示例は [Tooltip](../themes/tooltip.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
