# Marquee

`fandhe-frontend-pre-styled-ui` の `marquee` mod が提供するスタイル済み Marquee 部品です。

横方向に流れるテロップ表示部品です。content パーツを内部で 2 回複製しシームレスループを実現します。root:hover/:focus-within で常時一時停止し、prefers-reduced-motion: reduce でアニメーションを停止します。edge: Fade で両端フェード（mask-image）を適用でき、--fandhe-marquee-duration / -delay / -loop-count / -gap / -edge-size の CSS カスタムプロパティで速度・間隔・両端フェード幅を調整できます。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md)
