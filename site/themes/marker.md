# Marker

`fandhe-frontend-pre-styled-ui` の `marker` mod が提供するスタイル済み
Marker 部品です。会話スレッド内のインライン注記行（システム注記・日付等の
区切り・ラベル付きセパレータ）を Root / Icon / Content の 3 パーツ構成で
表現します。

`variant`（`note` / `divider` / `label`）・`tone`（`neutral` / `info` /
`warning` / `danger`）は headless 層が出力する `data-variant` /
`data-tone` を CSS セレクタとして参照するだけで、class ベースの軸は
持ちません。`note` は既定のインライン注記表示（線なし）、`divider` は
行の下に境界線を伴う表示、`label` は中央ラベル + 左右の線を伴う表示です。
`tone` は `fandhe-frontend-pre-styled-ui` の `ColorPalette` と同名の
`neutral` / `info` / `warning` / `danger` の 4 値で、注記の文字色・線色を
切り替えます。

区切り線は疑似要素（`::before` / `::after`）を使わずに描画します。
`divider` 形態は DOM を増やさず `root` へ境界線（`border-bottom`）の CSS
規則を適用するだけですが、`label` 形態は呼び出し側の `children` を
スタイル済み Separator（水平・実線、`aria-hidden="true"`）2 個で挟んでから
headless 層へ委譲します。挟み込む Separator に `aria-hidden="true"` を
渡すのは、短いラベルの前後で `role="separator"` が 2 回読み上げられるのを
避けるためです。

`icon` は装飾スロットとして `aria-hidden="true"` を固定付与します
（呼び出し側の `aria-hidden="false"` 偽装は除去されます）。`content` は
注記の本文テキストを children で受け取るだけのスロットです。`root` には
`role` を固定付与しません。ストリーミング中の注記に `role="status"` を
付与したい場合は、呼び出し側が `attrs` で明示的に渡してください。

ストリーミング中判定・注記の自動分類・タイムスタンプ整形はアプリケーション
ロジックであり、この部品では実装しません。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
