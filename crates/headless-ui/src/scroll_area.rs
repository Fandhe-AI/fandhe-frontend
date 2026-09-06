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
//!
//! # 参考サイトとの突合（イシュー #1662）
//!
//! ark-ui/Zag.js（`scroll-area.anatomy.ts`）・Radix Primitives
//! （`packages/react/scroll-area`）・chakra-ui・Radix Themes（Primitives
//! 上のスタイル層）と本モジュールの anatomy / `data-*` / ARIA /
//! キーボード操作を突合した。
//!
//! - **anatomy**: ark-ui/Zag.js（root/viewport/content/scrollbar/thumb/corner
//!   の 6 パーツ）と完全一致。Radix Primitives は `content` を持たない
//!   （viewport 内部に自前生成する実装差）が、ark-ui/chakra-ui との一致を
//!   優先し `content` を維持する。パートの増減なし（Themes 側イシュー
//!   #1584 は closed 済みで通知不要）。
//! - **`data-*`（意図的に採用しない値）**: Zag.js の `data-overflow-x/y`・
//!   `data-at-top/bottom/left/right`・`data-hover`・`data-scrolling`・
//!   `data-dragging`、Radix の `data-state="visible"|"hidden"` は、いずれも
//!   DOM 計測（overflow 有無・端到達）またはポインタ操作の実行時状態から
//!   導出される値であり、SSR の静的マークアップでは真の値を決定できない。
//!   `docs/policy/intentional-non-adoption.md` §3.25 規則 2（装飾・
//!   レイアウト計測の関心を headless-ui へ持ち込まない）と、本ファイル
//!   冒頭「スコープ外」節（#825 由来）に基づき非採用とする。固定値
//!   （例: 常に `data-state="hidden"`）を出力すると実態と乖離するため
//!   出力しない。
//! - **ARIA**: Zag.js が付与する `role="presentation"` は追加しない。
//!   viewport は `tabindex="0"` を固定付与しておりフォーカス可能なため、
//!   WAI-ARIA 1.2 §5.4「Presentational Roles Conflict Resolution」により
//!   `presentation` は UA に無視され、Radix 側（`role` 非付与）とも整合
//!   する。viewport の `tabindex="0"` 固定は維持する（SSR では overflow の
//!   有無を判定できず、WCAG 2.1.1・axe `scrollable-region-focusable` に
//!   対して安全側に倒す）。`scrollbar`/`corner` の `aria-hidden="true"` は
//!   両参照にはない本実装独自の付与だが、いずれも可読コンテンツを持たない
//!   非フォーカス装飾要素であり、ネイティブスクロールバーとの意味重複を
//!   明示する目的で維持する。
//! - **キーボード操作**: Radix docs は「ネイティブスクロールに依拠し
//!   キーボードスクロールは既定で対応、プラットフォーム差があるため個別
//!   キーは規定せず独自のキーイベントリスナも追加しない」と明記する。
//!   ark-ui/chakra-ui にもキーボード表はない。本モジュールも独自の
//!   キーハンドラを持たず、この点は参照と整合する（是正は docs-site 原稿
//!   側でネイティブキー一覧を明示する形で対応、コード変更なし）。
//! - **是正した実装上の欠陥**: 呼び出し側 `attrs` による固定属性
//!   （viewport の `tabindex`、scrollbar の `aria-hidden`/`data-orientation`、
//!   thumb の `data-orientation`、corner の `aria-hidden`）のなりすまし・
//!   重複出力を防ぐため [`drop_reserved`] を導入した（`crate::breadcrumb`
//!   と同型）。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::aria_hidden;
use crate::data_attrs::{data_orientation, Orientation};
use fandhe_frontend_core::Node;

/// ScrollArea の anatomy（`data-scope="scroll-area"`）。
const ANATOMY: Anatomy = anatomy("scroll-area");

/// [`viewport`] が固定付与する予約キー（イシュー #1662）。
const VIEWPORT_RESERVED: &[&str] = &["tabindex"];

/// [`scrollbar`] が固定付与する予約キー（イシュー #1662）。
const SCROLLBAR_RESERVED: &[&str] = &["aria-hidden", "data-orientation"];

/// [`thumb`] が固定付与する予約キー（イシュー #1662）。
const THUMB_RESERVED: &[&str] = &["data-orientation"];

