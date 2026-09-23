# wireframe-ui golden テスト（バイト一致）の更新手順

> `docs/internal/` は `site/nav.toml` に登録しない内部設計記録です
> （`crates/docs-site/tests/site_nav.rs::docs_internal_notes_are_never_registered_in_nav`
> が fail-closed に検証します）。本リポジトリは public であり、
> 「サイト非掲載」は「非公開」を意味しません。

## 1. 目的とスコープ

`crates/wireframe-ui/tests/*_css.rs` に置かれた `pub const <PART>_CSS`・
実行時生成 CSS 関数（`tokens::css()` / `size::css()` /
`frame::frame_padding_css()`）・`wireframe_css()` 出力のバイト一致テスト
（golden テスト）を、`docs/design/wireframe-ui-architecture.md` §8 の
49 部品を横断して整備した手順を記す（イシュー #2666）。

pre-styled-ui の同種文書
（`docs/internal/pre-styled-ui-golden-test-update-guide.md`）と同じ節立てで
書く。ただし wireframe-ui は pre-styled-ui と違い、**大半の
`<PART>_CSS` が `&str` リテラル定数**である（`tokens::css()` /
`size::css()` / `frame::frame_padding_css()` の 3 つだけが
`size::SCALE`/`tokens::TOKENS` という共有値表から実行時に導出される）。
このため、部品ごとの golden の実効性には限界がある。

### 1.1 golden の価値の限界（正直な注記）

`<PART>_CSS` の大半は元ソースの `&str` リテラルをそのままダンプした
golden であり、golden と実装の実質的な内容は同じである。golden が主に
生むのは「変更時に 2 か所（実装・テスト）を編集しなければならない摩擦」
であり、これが意図しない変更（コピペミス・書き換えの取りこぼし）の検知に
つながる。実効性がとりわけ高いのは次の 3 点であり、golden 全体の価値を
過大に見積もらないこと。

1. **基盤 CSS 4 件の golden**（`tests/base_css.rs`）: `tokens::css()` /
   `size::css()` / `frame::frame_padding_css()` は `size::SCALE` /
   `tokens::TOKENS` という共有値表からループで生成されるため、値表側の
   変更が黙って全出力へ波及する。golden が実際に差分を検知する。
   `icon::ICON_GLYPH_CSS` はリテラルだが基盤（Phase 0、#2606）として同じ
   ファイルへ集約する。
2. **連結順の契約テスト**（`tests/wireframe_css_assembly.rs`）:
   `docs/design/wireframe-ui-architecture.md` §10.4 が定める出力順
   （tokens → size → `PARTS` 登録順 → frame padding）を、`css.rs` の実装
   から独立に組み立て直して照合する。
3. **カバレッジゲート**（`tests/golden_coverage.rs`）: `PARTS` に登録
   されている定数がすべて golden を持つこと、§8 の 49 部品のうち未
   マージのもの（`PENDING`）がまだ CSS を出力していないことを fail-closed
   に検証する。後続部品 PR の golden 追加漏れ・`PENDING` 削除漏れを
   検知する。

## 2. 期待値の在り処

- 期待値はすべて**テストファイル内のインライン `const EXPECTED_CSS: &str`**
  （raw 文字列リテラル）です。`include_str!` による外部 fixture ファイルは
  持ちません。
- 部品 golden は 1 部品 1 ファイル（`tests/<snake>_css.rs`）です。基盤
  4 件（`tokens::css()` / `size::css()` / `frame::frame_padding_css()` /
  `icon::ICON_GLYPH_CSS`）は `tests/base_css.rs` に集約します。
- raw 文字列の `#` の個数（`r#"..."#` / `r##"..."##`）は、期待値の内容に
  `"#` が含まれないぎりぎりの最小個数を使います（機械生成時に自動選定、
  §6 参照）。

## 3. 部品 → テストファイル対応表

イシュー #2666 実装時点（2026-09-24、origin/main 先頭 `fe5c194b`）の実測
です。`docs/design/wireframe-ui-architecture.md` §8 が定める 49 部品の
うち、42 部品が golden でカバーされ、7 部品が未マージ（`PENDING`）です。
後続部品 PR がマージされたら本表・`tests/golden_coverage.rs` の
`PENDING` 定数の両方を更新してください（更新しないと
`golden_coverage.rs` が fail-closed に検知します。§7 参照）。

### 3.1 基盤（4 件、`tests/base_css.rs`）

