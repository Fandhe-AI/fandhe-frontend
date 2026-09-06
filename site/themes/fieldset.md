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

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
