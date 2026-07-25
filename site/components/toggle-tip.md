# Toggle Tip

`fandhe-frontend-pre-styled-ui` の `toggle_tip` mod が提供するスタイル済み Toggle Tip 部品です。
クリック開閉の小型ヒントで、見た目は Tooltip（小型・非モーダル）、挙動は
Popover（クリックで開閉し明示的に閉じるまで持続）の変種として位置づけられます。
Tooltip・Popover のいずれとも異なり、trigger / content のいずれにも
`role="tooltip"` を付与しない独自の ARIA 表現を採用しています。

> [!IMPORTANT]
> Demo はトリガー起点のオーバーレイ部品を「開いた状態」で固定掲示しています。
> 本来の配置（画面全体を覆う・トリガー直下に重なる）ではページ内の他セクションと
> 重なるため、掲示専用 CSS（`assets/pre-styled-ui.css` の `.pre-styled-showcase`
> スコープ）でページの流れの中へ収めています。実アプリケーションでの overlay 配置は
> pre-styled-ui の recipe CSS がそのまま担います。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
