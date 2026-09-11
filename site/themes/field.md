# Field

`fandhe-frontend-pre-styled-ui` の `field` mod が提供するスタイル済み Field 部品です。
Root / Label / HelperText / ErrorText / RequiredIndicator の 5 パーツに加え、Group /
Content / Title / Separator（内部パーツ SeparatorLine / SeparatorContent）の 6 パーツ
（イシュー #2185 で純追加、shadcn/ui FieldGroup / FieldContent / FieldTitle /
FieldSeparator 相当）の計 11 パーツ構成で、ラベル・補助テキスト・エラーテキスト・
必須マークの型階層と `root` の余白レイアウトを提供します。配置軸は `orientation`
（既定 `vertical` の縦積み、`horizontal` の横並び、`responsive` の 3 値）のみを
持ちます。`responsive`（イシュー #2199）は Group（container）の内側に置いた場合
にのみ意味を持ち、Group の inline サイズが 448px 以上のときだけ `horizontal` と
同じ横並びへ切り替わります。Group の外に置いた場合は常に縦積みのままです。

Group は複数の Field（Root）を縦に束ねる外側コンテナ、Separator は線のみ／テキスト付き
の区切り線です。Content / Title は `<label for>` を結び付けにくい場面（複数コントロール
の見出し等）で Label の代替として使う見出し + 補助テキストの列で、Title は `for`/`id`
を自動導出しないため、呼び出し側が Title の `id` を対応するコントロールの
`aria-labelledby` へ渡して結び付けます。

コントロール（input/textarea/select）は本部品が所有せず、[Input](./input.md) /
[Textarea](./textarea.md) / [Native Select](./native-select.md) の各部品が同じ
headless-ui `field` scope を共有して提供します。`data-invalid` / `data-disabled` /
`data-required` / `data-readonly` はいずれも headless 層が出力する状態を CSS セレクタ
として参照して見た目を切り替えるだけで、値の妥当性判定・送信処理といったバリデーション
自体はこの部品では実装しません。

`data-invalid` が立っているとき、Label のテキスト色もエラー色（`--fandhe-color-danger`）へ
切り替わります。エラー内容は ErrorText のテキストで別途伝わるため、色のみに依存した
表示にはなりません。

ErrorText に単一のテキストではなく `<ul>`（各エラーを `<li>` で列挙）を子要素として渡すと、
複数エラーメッセージ向けのリスト整形（縦積み・字下げ・disc マーカー）が適用されます。
`<ul>`/`<li>` の組み立てや重複排除自体はこの部品の責務ではなく、呼び出し側が
`fandhe_frontend_core::el` / `text` で構築して渡してください。

関連 API: [fandhe-frontend-pre-styled-ui API](../../docs/api/pre-styled-ui-api.md) / [fandhe-frontend-headless-ui API](../../docs/api/headless-ui-api.md)
