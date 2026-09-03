# pre-styled-ui golden テスト（バイト一致）の更新手順

> `docs/internal/` は `site/nav.toml` に登録しない内部設計記録です
> （`crates/docs-site/tests/site_nav.rs::docs_internal_notes_are_never_registered_in_nav`
> が fail-closed に検証します）。本リポジトリは public であり、
> 「サイト非掲載」は「非公開」を意味しません。

## 1. 目的とスコープ

`crates/pre-styled-ui/tests/*_css.rs` に置かれた `css()` / `stylesheet()`
出力のバイト一致テスト（golden テスト）を、スタイル調整の PR（親イシュー
#1421「Phase 0: スタイル調整の共通基盤」配下、Phase 1 以降の各部品 issue）
で安全かつ迷わず更新できるようにするための手順書です。

対象は golden（バイト一致）テストのみです。以下は本ガイドの対象外で、
別イシューに委ねます（インラインコード参照のみとし、`docs/internal/` は
nav 未登録のためリンク化しません）。

- スクリーンショット比較の運用: イシュー #1428
- semver バンプ運用（`.claude/rules/coding-rust.md` の
  「公開済みクレートの実体変更時は semver バンプ必須」節を参照）: イシュー
  #1429

## 2. 期待値の在り処

- 期待値はすべて **テストファイル内のインライン `const`**（例:
  `const EXPECTED_CSS: &str = r#"..."#;`）です。`include_str!` による外部
  fixture ファイルは 1 件もありません。
- 定数名の慣例は `EXPECTED_CSS` / `EXPECTED_<PART>_CSS` / `<PART>_GOLDEN_CSS`
  など、テストファイルごとに異なります（統一の命名規則はありません）。
- 1 ファイルに複数部品の期待値が並ぶことがあります（例:
  `form_controls_css.rs` は input / textarea / native_select の 3 部品分の
  定数を持ちます）。

### 2.1 3 つの方式

| 方式 | 内容 | 該当ファイル |
|------|------|-------------|
| (a) golden バイト一致 | `assert_eq!(<module>::css(), EXPECTED)` で CSS 全文を固定 | 下記対応表の大多数 |
| (b) 契約アサーション方式 | CSS 全文ではなく「スロットセレクタの存在」「主要な状態セレクタの存在」「CSS breakout 不在」のみを固定 | `menubar_css.rs` / `navigation_menu_css.rs`（イシュー #992 実装計画で、多パーツ部品では宣言 1 個の増減でも無関係な diff が広範囲に生じる brittle さを避けるためと明記済み）、`download_trigger_css.rs`（固定 `const` を持たず、`button::css()` からの派生値との比較・存在確認で流用契約を固定する派生比較変種）、`table_data_list_css.rs`（ファイル冒頭 rustdoc に「決定性 + 重要規則の存在確認」形式と明記済み） |
| (c) golden 不在 | XSS 回帰・決定性テストのみで golden は未整備 | 下記「golden 不在の部品」参照 |

方式 (b) を新たに採用する場合は、(a) の方が向いている（パーツ数が少なく
diff が読みやすい）部品を安易に (b) へ切り替えないでください（§8 禁止事項
参照）。

## 3. 部品 → テストファイル対応表

以下は `grep -l '\b<snake>::' crates/pre-styled-ui/tests/*_css.rs` で機械的
に再生成した実測結果です（golden 系テストファイルのみを対象とする版。
「特定モジュールへの参照が golden 以外も含めどこにあるか」を広く探す場合
は §4 の `tests/*.rs` 版を使ってください）。手書きで追記した場合は次回
更新時にドリフトしていないか同コマンドで確認してください。

### 3.1 1 対 1 対応（規則: `<snake>_css.rs`）

