# Text

`fandhe-frontend-wireframe-ui` の単一行テキスト部品です。画面設計図中の見出し・
ラベル・短い文言を表す、非インタラクティブなローファイ部品で、複数行にわたる
ブロック本文を表現する [Paragraph（wireframe-ui）](./paragraph.md)とは、1 行固定
表示かどうかで役割が異なります。API は `text(content, size, bold)` の 3 引数
（詳細は下の引数表を参照）で、props 構造体は導入していません。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) の Text 部品ですが、
ライセンス上の理由によりスクリーンショットは掲載していません（詳細は
`docs/design/wireframe-ui-architecture.md` §2）。参照したい場合は外部リンク先を
直接確認してください。

スタイル済みの実部品が必要な場合は、[Text（Themes）](../themes/text.md)を
検討してください。

## 原案差分メモ

- **API は独自設計**: blocks.pm の Figma プロパティ構成をそのまま転写せず、
  `docs/design/wireframe-ui-architecture.md` §6 の Text/Paragraph 系共通
  規約（`Size` + `Bold`(bool) + `Text`(文字列) の 3 点セット）から独立設計
  しました（[Paragraph（wireframe-ui）](./paragraph.md)と同型の判断）。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、上記の外部リンクのみで代替します。
- **1 行固定 + 省略記号**: `content` に含まれる `\n` を改行として描画せず、
  `white-space: nowrap` + `overflow: hidden` + `text-overflow: ellipsis`
  のみで 1 行に固定表示します。幅を超える文言は末尾が省略記号で切り詰め
  られます（`white-space: pre-line` で複数行を許容する Paragraph とは
  対になる判断です）。
- **ルート要素は `<span>`**: docs サイト骨格 CSS（`.docs-content span`）には
  詳細度の競合が無いため、`.fw-wire-paragraph`（`<div>` ルート）とは異なり
  `<span>` をそのまま採用できます（インラインの 1 行要素であることを
  示す意図も兼ねます）。
- **`Bold` 軸のみで `Primary` は持たない**: Text/Paragraph 系は表示状態軸を
  持たず、`Bold` のみで強調を表します（[Annotation（wireframe-ui）](./annotation.md)が
  `Bold` を使わず `Primary` で強調を表すのとは対になる判断です）。
- **非インタラクティブ**: `role`/`aria-*`/`tabindex` は一切付与せず、
  `button`/`a[href]` のような対話要素も出力しません（設計文書 §5/§7）。
