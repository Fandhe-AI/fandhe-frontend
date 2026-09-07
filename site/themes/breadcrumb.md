# Breadcrumb

`fandhe-frontend-pre-styled-ui` の `breadcrumb` mod が提供するスタイル済み Breadcrumb 部品です。

現在位置までの階層を示すナビゲーション部品です。root は既定で aria-label="breadcrumb" を持ち、現在ページには aria-current="page" が付与されます。ページ内の段階的な進行を示す用途には [Steps](steps.md) を検討してください。

区切り文字は `separator()` が受け取る `children` で自由に差し替えられます（`/` 以外にも `›`・`·`・SVG アイコン等を渡すだけで対応可能で、recipe/CSS の変更は不要です）。中間項目を省略して表示したい場合は `ellipsis()`（装飾専用、非対話）と [Menu](menu.md) を組み合わせることで、省略記号をクリックすると省略された項目の一覧をドロップダウン表示するパターンを構築できます（Demo の 2 つ目・3 つ目の掲示を参照）。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
