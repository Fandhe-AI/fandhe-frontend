# Tour

`fandhe-frontend-pre-styled-ui` の `tour` mod が提供するスタイル済み Tour 部品です。
オンボーディング向けステップガイドで、`open`/`closed` の 2 値に加え
`skipped`/`completed` という終端状態を持つ独自状態機械 `Tour` を提供します。
対象要素の実座標追従（DOM 解決・スクロール/リサイズ再計算）は行わず、
`TourStep::target` を `data-target` 属性としてエスケープ済みで出力するのみです。

> [!IMPORTANT]
> Demo はトリガー起点のオーバーレイ部品を「開いた状態」で固定掲示しています。
> 本来の配置（画面全体を覆う・トリガー直下に重なる）ではページ内の他セクションと
> 重なるため、掲示専用 CSS（`assets/pre-styled-ui.css` の `.pre-styled-showcase`
> スコープ）でページの流れの中へ収めています。実アプリケーションでの overlay 配置は
> pre-styled-ui の recipe CSS がそのまま担います。

`close_trigger` はアイコン専用契約です（固定正方 + `overflow: hidden`）。
`close_trigger(state, vec![("aria-label", "Close")], vec![text("×")])` の
ように 1 グリフ（`×` 等）と `aria-label` を渡してください。テキストを
渡した場合は正方形の枠内で切り詰められます。

`action_trigger` を複数並べる場合はボタン自身が持つ余白で間隔が空くため、
コンテナ側で追加のレイアウトを組む必要はありません。ボタンを無効化する
には呼び出し側で `attrs` へ `("disabled", "")` を渡してください
（ネイティブ `disabled` 属性がそのまま素通しされ、不透明度低下・
カーソル変更・hover 抑止に反映されます）。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
