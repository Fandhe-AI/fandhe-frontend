//! 全部品 CSS の集約出力。
//!
//! pre-styled-ui `Theme::to_css`（`crates/pre-styled-ui/src/theme.rs`）と
//! 同型の役割を、wireframe-ui では [`wireframe_css`] 関数が担う。

/// 各部品モジュールが自分の `pub const <PART>_CSS: &str` を登録する
/// スライス（登録順 = 出力順）。Phase 1 以降の部品イシューは、自分の
/// CSS 定数をここへ 1 要素追記する以外の場所で CSS を出力してはならない
/// （`docs/design/wireframe-ui-architecture.md` §10 の追記契約）。
///
/// イシュー #2606 で最初の登録（[`crate::icon::ICON_GLYPH_CSS`]）が入った。
/// イシュー #2617 で [`crate::annotation::ANNOTATION_CSS`] が続いた。
/// イシュー #2611 で [`crate::grid::GRID_CSS`]・イシュー #2612 で
/// [`crate::divider::DIVIDER_CSS`]・イシュー #2610 で
/// [`crate::stack::STACK_CSS`]・イシュー #2618 で
/// [`crate::link::LINK_CSS`]・イシュー #2616 で
/// [`crate::rich_text::RICH_TEXT_CSS`]・イシュー #2621 で
/// [`crate::button::BUTTON_CSS`]・イシュー #2625 で
/// [`crate::checkbox::CHECKBOX_CSS`] が続いた。
pub const PARTS: &[&str] = &[
    crate::icon::ICON_GLYPH_CSS,
    crate::annotation::ANNOTATION_CSS,
    crate::grid::GRID_CSS,
    crate::divider::DIVIDER_CSS,
    crate::stack::STACK_CSS,
    crate::link::LINK_CSS,
    crate::rich_text::RICH_TEXT_CSS,
    crate::button::BUTTON_CSS,
    crate::checkbox::CHECKBOX_CSS,
];

static CSS: std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// 全部品 CSS を 1 つの `&'static str` として返す（イシューでの呼称
/// `WIREFRAME_CSS` に対応する、唯一の入口関数）。
///
/// 出力順は `:root` トークン（[`crate::tokens::css`]）→ `Size` 5 段の
/// スコープ付きカスタムプロパティ（[`crate::size::css`]）→ [`PARTS`]
/// 登録順、の順で連結する。内容は `OnceLock` により初回呼び出し時にのみ
/// 構築され、以降は同一の `&'static str` 実体を返す。
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
        out
    })
}
