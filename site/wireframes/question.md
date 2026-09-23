# Question

`fandhe-frontend-wireframe-ui` の質問項目風プレースホルダーです。
ラベル + 任意の補足説明 + フォームコントロール + 任意のヒントという
「1 つの質問項目」の配置イメージだけを示す非インタラクティブな部品です。
API は `question(label, description, control, hint, size)` の 5 引数
（詳細は下の引数表を参照）で、props 構造体は導入していません。
`control` は既存の wireframe-ui 部品（`select`/`switch` 等）の戻り値を
そのまま渡す `Node` スロットです。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) の Question 部品ですが、
ライセンス上の理由によりスクリーンショットは掲載していません（詳細は
`docs/design/wireframe-ui-architecture.md` §2）。参照したい場合は外部リンク先を
直接確認してください。

ラベルは `<label>` ではなく `div` で出力し、`<input>`/`<fieldset>`/`<legend>`
のいずれも実装しない表示専用プレースホルダーです。実際に操作可能な
質問項目が必要な場合は
[Field（Themes）](../themes/field.md) /
[Field（Primitives）](../primitives/field.md) を検討してください。

## 原案差分メモ

- **API は独自設計**: `docs/design/wireframe-ui-architecture.md` §2 の保留
  （イシュー #2602）に従い、blocks.pm の Figma プロパティ構成を参照・
  書き写さず、`docs/design/wireframe-ui-architecture.md` §6 の汎用変換規約
  （`Size` 軸 + テキスト + 省略可能テキスト + スロット）から独立に設計
  しました。
- **コントロールは `Node` スロットで受ける**: 「Input short / …」のような
  コントロール種別は専用の variant を持たず、`select`/`switch` 等の
  wireframe-ui 部品の戻り値をそのまま渡す `control: Node` 引数へ委ねます
  （設計文書 §6「instance swap は `Node` スロット引数で受ける」の典型）。
  `question` 自身がテキストフィールド anatomy を内蔵する案は、Forms A の
  任意部品を差し込める柔軟性を失うため採りませんでした。
- **表示状態軸を持たない**: `Bold`/`Primary`/`Active`/`Disabled` はいずれも
  使いません。ラベルは常に太字で、表示状態はスロットに渡すコントロール側
  （`select(..., Active(true), ...)` 等）が担います。`question` ルートには
  `data-*` を一切付与しません。
- **`size` は呼び出し側がコントロールにも渡す規約**: `question` の `size`
  はラベル/説明/ヒントのフォントサイズにのみ効きます。コントロールの
  高さ・フォントサイズは制御しないため、同じ `size` を `control` 側にも
  渡すことを推奨します。
- **`<label>`・`<input>`・`<fieldset>`・`<legend>` は出力しない**: 非対話
  制約（`docs/design/wireframe-ui-architecture.md` §7）により、`for` 関連
  付けのない `<label>` も含め対話要素風の意味論に踏み込む要素は一切
  出力しません。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、上記の外部リンクのみで代替します。
- **配色はグレースケール**: `--fw-wire-ink`/`--fw-wire-ink-muted` トークンの
  みを使い、`ColorPalette` には依存しません。