accordion / **alert（イシュー #1553 で golden 新設）** /
**badge（イシュー #1555 で golden 新設）** / callout / carousel /
checkbox / checkbox_card / checkbox_group /
color_picker / color_swatch / date_input / dialog / **download_trigger（方式 b）** /
drawer / editable / file_upload / floating_panel / highlight / hover_card /
image_cropper / listbox / marquee / menu / **menubar（方式 b）** /
**navigation_menu（方式 b）** / number_input / pagination / password_input /
pin_input / progress / qr_code / radio_card / radio_group / rating_group /
select / separator / skeleton / skip_nav / splitter / stat / steps / switch /
tab_nav / tags_input / timeline / timer / toast / toggle_tip / tour /
visually_hidden

menubar / navigation_menu / download_trigger はファイル名こそ
`<snake>_css.rs` の規則どおりですが、内容は §2.1 の方式 (b)（契約アサー
ション、CSS 全文の golden ではない）です。golden バイト一致を期待して
開くと構成が異なるので注意してください。download_trigger は固定
`const` を持たず、`button::css()` からの派生値（scope 置換した文字列）
との比較・存在確認で流用契約を固定する派生比較変種です（§2.1 参照）。

### 3.2 グルーピングファイル（複数部品を 1 ファイルに集約）

| テストファイル | 対象部品 |
|----------------|----------|
| `button_css.rs` | button（`download_trigger_css.rs` も button を参照する） |
| `typography_css.rs` | heading / text / em / mark / blockquote / list / quote / strong |
| `form_controls_css.rs` | input / textarea / native_select（`field` scope を共有） |
| `image_icon_css.rs` | image / icon |
| `tag_kbd_code_css.rs` | tag / kbd / code |
| `table_data_list_css.rs`（方式 b） | table / data_list |
| `status_empty_state_css.rs` | status / empty_state |
| `popover_tooltip_css.rs` | popover / tooltip |
| `pie_donut_chart_css.rs` | pie_chart / donut_chart |
| `charts_css.rs` | line_chart / area_chart / sparkline |
| `charts_parts_css.rs` | charts 内部パーツ: `charts::axis` / `charts::grid` / `charts::legend` / `charts::tooltip` |
| `scatter_radar_chart_css.rs` | `charts::scatter_chart` / `charts::radar_chart` |
| `tabs_css.rs` | tabs（`recipe_css.rs` は recipe 機構自体の golden であり tabs はその一実例として参照するのみ） |

`table_data_list_css.rs` は他のグルーピングファイル（`assert_eq!` による
CSS 全文の golden、方式 (a)）と異なり、§2.1 の方式 (b)（契約アサーション、
「決定性 + 重要規則の存在確認」形式）です。§5 の通常フロー（`assert_eq!`
の差分を実出力で貼り替える手順）は適用できません。更新時は該当アサー
ション（`assert!(css.contains(...))`）を意図した変更に合わせて書き換えて
ください。

### 3.3 golden 不在の部品

以下は `pub fn stylesheet()` / `pub fn css()` を持つものの、golden
（バイト一致・契約アサーションいずれも含む）テストが未整備です。`smoke.rs` /
`xss_escape.rs` / `xss_escape_styled.rs` によるレンダリング・エスケープ検証
のみが存在します。新設の要否は Phase 1 の各部品 issue の判断に委ねます。

- `stylesheet()` を持つ 18 部品: action_bar / angle_slider /
  breadcrumb / calendar / clipboard / combobox / date_picker /
  json_tree_view / link / link_overlay / nav_list / scroll_area /
  segment_group / signature_pad / slider / toggle / toggle_group /
  tree_view（`toolbar` はイシュー #1547、`avatar` はイシュー #1554 で
  golden 追加済みのため本リストから除外）
- `css()` を持つ 1 部品: spinner（`alert` はイシュー #1553、`badge` は
  イシュー #1555、`callout` はイシュー #1556、`card` はイシュー #1557 で
  それぞれ golden 新設済みのため本リストから除外）
- charts 内部パーツ 3 件（`css()` を持つが golden なし）: `charts::bar_chart` /
  `charts::bar_list` / `charts::bar_segment`

