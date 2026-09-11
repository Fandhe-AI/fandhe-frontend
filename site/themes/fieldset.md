# Fieldset

`fandhe-frontend-pre-styled-ui` の `fieldset` mod が提供するスタイル済み Fieldset 部品です。
Root / Legend / HelperText / ErrorText の 4 パーツ構成で、複数の Field をネイティブ
`<fieldset>` / `<legend>` でグループ化します。UA 既定の `<fieldset>` / `<legend>` 枠線・
padding をリセットしたうえで、`size`（`sm` / `md` / `lg`、既定 `md`）軸のみによる余白・
文字サイズの段階化を提供します（`orientation` / `colorPalette` 軸は持ちません）。

内側の各 Field（ラベル・入力欄・補助テキスト等）は本部品が所有せず、[Field](./field.md) /
[Input](./input.md) 等の各部品がそのまま担います。`data-disabled` / `data-invalid` は
いずれも headless 層が出力する状態を CSS セレクタとして参照して見た目を切り替えるだけで、
値の妥当性判定・送信処理といったバリデーション自体はこの部品では実装しません。

Legend は `legend_with_variant` で `legend`（既定・大見出し）/ `label`（`size` 軸の
1 段下、小見出し）の 2 段見出しサイズを選べます（shadcn/ui `FieldLegend` の `variant`
prop と突合、イシュー #2214）。既存の `legend` は `data-variant` を出力しない契約のまま
不変です。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
