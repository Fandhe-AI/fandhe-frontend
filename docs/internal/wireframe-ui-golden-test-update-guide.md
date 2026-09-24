# wireframe-ui golden テスト（値ベース全単射検証）の更新手順

> `docs/internal/` は `site/nav.toml` に登録しない内部設計記録です
> （`crates/docs-site/tests/site_nav.rs::docs_internal_notes_are_never_registered_in_nav`
> が fail-closed に検証します）。本リポジトリは public であり、
> 「サイト非掲載」は「非公開」を意味しません。

## 1. 目的とスコープ

`crates/wireframe-ui/tests/golden_css.rs`（+ サブモジュール群
`crates/wireframe-ui/tests/golden/`）に置かれた、`fandhe_frontend_wireframe_ui::css::PARTS`
（`pub const <PART>_CSS` の集約配列）・実行時生成 CSS 関数
（`tokens::css()` / `size::css()` / `frame::frame_padding_css()`）の
バイト一致テスト（golden テスト）を、`docs/design/wireframe-ui-architecture.md`
§8 の 49 部品を横断して整備した手順を記す（イシュー #2666）。

### 1.1 方式（イシュー #2666、2026-09-24 に全面刷新）

旧方式（`tests/<snake>_css.rs` を 49 ファイルへ分散させ、
`tests/golden_coverage.rs` がソーステキストを走査するヒューリスティック
で `assert_eq!` の使用状況を判定する）は、コメント・文字列リテラル中に
`assert_eq!(...)` らしき文字列を書くだけでカバー済みと誤判定される迂回が、
方式を強化するたびに繰り返し指摘された（codex-review、計 4 ラウンド）。

現方式は **ソーステキストの走査を一切行わない値ベースの全単射検証** へ
置き換えた。

- golden 期待値リテラルは `crates/wireframe-ui/tests/golden/<part>.rs`
  （`pub const EXPECTED_CSS: &str` のみを持つ独立モジュール）に置く。
  `golden/` は `tests/` 直下ではなくサブディレクトリのため、cargo の
  統合テスト自動検出の対象にならない（`tests/support/` と同型の規約）。
- 単一の integration test crate `crates/wireframe-ui/tests/golden_css.rs`
  が `mod golden;` で全サブモジュールを取り込み、`goldens(): Vec<GoldenEntry>`
  レジストリを構築する。各エントリは
  `(表示名, 実装への通常の Rust 参照から得た実際の CSS, golden 独立
  リテラル, PARTS 由来か否か)` を保持する。**実装への参照は
  `fandhe_frontend_wireframe_ui::<mod>::<CONST>` という通常のコンパイル時
  識別子解決であり、コメント・文字列偽装では成立し得ない。**
- 検証する不変条件は 3 つ:
  1. 各エントリで `actual`（実装の実際の出力）と `expected`（golden
     独立リテラル）がバイト一致する
     （`each_golden_matches_its_expected_literal`）。
  2. `PARTS` の値の多重集合と、`in_parts: true` な golden エントリの
     `actual` 値の多重集合が一致する
     （`parts_values_and_in_parts_golden_actuals_are_the_same_multiset`、
     `golden_count_matches_parts_and_base_functions`）。これにより
     golden 追加漏れ・不要な残留エントリの両方向を検知する。
  3. `wireframe_css()` の連結順契約は既存の `tests/wireframe_css_assembly.rs`
     が独立に検証する（変更なし）。
- 「新しい部品の golden 追加漏れで (2) が FAIL する」ことは、実データではなく
  ローカルの偽の値配列に対して同じ多重集合比較関数
  （`check_value_multiset_bijection`）を当てる単体テスト
  （`bijection_self_test` モジュール）で示す。
- 「実装定数を golden 期待値として再エクスポートする自己比較」の防止策
  として `ptr::eq` による同一メモリ判定を実装・実測したが、**rustc が
  同一バイト列の `&'static str` リテラルをコンパイル時に単一アロケーション
  へ統合するため（LLVM 最適化ではなく const 評価・インターン処理の段階）、
  独立して書いた golden リテラルと実装定数が常に `ptr::eq` 判定で
  真になってしまい、判定手段として機能しないことを実測で確認した**。
  このため `ptr::eq` 検証は不採用とし、`crates/wireframe-ui/tests/golden_css.rs`
  内にコメントとして経緯を残している（削除の判断は弱体化ではなく方式の
  非採用の明示的記録）。この残余リスクは「`tests/golden/<part>.rs` は
  `pub const EXPECTED_CSS: &str = r#"..."#;` 1 行のみを持ち、
  `use fandhe_frontend_wireframe_ui` 等の import を書かない」という規約と
  コードレビューに委ねる。

