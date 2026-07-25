# Dialog

`fandhe-frontend-pre-styled-ui` の `dialog` mod が提供するスタイル済み Dialog 部品です。
画面全体を覆うモーダルダイアログで、通常の確認ダイアログ（`role="dialog"`）と
警告用ダイアログ（`role="alertdialog"`）を `DialogRole` で切り替えられます。
フォーカストラップ・Escape キーでの閉鎖・外側クリックでの閉鎖といったクライアント
挙動は JS ランタイム側の責務とし、本レイヤーは決定的な SSR 属性出力のみを担います。

> [!IMPORTANT]
> Demo はトリガー起点のオーバーレイ部品を「開いた状態」で固定掲示しています。
> 本来の配置（画面全体を覆う・トリガー直下に重なる）ではページ内の他セクションと
> 重なるため、掲示専用 CSS（`assets/pre-styled-ui.css` の `.pre-styled-showcase`
> スコープ）でページの流れの中へ収めています。実アプリケーションでの overlay 配置は
> pre-styled-ui の recipe CSS がそのまま担います。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
