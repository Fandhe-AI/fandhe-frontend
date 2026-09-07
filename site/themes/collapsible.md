# Collapsible

`fandhe-frontend-pre-styled-ui` の `collapsible` mod が提供するスタイル済み Collapsible 部品です。
トリガー（Trigger）・開閉状態を示すインジケータ（Indicator）・パネル（Content）を持つ
Root / Trigger / Indicator / Content の 4 パーツ構成で、`data-state`（open/closed）と
`data-disabled` をトリガーの文字色強調・インジケータの回転・減光として視覚に反映します。
ページ内に収まる disclosure（開閉パネル）であり、他のセクションを覆うオーバーレイでは
ないため、掲示位置を中和する専用 CSS は不要です。パネルの開閉は headless 層が付与する
`hidden` 属性のみで行い、開閉時の高さアニメーション（Radix の `collapsedHeight` 相当）は
意図的に非採用としています。理由はコンテンツ高さの実測が JS 前提という点だけでなく、
closed 時に付与される `hidden` 属性を上書きすると閉状態でも表示されてしまう構造的な
制約にもよります（shadcn/ui にも JS レスの代替実装はありません）。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
