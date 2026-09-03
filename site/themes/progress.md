# Progress

`fandhe-frontend-pre-styled-ui` の `progress` mod が提供するスタイル済み Progress 部品です。

処理の進捗を示す部品です。linear（Track/Range）と circular（SVG の Circle/CircleTrack/CircleRange）の両方に対応します。track/circle 系は headless の inherent メソッドをそのまま呼び出す契約で、動的な `--fandhe-progress-percent` を持つ range のみ styled `range()` ラッパーを経由します。`ProgressProps`（`size`/`variant`/`palette` の 3 軸）を root へ渡すことで、サイズ（xs〜xl）・track の見た目（outline/subtle）・range の塗り色（accent/info/success/warning/danger/neutral）を切り替えられます。value が None（indeterminate）のときはアニメーション（linear は横スライド、circular は回転）が付与され、`prefers-reduced-motion: reduce` 環境では停止します。読み込み中であることのみを示す用途には [Spinner](spinner.md) を検討してください。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
