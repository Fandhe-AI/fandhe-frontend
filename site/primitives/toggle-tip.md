# Toggle Tip

クリック開閉の小型ヒントです。`fandhe-frontend-headless-ui` の
`toggle_tip` mod は Root / Trigger / Positioner / Content / Arrow /
ArrowTip の 6 anatomy パーツを提供します。見た目は Tooltip（小型・
非モーダル）、挙動は Popover（クリックで開閉し明示的に閉じるまで持続）の
変種と位置づけられ、trigger は `aria-expanded` を持ちますが
`aria-haspopup` は付与せず、content も `role="tooltip"` を持ちません
（Tooltip・Popover いずれとも異なる 3 者境界）。

スタイル済みの表示例は [Toggle Tip](../themes/toggle-tip.md) を参照してください。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
