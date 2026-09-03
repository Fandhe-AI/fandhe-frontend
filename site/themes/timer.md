# Timer

`fandhe-frontend-pre-styled-ui` の `timer` mod が提供するスタイル済み Timer 部品です。

`root` の `data-state`（`completed` / `paused`）に応じて `item-value` の文字色が切り替わります（`completed` は accent 色、`paused` は muted 色）。`item-value`/`separator` のフォントサイズは `--fandhe-timer-value-font-size`、色は `--fandhe-timer-value-color` の custom property で上書きできます（クラス付与不要）。size / variant 軸は提供しません（参照サイトの ark-ui Timer も unstyled でスケールを持たないため）。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