/// [`corner`] が固定付与する予約キー（イシュー #1662）。
const CORNER_RESERVED: &[&str] = &["aria-hidden"];

/// 呼び出し側 `attrs` から予約キー（本モジュールが固定付与する属性名）を
/// 除去する（ASCII 大文字小文字無視の完全一致）。`fandhe_frontend_core::el`
/// は属性の重複除去をしないため、これを経由しない呼び出しは同名属性の
/// 重複出力・状態属性のなりすましを許してしまう（`crate::breadcrumb::drop_reserved`
/// と同型、イシュー #1662）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

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
    let attrs = drop_reserved(attrs, VIEWPORT_RESERVED);
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
    let attrs = drop_reserved(attrs, SCROLLBAR_RESERVED);
    let mut merged: Vec<(&str, &str)> = vec![aria_hidden(true), data_orientation(orientation)];
    merged.extend(attrs);
    ANATOMY.part("scrollbar", "div", merged, children)
}

/// Thumb パーツ（`div`）。[`scrollbar`] の中でスクロール位置を示すつまみ。
#[must_use]
pub fn thumb(orientation: Orientation, attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, THUMB_RESERVED);
    let mut merged: Vec<(&str, &str)> = vec![data_orientation(orientation)];
    merged.extend(attrs);
    ANATOMY.part("thumb", "div", merged, children)
}

/// Corner パーツ（`div`）。横・縦の [`scrollbar`] が交差する角。
///
/// [`scrollbar`] と同じく装飾要素のため `aria-hidden="true"` を固定で付与する。
#[must_use]
pub fn corner(attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, CORNER_RESERVED);
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

    // --- 予約キー除去回帰（イシュー #1662、`crate::breadcrumb` と同型） ---

    #[test]
    fn caller_reserved_keys_are_dropped_case_insensitively() {
        let viewport_html = render(&viewport(vec![("TABINDEX", "-1")], vec![]));
        assert!(viewport_html.contains(r#"tabindex="0""#));
        assert!(!viewport_html.contains(r#"tabindex="-1""#));

        let scrollbar_html = render(&scrollbar(
            Orientation::Vertical,
            vec![("Aria-Hidden", "false"), ("DATA-ORIENTATION", "horizontal")],
            vec![],
        ));
        assert!(scrollbar_html.contains(r#"aria-hidden="true""#));
        assert!(!scrollbar_html.contains(r#"aria-hidden="false""#));
        assert!(scrollbar_html.contains(r#"data-orientation="vertical""#));
        assert!(!scrollbar_html.contains(r#"data-orientation="horizontal""#));

        let thumb_html = render(&thumb(
            Orientation::Horizontal,
            vec![("Data-Orientation", "vertical")],
            vec![],
        ));
        assert!(thumb_html.contains(r#"data-orientation="horizontal""#));
        assert!(!thumb_html.contains(r#"data-orientation="vertical""#));

        let corner_html = render(&corner(vec![("ARIA-HIDDEN", "false")], vec![]));
        assert!(corner_html.contains(r#"aria-hidden="true""#));
        assert!(!corner_html.contains(r#"aria-hidden="false""#));
    }

    // --- 規則 2 ガード: 参照サイトの計測・ポインタ由来 data-* を出力しない
    //     （`docs/policy/intentional-non-adoption.md` §3.25 規則 2、イシュー #1662） ---

    #[test]
    fn no_part_outputs_measurement_or_pointer_derived_state() {
        let html = render(&root(
            vec![],
            vec![viewport(
                vec![],
                vec![
                    content(vec![], vec![text("body")]),
                    scrollbar(
                        Orientation::Vertical,
                        vec![],
                        vec![thumb(Orientation::Vertical, vec![], vec![])],
                    ),
                    corner(vec![], vec![]),
                ],
            )],
        ));
        for forbidden in [
            "data-state",
            "data-overflow-x",
            "data-overflow-y",
            "data-at-top",
            "data-at-bottom",
            "data-at-left",
            "data-at-right",
            "data-hover",
            "data-scrolling",
            "data-dragging",
            "data-ownedby",
            " dir=",
            " id=",
        ] {
            assert!(
                !html.contains(forbidden),
                "unexpected attribute `{forbidden}` in: {html}"
            );
        }
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
