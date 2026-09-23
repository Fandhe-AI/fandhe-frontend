# Select

`fandhe-frontend-wireframe-ui` のドロップダウン選択欄風プレースホルダーです。
表示文言 + 任意の先頭アイコン + サイズ/強調/無効状態を持つ非インタラクティブな
部品です。API は `select(text_content, leading, size, active, disabled)` の
5 引数（詳細は下の引数表を参照）で、props 構造体は導入していません。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) の Select 部品ですが、
ライセンス上の理由によりスクリーンショットは掲載していません（詳細は
`docs/design/wireframe-ui-architecture.md` §2）。参照したい場合は外部リンク先を
直接確認してください。

「Select」という名前ですが、`<select>` 要素・開閉・リストボックス・
キーボード操作のいずれも実装しない表示専用プレースホルダーです。実際に
操作可能な select が必要な場合は
[Select（Themes）](../themes/select.md) /
[Select（Primitives）](../primitives/select.md) を検討してください。

## 原案差分メモ

- **`div` ルート・非対話制約**: `docs/design/wireframe-ui-architecture.md`
  §7 に従い、ルートは `div` とし `role`/`aria-*`/`tabindex`/`on*` は一切
  出力しません（アイコンが持つ `aria-hidden="true"` は装飾用途の既定属性で
  対話的 ARIA ではありません）。
- **アイコン有無は先頭アイコンスロットの有無に畳み込む**: `docs/design/
  wireframe-ui-architecture.md` §2 の保留（イシュー #2602）に従い、
  blocks.pm の Figma プロパティ構成を参照・書き写さず、`button`/`link` と
  同型の一般的な UI パターンから独立に設計しました。「アイコン付き」
  「テキストのみ」の区別に専用の列挙型は導入せず、`leading: Option<Node>`
  の `Some`/`None` へ畳み込みます。
- **ドロップダウン指示子は固定パート**: 末尾の指示子アイコン（⌄）は
  スロットではなく部品固有の固定パートとし、常に `icon::caret_down` を
  出力します（利用者が省略・差し替えできない部品の同一性を担う要素の
  ため）。
- **State は共通型 `Active`/`Disabled` へ畳み込む**: 専用の `SelectState`
  列挙は追加せず、フォーカス風の強調は `Active`、無効状態は `Disabled`
  （いずれも既存の共通型）で表現します。`true` のときそれぞれ
  `data-active=""`/`data-disabled=""` を付与します（見た目のみで、
  実際のフォーカス管理・操作不能を実装するものではありません）。
- **開いた状態（リストボックス表示）の variant は実装しない**: 非対話
  制約（`docs/design/wireframe-ui-architecture.md` §5/§7）により、開閉
  状態の表示専用再現は本イシューでは実装していません。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、上記の外部リンクのみで代替します。
- **配色はグレースケール**: 黒塗り二値ではなく `--fw-wire-*` トークン
  （`ink`/`ink-muted`/`line`/`fill-subtle`）で読みやすさを優先します。