### 1.2 golden の価値の限界（正直な注記）

`<PART>_CSS` の大半は元ソースの `&str` リテラルをそのままダンプした
golden であり、golden と実装の実質的な内容は同じである。golden が主に
生むのは「変更時に 2 か所（実装・テスト）を編集しなければならない摩擦」
であり、これが意図しない変更（コピペミス・書き換えの取りこぼし）の検知に
つながる。実効性がとりわけ高いのは次の 2 点であり、golden 全体の価値を
過大に見積もらないこと。

1. **基盤生成 CSS 3 件の golden**（`tests/golden/tokens.rs` /
   `tests/golden/size.rs` / `tests/golden/frame_padding.rs`）:
   `tokens::css()` / `size::css()` / `frame::frame_padding_css()` は
   `size::SCALE` / `tokens::TOKENS` という共有値表からループで生成
   されるため、値表側の変更が黙って全出力へ波及する。golden が実際に
   差分を検知する。
2. **連結順の契約テスト**（`tests/wireframe_css_assembly.rs`）:
   `docs/design/wireframe-ui-architecture.md` §10.4 が定める出力順
   （tokens → size → `PARTS` 登録順 → frame padding）を、`css.rs` の実装
   から独立に組み立て直して照合する。

## 2. 期待値の在り処

- 期待値はすべて `crates/wireframe-ui/tests/golden/<part>.rs` の
  `pub const EXPECTED_CSS: &str`（raw 文字列リテラル）です。
  `include_str!` による外部 fixture ファイルは持ちません。
- 部品 golden は 1 部品 1 サブモジュール（`tests/golden/<mod>.rs`）です。
  基盤生成関数 3 件は `tests/golden/tokens.rs` / `tests/golden/size.rs` /
  `tests/golden/frame_padding.rs`、`PARTS` 先頭のグリフ基底 class
  （`icon::ICON_GLYPH_CSS`）は `tests/golden/icon_glyph.rs` です（表示
  部品としての `icon::ICON_CSS` は `tests/golden/icon.rs` に別途ある、
  §3 参照）。
- raw 文字列の `#` の個数（`r#"..."#` / `r##"..."##`）は、期待値の内容に
  `"#` が含まれないぎりぎりの最小個数を使います（機械生成時に自動選定、
  §6 参照）。
- `tests/golden/<part>.rs` は `pub const EXPECTED_CSS: &str = r#"..."#;`
  のみを持ち、`use` 文や実装クレートへの参照を書きません（§1.1 の
  `ptr::eq` 不採用を踏まえた、独立リテラルであることを保つ規約）。

## 3. 部品 → golden サブモジュール対応表

イシュー #2666 実装完了時点（2026-09-24、origin/main 先頭 `e596c09c`
「Brand（brand）部品を追加する」#2726 取り込み後）の実測です。
`docs/design/wireframe-ui-architecture.md` §8 が定める 49 部品すべてが
main へマージ済みであり、49 部品全件が golden でカバーされています。

### 3.1 基盤（4 件、`PARTS` を経由しない 3 件 + `PARTS` 先頭の 1 件）

| 対象 | 実装参照 | golden サブモジュール |
|---|---|---|
| tokens | `fandhe_frontend_wireframe_ui::tokens::css()` | `tests/golden/tokens.rs` |
| size | `fandhe_frontend_wireframe_ui::size::css()` | `tests/golden/size.rs` |
| frame_padding | `fandhe_frontend_wireframe_ui::frame::frame_padding_css()` | `tests/golden/frame_padding.rs` |
| icon_glyph | `fandhe_frontend_wireframe_ui::icon::ICON_GLYPH_CSS` | `tests/golden/icon_glyph.rs` |

### 3.2 部品（49 件、`PARTS` 由来、アルファベット順）

