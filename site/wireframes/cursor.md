# Cursor

`fandhe-frontend-wireframe-ui` のマウスカーソル・プレースホルダーです。
矢印・手のひらの 2 種のグリフに、任意の名前タグを添えて示します。共同編集
カーソル（他利用者の名前チップ付きポインタ）のような配置イメージを静的に
示す用途を想定しています。API は `cursor(kind, label, size)` の 3 引数
（詳細は下の引数表を参照）で、props 構造体は導入していません。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) の Cursor 部品ですが、
ライセンス上の理由によりスクリーンショットは掲載していません（詳細は
`docs/design/wireframe-ui-architecture.md` §2）。参照したい場合は外部リンク先を
直接確認してください。

本部品は表示専用のため、実際にポインタ追従するカーソルとしては出力しません。
実際に追従する・hover でバリアントが変わるカスタムカーソルが必要な場合は
[Cursor hover cards（Blocks）](../blocks/cursor-hover-cards.md)（wasm-full の
`cursor` feature）を検討してください。

## 原案差分メモ

- **既存アイコンでは表現できないため新規アイコンを追加**: `link`
  （イシュー #2618）・`tag`（イシュー #2619）・`file_drop`（イシュー #2633）は
  `fandhe-frontend-wireframe-ui` の既存アイコンセットから呼び出し側が選んで
  渡す `Option<Node>` スロット規約を採用しましたが、本部品はグリフそのもの
  （矢印・手のひらの形）が部品の本体であり、代わりに使える既存アイコンが
  1 つもありませんでした。そのため `icon::cursor_arrow`/`icon::cursor_hand`
  の 2 種を新規追加し（`icon::ALL` へ登録、全体で 23 種）、`cursor` は
  `CursorKind` に応じて内部でどちらか一方を呼ぶ設計としました（`ratings`
  が `icon::star` を再利用するのと同型に、本部品は自分専用のグリフを
  アイコン基盤側へ持ちます）。
- **`CursorKind` は部品ローカルの列挙型**: `docs/design/wireframe-ui-architecture.md`
  §2 の保留（イシュー #2602）に従い、blocks.pm の Figma「Type」プロパティの
  値を転写していません。本イシューの要求範囲（矢印・手のひら）のみを持つ
  最小構成とし、`crate::props` へは昇格させていません（部品をまたいだ
  再利用が現時点で見えていないため）。クレートルートから再エクスポートする
  初めての部品ローカル列挙型です。
- **`Active`/`Disabled` は持たない**: hover 状態相当は `CursorKind::Hand` で
  表現できるため、`Active` 軸を追加すると意味が重複すると判断しました。
- **名前タグは `Node` スロットではなく `Option<&str>` の内部パート**:
  `nav_item` の `counter: Option<&str>` と同型の設計です。共同編集者名の
  チップという用途に対して `Node` スロットにする必要性が見えないため、
  最小構成としています。
- **配色はグレースケール反転**: グリフの塗り（`fill`）を `--fw-wire-paper`
  （白系）、輪郭（`stroke`）を `currentColor`（`--fw-wire-ink`）にすることで、
  線画のみのアイコン基盤（`icon::ICON_GLYPH_CSS`）にモノクロの立体感を
  足しています。`ColorPalette` には依存しません。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、上記の外部リンクのみで代替します。
