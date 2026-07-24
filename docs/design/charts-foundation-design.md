# charts 基盤設計（イシュー #846）

## 1. 背景

chakra-ui の charts 群（16 項目、`docs/design/component-coverage-map.md` の
`.agents/skills/chakra-ui/references/charts/` 節）は
`docs/policy/intentional-non-adoption.md` §7 で「保留」区分だった。理由は、
recharts（chakra-ui charts が内部で使う React チャートライブラリ）が
外部 JS ランタイム・追加依存を前提にしており、本フレームワークの
外部依存ゼロ方針（`core` は外部依存ゼロ、`headless-ui`/`pre-styled-ui` も
最小依存、REQ-3）とそのままでは両立しないためである。

保留解除トリガーは「外部依存ゼロを維持したまま SVG ノード木生成のみで
実装できる設計の確立」。親 Phase #845 はこのトリガーを充足する基盤（座標
スケーリング・SVG ノード木生成・`ChartData` モデル）を先行実装し、
配置先判断を本文書に記録することを決定した。個々のチャート部品
（Area/Bar/Line/Pie 等）・軸/グリッド/凡例/ツールチップは後続イシュー
（#847〜#851）でこの基盤の上に実装する。

## 2. 配置先判断

**新クレートは作らず、`crates/pre-styled-ui/src/charts/`（ディレクトリ
モジュール）として実装した。**

理由:

1. **依存グラフ上限（REQ-3、60 件/深さ 6）への影響ゼロ**: 新クレートを
   ワークスペースへ追加すると、`fw new` 生成プロジェクトの標準構成
   （`crates/xtask` の依存グラフ計測対象）に新たな member を加えることに
   なり、計測・保守対象が増える。既存 `pre-styled-ui` 配下のモジュール
   追加であれば、依存グラフの形（メンバー数・エッジ数）自体は変化しない。
2. **`theme`/`recipe`/`css` 基盤の再利用**: 系列配色（[`series_color_var`]）
   は `theme.rs` の色トークン基盤（`DEFAULT_COLORS`・`color_var`）にそのまま
   乗せられる。新クレートに分離すると、この基盤へのアクセスに
   `pre-styled-ui` への逆依存が必要になり、依存方向が複雑化する。
3. **`pre-styled-ui` の不変条件をそのまま継承できる**: 外部依存は
   `fandhe-frontend-headless-ui` のみ・`#![forbid(unsafe_code)]`・
   `raw_html()` 不使用という既存クレートの不変条件（`crates/pre-styled-ui/src/lib.rs`
   クレート doc）を、新クレートで再定義することなくそのまま満たせる。

### 再検討トリガー（別クレート化を再評価する条件）

以下のいずれかが恒常的に成立した場合、`fandhe-frontend-charts`（仮称）への
分離を再評価する。

- charts 関連コードの追加により `pre-styled-ui` 単体のフルビルド時間が
  他クレートに比べ明確に増大し、開発体験（AI 開発・保守を含む）を損なう。
- charts の公開 API 表面（型・関数の総数）が既存の非 charts 部品群と
  同程度以上に肥大化し、単一クレートの責務境界としての一貫性を欠く。
- charts 部品が `headless-ui`/`pre-styled-ui` のいずれにも属さない
  独自の anatomy 抽象（例: 座標系を跨ぐ複合コンポーネント基盤）を要求し、
  既存 2 層構造（headless/pre-styled）に収まらなくなる。

## 3. API 方針（useChart → ChartData への吸収対応）

chakra-ui `charts/use-chart.md` の `useChart`（data/series/集計/フォーマット
を束ねる React hook）は、JS ランタイムを持たない本フレームワークでは
「明示的な Rust 構造体 + 決定的純関数」として吸収する。

| chakra-ui `useChart` | 本実装 |
|---|---|
| `chart.data` | [`ChartData::categories`]/[`ChartData::series`] |
| `getTotal` | [`data::total`] |
| `getMin` | [`data::min`] |
| `getMax` | [`data::max`] |
| `getValuePercent` | [`data::value_percent`] |
| ソート（`sort` オプション） | [`ChartData::sort_by_series`]（安定ソート、方向は [`data::SortDirection`]） |
| 色パレット（`chart.color(name)`） | [`charts::series_color_var`]（`theme.rs` の `chart-1`〜`chart-6` トークンを循環参照） |

`charts/installation.md`（recharts 導入手順）は「追加依存なし
（`fandhe-frontend-pre-styled-ui` のみ、`cargo add fandhe-frontend-pre-styled-ui`
のみで完結）」に置き換わる。`docs/design/component-coverage-map.md` の
該当 2 行（`use-chart.md`/`installation.md`）を「保留」→「実装済み」へ
更新した。

## 4. 決定性の規則

### 4.1 数値の決定的文字列化（`svg::fmt_coord`）

座標・寸法・tick 値の文字列化は `svg::fmt_coord` にのみ実装を一元化する。

