//! ScrollArea（カスタムスクロール領域）headless コンポーネント（イシュー
//! #825、`docs/design/component-coverage-map.md` 保留解除、
//! `docs/policy/intentional-non-adoption.md` §7 装飾系保留 #735 の対象）。
//!
//! ark-ui の `disclosure/scroll-area.md`・chakra-ui の `layout/scroll-area.md`
//! を参考に、CSS `overflow` を主体とするスクロール領域として Root /
//! Viewport / Content / Scrollbar / Thumb / Corner の 6 anatomy パーツを
//! 提供する。[`mod@crate::breadcrumb`]/[`mod@crate::nav_list`]/[`mod@crate::link`]
//! と同型で、開閉のような時間変化する内部状態を持たないため
//! [`mod@crate::state`] の状態機械は適用しない（自由関数のみ）。
//!
//! # 呼び出し文脈
//!
//! - 上層の [`crate::anatomy::Anatomy`]・[`crate::aria`]・[`crate::data_attrs`]
//!   へ薄く委譲するのみで、独自の出力経路・独自のエスケープ処理は持たない。
//! - `fandhe-frontend-pre-styled-ui` の `scroll_area`（イシュー #825）が本
//!   モジュールを再エクスポートし、`data-scope="scroll-area"`/`data-part="..."`
//!   セレクタを前提にスタイル（`overflow: auto` + カスタムスクロールバー
//!   表現）を当てる。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - **JS によるスクロール位置追従**（thumb の位置・サイズをスクロール量に
//!   応じて同期する処理）・**thumb の drag 操作**は本イシューのスコープ外
//!   とする。本モジュールが提供する `scrollbar`/`thumb`/`corner` パーツは
//!   将来 JS 追従を実装する際の受け皿となる静的マークアップのみであり、
//!   pre-styled-ui 側では初期実装として非表示（`display: none`）にし
//!   ネイティブスクロールバーの装飾で代替する（`crates/pre-styled-ui/src/scroll_area.rs`
//!   参照）。
//! - ネイティブスクロールバーを非表示化して独自スクロールバーへ完全に
//!   置き換える JS（`scrollbar-width: none` 相当のクロスブラウザ制御）も
//!   同じ理由で対象外。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`tabindex`）はすべて `&'static str` リテラル
//!   で固定しており、動的値が属性名スロットへ混入する経路はない
//!   （[`mod@crate::anatomy`]/[`crate::aria`]/[`crate::data_attrs`] の既存
//!   不変条件をそのまま継承する）。
//! - 呼び出し側 `attrs`/`children` の動的値はすべて
//!   [`fandhe_frontend_core::render`] の既定エスケープ（REQ-1）を必ず経由
//!   する。本モジュールは `raw_html()` を使用せず、HTML 文字列を直接組み
//!   立てない。
//! - `data-orientation` 値語彙は [`crate::data_attrs::Orientation`] に
//!   一元化されており、本モジュールで独自の値を作らない。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::aria_hidden;
use crate::data_attrs::{data_orientation, Orientation};
use fandhe_frontend_core::Node;

/// ScrollArea の anatomy（`data-scope="scroll-area"`）。
const ANATOMY: Anatomy = anatomy("scroll-area");

/// Root パーツ（`div`）。ScrollArea 全体の外枠。
#[must_use]
pub fn root(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("root", "div", attrs, children)
}

/// Viewport パーツ（`div`）。実際にスクロールする領域。
///
/// キーボードでスクロール可能な領域とする WAI 慣行（矢印キー/Page キーで
/// フォーカス済み要素をスクロールできる）に従い `tabindex="0"` を固定で
/// 付与する。読み上げ名が必要な場合は呼び出し側で `aria-label`/
/// `aria-labelledby` を `attrs` へ付与することを推奨する。
#[must_use]
pub fn viewport(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&str, &str)> = vec![("tabindex", "0")];
    merged.extend(attrs);
    ANATOMY.part("viewport", "div", merged, children)
}

/// Content パーツ（`div`）。[`viewport`] の内側に置くスクロール対象コンテンツ。
#[must_use]
pub fn content(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("content", "div", attrs, children)
}

/// Scrollbar パーツ（`div`）。カスタムスクロールバー表現の外枠。
///
/// ネイティブスクロールバー（[`viewport`] の `overflow` が生成するもの）と
/// 意味が重複する装飾要素であるため `aria-hidden="true"` を固定で付与する。
#[must_use]
pub fn scrollbar(orientation: Orientation, attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&str, &str)> = vec![aria_hidden(true), data_orientation(orientation)];
    merged.extend(attrs);
    ANATOMY.part("scrollbar", "div", merged, children)
}

/// Thumb パーツ（`div`）。[`scrollbar`] の中でスクロール位置を示すつまみ。
#[must_use]
pub fn thumb(orientation: Orientation, attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&str, &str)> = vec![data_orientation(orientation)];
    merged.extend(attrs);
    ANATOMY.part("thumb", "div", merged, children)
}

/// Corner パーツ（`div`）。横・縦の [`scrollbar`] が交差する角。
///
/// [`scrollbar`] と同じく装飾要素のため `aria-hidden="true"` を固定で付与する。
#[must_use]
pub fn corner(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&str, &str)> = vec![aria_hidden(true)];
    merged.extend(attrs);
    ANATOMY.part("corner", "div", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(vec![], vec![]));
        assert!(html.contains(r#"data-scope="scroll-area""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn viewport_has_tabindex_zero() {
        let html = render(&viewport(vec![], vec![]));
        assert!(html.contains(r#"data-part="viewport""#));
        assert!(html.contains(r#"tabindex="0""#));
    }

    #[test]
    fn content_outputs_scope_and_part() {
        let html = render(&content(vec![], vec![text("scrollable body")]));
        assert!(html.contains(r#"data-part="content""#));
        assert!(html.contains("scrollable body"));
    }

    #[test]
    fn scrollbar_has_aria_hidden_and_orientation() {
        let vertical = render(&scrollbar(Orientation::Vertical, vec![], vec![]));
        assert!(vertical.contains(r#"data-part="scrollbar""#));
        assert!(vertical.contains(r#"aria-hidden="true""#));
        assert!(vertical.contains(r#"data-orientation="vertical""#));

        let horizontal = render(&scrollbar(Orientation::Horizontal, vec![], vec![]));
        assert!(horizontal.contains(r#"data-orientation="horizontal""#));
    }

    #[test]
    fn thumb_has_orientation_and_no_aria_hidden() {
        let html = render(&thumb(Orientation::Horizontal, vec![], vec![]));
        assert!(html.contains(r#"data-part="thumb""#));
        assert!(html.contains(r#"data-orientation="horizontal""#));
        assert!(!html.contains("aria-hidden"));
    }

    #[test]
    fn corner_has_aria_hidden() {
        let html = render(&corner(vec![], vec![]));
        assert!(html.contains(r#"data-part="corner""#));
        assert!(html.contains(r#"aria-hidden="true""#));
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="scroll-area""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- XSS 回帰: attrs/children にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&viewport(vec![("data-testid", ATTR_BREAK_PAYLOAD)], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&content(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
