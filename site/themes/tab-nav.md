# Tab Nav

`fandhe-frontend-pre-styled-ui` の `tab_nav` mod が提供するスタイル済み Tab Nav 部品です。

見た目は [Tabs](tabs.md) と同じ下線付き水平タブですが、意味論はページ遷移を伴う通常のナビゲーションリンク集合です。`role="tablist"`/`role="tab"` を一切出力せず、素の `nav`/`a` の暗黙 ARIA ロール（`navigation`/`link`）のみを使い、現在ページは `aria-current="page"` で示します。パネル切り替え UI が必要な場合は [Tabs](tabs.md) を、縦方向の文書ナビ（サイドバー用途）が必要な場合は [Nav List](nav-list.md) を使ってください。

`root` へ size（`xs`/`sm`/`md`/`lg`/`xl`、既定 `md`）を指定できます。hover・キーボードフォーカスリングも備えます。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md)
