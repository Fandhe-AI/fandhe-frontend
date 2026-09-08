//! styled Item（イシュー #2066、親 #2064）の決定的 CSS 出力ゴールデン
//! テスト。
//!
//! `crates/pre-styled-ui/tests/input_group_css.rs` と同型の golden fixture
//! テスト。`item` recipe は `root`/`media`/`content`/`title`/`description`/
//! `actions`/`header`/`footer`/`group`/`separator` の 10 slot を宣言し、
//! variant/size は headless の `data-variant`/`data-size` を `AttrEq` で
//! 参照するのみで class ベースの軸を持たない（`src/item.rs` モジュール doc
//! 「variant / size の表現」節参照）。

use fandhe_frontend_pre_styled_ui::item;

const ITEM_GOLDEN_CSS: &str = "[data-scope=\"item\"][data-part=\"root\"] {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--fandhe-space-4);
  width: 100%;
  min-width: 0;
  box-sizing: border-box;
  padding: var(--fandhe-item-padding, var(--fandhe-space-4));
  border: 1px solid transparent;
  border-radius: var(--fandhe-radius-md);
  color: var(--fandhe-color-fg);
  font-size: var(--fandhe-font-font-size-sm);
  --fandhe-item-bg: transparent;
  background: var(--fandhe-item-bg);
  transition-property: background, border-color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope=\"item\"][data-part=\"media\"] {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  gap: var(--fandhe-space-2);
}

[data-scope=\"item\"][data-part=\"content\"] {
  display: flex;
  flex: 1 1 0%;
  flex-direction: column;
  gap: var(--fandhe-space-1);
  min-width: 0;
}

[data-scope=\"item\"][data-part=\"title\"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-2);
  font-weight: var(--fandhe-font-font-weight-medium);
  line-height: var(--fandhe-font-line-height-tight);
}

[data-scope=\"item\"][data-part=\"description\"] {
  margin: 0;
  color: var(--fandhe-color-fg-muted);
  font-size: var(--fandhe-font-font-size-sm);
  line-height: var(--fandhe-font-line-height-normal);
}

[data-scope=\"item\"][data-part=\"actions\"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-2);
}

[data-scope=\"item\"][data-part=\"header\"] {
  flex-basis: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fandhe-space-2);
}

[data-scope=\"item\"][data-part=\"footer\"] {
  flex-basis: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fandhe-space-2);
}

[data-scope=\"item\"][data-part=\"group\"] {
  display: flex;
  flex-direction: column;
}

[data-scope=\"item\"][data-part=\"separator\"] {
  border: 0;
  border-top: 1px solid var(--fandhe-color-border);
  margin: 0;
  height: 0;
  width: 100%;
}

[data-scope=\"item\"][data-part=\"root\"][data-variant=\"outline\"] {
  border-color: var(--fandhe-color-border);
}

[data-scope=\"item\"][data-part=\"root\"][data-variant=\"muted\"] {
  --fandhe-item-bg: var(--fandhe-color-bg-muted);
}

[data-scope=\"item\"][data-part=\"root\"][data-size=\"sm\"] {
  --fandhe-item-padding: var(--fandhe-space-2) var(--fandhe-space-3);
  gap: var(--fandhe-space-2);
}

[data-scope=\"item\"][data-part=\"root\"][href] {
  cursor: pointer;
  text-decoration: none;
  --fandhe-item-hover-bg: var(--fandhe-color-bg-subtle);
}

[data-scope=\"item\"][data-part=\"root\"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope=\"item\"][data-part=\"media\"][data-variant=\"icon\"] {
  width: var(--fandhe-space-8);
  height: var(--fandhe-space-8);
  background: var(--fandhe-color-bg-muted);
  border-radius: var(--fandhe-radius-sm);
}

[data-scope=\"item\"][data-part=\"media\"][data-variant=\"image\"] {
  width: var(--fandhe-space-10);
  height: var(--fandhe-space-10);
  border-radius: var(--fandhe-radius-sm);
  overflow: hidden;
}

@media (hover: hover) {
  [data-scope=\"item\"][data-part=\"root\"]:hover:not([data-disabled]) {
    background: var(--fandhe-item-hover-bg, var(--fandhe-item-bg));
  }
}

[data-scope=\"item\"][data-part=\"media\"][data-variant=\"image\"] > img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
";

#[test]
fn item_css_matches_golden_fixture() {
    assert_eq!(item::stylesheet(), ITEM_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(item::stylesheet(), item::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = item::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

/// raw CSS 追記の対象セレクタ（`media[data-variant="image"] > img` の
/// `object-fit: cover` リセット）が存在することを固定する（`src/item.rs`
/// モジュール doc「raw CSS 追記の理由」節参照）。
#[test]
fn css_appends_media_image_child_img_rule() {
    let css = item::stylesheet();
    assert!(css.contains(
        "[data-scope=\"item\"][data-part=\"media\"][data-variant=\"image\"] > img {\n  width: 100%;\n  height: 100%;\n  object-fit: cover;\n}"
    ));
}

/// `:focus-visible` によるフォーカスリングがトークン参照（palette 軸を
/// 持たないため常に `Token`）であることを固定する。
#[test]
fn css_focus_visible_ring_uses_token_color() {
    let css = item::stylesheet();
    assert!(css.contains("[data-scope=\"item\"][data-part=\"root\"]:focus-visible {"));
    assert!(css.contains("var(--fandhe-color-focus-ring, var(--fandhe-color-accent))"));
}

/// hover 規則が fallback 付き（`var(--fandhe-item-hover-bg,
/// var(--fandhe-item-bg))`）であることを固定する（`src/item.rs` モジュール
/// doc「落とし穴」節参照）。fallback を欠くと `href` を持たない `div` root
/// で背景が透明化してしまう回帰を防ぐ。
#[test]
fn css_hover_background_has_fallback_to_item_bg() {
    let css = item::stylesheet();
    assert!(css.contains(
        "[data-scope=\"item\"][data-part=\"root\"]:hover:not([data-disabled]) {\n    background: var(--fandhe-item-hover-bg, var(--fandhe-item-bg));\n  }"
    ));
}

/// variant/size は headless の `data-variant`/`data-size` を `AttrEq` で
/// 参照するのみで、class ベースの variant/size 軸を持たない
/// （`fd-item--` プレフィックスのクラスセレクタを生成しない）ことを固定
/// する。
#[test]
fn css_does_not_generate_class_based_variant_or_size_selectors() {
    let css = item::stylesheet();
    assert!(!css.contains("fd-item--"));
    assert!(!css.contains("class="));
}

/// `root`/`media` の variant・size 状態規則が漏れなく存在することを固定
/// する。
#[test]
fn css_declares_all_variant_and_size_state_rules() {
    let css = item::stylesheet();
    for variant in ["outline", "muted"] {
        assert!(css.contains(&format!(
            "[data-scope=\"item\"][data-part=\"root\"][data-variant=\"{variant}\"] {{"
        )));
    }
    assert!(css.contains("[data-scope=\"item\"][data-part=\"root\"][data-size=\"sm\"] {"));
    for media_variant in ["icon", "image"] {
        assert!(css.contains(&format!(
            "[data-scope=\"item\"][data-part=\"media\"][data-variant=\"{media_variant}\"] {{"
        )));
    }
}
