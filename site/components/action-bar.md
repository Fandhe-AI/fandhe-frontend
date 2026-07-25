# Action Bar

`fandhe-frontend-pre-styled-ui` の `action_bar` mod が提供するスタイル済み Action Bar 部品です。
複数項目を選択した状態で画面下部に現れる操作バー（chakra-ui の ActionBar 相当）で、
選択件数から開閉を自動導出する糖衣 API は持たず、「選択操作 → 開閉状態の決定」は
呼び出し側アプリケーションの責務とする設計です。開閉状態そのものは Dialog と同じ
`Disclosure` 状態機械を埋め込んで管理します。

> [!IMPORTANT]
> Demo はトリガー起点のオーバーレイ部品を「開いた状態」で固定掲示しています。
> 本来の配置（画面全体を覆う・トリガー直下に重なる）ではページ内の他セクションと
> 重なるため、掲示専用 CSS（`assets/pre-styled-ui.css` の `.pre-styled-showcase`
> スコープ）でページの流れの中へ収めています。実アプリケーションでの overlay 配置は
> pre-styled-ui の recipe CSS がそのまま担います。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
