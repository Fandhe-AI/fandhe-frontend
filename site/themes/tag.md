# Tag

`fandhe-frontend-pre-styled-ui` の `tag` mod が提供するスタイル済み Tag 部品です。

削除可能なラベル表示部品です。close_trigger（type="button"）を組み合わせることで削除操作を持つ Tag を構成できますが、aria-label・視覚内容（×等）は呼び出し側が渡す責務です。削除操作を持たない単純なラベル表示には [Badge](badge.md) を検討してください。

variant（Solid/Subtle（既定）/Outline/Surface）・size（Xs〜Xl）・colorPalette（6 値、既定 Accent）の 3 軸を持ちます（イシュー #1573 で参照サイト〔chakra-ui〕基準へ調整し、`Surface` variant・close-trigger の hover/キーボードフォーカスリングを追加）。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md)
