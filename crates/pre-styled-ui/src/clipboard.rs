//! styled Clipboard（headless ラッパー、イシュー #773、親トラッキング #520）。
//!
//! `fandhe_frontend_headless_ui::clipboard`（イシュー #773）の Root / Label /
//! Control / Input / Trigger / Indicator / ValueText 7 anatomy パーツを薄く
//! 再利用し、[`stylesheet`] で既定 CSS を追加提供する。薄い委譲の根拠・
//! スコープ外事項は [`crate::breadcrumb`]/[`crate::nav_list`] の rustdoc と
//! 同じ方針に従う。
//!
//! # 選択的 re-export（`root` のみ再定義する理由）
//!
//! [`crate::nav_list`]/[`crate::breadcrumb`] と同型で、styled `root`
//! （呼び出し側 `class` を [`drop_class_attr`] で除去する唯一のパーツ）と
//! headless の自由関数 `root` が名前衝突するため、それ以外のパーツ
//! （[`label`]/[`control`]/[`input`]/[`trigger`]/[`indicator`]/
//! [`value_text`]）のみを選択的に再エクスポートする。
//!
//! `Clipboard` 状態機械はあえて再エクスポートしない（[`crate::avatar`]/
//! [`crate::switch`] と同じ理由）。状態管理・hydration が必要な呼び出し側は
//! `fandhe_frontend_headless_ui::clipboard::Clipboard` を直接 import する。
//!
//! # variant を提供しない判断
//!
//! [`crate::lib`] rustdoc「複合部品の variant 統一方針」節が挙げる
//! `size`/`color-palette` は本イシューでは提供しない
//! （[`crate::hover_card`]/[`crate::toggle_tip`] と同じ判断。variant 展開は
//! 別イシューで一括検討する）。
//!
//! # Indicator の可視性切り替え（`data-state` + `hidden`）
//!
//! headless 層（[`fandhe_frontend_headless_ui::clipboard::indicator`]）は
//! 非表示側の変種に `hidden` 存在属性を付与し、UA 既定
//! `[hidden] { display: none }` に依存して JS なし SSR の表示制御を成立
//! させる。[`recipe`] の `indicator` base 規則で `display` を宣言すると
//! `[data-scope][data-part]`（詳細度 (0,2,0)）が `[hidden]`（詳細度
//! (0,1,0)）に勝ってしまい表示制御が壊れるため、`display` は base では
//! 宣言せず、`data-state="hidden"` 一致時の `display: none`
//! （[`crate::recipe::StateCondition::AttrEq`]、詳細度 (0,3,0)）としてのみ
//! 多層防御で登録する（[`crate::avatar`] の image/fallback と同型のパターン、
//! モジュール rustdoc 参照）。
//!
//! # セキュリティ不変条件
//!
//! - HTML 文字列の直接組み立てを行わず、すべての出力は headless 層 →
//!   [`fandhe_frontend_core::render`] の既定エスケープを経由する
//!   （`raw_html()` の新規使用なし）。
//! - 呼び出し側 `attrs` に含まれる `class` は [`drop_class_attr`] で
//!   [`root`] から除去する（呼び出し側からのクラス偽装混入を防ぐ、
//!   [`crate::nav_list::root`] と同じ判断）。
//! - styled [`root`] は headless
//!   [`fandhe_frontend_headless_ui::clipboard::root`] へ委譲するため、
//!   呼び出し側 `attrs` の `data-scope`/`data-part` 偽装除去（headless
//!   anatomy の fail-closed 挙動）をそのまま継承する。
//! - コピー対象値（`value`）はパスワード等の機微情報を含みうるため、
//!   本モジュールは `value` を CSS・ログのいずれにも出力しない
//!   （headless 層の既存不変条件をそのまま継承、
//!   `crates/headless-ui/src/clipboard.rs` rustdoc 参照）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `examples/headless-pre-styled-ui` の追随・crates.io への公開は公開
//!   イシュー側のスコープ。
//! - `navigator.clipboard` 実配線・タイムアウトによる自動リセットは
//!   `fandhe-frontend-wasm-full`（#773 後続）のスコープ。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{SlotRecipe, StateCondition};
pub use fandhe_frontend_headless_ui::clipboard::{
    control, indicator, input, label, trigger, value_text, ClipboardAction,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

/// [`SlotRecipe::new`] に渡す slot 一覧（`crates/headless-ui/src/clipboard.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`]
/// が一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "control",
    "input",
    "trigger",
    "indicator",
    "value-text",
];

