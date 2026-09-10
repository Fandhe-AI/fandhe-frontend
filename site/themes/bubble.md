# Bubble

`fandhe-frontend-pre-styled-ui` の `bubble` mod が提供するスタイル済み
Bubble 部品です。AI チャット UI の「吹き出し 1 個」を表現する Root /
Content / Reactions / Reaction / CollapseTrigger / CollapseContent の
6 パーツ構成で、塗り・枠線・無装飾の 3 形態と連続発言の角丸連結を見た目に
反映します。

`variant`（`solid` / `outline` / `plain`）・`align`（`start` / `end`）・
`group-position`（`single` / `first` / `middle` / `last`）・`selected`・
`state`（`open` / `closed`）は headless 層が出力する `data-variant` /
`data-align` / `data-group-position` / `data-selected` / `data-state` を
CSS セレクタとして参照するだけで、class ベースの軸は持ちません。
`variant` ごとに背景・文字色・枠線が切り替わり（`solid` はアクセント塗り、
`outline` は枠線のみ、`plain` は無装飾）、`align="end"` に応じて連続発言の
隣接辺の角丸が潰れます（`group-position` の `first` / `middle` / `last`。
`single` は角丸が潰れません）。実際の色調選択（アクセント色以外）は
`ColorPalette` 軸の責務ではなく、`root` が公開する
`--fandhe-bubble-bg` / `--fandhe-bubble-fg` / `--fandhe-bubble-border` の
3 custom property を呼び出し側が上書きするフックとして提供します。

`reactions` / `reaction` はリアクションチップの表示のみを担う非対話パーツ
です。押下・集計・トグルはこの部品では実装しません。`collapse-trigger` /
`collapse-content` は折りたたみ詳細を表現し、開閉は headless 層が出力する
`hidden` 属性で行いますが、JS 有効時は `fandhe-frontend-wasm-full` が実測
した高さを CSS 変数（`--fandhe-content-height`）へ書き込む経路と組み合わ
さった場合に `@starting-style` + `transition-behavior: allow-discrete` に
よる高さトランジションとして開閉が表現されます（`collapsible` /
`accordion` と同じ共通機構。`hidden` 契約自体は維持したままの CSS ネイ
ティブな遷移）。JS 無効時（変数未設定）は `auto` フォールバックにより従来
どおり `hidden` による即時切り替えのまま動作します。

「リスト中の何番目か」「前後の発言者が同じか」を突き合わせて
`group-position` を選ぶ計算、リアクションの押下・集計・トグル、折りたたみ
のトグル自体はアプリケーションロジックであり、この部品では実装しません。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
