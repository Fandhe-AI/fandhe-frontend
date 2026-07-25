# Popover

`fandhe-frontend-pre-styled-ui` の `popover` mod が提供するスタイル済み Popover 部品です。
トリガー起点のオーバーレイで、`content` に `role="dialog"` を固定付与し、
`title`/`description` が設定されているときのみ `aria-labelledby`/
`aria-describedby` をセットで付与します。開閉は `Disclosure` を埋め込んだ
状態機械 `Popover` が管理します。

> [!IMPORTANT]
> Demo はトリガー起点のオーバーレイ部品を「開いた状態」で固定掲示しています。
> 本来の配置（画面全体を覆う・トリガー直下に重なる）ではページ内の他セクションと
> 重なるため、掲示専用 CSS（`assets/pre-styled-ui.css` の `.pre-styled-showcase`
> スコープ）でページの流れの中へ収めています。実アプリケーションでの overlay 配置は
> pre-styled-ui の recipe CSS がそのまま担います。
