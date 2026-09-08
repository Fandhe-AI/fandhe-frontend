//! styled Button Group（イシュー #2060、親 #2058。headless 側 anatomy は
//! #2059）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/input_group_css.rs` と同型の golden fixture
//! テスト。`button_group` recipe は `root`/`separator`/`text` の 3 slot を
//! 宣言し、変数（size/variant/color-palette）を一切持たない
//! （`src/button_group.rs` モジュール doc「variant 軸: 持たない」節参照）。

use fandhe_frontend_pre_styled_ui::button_group;

const BUTTON_GROUP_GOLDEN_CSS: &str = "[data-scope=\"button-group\"][data-part=\"root\"] {
  display: inline-flex;
  align-items: stretch;
  width: fit-content;
  max-width: 100%;
  min-width: 0;
  box-sizing: border-box;
}

[data-scope=\"button-group\"][data-part=\"separator\"] {
  background: var(--fandhe-color-border);
  align-self: stretch;
  flex-shrink: 0;
  margin: 0;
}

[data-scope=\"button-group\"][data-part=\"text\"] {
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-2);
  box-sizing: border-box;
  padding: 0 var(--fandhe-space-4);
  background: var(--fandhe-color-bg-muted);
  color: var(--fandhe-color-fg);
  font-size: var(--fandhe-font-font-size-sm);
  font-weight: var(--fandhe-font-font-weight-medium);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  box-shadow: var(--fandhe-shadow-xs);
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"vertical\"] {
  flex-direction: column;
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"horizontal\"] {
  flex-direction: row;
}

[data-scope=\"button-group\"][data-part=\"separator\"][data-orientation=\"vertical\"] {
  width: 1px;
}

[data-scope=\"button-group\"][data-part=\"separator\"][data-orientation=\"horizontal\"] {
  height: 1px;
  width: 100%;
}

[data-scope=\"button-group\"][data-part=\"root\"] > [data-scope=\"menu\"][data-part=\"root\"] {
  display: inline-flex;
  align-self: stretch;
}

