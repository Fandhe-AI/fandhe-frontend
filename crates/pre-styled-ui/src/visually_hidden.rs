//! styled VisuallyHidden（headless ラッパー、イシュー #776、親 #766 Phase 6）。
//!
//! `fandhe_frontend_headless_ui::visually_hidden`（イシュー #776）の唯一の
//! anatomy パーツ `root` を薄く再利用し、[`css`] で clip 手法（chakra-ui の
//! VisuallyHidden 相当）の既定 CSS を追加提供する。薄い委譲の根拠・スコープ外
//! 事項は [`crate::separator`]/[`crate::skeleton`] の rustdoc と同じ方針に
//! 従う（headless 状態機械を要しない静的部品）。
//!
//! # clip 手法（[`clip_declarations`] を skip_nav と共有）
//!
//! `position: absolute` + `width`/`height: 1px` + `clip: rect(0 0 0 0)` +
//! `overflow: hidden` の組み合わせで、要素を視覚的には 1px 四方へ縮小しつつ
//! DOM 上には残す（スクリーンリーダーは DOM 上の要素として読み上げ続ける）。
//! `display: none`/`visibility: hidden` を使わない理由はまさにこの一点で、
//! それらは支援技術からも要素を除外してしまう。
//!
//! [`clip_declarations`] は `pub(crate)` として公開し、
//! [`crate::skip_nav::link`] の「focus していないときは視覚的に隠す」base
//! 宣言としても再利用する（同じ clip 手法の単一情報源、モジュール冒頭
//! rustdoc 参照）。
//!
//! ## `overflow-wrap: normal`（イシュー #1587、参考サイト基準との一致）
//!
//! Radix Primitives `VisuallyHidden`・Radix Themes（同実装の再エクスポート）・
//! ark-ui / zag-js の `visuallyHiddenStyle` はいずれも `white-space: nowrap`
//! に加えて `word-wrap: normal`（`overflow-wrap` の別名。本クレートでは
//! [`crate::card`] の `overflow-wrap: break-word` と表記を揃えるため標準名
//! `overflow-wrap` を使う）を宣言する。この部品はそれを欠いていたため追加
//! した。実害の根拠: [`crate::card::root`] は `overflow-wrap: break-word` を
//! 宣言しており、Card 内に置かれた VisuallyHidden はこれを継承する。1px
//! 四方に縮小された箱の中で `break-word` が有効なままだと、支援技術によっては
//! 単語が 1 文字ずつ改行されているものとして扱われ、読み上げが単語単位で
//! なく文字単位に分断されるおそれがある。`white-space: nowrap` だけでは
//! `break-word` の継承を打ち消せないため、`overflow-wrap: normal` を併記して
//! 祖先からの継承を明示的に遮断する。
//!
//! 意図的に参照実装へ合わせない点:
//! - chakra-ui の `border: 0` に対し `border-width: 0` を維持する（視覚的に
//!   等価な最小宣言。`border-style`/`border-color` まで上書きする必要はない）。
//! - `clip-path: inset(50%)` は Radix Primitives / Radix Themes / ark-ui の
//!   いずれにも存在しないため追加しない（`clip` は CSS Masking 仕様上
//!   非推奨だが主要ブラウザで動作し、参照実装も `clip` のみを使う）。
//!
//! # variant 軸を持たない理由
//!
//! VisuallyHidden は見た目のバリエーションを持たない単一の振る舞い
//! （常に clip される）であり、`size`/`color-palette` いずれの標準軸も
//! 意味を持たない（[`crate::separator`] が `color-palette` を持たないと
//! した判断と同型の整理）。
//!
//! # セキュリティ不変条件
//!
//! - HTML 文字列の直接組み立てを行わず、すべての出力は headless 層 →
//!   [`fandhe_frontend_core::render`] の既定エスケープを経由する
//!   （`raw_html()` の新規使用なし）。
//! - 呼び出し側 `attrs` に含まれる `class` は
//!   [`crate::class_attr::drop_class_attr`] で除去してから recipe 生成
//!   クラスと合成するため、`class` 属性は常に単一（呼び出し側からのクラス
//!   偽装・重複混入を防ぐ）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `examples/headless-pre-styled-ui` の追随・crates.io への公開は公開
//!   イシュー側のスコープ。

