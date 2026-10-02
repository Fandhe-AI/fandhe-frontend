# Drawer

`fandhe-frontend-pre-styled-ui` の `drawer` mod が提供するスタイル済み Drawer 部品です。
画面端からスライドインするパネルで、WAI-ARIA 上は Dialog パターンの変種のため
新規状態機械を作らず `dialog` の `Disclosure` 状態機械をそのまま再利用します。
`DrawerPlacement`（始端/終端/上端/下端、既定は終端）でどの端から出現するかを
切り替えられます。`close_trigger_with_variant(CloseTriggerVariant::Text, ...)`
で `dialog` と対称の平文ボタン見た目を持たせることもできますが、
`fandhe-frontend-wasm-full` が drawer scope の click 配線を未対応のため、
アクション行に置いても現状クリックでは閉じません（別イシューで追跡）。
`body`/`footer` は headless anatomy を変更しない pre-styled-only のレイアウト
専用パートです。`content` の attrs へ `data-has-body` を付けると opt-in で
flex column 化され、`body` が残り高さを埋めてスクロールします（付けない場合は
`body` を使っても従来どおり `content` 全体がスクロールします）。

`header` も同様に headless anatomy を変更しない pre-styled-only のパートで、
パネル上端まで広がる見出し帯のレイアウトのみを担います。`header`/
`description`/`close_trigger` へ `data-tone="accent"` を付けると、3 パートが
連動してアクセント色の塗りに切り替わります（個別の opt-in 属性のため、塗りを
適用したい各パートへ呼び出し側がそれぞれ付与してください）。`close_trigger`
は `content` へ `data-close-outside`、`close_trigger` 自身へ
`data-close-outside="<start|end|top|bottom>"`（`DrawerPlacement` の値と一致
させてください）を付けると、パネルの外側（暗幕側）へ配置できます。長い本文は
`data-has-body` + `body` と組み合わせてスクロールさせる前提です。

> [!IMPORTANT]
> Demo はトリガー起点のオーバーレイ部品を「開いた状態」で固定掲示しています。
> 本来の配置（画面全体を覆う・トリガー直下に重なる）ではページ内の他セクションと
> 重なるため、掲示専用 CSS（`assets/pre-styled-ui.css` の `.pre-styled-showcase`
> スコープ）でページの流れの中へ収めています。実アプリケーションでの overlay 配置は
> pre-styled-ui の recipe CSS がそのまま担います。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