[data-scope=\"button-group\"][data-part=\"root\"] > [data-scope=\"select\"][data-part=\"root\"] {
  display: inline-flex;
  align-self: stretch;
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"horizontal\"] > [data-scope=\"button\"][data-part=\"root\"]:not(:first-child) {
  border-start-start-radius: 0;
  border-end-start-radius: 0;
  border-inline-start-width: 0;
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"horizontal\"] > [data-scope=\"button\"][data-part=\"root\"]:not(:last-child) {
  border-start-end-radius: 0;
  border-end-end-radius: 0;
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"horizontal\"] > [data-scope=\"field\"][data-part=\"input\"]:not(:first-child) {
  border-start-start-radius: 0;
  border-end-start-radius: 0;
  border-inline-start-width: 0;
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"horizontal\"] > [data-scope=\"field\"][data-part=\"input\"]:not(:last-child) {
  border-start-end-radius: 0;
  border-end-end-radius: 0;
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"horizontal\"] > [data-scope=\"button-group\"][data-part=\"text\"]:not(:first-child) {
  border-start-start-radius: 0;
  border-end-start-radius: 0;
  border-inline-start-width: 0;
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"horizontal\"] > [data-scope=\"button-group\"][data-part=\"text\"]:not(:last-child) {
  border-start-end-radius: 0;
  border-end-end-radius: 0;
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"horizontal\"] > [data-scope=\"menu\"][data-part=\"root\"]:not(:first-child) > [data-scope=\"menu\"][data-part=\"trigger\"] {
  border-start-start-radius: 0;
  border-end-start-radius: 0;
  border-inline-start-width: 0;
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"horizontal\"] > [data-scope=\"menu\"][data-part=\"root\"]:not(:last-child) > [data-scope=\"menu\"][data-part=\"trigger\"] {
  border-start-end-radius: 0;
  border-end-end-radius: 0;
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"horizontal\"] > [data-scope=\"select\"][data-part=\"root\"]:not(:first-child) > [data-scope=\"select\"][data-part=\"control\"] > [data-scope=\"select\"][data-part=\"trigger\"] {
  border-start-start-radius: 0;
  border-end-start-radius: 0;
  border-inline-start-width: 0;
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"horizontal\"] > [data-scope=\"select\"][data-part=\"root\"]:not(:last-child) > [data-scope=\"select\"][data-part=\"control\"] > [data-scope=\"select\"][data-part=\"trigger\"] {
  border-start-end-radius: 0;
  border-end-end-radius: 0;
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"horizontal\"] > [data-scope=\"field\"][data-part=\"input\"] {
  flex: 1 1 0%;
  min-width: 0;
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"horizontal\"] > [data-scope=\"button-group\"][data-part=\"root\"]:not(:first-child) {
  margin-inline-start: var(--fandhe-space-2);
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"vertical\"] > [data-scope=\"button\"][data-part=\"root\"]:not(:first-child) {
  border-start-start-radius: 0;
  border-start-end-radius: 0;
  border-block-start-width: 0;
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"vertical\"] > [data-scope=\"button\"][data-part=\"root\"]:not(:last-child) {
  border-end-start-radius: 0;
  border-end-end-radius: 0;
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"vertical\"] > [data-scope=\"field\"][data-part=\"input\"]:not(:first-child) {
  border-start-start-radius: 0;
  border-start-end-radius: 0;
  border-block-start-width: 0;
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"vertical\"] > [data-scope=\"field\"][data-part=\"input\"]:not(:last-child) {
  border-end-start-radius: 0;
  border-end-end-radius: 0;
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"vertical\"] > [data-scope=\"button-group\"][data-part=\"text\"]:not(:first-child) {
  border-start-start-radius: 0;
  border-start-end-radius: 0;
  border-block-start-width: 0;
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"vertical\"] > [data-scope=\"button-group\"][data-part=\"text\"]:not(:last-child) {
  border-end-start-radius: 0;
  border-end-end-radius: 0;
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"vertical\"] > [data-scope=\"menu\"][data-part=\"root\"]:not(:first-child) > [data-scope=\"menu\"][data-part=\"trigger\"] {
  border-start-start-radius: 0;
  border-start-end-radius: 0;
  border-block-start-width: 0;
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"vertical\"] > [data-scope=\"menu\"][data-part=\"root\"]:not(:last-child) > [data-scope=\"menu\"][data-part=\"trigger\"] {
  border-end-start-radius: 0;
  border-end-end-radius: 0;
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"vertical\"] > [data-scope=\"select\"][data-part=\"root\"]:not(:first-child) > [data-scope=\"select\"][data-part=\"control\"] > [data-scope=\"select\"][data-part=\"trigger\"] {
  border-start-start-radius: 0;
  border-start-end-radius: 0;
  border-block-start-width: 0;
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"vertical\"] > [data-scope=\"select\"][data-part=\"root\"]:not(:last-child) > [data-scope=\"select\"][data-part=\"control\"] > [data-scope=\"select\"][data-part=\"trigger\"] {
  border-end-start-radius: 0;
  border-end-end-radius: 0;
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"vertical\"] > [data-scope=\"menu\"][data-part=\"root\"] > [data-scope=\"menu\"][data-part=\"trigger\"] {
  width: 100%;
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"vertical\"] > [data-scope=\"select\"][data-part=\"root\"] > [data-scope=\"select\"][data-part=\"control\"] {
  width: 100%;
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"vertical\"] > [data-scope=\"select\"][data-part=\"root\"] > [data-scope=\"select\"][data-part=\"control\"] > [data-scope=\"select\"][data-part=\"trigger\"] {
  width: 100%;
}

[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"vertical\"] > [data-scope=\"button-group\"][data-part=\"root\"]:not(:first-child) {
  margin-block-start: var(--fandhe-space-2);
}