`charts::pie` はジオメトリ計算のみを行う機構モジュール（`Node` を生成せず
`css()`/`stylesheet()` を持たない）であり、部品としての golden 対象外です
（`charts::data` / `charts::scale` / `charts::svg` も同様に機構モジュール）。

## 4. 解決コマンド

```sh
grep -l '\b<snake>::' crates/pre-styled-ui/tests/*.rs | xargs -n1 basename
```

- `<snake>` は部品モジュール名（例: `button`、`native_select`）。
- 複数ヒットする場合があります。例:
  - `button::` → `button_css.rs` と `download_trigger_css.rs`
  - `tooltip::` → `charts_parts_css.rs`（`charts::tooltip`）と
    `popover_tooltip_css.rs`（styled `tooltip`）の 2 件。どちらの `tooltip`
    かはヒット元ファイルの `use` 文で判別してください
  - `avatar::` は `avatar_css.rs`（イシュー #1554 で追加した golden）に加え
    `checkbox_card_css.rs` / `data_attr_vocabulary.rs` / `hover_card_css.rs` /
    `radio_card_css.rs` / `smoke.rs` / `xss_escape_styled.rs` にもヒットし
    ます（golden 不在部品を誤って golden ありと判断しないよう、ヒット先の
    ファイル名が `_css.rs` でも §3.3 記載の golden 不在部品でないか必ず
    確認してください）
- `charts::` 配下の部品は `charts::<sub>::` で grep してください（例:
  `charts::axis::`）。単に `axis::` だけで grep すると無関係なヒットが
  混ざることがあります。

## 5. 更新手順（通常フロー）

1. `crates/pre-styled-ui/src/<part>.rs`（または `charts/<part>.rs`）の
   recipe 宣言を変更する。
2. 該当テストのみを実行する: `cargo test -p fandhe-frontend-pre-styled-ui --test <name>_css`
3. `assert_eq!` の差分を読む。**`left` が実出力・`right` が期待値**です
   （取り違えに注意）。
4. 差分が意図した宣言変更のみに由来することを確認してから、期待値
   `const` を実出力で貼り替える。
5. 同ファイル内の他アサーション（決定性チェック・headless 側との接続
   照合・breakout 不在アサーション等）が独立して通ることを確認する。
6. クレート全体を実行する: `cargo test -p fandhe-frontend-pre-styled-ui`
7. `make lint` を通す。

## 6. 差分の読み方

- セレクタ行 `[data-scope="<scope>"][data-part="<part>"]` の `scope` /
  `part` は `SlotRecipe::new(scope, SLOTS)`（`src/recipe.rs`）の宣言に由来
  します。`field` scope を input / textarea / native_select が共有するなど、
  複数部品が同じ scope を持つ場合があります（`form_controls_css.rs`）。
- `.fd-<scope>--<axis>-<value>` は variant 軸のクラス名です。
- 属性セレクタ（`[data-*]`）や `:hover` 等の後置セレクタは
  `SlotRecipe::state`（states）由来です。
- `--fandhe-palette*` カスタムプロパティは `palette_declarations` 由来、
  `var(--fandhe-*)` はテーマトークン参照（`src/theme.rs`）由来です。
  トークン名の変更（親イシュー #1421 配下の #1422/#1423 のようなトークン
  設計変更）は複数の golden ファイルへ同時に波及します。
- 出力順は `SlotRecipe::css` の契約により **base（slots 宣言順）→
  variants（登録順）→ compound variants（登録順）→ states（登録順）** に
  固定されています。**宣言の並び替えだけ**でも差分が発生する点に注意して
  ください（値を変えていなくても diff が出るのは実装ミスではありません）。

## 7. 横断テストが落ちたときの読み解き方

golden テストと異なり、以下の横断テストは通常のスタイル調整では原則落ち
ません。落ちた場合は **期待値ではなく実装側の変更を疑ってください**。

