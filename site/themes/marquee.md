# Marquee

`fandhe-frontend-pre-styled-ui` の `marquee` mod が提供するスタイル済み Marquee 部品です。

横方向に流れるテロップ表示部品です。content パーツを内部で 2 回複製しシームレスループを実現します。root:hover/:focus-within で常時一時停止し、prefers-reduced-motion: reduce では停止した上で root を横スクロール可能にし両端フェードを無効化します。速度（--fandhe-marquee-duration）・方向（--fandhe-marquee-direction）・間隔（--fandhe-marquee-gap、既定 --fandhe-space-4）・両端フェード（--fandhe-marquee-fade、既定 0px）は CSS custom property の上書きで調整できます。root の枠（上下 padding は --fandhe-marquee-padding-y、背景は --fandhe-marquee-bg、枠線は --fandhe-marquee-border、角丸は --fandhe-marquee-radius。いずれも既定は中立値）も custom property の上書きで opt-in できます。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md)
