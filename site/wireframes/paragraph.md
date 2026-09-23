# Paragraph

`fandhe-frontend-wireframe-ui` の複数行本文部品です。画面設計図中の本文
プレースホルダーを表す、非インタラクティブなローファイ部品で、単一行の
見出し/ラベル相当（Phase 2「テキスト・注釈」の `text`）とは異なり複数行に
わたるブロック本文を表現します。API は `paragraph(content, size, bold)` の
3 引数（詳細は下の引数表を参照）で、props 構造体は導入していません。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) の Paragraph 部品ですが、
ライセンス上の理由によりスクリーンショットは掲載していません（詳細は
`docs/design/wireframe-ui-architecture.md` §2）。参照したい場合は外部リンク先を
直接確認してください。

単一行の見出し/ラベル相当を表す `text`（Phase 2「テキスト・注釈」で別途
実装、イシュー #2614）とは、単一行か複数行のブロック本文かで役割が
異なります。スタイル済みの実部品が必要な場合は、
[Text（Themes）](../themes/text.md)を検討してください。

## 原案差分メモ

- **API は独自設計**: blocks.pm の Figma プロパティ構成をそのまま転写せず、
  `docs/design/wireframe-ui-architecture.md` §6 の Text/Paragraph 系共通
  規約（`Size` + `Bold`(bool) + `Text`(文字列) の 3 点セット）から独立設計
  しました。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、上記の外部リンクのみで代替します。
- **複数行は CSS 表現のみ**: `content` に含まれる `\n` は `<br>` へ変換せず、
  `white-space: pre-line`（CSS）のみで改行として描画します。`raw_html` は
  使わず、テキスト → マークアップの変換経路自体を作らない判断です
  （既定エスケープ REQ-1 の議論を持ち込まないための設計判断）。
- **ルート要素は `<p>` ではなく `<div>`**: docs サイト骨格 CSS
  （`.docs-content p`）の詳細度に `.fw-wire-paragraph` が負け `Size` 軸の
  フォントサイズ差が Demo 上で消えるため、`<div>` ルートを採用しています
  （[Annotation（wireframe-ui）](./annotation.md)も同じ理由で `<div>` ルート）。
- **`Bold` 軸のみで `Primary` は持たない**: Text/Paragraph 系は表示状態軸を
  持たず、`Bold` のみで強調を表します（Annotation が `Bold` を使わず
  `Primary` で強調を表すのとは対になる判断です）。
- **非インタラクティブ**: `role`/`aria-*`/`tabindex` は一切付与せず、
  `button`/`a[href]` のような対話要素も出力しません（設計文書 §5/§7）。
