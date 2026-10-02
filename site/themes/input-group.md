# Input Group

`fandhe-frontend-pre-styled-ui` の `input_group` mod が提供するスタイル済み Input Group
部品です。Root / Addon / Text / Button の 4 パーツ構成で、入力欄の前後にテキスト・アイコン・
ボタンの addon を配置します。Root がコンテナ側の枠線・角丸・`:focus-within` フォーカスリング
を所有し、内側の [Input](./input.md) / [Textarea](./textarea.md) / [NativeSelect](./native-select.md) /
[Select](./select.md) は枠線なし・背景透明へリセットされます（NativeSelect / Select は
`root` の直接の子として配置した場合のみリセットが効き、内容幅のままインライン配置されます）。
`data-align`（`inline-start` / `inline-end` / `block-start` / `block-end`）で addon の配置を
切り替えられます。`size` / `variant` / `color-palette` いずれの軸も持たず、寸法・文字サイズは
内側の Input / Textarea / NativeSelect / Select に従属します。

ラベル・補助テキストは [Field](./field.md) が担います。`data-disabled` / `data-invalid` は
いずれも headless 層が出力する状態を CSS セレクタとして参照して見た目を切り替えるだけで、
値の妥当性判定・送信処理・addon クリックでのフォーカス移動といった処理はこの部品では
実装しません。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
