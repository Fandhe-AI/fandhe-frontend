# Table

`fandhe-frontend-wireframe-ui` の N 列 × M 行のデータ表配置部品です。
「ここにデータ表がある」という配置イメージだけを示す非インタラクティブな
部品です。API は `table(headers, rows, size)` の 3 引数（詳細は下の引数表を
参照）で、props 構造体は導入していません。

視覚的な参照元は [Table（blocks.pm）](https://www.blocks.pm/tags/table) です。
ライセンス上の理由により、本ページには blocks.pm のスクリーンショットや
外観・プロパティ構成をそのまま取り込みません（詳細は
`docs/design/wireframe-ui-architecture.md` §2）。参照したい場合は外部リンク
先を直接確認してください。

実際のデータ表示・並べ替え・行選択のような機能が必要な場合は、
[Table（Themes）](../themes/table.md) / [Data table（Themes）](../themes/data-table.md) /
[Data table（Primitives）](../primitives/data-table.md) を検討してください。

## 原案差分メモ

- **`<table>`/`<th>` は使わない**: [Calendar](./calendar.md) の先例
  （「データテーブル意味論を持ち込まず [Grid](./grid.md) と同じ手法を使う」）
  に揃え、`div`/`span` + CSS grid で表現します。プレースホルダーの文言を
  支援技術へ「データ表」として伝えないための一貫した判断です。
- **ヘッダー有無は bool ではなくスライスの空判定で表す**: 原案の
  Header(bool) は `headers.is_empty()` へ畳みました。空ならヘッダー行を
  出力しません。別に bool を持つと「`header=true` なのに `headers=[]`」
  という矛盾した入力を生むため採りませんでした。
- **列数は明示引数にしない**: `headers`/`rows` の形から導きます
  （`headers.len()` と各行長の最大値のうち大きい方を `1..=12` へ丸める）。
  原案の「Columns #」に相当する引数は持ちません。
- **行数は明示引数にしない**: `rows.len()` をそのまま使い、
  `MAX_TABLE_ROWS`（20 行）を超える入力は先頭のみを描画します
  （A05 資源有界化、`textarea::MAX_ROWS`/`calendar::MAX_WEEKS` と同型）。
  原案の「Rows #」に相当する引数は持ちません。
- **短い行は空セルで埋める**: 列数より短いヘッダー行・データ行は、
  不足分を空セル（`fw-wire-table-cell-empty`。[Calendar](./calendar.md) の
  `day-empty` と同じ作り）で埋めます。列数より長い行は先頭の列数分だけを
  使います。こうして全行が同じセル数になり、見た目の列が崩れません。
- **セル種別は持たない**: 原案の「セル種別」（テキスト/バー状
  プレースホルダー/アバターやチェックボックスの差し込み等）は採らず、
  全セルをテキストのみとしました。利用実績が出た時点で拡張を検討します。
- **配色はグレースケール**: 原案の黒塗りヘッダーは
  `--fw-wire-fill-subtle`（グレースケール）へ調整しました。
  `--fw-wire-ink`/`--fw-wire-line`/`--fw-wire-line-subtle` トークンのみを
  使い、`ColorPalette` には依存しません。
- **`style` 属性・ネイティブ対話要素は一切出力しない**: `role`/`aria-*`/
  `tabindex`/`style`/`on*`/`<table>`/`<th>`/`<button>`/`<input>`/
  `<select>`/`<a href>` は出力しません
  （`docs/design/wireframe-ui-architecture.md` §7）。ルートは `class` の
  みを持ちます（`question`/`calendar` の「ルートは class のみ」判断と同型）。
- **スクリーンショット非掲載**: `docs/design/reference-screenshots/` への
  blocks.pm 画像取り込みは #2602 で fail-closed に非掲載と確定しています。
