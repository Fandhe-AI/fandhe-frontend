# Nav item

`fandhe-frontend-wireframe-ui` のナビゲーション項目 1 行分のプレースホルダーです。
先頭アイコン + ラベル + 件数表示（カウンター）+ 任意の末尾アイコンを幅いっぱいの
1 行（または縦積み）で並べます。API は `nav_item(label, leading, trailing, counter,
size, active, orientation)` の 7 引数（詳細は下の引数表を参照）で、props 構造体は
導入していません。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) の Nav item 部品ですが、
ライセンス上の理由によりスクリーンショットは掲載していません（詳細は
`docs/design/wireframe-ui-architecture.md` §2）。参照したい場合は外部リンク先を
直接確認してください。

本部品は表示専用のため、実際に遷移するリンクとしては出力しません。実際に操作
できるナビゲーションが必要な場合は [Nav list（Themes）](../themes/nav-list.md)・
[Navigation menu（Themes）](../themes/navigation-menu.md)・
[Nav list（Primitives）](../primitives/nav-list.md) を検討してください。

## 原案差分メモ

- **API は独自設計**: `docs/design/wireframe-ui-architecture.md` §6/§11.4 と
  既存部品（`rich_text` のスロット + ラベル + `Orientation`、`input` の
  `Active` `.attr()` 消費形）から独立設計しました。同文書 §2 の保留
  （イシュー #2602）に従い、blocks.pm の Figma プロパティ構成を参照・
  書き写していません。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、上記の外部リンクのみで代替します。
- **アクティブ状態はグレースケール反転**: 黒塗りバーの原案イメージは
  `data-active` による背景・文字色の反転（`--fw-wire-ink`/`--fw-wire-paper`）
  で表現しました。アイコンは `currentColor` を使うため自動で追従します。
  反転時は件数表示（カウンター）の配色も可読性のため明るいトークンへ切り替え
  ます（`.fw-wire-nav-item[data-active] .fw-wire-nav-item-counter`）。
- **件数表示は `Node` スロットではなく内部パート**: `counter: Option<&str>`
  として持ち、専用の `Node` スロットにはしていません。Phase 7 に単独部品
  `counter`（#2655）が予定されていますが、本イシュー時点では未実装のため
  依存できないこと、ナビ行内の件数ピルはアクティブ時の反転配色を含む行
  レイアウトと一体であることの 2 点が理由です。
- **`Disabled` は持たない**: イシューの要求範囲外であり、引数がちょうど 7 個
  （`too_many_arguments` の上限は 8 個以上）に収まるようにしています。
- **`rich_text` との違い**: アクティブ状態・件数表示・幅いっぱいのサイドバー
  行レイアウトを持つ点が異なります。
- **非インタラクティブ**: `<a href>` にはしません（同文書 §7 の非対話制約）。
  `role`/`aria-*`/`tabindex` も一切出力しません。