| テスト | 疑うべきこと |
|--------|-------------|
| `data_attr_vocabulary.rs` | 非 anatomy `data-*`（`data-current`/`data-loading`/`data-action`/`data-value`/`data-series`）の付与条件を変えていないか、予約キー偽装除去のロジックを壊していないか |
| `xss_escape_styled.rs` / `xss_escape.rs` | 新しい動的値の出力経路を `text()` / 属性エスケープ経路以外の方法で追加していないか（REQ-1 回帰） |
| `recipe_css.rs` | scope / slot 名が headless-ui 側の実出力（`crates/headless-ui`）とずれていないか |
| `theme_css.rs` | テーマトークン名（`--fandhe-<group>-<name>`）を変更していないか |
| `recipe_determinism.rs` | `SlotRecipe` の内部実装に `HashMap`/`HashSet` 等、反復順序が不定な型を持ち込んでいないか |

## 8. 禁止事項

以下は「golden テストを通すため」の理由であっても行わないでください
（`.claude/rules/coding-rust.md` 「テスト」節: XSS 回帰テストの削除・弱体化
禁止、`#[ignore]` 追加でのごまかし禁止、を参照）。

- テストへの `#[ignore]` 付与
- `assert_eq!` から `contains` 等への緩和、または golden 方式（(a)）から
  契約アサーション方式（(b)）への安易な切り替え（brittle さの回避が明確な
  理由でない限り不可）
- エスケープ回帰テスト（`xss_escape.rs`/`xss_escape_styled.rs`）・横断テスト
  （`data_attr_vocabulary.rs` 等）の削除・弱体化
- 差分を読まずに実出力をそのまま期待値へ丸ごと貼り付ける運用（自動
  bless）。§5 の手順どおり、差分が意図した変更に由来することを確認して
  から貼り替えること
- テスト側の期待値だけを直し、`src/` 側の意図しない変更（レビュー時に
  気づかれなかった副作用）を残したままにすること

## 9. PR チェックリスト

- [ ] 変更した部品ごとに golden（該当する場合）を更新したか
- [ ] 横断テスト（§7 の一覧）が green か
- [ ] `cargo test -p fandhe-frontend-pre-styled-ui` 全体が green か
- [ ] `make lint` を通したか
- [ ] `crates/pre-styled-ui/**` の変更は `docs-site.yml` の再ビルド対象
      （`.claude/rules/ci.md` の paths フィルタ契約）であることを確認したか
- [ ] semver バンプは #1429 の運用に従ったか（本イシュー #1427 のスコープ外）

## 10. Phase 1 での検証記録

イシュー #1679（`mark`/`blockquote` への `ColorPalette::Neutral` 追加 +
`palette_declarations` → `palette_scale_declarations`〔6 役割版〕移行）が
Phase 1 で本ガイドに従って golden 更新を行った最初の事例です。

- §3.2「グルーピングファイル」の対応表どおり `mark`/`blockquote` の両 golden
  は `crates/pre-styled-ui/tests/typography_css.rs` の
  `MARK_GOLDEN_CSS`/`BLOCKQUOTE_GOLDEN_CSS` に集約されており、対応表と実態に
  齟齬はありませんでした。
- §5「更新手順（通常フロー）」どおり、まず `cargo test -p
  fandhe-frontend-pre-styled-ui --test typography_css` を実行して
  `assert_eq!` の diff（left = 実出力 / right = 旧期待値）を確認し、
  差分が「既存 palette 5 ブロックへの 3 宣言追加 + `--color-palette-neutral`
  ブロックの新規追加」という**純追加のみ**であることを目視確認してから
  実出力で期待値定数を置き換えました。手順どおり `#[ignore]` 追加・比較
  緩和は行っていません。
- §7「横断テストが落ちたときの読み解き方」の想定どおり、golden 更新と
  同一 PR 内で `crates/docs-site/tests/css_var_scope_prefix.rs` の
  `SHARED_VARS`（新規参照した `--fandhe-palette-subtle`/`-muted`/
  `-fg-subtle` の 3 件）を追随更新する必要がありました。`site_typography_
  contract.rs`（variant 宣言の内容は変更していないため）は無変更のまま
  green を維持できました。
