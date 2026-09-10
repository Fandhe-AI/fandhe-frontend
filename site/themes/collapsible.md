# Collapsible

`fandhe-frontend-pre-styled-ui` の `collapsible` mod が提供するスタイル済み Collapsible 部品です。
トリガー（Trigger）・開閉状態を示すインジケータ（Indicator）・パネル（Content）を持つ
Root / Trigger / Indicator / Content の 4 パーツ構成で、`data-state`（open/closed）と
`data-disabled` をトリガーの文字色強調・インジケータの回転・減光として視覚に反映します。
ページ内に収まる disclosure（開閉パネル）であり、他のセクションを覆うオーバーレイでは
ないため、掲示位置を中和する専用 CSS は不要です。パネルの開閉は headless 層が付与する
`hidden` 属性で行いますが、JS 有効時は `fandhe-frontend-wasm-full` が実測した content の
高さを CSS 変数（`--fandhe-content-height`）へ書き込み、`@starting-style` +
`transition-behavior: allow-discrete` によって開閉を高さトランジションとして表現します
（`hidden` 契約自体は維持したままの CSS ネイティブな遷移、Radix の `collapsedHeight` の
ような JS 実測依存の部分表示は再現しません）。JS 無効時（変数未設定）は `auto` フォール
バックにより従来どおり `hidden` による即時切り替えのまま動作します。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
