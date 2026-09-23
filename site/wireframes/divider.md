# Divider

`fandhe-frontend-wireframe-ui` の区切り線部品です。水平/垂直の線に、任意で
中央へラベルを置ける非インタラクティブなローファイ・プレースホルダーで、
セクション間・項目間の視覚的な区切りを表現する用途を想定しています。API は
`divider(label, size, orientation)` の 3 引数（詳細は下の引数表を参照）で、
props 構造体は導入していません。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) の Divider 部品ですが、
ライセンス上の理由によりスクリーンショットは掲載していません（詳細は
`docs/design/wireframe-ui-architecture.md` §2）。参照したい場合は外部リンク先を
直接確認してください。

Primitives・Themes には同名の "Divider" 部品はありません（本部品は
`fandhe-frontend-wireframe-ui` 固有です）。実際に操作可能・アクセシブルな
区切り線（`role="separator"` + `aria-orientation` 連動）が必要な場合は
[Separator（Themes）](../themes/separator.md) を検討してください。

## 原案差分メモ

- **API は独自設計**: blocks.pm の Figma プロパティ構成（`Size`/`Label`/
  `Vertical` 相当のトグル群）をそのまま転写せず、
  `docs/design/wireframe-ui-architecture.md` §6 の汎用変換規約（`Size` 軸 +
  方向 + 省略可能テキスト）から独立設計しました。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、上記の外部リンクのみで代替します。
- **`hr` は使わない**: 垂直・ラベル付きを表現できないうえ、variant によって
  タグが変わると「`role`/`data-`/`style` 不在」契約テストが複雑化するため、
  ルートは `div` + `::before`/`::after` 擬似要素で線を描く構成にしました。
  `role="separator"` も付与しません（`role`/`aria-*` を一切出力しない本
  クレートの方針、設計文書 §5/§7）。
- **`Size` の意味**: ラベルのフォントサイズと垂直方向の最小長さ
  （`--fw-wire-control-size`）に反映されますが、線の太さ
  （`--fw-wire-line-width`）は固定で `size` に連動しません。
- **非インタラクティブ**: `role`/`aria-*`/`tabindex` は一切付与せず、
  `button`/`a[href]` のような対話要素も出力しません（設計文書 §5/§7）。
