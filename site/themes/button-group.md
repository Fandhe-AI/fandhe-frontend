# Button Group

`fandhe-frontend-pre-styled-ui` の `button_group` mod が提供するスタイル済み Button Group
部品です。Root / Separator / Text の 3 パーツ構成で、関連するボタンを角丸・境界線で
1 つの連結表示にまとめます。先頭・末尾以外の隣接要素の角丸・開始側境界線を無効化することで
連結表示を作り、対象は Button だけでなく Input / Select trigger / Menu trigger にも及びます。
`data-orientation` によって横並び（既定）と縦積みを切り替えられます。`role="group"` の
静的なグループであり、状態機械を持つ [Toolbar](./toolbar.md) の roving tabindex とは異なり、
子ボタンのフォーカス順序はネイティブの Tab 順序に委ねます。`size` / `variant` /
`color-palette` いずれの軸も持たず、寸法・文字サイズは内側のボタン・入力欄に従属します。
縦横混在のレイアウト（上段に入力欄、下段に横並びボタン等）はグループの入れ子で表現できます。
内側グループへ値なし属性 `data-attached` を付与すると、既定の margin 間隔を打ち消し、
外側グループと枠線を共有する 1 つの連結表示にする opt-in 表現も使えます（Examples 節参照）。

バリデーション・送信処理・クリックハンドラといったアプリケーションロジックはこの部品では
実装しません。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
