# Command

`fandhe-frontend-pre-styled-ui` の `command` mod が提供するスタイル済み
Command（cmdk 由来のコマンドパレット、shadcn/ui `Command` 相当）部品です。
Root / Input / List / Empty / Group / GroupHeading / Item / Shortcut /
Separator / Dialog の 10 パーツ構成で、入力欄・リスト・group 見出し・選択
行の背景・shortcut の右寄せ・dialog 型の幅を持つ意匠を重ねます。

`size` / `variant` / `color-palette` いずれの軸も持ちません。絞り込み結果
0 件は `empty` slot が既定 `display: none` で、headless 層が付与する
`data-empty` が付いたときのみ `display: block` へ切り替わります。`item`
の選択行は `data-selected` の背景色で表し、hover はその背景色を洗い流さな
いよう選択行を除外します。`shortcut` は右寄せのみを担い、`children` へ
[Kbd](../kbd/) を渡すことでキー表示を合成できます。`dialog` は
`--fandhe-command-dialog-max-width`（既定 32rem）で幅を決め、closed 時は
`hidden` で確実に非表示化します。

絞り込み配線・Enter 実行・Cmd/Ctrl+K のグローバルショートカット・
フォーカストラップといったアプリケーションロジックはこの部品では実装
しません。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
