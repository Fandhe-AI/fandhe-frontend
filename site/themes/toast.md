# Toast

`fandhe-frontend-pre-styled-ui` の `toast` mod が提供するスタイル済み Toast 部品です。
一時的な通知を有界なキューとして管理する状態機械 `Toaster` を持ち、`aria-live`
は通知の状態（`ToastStatus`）から決定的に導出します（`Error` のみ `assertive`、
他は `polite`）。`aria-atomic="true"` を併用し通知全体を単位として読み上げさせ
ます。

> [!IMPORTANT]
> Demo はトリガー起点のオーバーレイ部品と同じく、画面端固定の通知を「表示中の状態」で
> 固定掲示しています。本来の配置（画面端固定・複数通知の積み上げ）ではページ内の
> 他セクションと重なるため、掲示専用 CSS（`assets/pre-styled-ui.css` の
> `.pre-styled-showcase` スコープ）でページの流れの中へ収めています。実アプリケーション
> での overlay 配置は pre-styled-ui の recipe CSS がそのまま担います。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
