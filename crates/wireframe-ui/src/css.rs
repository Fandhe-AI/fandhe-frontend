//! 全部品 CSS の集約出力。
//!
//! pre-styled-ui `Theme::to_css`（`crates/pre-styled-ui/src/theme.rs`）と
//! 同型の役割を、wireframe-ui では [`wireframe_css`] 関数が担う。

/// 各部品モジュールが自分の `pub const <PART>_CSS: &str` を登録する
/// スライス（登録順 = 出力順）。Phase 1 以降の部品イシューは、自分の
/// CSS 定数をここへ 1 要素追記する以外の場所で CSS を出力してはならない
/// （`docs/design/wireframe-ui-architecture.md` §10 の追記契約）。
///
/// **例外（`size::SCALE` から動的に導出する生成 CSS）**: [`crate::size::css`]
/// （既存、`Size` 5 段のスコープ付きカスタムプロパティ）と
/// [`crate::frame::frame_padding_css`]（イシュー #2609/PR #2679、Frame の
/// padding 5 段）は `pub const <PART>_CSS: &str` として `const` 化できない
/// （値が `size::SCALE` という実行時に走査する配列から導出されるため）。
/// この 2 つに限り [`wireframe_css`] が `PARTS` を経由せず個別に連結する
/// 唯一の許容経路とする。新規の非 `const` CSS 生成関数を追加する場合は
/// 「`size::SCALE` 等の共有値表から動的に導出する必要がある」ことを
/// 追加条件とし、単に `const` 化が面倒という理由での逸脱は許容しない。
/// **未反映の残課題**: `docs/design/wireframe-ui-architecture.md` §10.4
/// の追記契約本文はこの例外を明文化していない（本 PR のスコープは
/// `crates/wireframe-ui/` に限定されるため）。同文書側の追随は
/// docs-writer への別途委譲が必要。
///
/// イシュー #2606 で最初の登録（[`crate::icon::ICON_GLYPH_CSS`]）が入った。
/// イシュー #2617 で [`crate::annotation::ANNOTATION_CSS`] が続いた。
/// イシュー #2611 で [`crate::grid::GRID_CSS`]・イシュー #2612 で
/// [`crate::divider::DIVIDER_CSS`]・イシュー #2610 で
/// [`crate::stack::STACK_CSS`]・イシュー #2618 で
/// [`crate::link::LINK_CSS`]・イシュー #2616 で
/// [`crate::rich_text::RICH_TEXT_CSS`]・イシュー #2621 で
/// [`crate::button::BUTTON_CSS`]・イシュー #2624 で
/// [`crate::select::SELECT_CSS`]・イシュー #2626 で
/// [`crate::radio::RADIO_CSS`]・イシュー #2627 で
/// [`crate::switch::SWITCH_CSS`]・イシュー #2625 で
/// [`crate::checkbox::CHECKBOX_CSS`]・イシュー #2615 で
/// [`crate::paragraph::PARAGRAPH_CSS`]・イシュー #2623 で
/// [`crate::textarea::TEXTAREA_CSS`]・イシュー #2628 で
/// [`crate::slider::SLIDER_CSS`]・イシュー #2609 で
/// [`crate::frame::FRAME_CSS`]・イシュー #2619 で
/// [`crate::tag::TAG_CSS`]・イシュー #2622 で
/// [`crate::input::INPUT_CSS`]・イシュー #2630 で
/// [`crate::question::QUESTION_CSS`]・イシュー #2631 で
/// [`crate::ratings::RATINGS_CSS`]・イシュー #2632 で
/// [`crate::calendar::CALENDAR_CSS`]・イシュー #2633 で
/// [`crate::file_drop::FILE_DROP_CSS`]・イシュー #2614 で
/// [`crate::text::TEXT_CSS`]・イシュー #2638 で
/// [`crate::tabs::TABS_CSS`]・イシュー #2634 で
/// [`crate::stepper::STEPPER_CSS`]・イシュー #2636 で
/// [`crate::nav_item::NAV_ITEM_CSS`]・イシュー #2640 で
/// [`crate::pagination::PAGINATION_CSS`] が続いた。
pub const PARTS: &[&str] = &[
    crate::icon::ICON_GLYPH_CSS,
    crate::annotation::ANNOTATION_CSS,
    crate::grid::GRID_CSS,
    crate::divider::DIVIDER_CSS,
    crate::stack::STACK_CSS,
    crate::link::LINK_CSS,
    crate::rich_text::RICH_TEXT_CSS,
    crate::button::BUTTON_CSS,
    crate::select::SELECT_CSS,
    crate::radio::RADIO_CSS,
    crate::switch::SWITCH_CSS,
    crate::checkbox::CHECKBOX_CSS,
    crate::paragraph::PARAGRAPH_CSS,
    crate::textarea::TEXTAREA_CSS,
    crate::slider::SLIDER_CSS,
    crate::frame::FRAME_CSS,
    crate::tag::TAG_CSS,
    crate::input::INPUT_CSS,
    crate::question::QUESTION_CSS,
    crate::ratings::RATINGS_CSS,
    crate::calendar::CALENDAR_CSS,
    crate::file_drop::FILE_DROP_CSS,
    crate::text::TEXT_CSS,
    crate::tabs::TABS_CSS,
    crate::stepper::STEPPER_CSS,
    crate::nav_item::NAV_ITEM_CSS,
    crate::pagination::PAGINATION_CSS,
];

static CSS: std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// 全部品 CSS を 1 つの `&'static str` として返す（イシューでの呼称
/// `WIREFRAME_CSS` に対応する、唯一の入口関数）。
///
/// 出力順は `:root` トークン（[`crate::tokens::css`]）→ `Size` 5 段の
/// スコープ付きカスタムプロパティ（[`crate::size::css`]）→ [`PARTS`]
/// 登録順 → Frame の padding 5 段（[`crate::frame::frame_padding_css`]）、
/// の順で連結する。内容は `OnceLock` により初回呼び出し時にのみ構築され、
/// 以降は同一の `&'static str` 実体を返す。
///
/// Frame の padding は [`crate::size::css`] と同じく [`PARTS`] を経由
/// しない（[`crate::frame::FRAME_CSS`] は `const` のため `size::SCALE`
/// から動的に導出する padding 値を持てず、`crate::frame::frame_padding_css`
/// が別途生成する。padding 値の複製を避けるための設計、コードレビュー
/// 指摘、イシュー #2609/PR #2679）。
pub fn wireframe_css() -> &'static str {
    CSS.get_or_init(|| {
        let mut out = String::new();
        out.push_str(&crate::tokens::css());
        out.push('\n');
        out.push_str(&crate::size::css());
        for part in PARTS {
            out.push('\n');
            out.push_str(part);
        }
        out.push('\n');
        out.push_str(&crate::frame::frame_padding_css());
        out
    })
}