1. `format!("{:.2}", v)`（Rust 標準の小数第 2 位への丸め）
2. 小数点を含む場合、末尾の連続する `0` を除去し、続けて末尾の `.` も除去
3. 結果が `"-0"` の場合は `"0"` に正規化する

出力文字集合は `[0-9.-]` に閉じる（`PathBuilder` の `d` 属性値はこれに
座標区切りの `,` と `M`/`L`/`Z` コマンド文字を加えた `[0-9.\-, MLZ]` に
閉じる）。`v` が非有限（`NaN`/`±inf`）の場合の挙動は未規定とし、
`fmt_coord` へ到達する前に必ず有限性を検証する契約とする（§4.2）。

### 4.2 fail-closed な数値検証

`ChartData::new`/`LinearScale::new` の構築時に非有限値を拒否する
（[`ChartError::NonFiniteValue`]）。この検証を経由した値のみが
`fmt_coord`/`LinearScale::scale` へ渡る契約とすることで、フォーマット段
（マークアップ生成）に `NaN`/`±inf` が到達する経路を構造的に排除する。

### 4.3 nice tick の 1-2-5 ステップアルゴリズム

`LinearScale::ticks`/`LinearScale::nice` は d3（`d3-array` の
`tickStep`）が採用するアルゴリズムを踏襲する。生の刻み幅
`raw_step = (domain 幅) / target` に対し、10 のべき乗 `magnitude` を
取り出し、正規化誤差 `error = raw_step / magnitude` を
`sqrt(2)`/`sqrt(10)`/`sqrt(50)` の 3 閾値と比較して倍率
`{1, 2, 5, 10}` を選ぶ（単純な算術中間点ではなく幾何学的閾値を使うことで、
期待 tick 本数から実際の本数が系統的に増減しないようにする、d3 の既知の
設計）。`target` は 1..=50 に検証し、無限ループ・過大メモリ割当を構造的に
排除する（`ticks` 内の `MAX_TICKS` 上限と合わせた二重の安全策）。

## 5. セキュリティ不変条件

- SVG を含む全マークアップは [`fandhe_frontend_headless_ui::fandhe_frontend_core::el`]/`text`
  経由のノード木 API のみで生成し、`raw_html()` は 0 箇所（クレート全体の
  `disallowed_methods` clippy lint が機械強制）。
- 数値由来の属性値（`d`/`viewBox`/座標）は検証済み有限 `f64` →
  `fmt_coord`/`PathBuilder` の閉じた文字集合のみで構成され、文字列注入
  経路を持たない。
- 系列名・カテゴリ名等の任意文字列はテキストノード・属性値としてのみ
  扱われ、`fandhe_frontend_core::render` の既定エスケープ（REQ-1）を必ず
  通る（`crates/pre-styled-ui/tests/charts_foundation.rs` の XSS 回帰
  テストで固定）。
- `svg::svg_root` は呼び出し側 `attrs` に含まれる `viewBox`/`role`
  （大文字小文字を無視）を黙って除去してから既定値を付与する
  （`fandhe_frontend_core::render` は同名属性を重複除去しない契約のため、
  除去せず連結すると無効な HTML を生みかねない。`class_attr::drop_class_attr`
  と同型の判断）。

## 6. 後続イシュー（#847〜#851）への引き継ぎ事項

- 本イシュー（#846）は charts 基盤自体の CSS（`stylesheet()`）を持たない。
  チャート部品固有の CSS（軸線・グリッド線・凡例・ツールチップの既定
  スタイル等）を追加する際は、`stylesheet.rs` の一元化リスト
  （`all_styled_component_css`）への登録が必要になる。
- `stylesheet.rs` のドリフト検知テストは `src/` 直下の `.rs` ファイルのみを
  走査する設計であり、`src/charts/` のようなディレクトリモジュール配下は
  自動走査の対象外である（本イシューは CSS を持たないため影響しないが、
  #847 以降で CSS を追加する場合はこの走査範囲の盲点を踏まえ、手動登録漏れ
  がないか個別に確認する必要がある）。走査範囲をディレクトリモジュールへ
  拡張するかどうかは、CSS 追加が実際に必要になった時点で判断する
  （現時点では過剰な先回り拡張を避ける）。
- **上記の走査範囲拡張はイシュー #849（BarChart/BarList/BarSegment）で
  実施済み**。`bar_chart`/`bar_list`/`bar_segment` が本イシュー（#846）以降で
  最初に CSS（`css()`）を持つ charts 部品となったため、
  `stylesheet.rs::tests::all_styled_component_css_covers_every_component_module`
  の走査対象を `src/` 直下に加え `src/charts/`（非再帰の 1 階層）へ拡張し、
  `all_styled_component_css` へ 3 部品を登録した。`charts/mod.rs`（モジュール
  宣言のみ、CSS を持たない）は `lib.rs`/`stylesheet.rs` と同じく走査対象から
  除外している。並列実行される #847/#848/#850 が同じ拡張を独立に行う可能性
  があるため、マージ順序によっては後続 PR 側で重複実装を破棄し先行実装へ
  合流する調整が必要になる（本ファイル冒頭の注意と同型）。
