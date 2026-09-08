# Input Group

`fandhe-frontend-pre-styled-ui` の `input_group` mod が提供するスタイル済み Input Group
部品です。Root / Addon / Text / Button の 4 パーツ構成で、入力欄の前後にテキスト・アイコン・
ボタンの addon を配置します。Root がコンテナ側の枠線・角丸・`:focus-within` フォーカスリング
を所有し、内側の [Input](./input.md) / [Textarea](./textarea.md) は枠線なし・背景透明へ
リセットされます。`data-align`（`inline-start` / `inline-end` / `block-start` / `block-end`）
で addon の配置を切り替えられます。`size` / `variant` / `color-palette` いずれの軸も持たず、
寸法・文字サイズは内側の Input / Textarea に従属します。

ラベル・補助テキストは [Field](./field.md) が担います。`data-disabled` / `data-invalid` は
いずれも headless 層が出力する状態を CSS セレクタとして参照して見た目を切り替えるだけで、
値の妥当性判定・送信処理・addon クリックでのフォーカス移動といった処理はこの部品では
実装しません。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
