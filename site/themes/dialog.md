# Dialog

`fandhe-frontend-pre-styled-ui` の `dialog` mod が提供するスタイル済み Dialog 部品です。
画面全体を覆うモーダルダイアログで、通常の確認ダイアログ（`role="dialog"`）と
警告用ダイアログ（`role="alertdialog"`）を `DialogRole` で切り替えられます。
フォーカストラップ・Escape キーでの閉鎖・外側クリックでの閉鎖といったクライアント
挙動は JS ランタイム側の責務とし、本レイヤーは決定的な SSR 属性出力のみを担います。

alert-dialog（確認ダイアログ、Radix Primitives / Radix Themes の Alert Dialog に相当）は
独立部品や新しい variant 軸としては追加せず、`DialogRole::Alertdialog`（`role="alertdialog"`）と
`footer`（pre-styled-only のアクション列レイアウトパート）に既存の `button` の
variant / colorPalette 軸（Solid + Danger と Outline の組み合わせ等）を組み合わせて構成します。
`role="alertdialog"` の dialog は wasm-full 層が外側クリックでの閉鎖を既定で無効化します。
構成例は下記 Demo の Examples 節（Alert dialog）を参照してください。

> [!IMPORTANT]
> Demo はトリガー起点のオーバーレイ部品を「開いた状態」で固定掲示しています。
> 本来の配置（画面全体を覆う・トリガー直下に重なる）ではページ内の他セクションと
> 重なるため、掲示専用 CSS（`assets/pre-styled-ui.css` の `.pre-styled-showcase`
> スコープ）でページの流れの中へ収めています。実アプリケーションでの overlay 配置は
> pre-styled-ui の recipe CSS がそのまま担います。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
