//! styled Message（イシュー #2106、親 #2104）の決定的 CSS 出力ゴールデン
//! テスト。
//!
//! `crates/pre-styled-ui/tests/item_css.rs` と同型の golden fixture
//! テスト。`message` recipe は `root`/`avatar`/`header`/`content`/
//! `footer`/`group` の 6 slot を宣言し、role/align/loading/error は
//! headless の `data-role`/`data-align`/`data-loading`/`data-error` を
//! `AttrEq`/`Attr` で参照するのみで class ベースの軸を持たない
//! （`src/message.rs` モジュール doc「role / align / loading / error の
//! 表現」節参照）。`group` 配下の連続発言に対する raw CSS 追記
//! （`:not(:first-child)`）も golden に含む。

use fandhe_frontend_pre_styled_ui::message;

const MESSAGE_GOLDEN_CSS: &str = "[data-scope=\"message\"][data-part=\"root\"] {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  column-gap: var(--fandhe-space-3);
  row-gap: var(--fandhe-space-1);
  align-items: start;
  max-width: var(--fandhe-message-max-width, 42rem);
  min-width: 0;
  align-self: flex-start;
  color: var(--fandhe-color-fg);
  font-size: var(--fandhe-font-font-size-sm);
  --fandhe-message-content-bg: var(--fandhe-color-bg-muted);
  --fandhe-message-content-fg: var(--fandhe-color-fg);
  --fandhe-message-content-border: transparent;
}

[data-scope=\"message\"][data-part=\"avatar\"] {
  grid-column: 1;
  grid-row: 1 / span 3;
  width: var(--fandhe-space-8);
  height: var(--fandhe-space-8);
  border-radius: var(--fandhe-radius-full);
}

[data-scope=\"message\"][data-part=\"header\"] {
  grid-column: 2;
  font-size: var(--fandhe-font-font-size-xs);
  color: var(--fandhe-color-fg-muted);
  display: flex;
  gap: var(--fandhe-space-2);
}

[data-scope=\"message\"][data-part=\"content\"] {
  grid-column: 2;
  padding: var(--fandhe-space-2) var(--fandhe-space-3);
  border-radius: var(--fandhe-radius-lg);
  background: var(--fandhe-message-content-bg);
  color: var(--fandhe-message-content-fg);
  border: 1px solid var(--fandhe-message-content-border);
  min-width: 0;
  overflow-wrap: anywhere;
}

[data-scope=\"message\"][data-part=\"footer\"] {
  grid-column: 2;
  display: flex;
  gap: var(--fandhe-space-2);
  align-items: center;
  font-size: var(--fandhe-font-font-size-xs);
  color: var(--fandhe-color-fg-muted);
}

[data-scope=\"message\"][data-part=\"group\"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-1);
  list-style: none;
  margin: 0;
  padding: 0;
}

[data-scope=\"message\"][data-part=\"root\"][data-align=\"end\"] {
  align-self: flex-end;
  grid-template-columns: minmax(0, 1fr) auto;
  margin-inline-start: auto;
}

[data-scope=\"message\"][data-part=\"root\"][data-role=\"user\"] {
  --fandhe-message-content-bg: var(--fandhe-color-accent-subtle);
}

[data-scope=\"message\"][data-part=\"root\"][data-role=\"assistant\"] {
  --fandhe-message-content-bg: var(--fandhe-color-bg-muted);
}

[data-scope=\"message\"][data-part=\"root\"][data-role=\"system\"] {
  --fandhe-message-content-bg: transparent;
  --fandhe-message-content-fg: var(--fandhe-color-fg-muted);
  font-style: italic;
}

[data-scope=\"message\"][data-part=\"root\"][data-loading] {
  opacity: 0.7;
}

[data-scope=\"message\"][data-part=\"root\"][data-error] {
  --fandhe-message-content-bg: var(--fandhe-color-danger-subtle);
  --fandhe-message-content-fg: var(--fandhe-color-danger-fg-subtle);
  --fandhe-message-content-border: var(--fandhe-color-danger);
}

[data-scope=\"message\"][data-part=\"root\"][data-align=\"end\"] > [data-scope=\"message\"][data-part=\"avatar\"] {
  grid-column: 2;
}

[data-scope=\"message\"][data-part=\"root\"][data-align=\"end\"] > [data-scope=\"message\"][data-part=\"header\"] {
  grid-column: 1;
}

[data-scope=\"message\"][data-part=\"root\"][data-align=\"end\"] > [data-scope=\"message\"][data-part=\"content\"] {
  grid-column: 1;
}

[data-scope=\"message\"][data-part=\"root\"][data-align=\"end\"] > [data-scope=\"message\"][data-part=\"footer\"] {
  grid-column: 1;
}

[data-scope=\"message\"][data-part=\"root\"]:not(:has(> [data-scope=\"message\"][data-part=\"avatar\"])) {
  grid-template-columns: minmax(0, 1fr);
}

