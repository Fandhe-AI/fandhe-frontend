# Floating Panel

`fandhe-frontend-pre-styled-ui` の `floating_panel` mod が提供するスタイル済み Floating Panel 部品です。
ドラッグ移動・リサイズ可能な浮遊パネルで、開閉に加えて `default`/`minimized`/
`maximized` の 3 値を持つ独自状態 `Stage` を管理します。非モーダル overlay の
ため `content` は `role="dialog"` のみを付与し `aria-modal` は出力しません
（ユーザーは他の要素を操作し続けられます）。

> [!IMPORTANT]
> Demo はトリガー起点のオーバーレイ部品を「開いた状態」で固定掲示しています。
> 本来の配置（画面全体を覆う・トリガー直下に重なる）ではページ内の他セクションと
> 重なるため、掲示専用 CSS（`assets/pre-styled-ui.css` の `.pre-styled-showcase`
> スコープ）でページの流れの中へ収めています。実アプリケーションでの overlay 配置は
> pre-styled-ui の recipe CSS がそのまま担います。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
