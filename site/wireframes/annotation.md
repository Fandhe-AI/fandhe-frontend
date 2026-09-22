# Annotation

`fandhe-frontend-wireframe-ui` の注釈ボックス部品です。太字タイトル + 任意の
説明文を持つ非インタラクティブなローファイ・プレースホルダーで、画面設計図の
余白に設計意図を書き込む用途を想定しています。API は `annotation(title,
description, size, primary)` の 4 引数（詳細は下の引数表を参照）で、
props 構造体は導入していません。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) の Annotation 部品ですが、
ライセンス上の理由によりスクリーンショットは掲載していません（詳細は
`docs/design/wireframe-ui-architecture.md` §2）。参照したい場合は外部リンク先を
直接確認してください。

Primitives・Themes には同名の "Annotation" 部品はありません（本部品は
`fandhe-frontend-wireframe-ui` 固有です）。実際に操作可能な注釈・ツールチップが
必要な場合は [Tooltip（Primitives）](../primitives/tooltip.md) を検討してください。

## 原案差分メモ

- **API は独自設計**: blocks.pm の Figma プロパティ構成（`Type`/`Emphasis`
  相当のトグル群）をそのまま転写せず、
  `docs/design/wireframe-ui-architecture.md` §6 の汎用変換規約（`Size` 軸 +
  強調 bool + テキスト + 省略可能テキスト）から独立設計しました。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、上記の外部リンクのみで代替します。
- **配色は黒塗り二値ではなく反転**: blocks.pm の原案に相当する黒塗り
  プレースホルダーを機械的に再現するのではなく、`Primary(true)` 指定時のみ
  背景・文字色を反転するグレースケール配色を採用しています（
  `docs/design/wireframe-ui-architecture.md` §3「黒塗り二値を強制せず
  読みやすさ優先」の判断軸）。反転時の説明文には `--fw-wire-ink-muted` では
  なく `--fw-wire-fill`（明るいトークン）を使い、暗地上でのコントラスト
  不足を避けています。
- **非インタラクティブ**: `role`/`aria-*`/`tabindex` は一切付与せず、
  `button`/`a[href]` のような対話要素も出力しません（設計文書 §5/§7）。
