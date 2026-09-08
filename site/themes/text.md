# Text

`fandhe-frontend-pre-styled-ui` の `text` mod が提供するスタイル済み Text 部品です。

size（xs〜xl4 の 8 段階、イシュー #1442）・weight（normal/medium/semibold/bold、イシュー #1442）・variant（plain（既定）/muted）の 3 軸を持ちます。イシュー #2055 で shadcn/ui Typography と突合し、前景色を弱める `variant=muted`（テーマトークン `fg-muted`）を追加しました。shadcn/ui の `lead`/`large`/`small`/`muted` プリセットはプリセット名を持ち込まず、既存の size/weight/variant 軸の組み合わせで再現します。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md)
