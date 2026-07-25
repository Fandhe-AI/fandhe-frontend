# Drawer

`fandhe-frontend-pre-styled-ui` の `drawer` mod が提供するスタイル済み Drawer 部品です。
画面端からスライドインするパネルで、WAI-ARIA 上は Dialog パターンの変種のため
新規状態機械を作らず `dialog` の `Disclosure` 状態機械をそのまま再利用します。
`DrawerPlacement`（始端/終端/上端/下端、既定は終端）でどの端から出現するかを
切り替えられます。

> [!IMPORTANT]
> Demo はトリガー起点のオーバーレイ部品を「開いた状態」で固定掲示しています。
> 本来の配置（画面全体を覆う・トリガー直下に重なる）ではページ内の他セクションと
> 重なるため、掲示専用 CSS（`assets/pre-styled-ui.css` の `.pre-styled-showcase`
> スコープ）でページの流れの中へ収めています。実アプリケーションでの overlay 配置は
> pre-styled-ui の recipe CSS がそのまま担います。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
