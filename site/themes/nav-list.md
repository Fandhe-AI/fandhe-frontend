# Nav List

`fandhe-frontend-pre-styled-ui` の `nav_list` mod が提供するスタイル済み Nav List 部品です。`root`（`nav`）/ `heading`（`h2`）/ `list`（`ul`）/ `item`（`li`）/ `link`（`a`）の 5 anatomy パーツで構成する縦方向の文書ナビ部品で、`role` を一切出力せず素の HTML の暗黙 ARIA ロールのみを使います。本サイトのサイドバー自体もこの部品の headless 実装で組み立てられています。現在ページは `aria-current="page"` で示します。水平方向のページ内ナビ（タブ状の見た目）が必要な場合は [Tab Nav](tab-nav.md) を使ってください。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
