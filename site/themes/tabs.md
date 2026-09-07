# Tabs

`fandhe-frontend-pre-styled-ui` の `tabs` mod が提供するスタイル済み Tabs 部品です。
WAI-ARIA APG の Tabs パターン（`role="tablist"`/`"tab"`/`"tabpanel"`・
`aria-selected`・相互参照する `aria-controls`/`aria-labelledby`）に準拠した
マークアップを SSR 時点の静的な選択状態から組み立てます。ページ内に収まる
コンテンツ切り替え UI であり、他のセクションを覆うオーバーレイではないため、
掲示位置を中和する専用 CSS は不要です。`tabs()` の第 1 引数 `variant`
（`TabsVariant`）で下線スタイル（`Line`、既定）とセグメント/ピル型スタイル
（`Enclosed`、shadcn/ui 既定 variant 相当）を選べます。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
