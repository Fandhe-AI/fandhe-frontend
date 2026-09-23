# Tabs

`fandhe-frontend-wireframe-ui` のタブ列風プレースホルダーです。
タブ項目の並び + 選択中インジケータの配置イメージだけを示す非インタラクティブな
部品です。API は `tabs(items, active, orientation, size)` の 4 引数（詳細は下の
引数表を参照）で、props 構造体は導入していません。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) の Tabs 部品ですが、
ライセンス上の理由によりスクリーンショットは掲載していません（詳細は
`docs/design/wireframe-ui-architecture.md` §2）。参照したい場合は外部リンク先を
直接確認してください。

「Tabs」という名前ですが、`role="tablist"`/`role="tab"`・`aria-selected`・実際の
パネル切り替えのいずれも実装しない表示専用プレースホルダーです。実際に操作可能な
タブが必要な場合は
[Tabs（Themes）](../themes/tabs.md) /
[Tabs（Primitives）](../primitives/tabs.md) を検討してください。

## 原案差分メモ

- **`div` ルート・非対話制約**: `docs/design/wireframe-ui-architecture.md`
  §7 に従い、ルートは `div` とし `role`/`aria-*`/`tabindex`/`on*` は一切
  出力しません。
- **タブラベルは固定スロットではなくスライスで受ける**: blocks.pm の Tabs
  部品が持つ Figma プロパティ構成（`Size`/`Active tab`/`Tab1`〜`Tab5` の
  個別 bool+text スロット）は §2 の保留（イシュー #2602）に従い書き写さず、
  §6 の汎用変換規約から独立に設計しました。`items: &[&str]` 1 引数へ畳み、
  項目数の上限は設けていません。
- **選択状態は `active: Option<usize>` 1 引数で表す**: 専用の
  `Selected`/`Checked` 型は新設せず、既存の共通型 `Active` は項目ごとの
  `data-active` 出力にのみ用います（`radio`/`checkbox`/`switch` と同じ
  再利用判断）。ただし引数自体は項目数分の `Active` スライスではなく
  「どの添字が選択中か」を表す `Option<usize>` とし、「選択中は高々 1 件」
  という不変条件を型で保証しています。`None` または範囲外の値は、どの
  項目にも選択インジケータを付けない決定的な挙動へ倒し、`unwrap`/
  `expect`/`panic` は使いません。
- **`Orientation` を必須引数に取る**: `props.rs` の doc が消費者として
  「stack / divider / tabs / slider」を明記しているため、水平/垂直を
  切り替え可能にしています。
- **`Disabled`/`Bold`/`Primary`・アイコンスロット・パネル領域は持たない**:
  タブ項目の無効化・強調・アイコン付与や、選択中パネルの内容表示は本部品の
  責務としていません。パネルが必要な場合は `Frame` 等の他部品との合成で
  表現してください。
- **黒塗り二値ではなくグレースケール**: 選択中項目は `--fw-wire-ink`・
  非選択項目は `--fw-wire-ink-muted`・選択中の背景は `--fw-wire-fill-subtle`
  を使い、読みやすさを優先します（他の wireframe-ui 部品と同じ配色方針）。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、上記の外部リンクのみで代替します。