| kebab（golden エントリ名） | 実装参照 | golden サブモジュール |
|---|---|---|
| accordion | `fandhe_frontend_wireframe_ui::accordion::ACCORDION_CSS` | `tests/golden/accordion.rs` |
| alert | `fandhe_frontend_wireframe_ui::alert::ALERT_CSS` | `tests/golden/alert.rs` |
| annotation | `fandhe_frontend_wireframe_ui::annotation::ANNOTATION_CSS` | `tests/golden/annotation.rs` |
| avatar | `fandhe_frontend_wireframe_ui::avatar::AVATAR_CSS` | `tests/golden/avatar.rs` |
| brand | `fandhe_frontend_wireframe_ui::brand::BRAND_CSS` | `tests/golden/brand.rs` |
| breadcrumbs | `fandhe_frontend_wireframe_ui::breadcrumbs::BREADCRUMBS_CSS` | `tests/golden/breadcrumbs.rs` |
| button | `fandhe_frontend_wireframe_ui::button::BUTTON_CSS` | `tests/golden/button.rs` |
| calendar | `fandhe_frontend_wireframe_ui::calendar::CALENDAR_CSS` | `tests/golden/calendar.rs` |
| card_basic | `fandhe_frontend_wireframe_ui::card_basic::CARD_BASIC_CSS` | `tests/golden/card_basic.rs` |
| chart | `fandhe_frontend_wireframe_ui::chart::CHART_CSS` | `tests/golden/chart.rs` |
| checkbox | `fandhe_frontend_wireframe_ui::checkbox::CHECKBOX_CSS` | `tests/golden/checkbox.rs` |
| counter | `fandhe_frontend_wireframe_ui::counter::COUNTER_CSS` | `tests/golden/counter.rs` |
| cursor | `fandhe_frontend_wireframe_ui::cursor::CURSOR_CSS` | `tests/golden/cursor.rs` |
| divider | `fandhe_frontend_wireframe_ui::divider::DIVIDER_CSS` | `tests/golden/divider.rs` |
| emoji | `fandhe_frontend_wireframe_ui::emoji::EMOJI_CSS` | `tests/golden/emoji.rs` |
| file_drop | `fandhe_frontend_wireframe_ui::file_drop::FILE_DROP_CSS` | `tests/golden/file_drop.rs` |
| frame | `fandhe_frontend_wireframe_ui::frame::FRAME_CSS` | `tests/golden/frame.rs` |
| grid | `fandhe_frontend_wireframe_ui::grid::GRID_CSS` | `tests/golden/grid.rs` |
| icon | `fandhe_frontend_wireframe_ui::icon::ICON_CSS` | `tests/golden/icon.rs` |
| image | `fandhe_frontend_wireframe_ui::image::IMAGE_CSS` | `tests/golden/image.rs` |
| input | `fandhe_frontend_wireframe_ui::input::INPUT_CSS` | `tests/golden/input.rs` |
| link | `fandhe_frontend_wireframe_ui::link::LINK_CSS` | `tests/golden/link.rs` |
| list | `fandhe_frontend_wireframe_ui::list::LIST_CSS` | `tests/golden/list.rs` |
| map | `fandhe_frontend_wireframe_ui::map::MAP_CSS` | `tests/golden/map.rs` |
| media | `fandhe_frontend_wireframe_ui::media::MEDIA_CSS` | `tests/golden/media.rs` |
| menu | `fandhe_frontend_wireframe_ui::menu::MENU_CSS` | `tests/golden/menu.rs` |
| modal | `fandhe_frontend_wireframe_ui::modal::MODAL_CSS` | `tests/golden/modal.rs` |
| nav_item | `fandhe_frontend_wireframe_ui::nav_item::NAV_ITEM_CSS` | `tests/golden/nav_item.rs` |
| pagination | `fandhe_frontend_wireframe_ui::pagination::PAGINATION_CSS` | `tests/golden/pagination.rs` |
| paragraph | `fandhe_frontend_wireframe_ui::paragraph::PARAGRAPH_CSS` | `tests/golden/paragraph.rs` |
| progress | `fandhe_frontend_wireframe_ui::progress::PROGRESS_CSS` | `tests/golden/progress.rs` |
| question | `fandhe_frontend_wireframe_ui::question::QUESTION_CSS` | `tests/golden/question.rs` |
| radio | `fandhe_frontend_wireframe_ui::radio::RADIO_CSS` | `tests/golden/radio.rs` |
| ratings | `fandhe_frontend_wireframe_ui::ratings::RATINGS_CSS` | `tests/golden/ratings.rs` |
| rich_text | `fandhe_frontend_wireframe_ui::rich_text::RICH_TEXT_CSS` | `tests/golden/rich_text.rs` |
| select | `fandhe_frontend_wireframe_ui::select::SELECT_CSS` | `tests/golden/select.rs` |
| slider | `fandhe_frontend_wireframe_ui::slider::SLIDER_CSS` | `tests/golden/slider.rs` |
| spinner | `fandhe_frontend_wireframe_ui::spinner::SPINNER_CSS` | `tests/golden/spinner.rs` |
| stack | `fandhe_frontend_wireframe_ui::stack::STACK_CSS` | `tests/golden/stack.rs` |
| stat | `fandhe_frontend_wireframe_ui::stat::STAT_CSS` | `tests/golden/stat.rs` |
| stepper | `fandhe_frontend_wireframe_ui::stepper::STEPPER_CSS` | `tests/golden/stepper.rs` |
| switch | `fandhe_frontend_wireframe_ui::switch::SWITCH_CSS` | `tests/golden/switch.rs` |
| table | `fandhe_frontend_wireframe_ui::table::TABLE_CSS` | `tests/golden/table.rs` |
| tabs | `fandhe_frontend_wireframe_ui::tabs::TABS_CSS` | `tests/golden/tabs.rs` |
| tag | `fandhe_frontend_wireframe_ui::tag::TAG_CSS` | `tests/golden/tag.rs` |
| text | `fandhe_frontend_wireframe_ui::text::TEXT_CSS` | `tests/golden/text.rs` |
| textarea | `fandhe_frontend_wireframe_ui::textarea::TEXTAREA_CSS` | `tests/golden/textarea.rs` |
| toast | `fandhe_frontend_wireframe_ui::toast::TOAST_CSS` | `tests/golden/toast.rs` |
| tooltip | `fandhe_frontend_wireframe_ui::tooltip::TOOLTIP_CSS` | `tests/golden/tooltip.rs` |

