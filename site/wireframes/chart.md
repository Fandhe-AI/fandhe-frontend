# Chart

`fandhe-frontend-wireframe-ui` の棒グラフ配置イメージプレースホルダーです。
`<svg>`/`<canvas>` を一切使わず `div` の入れ子だけで組み立てる、非インタラ
クティブな表示専用部品です。API は `chart(values, orientation, size)` の
3 引数（詳細は下の引数表を参照）です。

実データを描画するグラフが必要な場合は
[Bar Chart（Themes）](../themes/bar-chart.md) や
[Charts（Themes）](../themes/charts.md) を検討してください。参照元
（blocks.pm）のスクリーンショットは掲載しません（ライセンス保留、
`docs/design/wireframe-ui-architecture.md` §2/§7 参照）。

## 原案差分メモ

- **API は独自設計**: `docs/design/wireframe-ui-architecture.md` §2/§7 に
  より、blocks.pm の Figma プロパティ（種別列挙・系列数など）は書き写して
  いません。§4/§6 の汎用変換規約と、クレート内の既存パターン（`progress`
  の 5 刻み量子化、`grid::MAX_COLUMNS` の資源有界化、`props::Orientation`
  の再利用）から独自に設計しました。
- **`props::Orientation` の再利用**: divider/stack/slider に続く 4 例目の
  消費者です。部品ローカルの `ChartKind` 等は新設していません。
- **5 刻み量子化**: 棒 1 本の値は 0〜100 へクランプしたのち 5 刻みへ量子化
  （四捨五入相当）し、固定 class（`fw-wire-chart-value-<q>`、21 種）を付与
  します。`style="--…: 42%"` のような属性値の動的組み立てや `data-value`
  の出力は行いません。
- **`MAX_BARS`（12）による資源有界化**: `grid::MAX_COLUMNS` と同じ値を
  採用し、超過分は先頭 12 本だけを描画します（panic しません）。
- **text 引数を持たない**: 軸ラベル・凡例・タイトルは発明していません。
  動的入力は `u8` 列だけで固定 class 集合へ量子化されるため、既定エスケー
  プ（REQ-1）の対象となる動的文字列は本部品に構造的に存在しません
  （`crates/wireframe-ui/tests/chart.rs` 冒頭コメント参照）。
- **SVG/canvas を使わない**: `div` の入れ子だけで anatomy を組み立てます
  （プロット領域 + 棒 N 本）。
- **グレースケール配色**: `ColorPalette` には依存せず、`--fw-wire-ink-muted`
  系の単色で棒を塗ります。
- **スコープ外**: 折れ線・面・円・散布などの種別、凡例、軸ラベル、タイト
  ル、複数系列（グループ化・積み上げ棒）は対象外です。
- **非対話**: `role`/`aria-*`/`tabindex`/`style`/`on*`/`<svg>`/`<canvas>` は
  一切出力しません（同文書 §7 の非対話制約）。
