# Menu

`fandhe-frontend-wireframe-ui` のドロップダウン風メニューのプレースホルダーです。
検索欄（任意）と、有効・無効が混在する項目リストからなるパネルの配置イメージを
示します。API は `menu(items, active, search, size)` の 4 引数（詳細は下の
引数表を参照）で、項目は `MenuItem::new`/`MenuItem::disabled` で構築します。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) の Menu 部品ですが、
ライセンス上の理由によりスクリーンショットは掲載していません（詳細は
`docs/design/wireframe-ui-architecture.md` §2）。参照したい場合は外部リンク先を
直接確認してください。

本部品は表示専用のため、`role="menu"`/`menuitem`・開閉・キーボードナビゲーションは
実装しません。実際に操作できるメニューが必要な場合は
[Menu（Themes）](../themes/menu.md)・[Menu（Primitives）](../primitives/menu.md)
を検討してください。

## 原案差分メモ

- **API は独自設計**: `docs/design/wireframe-ui-architecture.md` §6 と既存部品
  （`tabs` の `active: Option<usize>`、`nav_item` の `Option<&str>` 内部パート、
  `select` の固定パートによる指示子）から独立設計しました。同文書 §2 の保留
  （イシュー #2602）に従い、blocks.pm の `Option1`〜`Option5` 個別 bool+text
  スロット構成は参照・書き写していません。
- **項目はスライスへ畳む**: 固定 5 スロットの bool+text ではなく、公開の
  `MenuItem` 構造体（ラベル + 無効状態）のスライス `items: &[MenuItem]` で
  受けています。項目数に上限は設けていません。
- **強調状態は `Option<usize>`、無効項目を優先**: `tabs` と同じく「強調中は
  高々 1 件」を型で保証します。`active` が無効項目を指している場合は強調を
  付与しません（決定的で見た目の矛盾を避けるため）。範囲外の添字・`None` も
  同様に安全側へ倒します。
- **検索欄は `Option<&str>` + 固定アイコン**: bool ではなく検索行の
  プレースホルダー文言を持つ内部パートとしました。先頭には常に固定パートの
  `icon::search` を出力し、アイコンスロットにはしていません（`select` の
  ドロップダウン指示子と同じ判断: 利用者が省略・差し替えできない部品の
  同一性を担う要素のため）。`<input>` は出力しません。
- **アイコンスロットは非採用**: `docs/design/wireframe-ui-architecture.md`
  §11.4 は Menu をこの規約の適用先の例に挙げていますが、本イシューでは
  項目ごとのアイコン差し替えを対応範囲外としました。将来必要になった場合は
  別イシューで検討します。
- **`Orientation`/`Bold`/`Primary`/全体の `Disabled` は持たない**: イシューの
  要求範囲外です。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、上記の外部リンクのみで代替します。
- **非インタラクティブ**: `role="menu"`/`aria-expanded`/`aria-haspopup`・
  `tabindex`・実際の開閉は一切出力しません（同文書 §7 の非対話制約）。
