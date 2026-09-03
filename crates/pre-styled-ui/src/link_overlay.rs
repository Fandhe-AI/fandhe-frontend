//! styled LinkOverlay（headless ラッパー、イシュー #756、#716 追加候補の
//! 消化）。
//!
//! `fandhe_frontend_headless_ui::link_overlay`（イシュー #756）の Root /
//! Overlay 2 anatomy パーツを薄く再利用し、[`stylesheet`] で「カード全面
//! クリック化」の既定 CSS を追加提供する。薄い委譲の根拠・スコープ外事項は
//! [`crate::breadcrumb`]/[`crate::avatar`] の rustdoc と同じ方針に従う。
//!
//! # 全面拡張の CSS 実装
//!
//! `::before` 疑似要素を使わず `overlay` 自身を展開する方式を採る理由は
//! headless 層（`crates/headless-ui/src/link_overlay.rs`）の rustdoc
//! 「全面拡張の実装方針」を参照。[`recipe`] は `root` に `position: relative`、
//! `overlay` に `position: absolute; inset: 0;` を登録する。呼び出し側は
//! `overlay` 以外の子ノード（見出し・画像等）で `root` の高さを確立する
//! 契約を維持する。
//!
//! # セキュリティ不変条件
//!
//! [`crate::link`] と同じ（headless 層 → [`fandhe_frontend_core::render`]
//! の既定エスケープを必ず経由し、`raw_html()` の新規使用なし、`href` の URL
//! スキーム検証は headless 層が担う）。
//!
//! # イシュー #1580 の参照サイト比較（7 軸チェック）
//!
//! 参照サイト（chakra-ui `LinkBox`/`LinkOverlay`。ark-ui / Radix Primitives /
//! Radix Themes には対応する部品が存在しない、`docs/design/
//! component-coverage-map.md` 参照）と 7 軸（サイズ / バリアント / 色 /
//! 状態 `data-*` / ダーク / フォーカス / 余白・角丸・影）で比較した結果:
//!
//! - **サイズ / バリアント / 色 / 状態 `data-*`**: chakra の `LinkOverlay`
//!   自身はいずれの軸も持たない（見た目はコンテナ `LinkBox` 側の責務）ため
//!   本部品でも追加しない。
//! - **フォーカス（是正）**: `overlay` に `:focus-visible` のリングが
//!   未定義（UA 既定 outline 任せ）だったため是正する。本部品は
//!   `ColorPalette` 軸を持たないため
//!   [`crate::recipe::FocusRingColor::Token`] を使う（`docs/design/
//!   pre-styled-ui-focus-ring-and-size-conventions.md` §6 手順 2）。
//!   `overlay` は `root` 全面に `inset: 0` で展開されるため
//!   [`crate::recipe::FocusRingOffset::Outside`] が chakra の LinkBox 外周
//!   リングと一致する。
//! - **余白・角丸・影**: `overlay` 自身は余白・影を持たないが、`card` 等の
//!   角丸を持つ `root` と合成した際にフォーカスリングが角丸に沿うよう
//!   `border-radius: inherit` を追加する。
//! - **hover（意図的非採用）**: `overlay` は `z-index: 0` で通常フローの
//!   子要素（見出し・本文）より**上**に positioned されるため、
//!   `hover_surface_declarations` 相当の背景を敷くとカード内容を覆い隠す。
//!   chakra の `LinkOverlay` 自身も hover 装飾を持たない。
//! - **disabled（意図的非採用）**: `<a>` に disabled 概念がない（[`crate::link`]
//!   と同じ判断）。
//! - **transition（意図的非採用）**: 遷移する視覚属性を追加しないため不要。
//! - **入れ子リンクの前面化（構造的に非採用）**: chakra は `LinkBox` 側で
//!   `a[href]:not(overlay) { z-index: 1 }` の子孫セレクタ規則を持つが、
//!   [`SlotRecipe`] は子孫セレクタに対応しない（イシュー #708 で確定済み）
//!   ため表現できない。headless 層 rustdoc の「呼び出し側責務」記述を維持
//!   する。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `examples/headless-pre-styled-ui` の追随・crates.io への公開は公開
//!   イシュー側のスコープ。
//! - `root` 内に `overlay` 以外の対話要素を配置する場合の z-index 調整は
//!   呼び出し側の責務（headless 層 rustdoc 参照）。
//! - 入れ子リンクの前面化（chakra `LinkBox` 相当）: `SlotRecipe` の子孫
//!   セレクタ非対応により構造的に表現不可（上記 7 軸チェック参照）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    focus_ring_declarations, FocusRingColor, FocusRingOffset, SlotRecipe, StateCondition,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::link_overlay::overlay;

/// [`SlotRecipe::new`] に渡す slot 一覧（`crates/headless-ui/src/link_overlay.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約）。
const SLOTS: &[&str] = &["root", "overlay"];

/// この styled LinkOverlay の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("link-overlay", SLOTS)
        .base("root", vec![decl("position", "relative")])
        .base(
            "overlay",
            vec![
                decl("position", "absolute"),
                decl("inset", "0"),
                decl("z-index", "0"),
                // card 等の角丸 root と合成した際にフォーカスリングが
                // 角丸に沿うようにする（イシュー #1580）。
                decl("border-radius", "inherit"),
                decl("cursor", "pointer"),
            ],
        )
        // イシュー #1580: 本部品は ColorPalette 軸を持たないため Token /
        // 外側リングを使う（モジュール冒頭 rustdoc 参照）。
        .state(
            "overlay",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
}

/// この styled LinkOverlay が生成する静的 CSS 全量を返す（決定的。
/// [`crate::avatar::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled `root` パーツ（位置決めコンテキスト）を組み立てる。呼び出し側
/// `attrs` の `class` は [`drop_class_attr`] で除去する（本部品は `root` に
/// variant クラスを持たないが、他 styled 部品との一貫性のため同様に扱う）。
/// 実体は [`fandhe_frontend_headless_ui::link_overlay::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::link_overlay;
///
/// let node = link_overlay::root(vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="link-overlay" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::link_overlay::root(drop_class_attr(attrs), children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(vec![], vec![]));
        assert!(html.contains(r#"data-scope="link-overlay""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn reexported_overlay_outputs_expected_tag() {
        let html = render(&overlay("/docs/next", vec![], vec![text("Next")]));
        assert!(html.contains("<a"));
        assert!(html.contains(r#"href="/docs/next""#));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="link-overlay""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn stylesheet_is_deterministic_and_contains_positioning_declarations() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains("position: relative"));
        assert!(a.contains("position: absolute"));
        assert!(a.contains("inset: 0"));
        assert!(a.contains("border-radius: inherit"));
        assert!(a.contains(":focus-visible"));
        assert!(a.contains("outline"));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn overlay_children_script_payload_is_escaped() {
        let html = render(&overlay(
            "/docs",
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }
}
