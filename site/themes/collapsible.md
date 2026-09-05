# Collapsible

`fandhe-frontend-pre-styled-ui` の `collapsible` mod が提供するスタイル済み Collapsible 部品です。
トリガー（Trigger）・開閉状態を示すインジケータ（Indicator）・パネル（Content）を持つ
Root / Trigger / Indicator / Content の 4 パーツ構成で、`data-state`（open/closed）と
`data-disabled` をトリガーの文字色強調・インジケータの回転・減光として視覚に反映します。
ページ内に収まる disclosure（開閉パネル）であり、他のセクションを覆うオーバーレイでは
ないため、掲示位置を中和する専用 CSS は不要です。パネルの開閉は headless 層が付与する
`hidden` 属性のみで行い、開閉時の高さアニメーション（Radix の `collapsedHeight` 相当）は
コンテンツ高さの実測が前提となる JS 計測の関心のため意図的に非採用としています。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