/// この styled Clipboard の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("clipboard", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("flex-direction", "column"),
                decl("gap", "var(--fandhe-space-1)"),
            ],
        )
        .base(
            "control",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
            ],
        )
        .base(
            "input",
            vec![
                decl("flex", "1"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("padding", "var(--fandhe-space-2)"),
                decl("background", "var(--fandhe-color-bg-subtle)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("font-family", "monospace"),
            ],
        )
        .base(
            "trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("gap", "var(--fandhe-space-1)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("cursor", "pointer"),
            ],
        )
        .state(
            "trigger",
            StateCondition::Attr("data-copied"),
            vec![
                decl(
                    "border-color",
                    "var(--fandhe-color-success, var(--fandhe-color-accent))",
                ),
                decl(
                    "color",
                    "var(--fandhe-color-success, var(--fandhe-color-accent))",
                ),
            ],
        )
        .base("indicator", vec![decl("align-items", "center")])
        // headless 層の `hidden` 存在属性（UA 既定 `[hidden] { display: none }`）
        // による JS なし SSR の表示制御を、`data-state="hidden"` 一致時の
        // 明示的な `display: none` で多層防御する（本モジュール冒頭の rustdoc
        // 「Indicator の可視性切り替え」節参照）。
        .state(
            "indicator",
            StateCondition::AttrEq("data-state", "hidden"),
            vec![decl("display", "none")],
        )
        .base(
            "value-text",
            vec![
                decl("font-family", "monospace"),
                decl("word-break", "break-all"),
            ],
        )
}

/// この styled Clipboard が生成する静的 CSS 全量を返す（決定的。
/// [`crate::avatar::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled `root` パーツを組み立てる（[`drop_class_attr`] により呼び出し側の
/// `class` は除去する）。実体は
/// [`fandhe_frontend_headless_ui::clipboard::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::clipboard;
///
/// let node = clipboard::root("https://example.com", false, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="clipboard" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    value: &'a str,
    copied: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::clipboard::root(value, copied, drop_class_attr(attrs), children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

    // --- anatomy ---

    #[test]
    fn root_outputs_scope_part_and_data_value() {
        let html = render(&root("v", false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="clipboard""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-value="v""#));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            "v",
            false,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="clipboard""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn class_attr_from_caller_is_dropped() {
        let html = render(&root(
            "v",
            false,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn reexported_parts_render_expected_tags() {
        let label_html = render(&label(vec![], vec![text("Link")]));
        assert!(label_html.contains("<label"));

        let control_html = render(&control(false, vec![], vec![]));
        assert!(control_html.contains(r#"data-part="control""#));

        let input_html = render(&input("secret", false, vec![]));
        assert!(input_html.contains(r#"type="text""#));
        assert!(input_html.contains(r#"readonly="""#));

        let trigger_html = render(&trigger(false, vec![], vec![text("Copy")]));
        assert!(trigger_html.contains(r#"type="button""#));

        let indicator_html = render(&indicator(true, true, vec![], vec![]));
        assert!(indicator_html.contains(r#"data-state="visible""#));

        let value_text_html = render(&value_text(vec![], vec![text("v")]));
        assert!(value_text_html.contains(r#"data-part="value-text""#));
    }

    // --- data-copied 連動 ---

    #[test]
    fn stylesheet_links_trigger_to_copied_state() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="clipboard"][data-part="trigger"][data-copied] {"#));
    }

    #[test]
    fn stylesheet_links_hidden_state_to_display_none_for_indicator() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="clipboard"][data-part="indicator"][data-state="hidden"] {
  display: none;
}"#
        ));
    }

    #[test]
    fn indicator_base_rule_does_not_declare_display() {
        // `[hidden]`（詳細度 (0,1,0)）に対し `[data-scope][data-part]`
        // （詳細度 (0,2,0)）が勝ってしまう回帰を防ぐ（モジュール rustdoc
        // 「Indicator の可視性切り替え」節参照）。
        let css = stylesheet();
        let start = css
            .find(r#"[data-scope="clipboard"][data-part="indicator"] {"#)
            .expect("indicator base rule must exist");
        let end = css[start..].find('}').map(|i| start + i).unwrap();
        assert!(!css[start..end].contains("display"));
    }

    #[test]
    fn stylesheet_is_deterministic() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    // --- 状態機械: headless 経由の SSR/hydration 往復 ---

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_clipboard_state_machine() {
        use fandhe_frontend_headless_ui::clipboard::Clipboard;

        let mut c = Clipboard::default();
        assert!(!c.is_copied());

        let ssr_html = render(&c.root("v", vec![], vec![]));
        assert!(!ssr_html.contains("data-copied"));

        assert!(dispatch(&mut c, "copy", ""));
        let hydrate_html = render(&render_for_hydration(&c));
        assert!(hydrate_html.contains(r#"data-hydrate-copied="copied""#));

        let restored = Clipboard::from_hydration_attrs(&c.hydration_attrs()).unwrap();
        assert_eq!(restored, c);
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_data_value_attribute_breakout_payload_is_escaped() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&root(PAYLOAD, false, vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_value_text_children_are_escaped_on_render() {
        let html = render(&value_text(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