[data-scope=\"message\"][data-part=\"root\"]:not(:has(> [data-scope=\"message\"][data-part=\"avatar\"])) > [data-scope=\"message\"][data-part=\"header\"] {
  grid-column: 1;
}

[data-scope=\"message\"][data-part=\"root\"]:not(:has(> [data-scope=\"message\"][data-part=\"avatar\"])) > [data-scope=\"message\"][data-part=\"content\"] {
  grid-column: 1;
}

[data-scope=\"message\"][data-part=\"root\"]:not(:has(> [data-scope=\"message\"][data-part=\"avatar\"])) > [data-scope=\"message\"][data-part=\"footer\"] {
  grid-column: 1;
}

[data-scope=\"message\"][data-part=\"root\"]:not(:has(> [data-scope=\"message\"][data-part=\"header\"])):not(:has(> [data-scope=\"message\"][data-part=\"footer\"])) > [data-scope=\"message\"][data-part=\"avatar\"] {
  grid-row: 1 / span 1;
}

[data-scope=\"message\"][data-part=\"root\"]:has(> [data-scope=\"message\"][data-part=\"header\"]):not(:has(> [data-scope=\"message\"][data-part=\"footer\"])) > [data-scope=\"message\"][data-part=\"avatar\"] {
  grid-row: 1 / span 2;
}

[data-scope=\"message\"][data-part=\"root\"]:not(:has(> [data-scope=\"message\"][data-part=\"header\"])):has(> [data-scope=\"message\"][data-part=\"footer\"]) > [data-scope=\"message\"][data-part=\"avatar\"] {
  grid-row: 1 / span 2;
}

[data-scope=\"message\"][data-part=\"group\"] > [data-scope=\"message\"][data-part=\"root\"]:not(:first-child) {
  margin-top: calc(-1 * var(--fandhe-space-1));
}

[data-scope=\"message\"][data-part=\"group\"] > [data-scope=\"message\"][data-part=\"root\"]:not(:first-child) > [data-scope=\"message\"][data-part=\"avatar\"] {
  visibility: hidden;
}
";

/// `stylesheet()` の全文バイト一致を固定する（golden）。意図しない宣言の
/// 混入・欠落を検知する（`docs/internal/pre-styled-ui-golden-test-update-guide.md`
/// の更新手順に従う）。
#[test]
fn css_matches_golden_snapshot() {
    assert_eq!(message::stylesheet(), MESSAGE_GOLDEN_CSS);
}

/// 出力は毎回同一（非決定性のハッシュ順走査等が混入していないこと）。
#[test]
fn css_is_deterministic() {
    assert_eq!(message::stylesheet(), message::stylesheet());
}

