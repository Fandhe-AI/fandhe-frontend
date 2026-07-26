# Timer

カウントダウン/カウントアップを表示する unstyled 部品です。時計 API を一切使わず、時間の前進は tick（デルタミリ秒）の明示的な注入のみで進む決定的な状態機械です。

Themes 版（`fandhe-frontend-pre-styled-ui`）はこの構造へ既定 CSS を追加するだけの薄いラッパーであり、CSS は持ちません。スタイル済みの表示例は [Timer](../themes/timer.md) を参照してください。

実際の計時駆動（`setInterval` 相当）はクライアントランタイム側の責務であり、本部品は経過値の表示 anatomy と現在フェーズの表示状態のみを提供します。

関連 API: [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