- 手順・対応表とも齟齬なく完走でき、本ガイドの記述を修正する必要は
  ありませんでした（親イシュー #1421 へコメント報告、本イシュー #1427 の
  受け入れ条件 2 に対応）。

イシュー #1681（`Size::Xs`/`Size::Xl` と `ColorPalette::Neutral` を
Interactive・Data Display・Charts の該当 24 部品〔既に該当軸を
registrar 済みの部品のみ〕へ適用）も同じ手順で完走しました。

- 対象再計測（`git grep`）の結果、調査時点の一覧（Interactive:
  pagination/splitter/steps/tabs/tour が palette、accordion/breadcrumb/
  carousel/dialog/drawer/menu/pagination/splitter/steps/tabs が size。
  Data Display: badge/callout/spinner/status/tag/timeline が palette、
  avatar/badge/callout/color_swatch/empty_state/icon/progress/qr_code/
  spinner/stat/status/table/tag/timeline が size。Charts: area_chart/
  donut_chart/line_chart/pie_chart/sparkline が size）と実態に齟齬は
  ありませんでした。`alert` は palette 軸を持たないため変更なし。
- §3.2 の対応表どおり、golden は `accordion_css.rs` / `callout_css.rs` /
  `carousel_css.rs` / `charts_css.rs`（line/area/sparkline）/
  `color_swatch_css.rs` / `dialog_css.rs` / `drawer_css.rs` /
  `image_icon_css.rs`（icon 分のみ）/ `menu_css.rs` / `pagination_css.rs` /
  `pie_donut_chart_css.rs` / `progress_css.rs` / `qr_code_css.rs` /
  `splitter_css.rs` / `stat_css.rs` / `status_empty_state_css.rs`
  （status/empty_state 双方）/ `steps_css.rs` / `tabs_css.rs` /
  `tag_kbd_code_css.rs`（tag 分のみ）/ `timeline_css.rs` /
  `tour_css.rs` の 24 test fn に分散していました（`avatar`/`table` は
  golden 不在〔決定性テストのみ〕のため対象外）。
- 差分はすべて「既存 5 段（Sm/Md/Lg）ブロックへの Xs/Xl 純追加」または
  「既存 5 値 palette ブロックへの Neutral 純追加（6 役割版 palette
  宣言への切替を含む）」のみであることを目視確認してから期待値定数を
  置き換えました。`#[ignore]` 追加・比較緩和は行っていません。
- Xs/Xl のリテラル値は「既存宣言が等差進行なら両端へ 1 段外挿」
  「トークン参照なら同系トークンの下限/上限（無ければ最も近い既存
  トークンで clamp）」を機械的な規則として適用し、部品ごとの導出根拠は
  `src/<part>.rs` 内のコメントに残しました（詳細は PR 本文）。
- 横断テスト（`data_attr_vocabulary.rs`/`xss_escape_styled.rs`/
  `recipe_determinism.rs`/`theme_css.rs`）は期待値変更なしで green の
  ままでした。新規トークン参照はいずれも #1422/#1423 で既に導入済みの
  ものだけを使ったため、`css_var_scope_prefix.rs` の `SHARED_VARS` 追随は
  不要でした。
- `crates/docs-site/tests/no_js_contract.rs` の `TempDir::new` が
  `pid + nanos` のみで一時出力先を一意化していたため、サイトビルドの
  所要時間が伸びたことで並列実行中の 2 テストが同一パスへ衝突し、
  片方の `Drop` がもう片方の出力ディレクトリを削除してしまう既存の
  レースが顕在化しました（golden 更新そのものとは無関係）。原子的な
  カウンタを追加してパスを一意化し解消しています（同ファイル参照）。
- 手順・対応表とも齟齬なく完走でき、本ガイドの記述を修正する必要は
  ありませんでした（親イシュー #1426 へコメント報告）。
