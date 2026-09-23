# Rich text

`fandhe-frontend-wireframe-ui` のリッチテキスト行部品です。アイコン + ラベル +
末尾アイコンを 1 行（または縦積み）で並べる非インタラクティブなローファイ・
プレースホルダーで、「アイコン付き行」の配置イメージ提示を想定しています。
API は `rich_text(label, leading, trailing, size, bold, orientation)` の
6 引数（詳細は下の引数表を参照）で、props 構造体は導入していません。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) の Rich text 部品ですが、
ライセンス上の理由によりスクリーンショットは掲載していません（詳細は
`docs/design/wireframe-ui-architecture.md` §2）。参照したい場合は外部リンク先を
直接確認してください。

Primitives・Themes には同名の "Rich text" 部品はありません（本部品は
`fandhe-frontend-wireframe-ui` 固有です）。実際に操作可能なリンク行・ボタンが
必要な場合は [Link（Themes）](../themes/link.md) や [Button（Themes）](../themes/button.md)
を検討してください。

## 原案差分メモ

- **API は独自設計**: blocks.pm の Figma プロパティ構成をそのまま転写せず、
  `docs/design/wireframe-ui-architecture.md` §6 の Text/Paragraph 系変換規約
  （`Size` + `Bold` + テキストの 3 点セット）と §11.4 の `leading`/`trailing`
  `Option<Node>` スロット規約を組み合わせて独立設計しました。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、上記の外部リンクのみで代替します。
- **`Orientation` は既存の共通型**: stack/divider/tabs と共用する
  `props::Orientation`（`Horizontal`/`Vertical`）をそのまま流用しており、
  blocks.pm 固有のプロパティ名を転写したものではありません。
- **`Bold`/`Orientation` の CSS は本部品スコープで初めて宣言**: 両 class は
  `props.rs` が class 名のみを定義していましたが、横断 CSS（`tokens::css`/
  `size::css`/`css::PARTS`）に対応する宣言が存在しませんでした（Annotation
  は `Bold` を意図的に不採用だったため未消費）。本部品が両 class の最初の
  消費者となり、`docs/design/wireframe-ui-architecture.md` §10「部品 CSS は
  自分のルート配下のみ」に従い部品スコープで宣言しています。
- **非インタラクティブ**: `role`/`aria-*`/`tabindex` は一切付与せず、
  `button`/`a[href]` のような対話要素も出力しません。末尾アイコンは装飾で
  あり、遷移を意味しません（設計文書 §5/§7）。
