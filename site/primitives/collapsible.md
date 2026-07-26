# Collapsible

単一の開閉パネルです。`fandhe-frontend-headless-ui` の `collapsible` mod は
Root / Trigger / Indicator / Content の 4 anatomy パーツと、開閉状態を表す
`data-state`・`aria-expanded`・`aria-controls` を提供します。開閉状態の
遷移は `open`/`close`/`toggle` の dispatch で行い、closed のときは
`hidden` 存在属性を付与して JS なしの SSR でも閉状態を表現します。
`fandhe-frontend-pre-styled-ui` にはまだ対応するスタイル済み部品がありません。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
