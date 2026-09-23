# Switch

`fandhe-frontend-wireframe-ui` のトグルスイッチ風プレースホルダーです。
楕円トラック + つまみ + 任意のラベルを持つ非インタラクティブな部品です。
API は `switch(label, size, active, disabled)` の 4 引数（詳細は下の引数表を
参照）で、props 構造体は導入していません。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) の Switch 部品ですが、
ライセンス上の理由によりスクリーンショットは掲載していません（詳細は
`docs/design/wireframe-ui-architecture.md` §2）。参照したい場合は外部リンク先を
直接確認してください。

「Switch」という名前ですが、`<input type="checkbox">`・`role="switch"`・
`aria-checked`・クリック操作のいずれも実装しない表示専用プレースホルダーです。
実際に操作可能なスイッチが必要な場合は
[Switch（Themes）](../themes/switch.md) /
[Switch（Primitives）](../primitives/switch.md) を検討してください。

## 原案差分メモ

- **API は独自設計**: `docs/design/wireframe-ui-architecture.md` §2 の保留
  （イシュー #2602）に従い、blocks.pm の Figma プロパティ構成を参照・
  書き写さず、`annotation` と同型の「bool + テキストを 1 スロットへ畳み込む」
  パターンから独立に設計しました。
- **ラベルは `Option<&str>` へ畳み込む**: Label(bool) + Text 相当の
  プロパティは `label: Option<&str>` 1 引数へ畳み込みます。`None` の
  ときはラベルのパート要素自体を出力しません（空要素を残さない）。
- **`Active` は ON 状態の意味**: 共通型 `Active` は
  [Select（Wireframes）](./select.md) 等ではフォーカス風の強調を表しますが、
  本部品では ON 状態そのものを表します（部品ごとに意味が異なる点に
  注意してください）。
- **`Disabled` は Forms A 家族との API 整合のための自己定義軸**: blocks.pm
  の Figma プロパティには存在しませんが、Phase 3「Forms A」の他部品
  （input/select 等）と状態軸を揃えるために併用します。
- **`<input>`・`role="switch"`・`aria-checked` は出力しない**: 非対話制約
  （`docs/design/wireframe-ui-architecture.md` §7）により、見た目上は
  操作可能に見えて実際には操作不能な UI を避けるため、実際に操作可能な
  スイッチは Themes/Primitives の Switch を案内します。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、上記の外部リンクのみで代替します。
- **配色はグレースケール反転**: 黒塗り二値ではなく `--fw-wire-ink` トークン
  1 段の反転で ON 状態を表現します。