| 対象 | 生成元 |
|---|---|
| `tokens::css()` | `crates/wireframe-ui/src/tokens.rs` |
| `size::css()` | `crates/wireframe-ui/src/size.rs` |
| `frame::frame_padding_css()` | `crates/wireframe-ui/src/frame.rs` |
| `icon::ICON_GLYPH_CSS` | `crates/wireframe-ui/src/icon.rs`（#2606 の SVG アイコン基盤） |

### 3.2 カバー済み部品（42 件）

| Phase | kebab | イシュー | テストファイル |
|---|---|---|---|
| 1 | frame | #2609 | `tests/frame_css.rs` |
| 1 | stack | #2610 | `tests/stack_css.rs` |
| 1 | grid | #2611 | `tests/grid_css.rs` |
| 1 | divider | #2612 | `tests/divider_css.rs` |
| 2 | text | #2614 | `tests/text_css.rs` |
| 2 | paragraph | #2615 | `tests/paragraph_css.rs` |
| 2 | rich-text | #2616 | `tests/rich_text_css.rs` |
| 2 | annotation | #2617 | `tests/annotation_css.rs` |
| 2 | link | #2618 | `tests/link_css.rs` |
| 2 | tag | #2619 | `tests/tag_css.rs` |
| 3 | button | #2621 | `tests/button_css.rs` |
| 3 | input | #2622 | `tests/input_css.rs` |
| 3 | textarea | #2623 | `tests/textarea_css.rs` |
| 3 | select | #2624 | `tests/select_css.rs` |
| 3 | checkbox | #2625 | `tests/checkbox_css.rs` |
| 3 | radio | #2626 | `tests/radio_css.rs` |
| 3 | switch | #2627 | `tests/switch_css.rs` |
| 3 | slider | #2628 | `tests/slider_css.rs` |
| 4 | question | #2630 | `tests/question_css.rs` |
| 4 | ratings | #2631 | `tests/ratings_css.rs` |
| 4 | calendar | #2632 | `tests/calendar_css.rs` |
| 4 | file-drop | #2633 | `tests/file_drop_css.rs` |
| 4 | stepper | #2634 | `tests/stepper_css.rs` |
| 5 | nav-item | #2636 | `tests/nav_item_css.rs` |
| 5 | menu | #2637 | `tests/menu_css.rs` |
| 5 | tabs | #2638 | `tests/tabs_css.rs` |
| 5 | breadcrumbs | #2639 | `tests/breadcrumbs_css.rs` |
| 5 | pagination | #2640 | `tests/pagination_css.rs` |
| 5 | accordion | #2641 | `tests/accordion_css.rs` |
| 5 | cursor | #2642 | `tests/cursor_css.rs` |
| 6 | tooltip | #2644 | `tests/tooltip_css.rs` |
| 6 | modal | #2645 | `tests/modal_css.rs` |
| 6 | alert | #2646 | `tests/alert_css.rs` |
| 6 | toast | #2647 | `tests/toast_css.rs` |
| 6 | progress | #2648 | `tests/progress_css.rs` |
| 6 | spinner | #2649 | `tests/spinner_css.rs` |
| 7 | avatar | #2651 | `tests/avatar_css.rs` |
| 7 | counter | #2655 | `tests/counter_css.rs` |
| 7 | emoji | #2654 | `tests/emoji_css.rs` |
| 7 | stat | #2656 | `tests/stat_css.rs` |
| 8 | image | #2660 | `tests/image_css.rs` |
| 8 | chart | #2663 | `tests/chart_css.rs` |

### 3.3 保留部品（7 件、golden 未整備）

`crates/wireframe-ui/tests/golden_coverage.rs` の `PENDING` 定数と同期
させること。

| Phase | kebab | イシュー | 状態 |
|---|---|---|---|
| 7 | icon（表示部品） | #2652 | PR 未作成 |
| 7 | brand | #2653 | PR 未作成 |
| 7 | list | #2657 | PR #2717 open |
| 7 | card-basic | #2658 | PR #2714 open |
| 8 | media | #2661 | PR #2718 open |
| 8 | table | #2662 | PR #2721 open |
| 8 | map | #2664 | PR #2722 open |

合計: 42（カバー済み）+ 7（保留）= **49**（`docs/design/wireframe-ui-architecture.md`
§8 の全部品数と一致）。

## 4. 解決コマンド

特定部品の golden ファイルを探す・存在確認する:

```sh
grep -l '\b<snake>::' crates/wireframe-ui/tests/*_css.rs
```

