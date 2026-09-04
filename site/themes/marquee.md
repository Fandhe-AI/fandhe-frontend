# Marquee

`fandhe-frontend-pre-styled-ui` の `marquee` mod が提供するスタイル済み Marquee 部品です。

横方向に流れるテロップ表示部品です。content パーツを内部で 2 回複製しシームレスループを実現します。root:hover/:focus-within で常時一時停止し、prefers-reduced-motion: reduce では単なる停止に留まらず、複製 content の除去・可視コピーの折り返し表示（全文が読める）・両端フェードの解除まで行います。速度（--fandhe-marquee-duration）・方向（--fandhe-marquee-direction）・間隔（--fandhe-marquee-gap、既定 --fandhe-space-4）・両端フェード（--fandhe-marquee-fade、既定 0px）・内側余白（--fandhe-marquee-padding、既定 0）は CSS custom property の上書きで調整できます。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md)