合計: 49（部品）+ 4（基盤）= **53** golden エントリ（`crates/wireframe-ui/tests/golden_css.rs`
の `goldens().len()`。うち `PARTS` 由来〔`in_parts: true`〕は 50 件 =
49 部品 + icon_glyph、`PARTS.len()` と一致することを
`parts_values_and_in_parts_golden_actuals_are_the_same_multiset` が検証する）。

## 4. 解決コマンド

特定部品の golden サブモジュールを探す・存在確認する:

```sh
ls crates/wireframe-ui/tests/golden/<snake>.rs
```

`PARTS` 定数一覧の再確認:

```sh
sed -n '/pub const PARTS/,/^];/p' crates/wireframe-ui/src/css.rs
```

## 5. 更新手順（通常フロー: 意図した CSS 変更）

1. `<PART>_CSS`（またはトークン・サイズ・padding 生成関数）を変更する。
2. `cargo test -p fandhe-frontend-wireframe-ui --test golden_css` を実行し、
   `each_golden_matches_its_expected_literal` の失敗差分（`left` = 実装
   の新しい出力、`right` = 旧 golden）を確認する。意図した変更のみが
   差分に現れていることを確認する。
3. 差分が正しければ、`tests/golden/<part>.rs`（基盤は
   `tests/golden/tokens.rs` 等）の `pub const EXPECTED_CSS` を新しい
   出力へ置き換える。手で書き写さず、§6 のダンプ手順で機械的に生成する
   こと。
4. `cargo test -p fandhe-frontend-wireframe-ui` を再実行し、
   `wireframe_css_assembly.rs`・`golden_css.rs` も含めて全 green に
   なることを確認する。

## 6. 期待値のダンプ手順（機械生成、手書き禁止）

期待値を手で書き写すと差分検知の意味がなくなるため、次のように使い捨て
の統合テストで実際の出力をダンプしてから golden へ貼り付けること。

1. `crates/wireframe-ui/tests/` に一時ファイル（例: `_dump.rs`、コミット
   しない）を置き、対象の `pub const`/関数を `println!("{}", ...)` で
   出力する。
2. `cargo test -p fandhe-frontend-wireframe-ui --test _dump -- --nocapture`
   で実行し、出力をそのままコピーする。
