# Link

`fandhe-frontend-wireframe-ui` の下線テキストリンク風プレースホルダーです。
下線付きラベル + 任意の末尾アイコン（外部リンク等）で「リンクらしさ」だけを
表現する非インタラクティブな部品です。API は `link(label, trailing, size,
bold)` の 4 引数（詳細は下の引数表を参照）で、props 構造体は導入していません。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) の Link 部品ですが、
ライセンス上の理由によりスクリーンショットは掲載していません（詳細は
`docs/design/wireframe-ui-architecture.md` §2）。参照したい場合は外部リンク先を
直接確認してください。

本部品は表示専用のため実際にページ遷移するリンクは出力しません。実際に
操作可能なリンクが必要な場合は [Link（Themes）](../themes/link.md) または
[Link（Primitives）](../primitives/link.md) を検討してください。

## 原案差分メモ

- **API は独自設計**: `docs/design/wireframe-ui-architecture.md` §6 の
  汎用変換規約から独立設計しました。
- **アイコンスロットで表現**: 末尾アイコンは `trailing: Option<Node>` で
  受け取り、外部リンク表示は呼び出し側が `Some(icon::external(size))` を
  渡すことで表現します（同文書 §11.4 の `Option<Node>` アイコンスロット
  規約の実例）。専用の型や第 2 の bool 引数、`external_link` のような
  便宜ラッパは追加していません。
- **`a[href]` は出力しない**: ルートは `span` とし、`href`/`rel`/`target`
  は一切出力しません（同文書 §7 の非対話制約）。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、上記の外部リンクのみで代替します。
- **配色はグレースケール**: 黒塗り二値ではなく `--fw-wire-ink`（文字色）・
  `--fw-wire-line`（下線色）のトークンを使います。