`PARTS` 定数一覧の再確認:

```sh
sed -n '/pub const PARTS/,/^];/p' crates/wireframe-ui/src/css.rs
```

## 5. 更新手順（通常フロー: 意図した CSS 変更）

1. `<PART>_CSS`（またはトークン・サイズ・padding 生成関数）を変更する。
2. `cargo test -p fandhe-frontend-wireframe-ui --test <snake>_css` を実行し、
   `assert_eq!` の失敗差分（`left` = 実装の新しい出力、`right` = 旧 golden）
   を確認する。意図した変更のみが差分に現れていることを確認する。
3. 差分が正しければ、`tests/<snake>_css.rs`（基盤は `tests/base_css.rs`）
   の `const EXPECTED_CSS` を新しい出力へ置き換える。手で書き写さず、
   §6 のダンプ手順で機械的に生成すること。
4. `cargo test -p fandhe-frontend-wireframe-ui` を再実行し、
   `wireframe_css_assembly.rs`・`golden_coverage.rs` も含めて全 green に
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

新しい部品を `PARTS` へ登録する PR（またはイシュー #2666 の `PENDING` に
載っている部品が main へマージされる PR）は、以下を**同じ PR 内で**行う
こと。

1. `<PART>_CSS` を定義し `crate::css::PARTS` へ追記する。
2. §6 の手順で `tests/<snake>_css.rs` を追加する。
3. その部品が `PENDING`（`tests/golden_coverage.rs`）に載っている場合は
   削除する。
4. 本手順書 §3.2 の対応表へ 1 行追加し、§3.3 から該当行を削除する。
5. `cargo test -p fandhe-frontend-wireframe-ui` を実行し、
   `golden_coverage.rs` の全テスト（`every_parts_const_has_a_golden_file`
   / `pending_entries_are_subset_of_all_parts_and_not_yet_shipped` /
   `shipped_parts_expose_their_root_selector` /
   `covered_count_matches_parts_minus_icon_glyph_base` 等）が green に
   なることを確認する。

## 8. 差分の読み方

- `<snake>_css.rs` の失敗: その部品の CSS 定数が変わった。意図した変更か
  確認する。
- `wireframe_css_assembly.rs` の失敗: `wireframe_css()` の連結順が
  `docs/design/wireframe-ui-architecture.md` §10.4 の契約からずれた
  （`PARTS` への登録漏れ・順序入れ替え・区切り文字の変更等）。
- `golden_coverage.rs` の失敗:
  - `every_parts_const_has_a_golden_file`: `PARTS` に登録した定数の
    golden ファイルが無い。§7 の手順を実行する。
  - `pending_entries_are_subset_of_all_parts_and_not_yet_shipped`:
    `PENDING` に載っている部品が実は既に出力されている（golden を追加
    して `PENDING` から外し忘れている）。
  - `shipped_parts_expose_their_root_selector`: カバー済みと扱っている
    部品のルートセレクタ（`.fw-wire-<kebab> {`）が見つからない
    （`ALL_PARTS` のエントリ誤り、または部品側実装の変更）。
  - `covered_count_matches_parts_minus_icon_glyph_base`: カバー数の
    整合が崩れた（`PENDING` の更新漏れが典型）。

## 9. 禁止事項

- golden の弱体化・削除（差分を隠すための緩和）を行わない。
- `#[ignore]` を golden テストへ付けない。
- `PENDING` へ、実際には main にマージ済みの部品を恣意的に残さない
  （`pending_entries_are_subset_of_all_parts_and_not_yet_shipped` が
  fail-closed に検知する）。
- 集約出力（`wireframe_css()` の全文）を 1 本の巨大なリテラル golden に
  固定しない（部品が増えるたびに書き換えが必要になり、並行 PR と必ず
  競合するため。連結順の検証は `wireframe_css_assembly.rs` の組み立て
  契約で足りる）。

## 10. PR チェックリスト

- [ ] `cargo test -p fandhe-frontend-wireframe-ui` が全 green
- [ ] 新規/変更した golden は §6 の機械ダンプ手順で生成した（手書きで
      ない）
- [ ] 新規部品を追加した場合は §7 の手順（golden 追加・`PENDING` 更新・
      本手順書 §3 更新）を全て行った
- [ ] `cargo fmt --all -- --check` / `cargo clippy -p fandhe-frontend-wireframe-ui
      --all-targets -- -D warnings` が通る
