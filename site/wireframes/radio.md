# Radio

`fandhe-frontend-wireframe-ui` のラジオボタン風プレースホルダーです。
円形コントロール + 任意のラベル + サイズ/選択/無効状態を持つ非インタラクティブな
部品です。API は `radio(label, size, active, disabled)` の 4 引数（詳細は下の
引数表を参照）で、props 構造体は導入していません。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) の Radio 部品ですが、
ライセンス上の理由によりスクリーンショットは掲載していません（詳細は
`docs/design/wireframe-ui-architecture.md` §2）。参照したい場合は外部リンク先を
直接確認してください。

「Radio」という名前ですが、`<input type="radio">` 要素・`role="radio"`・選択状態の
遷移のいずれも実装しない表示専用プレースホルダーです。実際に操作可能なラジオが
必要な場合は
[Radio Group（Themes）](../themes/radio-group.md) /
[Radio Group（Primitives）](../primitives/radio-group.md) を検討してください。

## 原案差分メモ

- **`div` ルート・非対話制約**: `docs/design/wireframe-ui-architecture.md`
  §7 に従い、ルートは `div` とし `role`/`aria-*`/`tabindex`/`on*` は一切
  出力しません。
- **Label(bool) + Text は `Option<&str>` へ畳み込む**: `docs/design/
  wireframe-ui-architecture.md` §2 の保留（イシュー #2602）に従い、
  blocks.pm の Figma プロパティ構成（`Size`/`Active`/`Label(bool)`/`Text`）を
  参照・書き写さず、§6 の汎用変換規約と Forms A 既存部品（`button`/
  `select`）の先例から独立に設計しました。「ラベルあり」「ラベルなし」の
  区別に専用の bool 引数は導入せず、`label: Option<&str>` の `Some`/`None`
  へ畳み込みます（`None` のときラベルパート要素自体を出力しません）。
- **選択状態は既存の共通型 `Active` を再利用する**: 専用の `Selected`/
  `Checked` 型は新設していません。`props.rs` は checkbox（イシュー #2625）・
  switch（イシュー #2627）も同じ共有ファイルとして触るため、選択状態を表す
  共通型の統一（`data-selected` 等への一本化）はこれらの部品が揃ってから
  別途判断する対象として本イシューでは意図的にスコープ外としています。
  `true` のとき `data-active=""` を付与し、内側の黒丸（選択済み表現）を
  `::after` の CSS で描きます。
- **`Disabled` は自己定義で追加**: Forms 部品として整合させるため、
  イシューのプロパティ列挙にはない `Disabled`（`docs/design/
  wireframe-ui-architecture.md` §5 の表示状態軸）を追加しました。
- **黒丸は `--fw-wire-ink` トークン経由**: 黒塗り二値ではなく
  `--fw-wire-*` トークンで読みやすさを優先します（他の wireframe-ui 部品と
  同じ配色方針）。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、上記の外部リンクのみで代替します。
- **グループ化・排他選択はモデル化しない**: 本部品は単体 1 個のみを表現し、
  複数項目の縦横並び・排他表現を行う「ラジオグループ」部品は実装していません
  （Demo は複数個の `radio` を `div` で並べて配置イメージのみを示します）。
