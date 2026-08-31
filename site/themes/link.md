# Link

`fandhe-frontend-pre-styled-ui` の `link` mod が提供するスタイル済み Link 部品です。素の `<a>` 要素 1 パーツ（anatomy `root`）のみで構成する最小部品で、Plain（下線なし）/Underline（常時下線）の 2 variant、colorPalette（6 値、既定 Accent）、現在ページ表示（`aria-current="page"`）、外部リンク（`target="_blank"` + `rel="noopener noreferrer"` を不可分に付与）を提供します。ホバー時は文字色を強調し、キーボード操作時（focus-visible）はフォーカスリングを表示します（色の変化には短いトランジションが伴います）。ページ遷移を伴う水平ナビが必要な場合は [Tab Nav](tab-nav.md) を、文書ナビ（サイドバー用途）が必要な場合は [Nav List](nav-list.md) を使ってください。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
