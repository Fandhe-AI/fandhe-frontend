# Pin Input

`fandhe-frontend-pre-styled-ui` の `pin_input` mod が提供するスタイル済み Pin Input 部品です。

桁グループ（例: 6 桁を 3-3 に分ける）を視覚的に区切りたい場合は `separator()` を `control()` の間に挟みます（pre-styled-only パートで headless-ui の anatomy には存在しません）。区切り文字は `children` で自由に差し替えられます（`-` 以外にも任意のテキスト・アイコンを渡せます）。`role="presentation"` + `aria-hidden="true"` を固定付与するため、スクリーンリーダーの読み上げには影響しません。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
