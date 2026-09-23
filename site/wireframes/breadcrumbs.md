# Breadcrumbs

`fandhe-frontend-wireframe-ui` のパンくずリスト風プレースホルダーです。
上位階層から現在ページへ至る経路の配置イメージだけを示す非インタラクティブな
部品です。API は `breadcrumbs(items, size)` の 2 引数（詳細は下の引数表を
参照）で、props 構造体は導入していません。

視覚的な参照元は [blocks.pm](https://www.blocks.pm/) の Breadcrumbs 部品
ですが、ライセンス上の理由によりスクリーンショットは掲載していません
（詳細は `docs/design/wireframe-ui-architecture.md` §2）。参照したい場合は
外部リンク先を直接確認してください。

「Breadcrumbs」という名前ですが、`nav`/`ol`/`li`/`a[href]`/`aria-current`の
いずれも実装しない表示専用プレースホルダーです。実際に操作可能なパンくずが
必要な場合は
[Breadcrumb（Themes）](../themes/breadcrumb.md) /
[Breadcrumb（Primitives）](../primitives/breadcrumb.md) を検討してください。

## 原案差分メモ

- **`div` ルート・非対話制約**: `docs/design/wireframe-ui-architecture.md`
  §7 に従い、ルートは `div` とし `role`/`aria-*`/`tabindex`/`on*`/`href` は
  一切出力しません。
- **階層ラベルは固定スロットではなくスライスで受ける**: blocks.pm の
  Breadcrumbs 部品が持つ Figma プロパティ構成（`Size`/`Levels`/`Level text`
  ×N の個別スロット）は §2 の保留（イシュー #2602）に従い書き写さず、§6
  の汎用変換規約から独立に設計しました。`items: &[&str]` 1 引数へ畳み、
  項目数の上限は設けていません。
- **現在階層（選択状態）は引数を持たず、常に最後の項目へ自動付与する**:
  `tabs` の `active: Option<usize>` とは異なり、選択専用の引数は設けて
  いません。パンくずは定義上「現在ページで終わる経路」であり、現在項目は
  常に最後の要素であるという不変条件を利用者に選ばせず構造で保証するため、
  `items` が空でない限り最後の項目へ常に既存の共通型 `Active`
  （`data-active`）を付与します。新規の `Current` 等の型は新設していません
  （`radio`/`tabs` の再利用判断と同じ）。
- **区切り記号は CSS 擬似要素のみで描く**: `stepper` の連結線（`::before`）
  と同じ先例です。DOM へ区切りノード・テキストを一切出力しません
  （`icon::caret_right` 等の SVG アイコンは `aria-hidden` 付き SVG を
  出力し非対話テストを複雑にするため不採用にしました）。
- **`Orientation`・`Disabled`/`Bold`/`Primary`・先頭のホームアイコン
  `Node` スロット・中間階層の省略（「…」折りたたみ）は持たない**:
  パンくずは水平専用とし、これらの軸は本部品の責務としていません。必要な
  場合は `rich_text`/`icon` 等の他部品との合成で表現してください。
- **黒塗り二値ではなくグレースケール**: 現在項目は `--fw-wire-ink`・
  非現在項目は `--fw-wire-ink-muted`・区切りは `--fw-wire-line` 系グレーを
  使い、読みやすさを優先します（他の wireframe-ui 部品と同じ配色方針）。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本ページも画像は置かず、上記の外部リンクのみで代替します。