use crate::class_attr::drop_class_attr;
use crate::css::{decl, Declaration};
use crate::recipe::SlotRecipe;
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

/// [`SlotRecipe::new`] に渡す slot 一覧（`crates/headless-ui/src/visually_hidden.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約）。
const SLOTS: &[&str] = &["root"];

/// clip 手法の宣言列（[`crate::skip_nav`] の `link` base とも共有する単一
/// 情報源。モジュール冒頭 rustdoc「clip 手法」節参照）。
pub(crate) fn clip_declarations() -> Vec<Declaration> {
    vec![
        decl("position", "absolute"),
        decl("width", "1px"),
        decl("height", "1px"),
        decl("padding", "0"),
        decl("margin", "-1px"),
        decl("overflow", "hidden"),
        decl("clip", "rect(0, 0, 0, 0)"),
        decl("white-space", "nowrap"),
        decl("overflow-wrap", "normal"),
        decl("border-width", "0"),
    ]
}

/// この styled VisuallyHidden の既定 CSS を組み立てる（内部ヘルパ、[`css`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("visually-hidden", SLOTS).base("root", clip_declarations())
}

/// この styled VisuallyHidden が生成する静的 CSS 全量を返す（決定的。
/// [`crate::separator::css`]/[`crate::skeleton::css`] と同じ契約）。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// styled `root` パーツを組み立てる。実体は
/// [`fandhe_frontend_headless_ui::visually_hidden::root`] へ委譲する。
/// 呼び出し側の `class` は [`drop_class_attr`] で除去してから合成する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{render, text};
/// use fandhe_frontend_pre_styled_ui::visually_hidden::root;
///
/// let node = root(vec![], vec![text("補足テキスト")]);
/// let html = render(&node);
/// assert!(html.contains(r#"data-scope="visually-hidden""#));
/// ```
#[must_use]
pub fn root<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    // 本部品は variant 軸を持たない（モジュール冒頭 rustdoc「variant 軸を
    // 持たない理由」参照）ため `class` 属性自体を付与しない
    // ([`crate::link_overlay::root`] と同型。呼び出し側 `class` は
    // 一貫性のため引き続き除去する）。
    fandhe_frontend_headless_ui::visually_hidden::root(drop_class_attr(attrs), children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(vec![], vec![text("補足テキスト")]));
        assert!(html.contains(r#"data-scope="visually-hidden""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(">補足テキスト<"));
    }

    #[test]
    fn root_never_emits_aria_hidden() {
        let html = render(&root(vec![], vec![]));
        assert!(!html.contains("aria-hidden"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="visually-hidden""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn caller_class_is_dropped_and_no_class_attr_is_emitted() {
        let html = render(&root(vec![("class", "attacker-controlled")], vec![]));
        assert_eq!(html.matches("class=\"").count(), 0);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn css_output_is_deterministic_and_declares_clip_technique() {
        let a = css();
        let b = css();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="visually-hidden"][data-part="root"]"#));
        assert!(a.contains("clip: rect(0, 0, 0, 0);"));
        assert!(a.contains("overflow: hidden;"));
        assert!(a.contains("overflow-wrap: normal;"));
    }

    #[test]
    fn css_never_contains_style_breakout_sequences() {
        let out = css();
        assert!(!out.contains("</style"));
        assert!(!out.contains('<'));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn children_script_payload_is_escaped() {
        let html = render(&root(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn attrs_value_breakout_payload_is_escaped() {
        let html = render(&root(vec![("data-x", "\" onmouseover=\"alert(1)")], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }
}