/// `<style>` タグからの脱出シーケンス（`</style`）や生の `<` を一切
/// 含まないことを固定する（REQ-1 と独立した CSS 埋め込み文脈の不変条件）。
#[test]
fn css_never_contains_style_breakout_sequences() {
    let css = message::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

/// role/align/loading/error の state 規則が漏れなく存在することを固定
/// する。
#[test]
fn css_declares_all_role_align_loading_error_state_rules() {
    let css = message::stylesheet();
    for role in ["user", "assistant", "system"] {
        assert!(css.contains(&format!(
            "[data-scope=\"message\"][data-part=\"root\"][data-role=\"{role}\"] {{"
        )));
    }
    assert!(css.contains("[data-scope=\"message\"][data-part=\"root\"][data-align=\"end\"] {"));
    assert!(css.contains("[data-scope=\"message\"][data-part=\"root\"][data-loading] {"));
    assert!(css.contains("[data-scope=\"message\"][data-part=\"root\"][data-error] {"));
}

/// role/align/loading/error は headless の `data-*` を参照するのみで、
/// class ベースのセレクタ（`fd-message--` プレフィックス）を生成しない
/// ことを固定する。
#[test]
fn css_does_not_generate_class_based_selectors() {
    let css = message::stylesheet();
    assert!(!css.contains("fd-message--"));
    assert!(!css.contains("class="));
}

/// `group` 配下の連続発言に対する raw CSS 追記（`:not(:first-child)`）が
/// 存在することを固定する（`src/message.rs` モジュール doc「raw CSS
/// 追記の理由」節参照）。
#[test]
fn css_appends_consecutive_group_root_and_avatar_rules() {
    let css = message::stylesheet();
    assert!(css.contains(
        "[data-scope=\"message\"][data-part=\"group\"] > [data-scope=\"message\"][data-part=\"root\"]:not(:first-child) {"
    ));
    assert!(css.contains(
        "[data-scope=\"message\"][data-part=\"group\"] > [data-scope=\"message\"][data-part=\"root\"]:not(:first-child) > [data-scope=\"message\"][data-part=\"avatar\"] {"
    ));
    assert!(css.contains("visibility: hidden;"));
}

/// `root` を 2 カラム grid とし、`avatar` が `header`/`content`/`footer`
/// の行を `grid-row: 1 / span 3` で縦に貫通することで「avatar + 縦積み
/// 本体」を表現していることを固定する（`src/message.rs` モジュール doc
/// 「slot 別の意匠」節参照）。
#[test]
fn css_lays_out_root_as_two_column_grid_with_avatar_spanning_rows() {
    let css = message::stylesheet();
    assert!(css.contains("[data-scope=\"message\"][data-part=\"root\"] {\n  display: grid;"));
    assert!(css.contains("grid-template-columns: auto minmax(0, 1fr);"));
    assert!(css.contains("grid-row: 1 / span 3;"));
}

/// `data-align="end"` 時、root の `grid-template-columns` 反転に加えて
/// `avatar`/`header`/`content`/`footer` の `grid-column` も raw CSS で
/// 入れ替わることを固定する（`src/message.rs` モジュール doc「raw CSS
/// 追記の理由」節参照）。
#[test]
fn css_swaps_grid_columns_for_data_align_end() {
    let css = message::stylesheet();
    assert!(css.contains(
        "[data-scope=\"message\"][data-part=\"root\"][data-align=\"end\"] > [data-scope=\"message\"][data-part=\"avatar\"] {\n  grid-column: 2;"
    ));
    for part in ["header", "content", "footer"] {
        assert!(css.contains(&format!(
            "[data-scope=\"message\"][data-part=\"root\"][data-align=\"end\"] > [data-scope=\"message\"][data-part=\"{part}\"] {{\n  grid-column: 1;"
        )));
    }
}

/// `avatar` が省略された `root` は 2 カラム grid を単一カラムへ縮退させ、
/// 空の 1 列目との境界に常時発生していた `column-gap`（アバターなし
/// メッセージの余分なインデント、Bugbot 指摘）を消すことを固定する
/// （`src/message.rs` モジュール doc「raw CSS 追記の理由」節参照）。
#[test]
fn css_collapses_root_to_single_column_when_avatar_is_absent() {
    let css = message::stylesheet();
    assert!(css.contains(
        "[data-scope=\"message\"][data-part=\"root\"]:not(:has(> [data-scope=\"message\"][data-part=\"avatar\"])) {\n  grid-template-columns: minmax(0, 1fr);"
    ));
    // `header`/`content`/`footer` の base 宣言は `grid-column: 2` 固定
    // のため、単一カラム化と対で列 1 へ明示的に移さないと暗黙の列 2 が
    // 生成されてしまう（`src/message.rs` モジュール doc「raw CSS 追記の
    // 理由」節参照）。
    for part in ["header", "content", "footer"] {
        assert!(css.contains(&format!(
            "[data-scope=\"message\"][data-part=\"root\"]:not(:has(> [data-scope=\"message\"][data-part=\"avatar\"])) > [data-scope=\"message\"][data-part=\"{part}\"] {{\n  grid-column: 1;"
        )));
    }
}

/// `avatar` の `grid-row` span が実在する `header`/`footer` の組み合わせ
/// （0 個・片方のみ）に応じて 1〜2 段へ動的に切り替わり、省略された分の
/// `row-gap` が下部の空行として残らないことを固定する（`src/message.rs`
/// モジュール doc「raw CSS 追記の理由」節参照。`header`/`footer` の両方が
/// 存在する既定ケースは `avatar_base` の `grid-row: 1 / span 3` のまま
/// 据え置くため、ここでは 3 パターンの上書きのみ検証する）。
#[test]
fn css_adjusts_avatar_row_span_for_missing_header_or_footer() {
    let css = message::stylesheet();
    assert!(css.contains(
        "[data-scope=\"message\"][data-part=\"root\"]:not(:has(> [data-scope=\"message\"][data-part=\"header\"])):not(:has(> [data-scope=\"message\"][data-part=\"footer\"])) > [data-scope=\"message\"][data-part=\"avatar\"] {\n  grid-row: 1 / span 1;"
    ));
    assert!(css.contains(
        "[data-scope=\"message\"][data-part=\"root\"]:has(> [data-scope=\"message\"][data-part=\"header\"]):not(:has(> [data-scope=\"message\"][data-part=\"footer\"])) > [data-scope=\"message\"][data-part=\"avatar\"] {\n  grid-row: 1 / span 2;"
    ));
    assert!(css.contains(
        "[data-scope=\"message\"][data-part=\"root\"]:not(:has(> [data-scope=\"message\"][data-part=\"header\"])):has(> [data-scope=\"message\"][data-part=\"footer\"]) > [data-scope=\"message\"][data-part=\"avatar\"] {\n  grid-row: 1 / span 2;"
    ));
}