3. 出力に `"#` が含まれる場合は `r#"..."#` ではなく `r##"..."##`（以降
   必要な個数だけ `#` を増やす）を使う。末尾改行の有無も実出力へ厳密に
   合わせる（`SPINNER_CSS` のように `}\n` で終わる定数がある）。
4. 一時ファイルを削除してからコミットする（`git status` で残っていない
   ことを確認する）。

## 7. 新規部品の追加手順

新しい部品を `PARTS` へ登録する PR は、以下を**同じ PR 内で**行うこと。

1. `<PART>_CSS` を定義し `crate::css::PARTS` へ追記する。
2. `crates/wireframe-ui/tests/golden/<snake>.rs` を新規追加し、§6 の
   手順で `pub const EXPECTED_CSS: &str` を書く（`use` 文は書かない、
   §2 参照）。
3. `crates/wireframe-ui/tests/golden/mod.rs` へ `pub mod <snake>;` を
   追記する。
4. `crates/wireframe-ui/tests/golden_css.rs` の `goldens()` へ
   `GoldenEntry { name: "<snake>", actual: w::<mod>::<CONST>.to_string(),
   expected: golden::<snake>::EXPECTED_CSS, in_parts: true }` を追記する。
5. 本手順書 §3.2 の対応表へ 1 行追加する。
6. `cargo test -p fandhe-frontend-wireframe-ui --test golden_css` を実行
   し、全テスト（`each_golden_matches_its_expected_literal` /
   `golden_count_matches_parts_and_base_functions` /
   `parts_values_and_in_parts_golden_actuals_are_the_same_multiset` /
   `golden_names_are_unique`）が green になることを確認する。golden の
   追加を忘れると `golden_count_matches_parts_and_base_functions` と
   `parts_values_and_in_parts_golden_actuals_are_the_same_multiset` が
   fail-closed に検知する。

## 8. 差分の読み方

- `each_golden_matches_its_expected_literal` の失敗: 該当部品の CSS
  定数が変わった。意図した変更か確認する。
- `wireframe_css_assembly.rs` の失敗: `wireframe_css()` の連結順が
  `docs/design/wireframe-ui-architecture.md` §10.4 の契約からずれた
  （`PARTS` への登録漏れ・順序入れ替え・区切り文字の変更等）。
- `golden_count_matches_parts_and_base_functions` /
  `parts_values_and_in_parts_golden_actuals_are_the_same_multiset` の
  失敗: `PARTS` に登録した定数の golden エントリが無い（§7 の手順を
  実行する）、または `PARTS` から削除された部品の golden エントリが
  `goldens()` に残留している（該当エントリを `goldens()`・
  `tests/golden/<snake>.rs`・`tests/golden/mod.rs` から削除する）。
- `golden_names_are_unique` の失敗: `goldens()` に同名エントリが重複
  している。

## 9. 禁止事項

- golden の弱体化・削除（差分を隠すための緩和）を行わない。
- `#[ignore]` を golden テストへ付けない。
- `goldens()` から、実際には `PARTS` に登録済みの部品のエントリを
  恣意的に外さない（`parts_values_and_in_parts_golden_actuals_are_the_same_multiset`
  が fail-closed に検知する）。
- 集約出力（`wireframe_css()` の全文）を 1 本の巨大なリテラル golden に
  固定しない（部品が増えるたびに書き換えが必要になり、並行 PR と必ず
  競合するため。連結順の検証は `wireframe_css_assembly.rs` の組み立て
  契約で足りる）。
- `tests/golden/<part>.rs` に実装クレートへの `use`・参照を書かない
  （§1.1・§2 参照。独立リテラルであることを保つ）。
- `tests/golden_css.rs`・`tests/golden/` の代わりにソーステキストを
  走査するヒューリスティックへ回帰しない（イシュー #2666 の 4 ラウンドの
  codex-review 指摘を踏まえた明示的な非採用判断）。

## 10. PR チェックリスト

- [ ] `cargo test -p fandhe-frontend-wireframe-ui` が全 green
- [ ] 新規/変更した golden は §6 の機械ダンプ手順で生成した（手書きで
      ない）
- [ ] 新規部品を追加した場合は §7 の手順（golden サブモジュール追加・
      `goldens()` 追記・本手順書 §3 更新）を全て行った
- [ ] `cargo fmt --all -- --check` / `cargo clippy -p fandhe-frontend-wireframe-ui
      --all-targets -- -D warnings` が通る
