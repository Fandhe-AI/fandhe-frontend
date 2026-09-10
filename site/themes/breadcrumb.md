# Breadcrumb

`fandhe-frontend-pre-styled-ui` の `breadcrumb` mod が提供するスタイル済み Breadcrumb 部品です。

現在位置までの階層を示すナビゲーション部品です。root は既定で aria-label="breadcrumb" を持ち、現在ページには aria-current="page" が付与されます。ページ内の段階的な進行を示す用途には [Steps](steps.md) を検討してください。

区切り文字は `separator()` が受け取る `children` で自由に差し替えられます（`/` 以外にも `›`・`·`・SVG アイコン等を渡すだけで対応可能で、recipe/CSS の変更は不要です）。中間項目を省略して表示したい場合は `item()` の中に [Menu](menu.md) を配置し、trigger の表示文字列を省略記号（"…"）にすることで、クリックすると省略された項目の一覧をドロップダウン表示するパターンを構築できます（`ellipsis()` は装飾専用の非対話 `<li>` 固定で trigger の子にはできないため使いません。Demo の 2 つ目・3 つ目の掲示を参照）。

項目間の余白（`list` の `gap`）はビューポート幅 640px 以上で `--fandhe-space-1-5`（0.375rem）から `--fandhe-space-2-5`（0.625rem）へ自動的に広がります（shadcn/ui の `sm:gap-2.5` 相当）。上書きしたい場合は `[data-scope="breadcrumb"][data-part="list"]` セレクタの `gap` を利用者側 CSS で指定してください。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
