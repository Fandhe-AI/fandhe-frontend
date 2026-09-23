# Avatar

`fandhe-frontend-wireframe-ui` のアバタープレースホルダーです。正方形または
円形の枠の中に人物の線画（既定）を置き、実際のアバター画像の配置イメージを
示す非インタラクティブな部品です。API は `avatar(content, size, circle)` の
3 引数（詳細は下の引数表を参照）で、props 構造体は導入していません。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) の Avatar 部品ですが、
ライセンス上の理由によりスクリーンショットは掲載していません（詳細は
`docs/design/wireframe-ui-architecture.md` §2）。参照したい場合は外部リンク先を
直接確認してください。

本部品は表示専用のため `<img>` は出力せず、画像 URL を受け取る API も
持ちません。実際に画像を表示する部品が必要な場合は [Avatar（Themes）](../themes/avatar.md)
または [Avatar（Primitives）](../primitives/avatar.md) を検討してください。

## 原案差分メモ

- **API は独自設計**: `docs/design/wireframe-ui-architecture.md` §4・§6 の
  Figma プロパティ変換規約から独立設計しました。
- **サイズは 5 段階へ畳み込み**: 原案は XXL までの段階を想定していますが、
  本クレート共通の `Size` 5 段（xs〜xl）へ畳み込んでいます（他部品と
  同じ判断）。
- **`content: None` は人物線画へフォールバック（`Option<Node>` スロット
  規約 §11.4 からの意図的な逸脱）**: `crate::icon` の `Option<Node>`
  アイコンスロット規約は「`None` ならスロット要素を出力しない」を
  原則としますが、本部品は中身のないアバター枠がワイヤーフレームとして
  意味を持たないため、`content` が `None` のとき既定で人物線画
  （`icon::user`）を差し込みます。`Some(node)` を渡した場合はそのノードを
  そのまま子要素にします（画像アイコンやイニシャルテキストへの差し替えが
  可能です）。
- **円形は部品固有の修飾 class**: `circle: bool` は `crate::frame` の
  `bordered: bool` と同型の判断で、部品固有の修飾 class
  （`fw-wire-avatar-circle`）として表現します。`props.rs` に新しい型は
  追加していません。
- **`<img>` は出力しない**: ルートは `div` とし、`src`/`href`/`style` は
  一切出力しません（同文書 §7 の非対話制約）。画像 URL を受け取る API も
  設けていません（外部リソース読み込みの経路を作らないため）。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、上記の外部リンクのみで代替します。
- **配色はグレースケール**: 黒塗り二値ではなく `--fw-wire-fill-subtle`
  （背景）・`--fw-wire-ink-muted`（グリフ色）・`--fw-wire-line`（枠線）の
  トークンを使います。
