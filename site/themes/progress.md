# Progress

`fandhe-frontend-pre-styled-ui` の `progress` mod が提供するスタイル済み Progress 部品です。

処理の進捗を示す部品です。linear（Track/Range）と circular（SVG の Circle/CircleTrack/CircleRange）の両方に対応します。track/circle 系は headless の inherent メソッドをそのまま呼び出す契約で、動的な `--fandhe-progress-percent` を持つ range のみ styled `range()` ラッパーを経由します。`ProgressProps`（`size`/`variant`/`palette` の 3 軸）を root へ渡すことで、サイズ（xs〜xl）・track の見た目（outline/subtle/plain）・range の塗り色（accent/info/success/warning/danger/neutral）を切り替えられます。value が None（indeterminate）のときはアニメーション（linear は横スライド、circular は回転）が付与され、`prefers-reduced-motion: reduce` 環境では停止します。circular の indeterminate は回転に加え circle-range へ円周の 1/4 分の固定弧（`stroke-dasharray`）を与え、完全なリング（complete）と視覚的に区別します。この弧に `animation` は使わないため、`prefers-reduced-motion: reduce` 環境でも静止した弧として残ります。読み込み中であることのみを示す用途には [Spinner](spinner.md) を検討してください。

イシュー #2049 で shadcn/ui と突合し、既存の Outline（中立トラック + 淡い内側枠線）・Subtle（palette 淡色トラック）では表現できなかった「枠線なしの中立トラック」を `ProgressVariant::Plain`（shadcn/ui 既定表現に相当）として補完しました。既存 2 variant の CSS 出力・既定 variant はバイト不変です。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
