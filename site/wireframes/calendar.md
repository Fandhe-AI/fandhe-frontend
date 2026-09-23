# Calendar

`fandhe-frontend-wireframe-ui` の月表示グリッド型カレンダー部品です。
「ここに月表示の日付ピッカーがある」という配置イメージだけを示す
非インタラクティブな部品です。API は
`calendar(month_label, weeks, selected_day, size)` の 4 引数（詳細は
下の引数表を参照）で、props 構造体は導入していません。

blocks.pm に対応する部品はありません（本部品は `fandhe-frontend-wireframe-ui`
独自の追加部品です）。参照元となる外部サイトはないため、[blocks.pm](https://www.blocks.pm/)
そのものへのリンクのみ掲載します（詳細は `docs/design/wireframe-ui-architecture.md`
§2）。

前月/翌月の送り矢印・曜日ヘッダーはテキストなしの固定パートとして表示され、
月送り・キーボード操作・範囲選択のような実インタラクティブな挙動は
実装していません。実際に操作可能な日付ピッカーが必要な場合は
[Calendar（Themes）](../themes/calendar.md) /
[Calendar（Primitives）](../primitives/calendar.md) /
[Date Picker（Themes）](../themes/date-picker.md) /
[Date Picker（Primitives）](../primitives/date-picker.md) を検討してください。

## 原案差分メモ

- **blocks.pm 対応なしの独自追加部品**: イシュー本文の想定（月ラベル文字列・
  週ごとに 7 マスの日付配列・任意の選択日）をそのまま `month_label: &str` /
  `weeks: &[[Option<u32>; 7]]` / `selected_day: Option<u32>` として受け取り、
  全既存部品と揃えるため `size: Size` を 4 番目の引数として追加しました
  （専用 props 構造体は導入せず、`question`/`radio` と同じ位置引数方式）。
- **ルートは表示状態軸を持たない**: `Active`/`Disabled` はルートには付与しません
  （`question` の「ルートは class のみ」判断と同型）。表示状態は選択日セル
  だけが持ちます。
- **選択日は既存 `props::Active` を再利用**: 新規の `Selected`/`Checked` 型は
  新設せず、共有ファイル `props.rs` は触りません（`radio`/`checkbox` の
  先例と同じ判断軸）。各日セルは `Some(day) == selected_day` の一致判定で
  `data-active` を付与し、同じ日付値が複数セルにある入力では一致する
  **全セル**に付与します（単純・決定的な仕様、テストで固定）。
- **`weeks` は 6 週へ飽和**: 1 か月は最大 6 週にまたがるため、
  `MAX_WEEKS = 6` を超える入力は先頭 6 週のみを描画します
  （`textarea::MAX_ROWS`・`grid::MAX_COLUMNS` と同型の資源有界化、A05）。
  空スライスを渡した場合は曜日ヘッダーのみを描画し panic しません。
- **日付値は検証しない**: `Some(d)` は `u32::to_string()` でそのまま表示します。
  `0` や 32 以上のような妥当性検証はアプリケーションロジックの責務であり、
  UI コンポーネント層の責務外と判断しました（`docs/policy/intentional-non-adoption.md`
  §3.25 の判断軸と同型）。`None` は中身なしの空セル
  （`fw-wire-calendar-day-empty` 修飾）として出力し、7 列の配置を保ちます。
- **`<table>`/`<th>` は使わない**: データテーブル意味論を持ち込まず、
  `div`/`span` + CSS grid のみで組み立てます（[Grid](./grid.md) と同じ手法）。
- **曜日ヘッダーはテキストなしの固定パート**: 曜日ラベル引数はイシューに
  なく、ロケール依存文字列（「日」「Su」等）をハードコードしないため、
  7 個のテキストなしプレースホルダーを固定パートとして出力します。
- **前月/翌月の送り矢印は装飾アイコンの固定パート**: ヘッダーの月ラベル
  左右に `icon::caret_left`/`icon::caret_right` を配置しますが、`<button>`
  は出力しません（アイコン基盤が付与する装飾用の `aria-hidden="true"` の
  みが現れ、対話的 ARIA は出ません）。
- **`style` 属性・ネイティブ対話要素は一切出力しない**: `role`/`aria-*`
  （アイコンの `aria-hidden` を除く）/`tabindex`/`style`/`on*`/`<button>`/
  `<input>`/`<a href>` は出力しません
  （`docs/design/wireframe-ui-architecture.md` §7）。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
  本部品は blocks.pm 対応なしの独自追加部品でもあるため、そもそも参照
  スクリーンショットを持ちません。
- **配色はグレースケール**: `--fw-wire-ink`/`--fw-wire-ink-muted`/
  `--fw-wire-fill`/`--fw-wire-fill-subtle` トークンのみを使い、
  `ColorPalette` には依存しません。