[data-scope=\"button-group\"][data-part=\"root\"] > [data-scope=\"button\"][data-part=\"root\"]:focus-visible {
  position: relative;
  z-index: 1;
}

[data-scope=\"button-group\"][data-part=\"root\"] > [data-scope=\"field\"][data-part=\"input\"]:focus-visible {
  position: relative;
  z-index: 1;
}

[data-scope=\"button-group\"][data-part=\"root\"] > [data-scope=\"menu\"][data-part=\"root\"] > [data-scope=\"menu\"][data-part=\"trigger\"]:focus-visible {
  position: relative;
  z-index: 1;
}

[data-scope=\"button-group\"][data-part=\"root\"] > [data-scope=\"select\"][data-part=\"root\"] > [data-scope=\"select\"][data-part=\"control\"] > [data-scope=\"select\"][data-part=\"trigger\"]:focus-visible {
  position: relative;
  z-index: 1;
}
";

#[test]
fn button_group_css_matches_golden_fixture() {
    assert_eq!(button_group::stylesheet(), BUTTON_GROUP_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(button_group::stylesheet(), button_group::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = button_group::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

/// horizontal/vertical それぞれの向きで `flex-direction` の state 規則が
/// 明示されることを固定する（`src/button_group.rs` モジュール doc
/// 「headless は `data-orientation` を常に出力する」節参照。連結順序への
/// 依存を避けるため horizontal も明示する）。
#[test]
fn css_declares_both_orientation_flex_direction_rules() {
    let css = button_group::stylesheet();
    assert!(css.contains(
        "[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"vertical\"] {\n  flex-direction: column;\n}"
    ));
    assert!(css.contains(
        "[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"horizontal\"] {\n  flex-direction: row;\n}"
    ));
}

/// `separator` の向き別規則（グループ自身と直交する headless
/// `data-orientation` をそのままセレクタに使う）を固定する。
#[test]
fn css_declares_separator_orientation_rules() {
    let css = button_group::stylesheet();
    assert!(css.contains(
        "[data-scope=\"button-group\"][data-part=\"separator\"][data-orientation=\"vertical\"] {\n  width: 1px;\n}"
    ));
    assert!(css.contains(
        "[data-scope=\"button-group\"][data-part=\"separator\"][data-orientation=\"horizontal\"] {\n  height: 1px;\n  width: 100%;\n}"
    ));
}

/// 角丸連結・境界線二重描画解消の raw CSS 追記が、明示列挙した 5 種の
/// 直接の子（button/input/text/menu-trigger/select-trigger）すべてに
/// horizontal/vertical 双方で存在することを固定する（`src/button_group.rs`
/// モジュール doc「raw CSS 追記の理由」節参照）。
#[test]
fn css_declares_connectable_child_radius_and_border_rules_for_both_orientations() {
    let css = button_group::stylesheet();
    // (direct_child, descendant_suffix)。位置擬似クラスは常に direct 側へ
    // 付与し、そこから suffix で子孫（menu/select の trigger）へ降りる
    // （`src/button_group.rs` の `connectable_target` doc「位置擬似クラス
    // は直接の子へ付与する」節参照。擬似クラスを子孫の末尾へ付けると
    // trigger が常にその親の唯一の子のため never-match の dead CSS に
    // なる、実装時に発見・修正した回帰）。
    let targets = [
        ("[data-scope=\"button\"][data-part=\"root\"]", ""),
        ("[data-scope=\"field\"][data-part=\"input\"]", ""),
        ("[data-scope=\"button-group\"][data-part=\"text\"]", ""),
        (
            "[data-scope=\"menu\"][data-part=\"root\"]",
            " > [data-scope=\"menu\"][data-part=\"trigger\"]",
        ),
        (
            "[data-scope=\"select\"][data-part=\"root\"]",
            " > [data-scope=\"select\"][data-part=\"control\"] > [data-scope=\"select\"][data-part=\"trigger\"]",
        ),
    ];
    for orientation in ["horizontal", "vertical"] {
        let width_prop = if orientation == "horizontal" {
            "border-inline-start-width"
        } else {
            "border-block-start-width"
        };
        for (direct, suffix) in targets {
            let not_first = format!(
                "[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"{orientation}\"] > {direct}:not(:first-child){suffix}"
            );
            assert!(
                css.contains(&not_first),
                "missing not(:first-child) rule for {direct}{suffix} ({orientation})"
            );
            let not_last = format!(
                "[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"{orientation}\"] > {direct}:not(:last-child){suffix}"
            );
            assert!(
                css.contains(&not_last),
                "missing not(:last-child) rule for {direct}{suffix} ({orientation})"
            );
        }
        assert!(css.contains(width_prop));
    }

    // 回帰防止: 擬似クラスを子孫（trigger）の末尾へ付けた never-match な
    // 形は出力されないことを固定する。
    assert!(!css.contains("[data-scope=\"menu\"][data-part=\"trigger\"]:not(:first-child)"));
    assert!(!css.contains("[data-scope=\"select\"][data-part=\"trigger\"]:not(:first-child)"));
}

/// menu/select は直接の子（`root`、中間コンテナ）自体を `display:
/// inline-flex; align-self: stretch` で伸長させ、角丸連結の対象へは
/// 含めないことを固定する（`src/button_group.rs` モジュール doc「raw CSS
/// 追記の理由」節参照）。
#[test]
fn css_stretches_menu_and_select_wrapper_root_without_radius_rules() {
    let css = button_group::stylesheet();
    assert!(css.contains(
        "[data-scope=\"button-group\"][data-part=\"root\"] > [data-scope=\"menu\"][data-part=\"root\"] {\n  display: inline-flex;\n  align-self: stretch;\n}"
    ));
    assert!(css.contains(
        "[data-scope=\"button-group\"][data-part=\"root\"] > [data-scope=\"select\"][data-part=\"root\"] {\n  display: inline-flex;\n  align-self: stretch;\n}"
    ));
    assert!(!css.contains(
        "[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"horizontal\"] > [data-scope=\"menu\"][data-part=\"root\"]:not(:first-child) {"
    ));
    assert!(!css.contains(
        "[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"horizontal\"] > [data-scope=\"select\"][data-part=\"root\"]:not(:first-child) {"
    ));
}

/// ネスト（`root` の子に別の `root`）は角丸連結対象へ含めず、代わりに
/// 先頭以外の内側グループへ間隔のみ付与することを固定する（`:has()`
/// 不採用の代替、`src/button_group.rs` モジュール doc「ネスト」節参照）。
#[test]
fn css_declares_nested_root_margin_rules_without_radius_rules() {
    let css = button_group::stylesheet();
    assert!(css.contains(
        "[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"horizontal\"] > [data-scope=\"button-group\"][data-part=\"root\"]:not(:first-child) {\n  margin-inline-start: var(--fandhe-space-2);\n}"
    ));
    assert!(css.contains(
        "[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"vertical\"] > [data-scope=\"button-group\"][data-part=\"root\"]:not(:first-child) {\n  margin-block-start: var(--fandhe-space-2);\n}"
    ));
    assert!(!css.contains(
        "[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"horizontal\"] > [data-scope=\"button-group\"][data-part=\"root\"]:not(:first-child) {\n  border-start-start-radius"
    ));
}

/// フォーカスリングの重なり回避（`position: relative; z-index: 1`）が
/// 実フォーカス対象（menu/select は trigger）へ付与され、中間コンテナ
/// （root）への dead CSS を生まないことを固定する。
#[test]
fn css_declares_focus_visible_z_index_rules_on_real_focus_targets_only() {
    let css = button_group::stylesheet();
    assert!(css.contains(
        "[data-scope=\"button-group\"][data-part=\"root\"] > [data-scope=\"button\"][data-part=\"root\"]:focus-visible {\n  position: relative;\n  z-index: 1;\n}"
    ));
    assert!(css.contains(
        "[data-scope=\"button-group\"][data-part=\"root\"] > [data-scope=\"menu\"][data-part=\"root\"] > [data-scope=\"menu\"][data-part=\"trigger\"]:focus-visible {\n  position: relative;\n  z-index: 1;\n}"
    ));
    assert!(css.contains(
        "[data-scope=\"button-group\"][data-part=\"root\"] > [data-scope=\"select\"][data-part=\"root\"] > [data-scope=\"select\"][data-part=\"control\"] > [data-scope=\"select\"][data-part=\"trigger\"]:focus-visible {\n  position: relative;\n  z-index: 1;\n}"
    ));
    assert!(!css.contains(
        "[data-scope=\"button-group\"][data-part=\"root\"] > [data-scope=\"menu\"][data-part=\"root\"]:focus-visible"
    ));
    assert!(!css.contains(
        "[data-scope=\"button-group\"][data-part=\"root\"] > [data-scope=\"select\"][data-part=\"root\"]:focus-visible"
    ));
    // `text` はネイティブ `<div>` でありフォーカス不能なため対象から除外
    // する（dead CSS を書かない）。
    assert!(!css.contains(
        "[data-scope=\"button-group\"][data-part=\"root\"] > [data-scope=\"button-group\"][data-part=\"text\"]:focus-visible"
    ));
}

/// 横並び時のみ、子 `field/input`（[`crate::input`] の `width: 100%`
/// 基底規則）を `flex: 1 1 0%; min-width: 0` で縮小可能にすることを固定
/// する（codex-review/Bugbot 指摘の回帰防止。`src/button_group.rs`
/// `stylesheet` 内コメント「横並び時のみ」節参照）。縦積み時は主軸が
/// block 方向のため本規則を出力しない（適用すると意味が変わるため）。
#[test]
fn css_shrinks_input_in_horizontal_orientation_only() {
    let css = button_group::stylesheet();
    assert!(css.contains(
        "[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"horizontal\"] > [data-scope=\"field\"][data-part=\"input\"] {\n  flex: 1 1 0%;\n  min-width: 0;\n}"
    ));
    assert!(!css.contains(
        "[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"vertical\"] > [data-scope=\"field\"][data-part=\"input\"] {\n  flex: 1 1 0%;"
    ));
}

/// 縦積み時、menu/select の中間ラッパー root への `align-self: stretch`
/// だけでは内側の trigger（select は `control` 経由）まで伸長が伝播しない
/// ため、`width: 100%` を明示的に伝播させることを固定する（codex-review
/// 指摘の回帰防止。`src/button_group.rs` `stylesheet` 内コメント「縦積み
/// 時、menu/select の中間ラッパー root」節参照）。横並び時は主軸が
/// inline 方向のため本規則を出力しない。
#[test]
fn css_propagates_stretch_width_into_menu_select_targets_in_vertical_orientation_only() {
    let css = button_group::stylesheet();
    assert!(css.contains(
        "[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"vertical\"] > [data-scope=\"menu\"][data-part=\"root\"] > [data-scope=\"menu\"][data-part=\"trigger\"] {\n  width: 100%;\n}"
    ));
    assert!(css.contains(
        "[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"vertical\"] > [data-scope=\"select\"][data-part=\"root\"] > [data-scope=\"select\"][data-part=\"control\"] {\n  width: 100%;\n}"
    ));
    assert!(css.contains(
        "[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"vertical\"] > [data-scope=\"select\"][data-part=\"root\"] > [data-scope=\"select\"][data-part=\"control\"] > [data-scope=\"select\"][data-part=\"trigger\"] {\n  width: 100%;\n}"
    ));
    assert!(!css.contains(
        "[data-scope=\"button-group\"][data-part=\"root\"][data-orientation=\"horizontal\"] > [data-scope=\"menu\"][data-part=\"root\"] > [data-scope=\"menu\"][data-part=\"trigger\"] {\n  width: 100%;\n}"
    ));
}
