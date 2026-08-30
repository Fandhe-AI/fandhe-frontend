//! styled Avatar（headless ラッパー、イシュー #684、親 #680/#681）。
//!
//! `fandhe_frontend_headless_ui::avatar`（イシュー #543/#569）の Root /
//! Image / Fallback 3 anatomy パーツと [`Avatar`] 状態機械を薄く再利用し、
//! [`stylesheet`] で既定 CSS を追加提供する。薄い委譲の根拠・スコープ外事項は
//! [`crate::dialog`]/[`crate::tooltip`] の rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、`Avatar` 型を
//! 再エクスポートしない理由）
//!
//! [`crate::tooltip`]/[`crate::popover`] は headless モジュールを
//! `pub use ...::*` で丸ごと再エクスポートするが、本モジュールは styled
//! `root`（variant クラス付与のため本モジュールで再定義、`crate::card::root`
//! と同型）と headless の自由関数 `root` が名前衝突するため、必要な識別子
//! のみを選択的に再エクスポートする。
//!
//! 状態機械 [`fandhe_frontend_headless_ui::avatar::Avatar`] は**あえて**
//! 再エクスポートしない（PR #695 Bugbot 指摘、イシュー #684）。`Avatar` は
//! `.root(attrs, children)` という inherent メソッドを持つが、これは
//! headless 自由関数 `root` へそのまま委譲するのみで `size`/`shape`
//! variant クラスを一切付与しない（[`root`] とは別の、未スタイルの実体）。
//! 本モジュールが `Avatar` を丸ごと再エクスポートすると、呼び出し側が
//! （styled 層のつもりで）`avatar_instance.root(...)` を呼んでしまい、
//! base 属性のスタイルは効くが `size`/`shape` が付与されずレイアウトが
//! 静かに崩れる事故を誘発する（Rust の可視性機構では外部型の inherent
//! メソッドだけを選択的に隠せないため、型自体を再エクスポートしないことが
//! 唯一の fail-closed な対策）。`Avatar` による状態管理・hydration が
//! 必要な呼び出し側は `fandhe_frontend_headless_ui::avatar::Avatar` を
//! 直接 import し、実際の描画は本モジュールの styled [`root`]（および
//! 再エクスポート済みの [`image`]/[`fallback`]、`status()` は
//! `Avatar::status()` から取得）を組み合わせて構築すること。
//!
//! # variant（size/shape）について
//!
//! Avatar は本クレート最初の styled 部品として、単一 recipe に 2 軸の
//! variant（[`Size`]・[`AvatarShape`]）を持つ（chakra-ui Avatar の
//! size/shape を最小構成へ縮約）。variant クラスは `root` パーツのみに
//! 付与し、`image`/`fallback` はクラスを持たない
//! （[`crate::card::root`] が variant クラスを root のみへ付与する判断と
//! 同型）。
//!
//! # `image`/`fallback` の base 規則が `display` を宣言しない理由
//!
//! headless 層（[`fandhe_frontend_headless_ui::avatar::image`]/
//! [`fandhe_frontend_headless_ui::avatar::fallback`]）は非表示側に `hidden`
//! 存在属性を付与し、UA 既定 `[hidden] { display: none }` に依存して JS
//! なし SSR の表示制御を成立させる。[`recipe`] の `image`/`fallback` base
//! 規則で `display` を宣言すると、`[data-scope][data-part]`（詳細度
//! (0,2,0)）が `[hidden]`（詳細度 (0,1,0)）に勝ってしまい表示制御が壊れる
//! （[`crate::tooltip`] の positioner 節・PR #575 Bugbot 指摘と同じ構造的な
//! 回避）。`data-state` に応じた `display: none` の明示は [`SlotRecipe::state`]
//! （[`crate::recipe::StateCondition::AttrEq`]）で `[data-state="hidden"]`
//! （詳細度 (0,3,0)）としてのみ登録し、常に `[hidden]` より詳細度で勝つ
//! ことで多層防御にする。
//!
//! # セキュリティ不変条件
//!
//! - HTML 文字列の直接組み立てを行わず、すべての出力は headless 層 →
//!   [`fandhe_frontend_core::render`] の既定エスケープを経由する
//!   （`raw_html()` の新規使用なし）。
//! - variant クラス名は [`recipe::SlotRecipe::variant_classes`] が
//!   `&'static str` enum 値から決定的に生成し、動的文字列合成を行わない。
//! - 呼び出し側 `attrs` に含まれる `class` は
//!   [`crate::class_attr::drop_class_attr`] で除去してから recipe 生成
//!   クラスと合成するため、`class` 属性は常に単一（呼び出し側からの
//!   クラス偽装・重複混入を防ぐ）。
//! - styled `root` は headless [`fandhe_frontend_headless_ui::avatar::root`]
//!   へ委譲するため、呼び出し側 `attrs` の `data-scope`/`data-part` 偽装除去
//!   （headless anatomy の fail-closed 挙動）をそのまま継承する。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `examples/headless-pre-styled-ui` の手書き avatar CSS 撤去・本モジュール
//!   への切り替えは #680 配下の後続イシューのスコープ（既存トラッキング
//!   あり）。
//! - crates.io への公開・利用側依存追随は #686 のスコープ。
//! - headless 共通型の再エクスポート整備は #685 のスコープ。
//! - 画像 `load`/`error` イベントの wasm グルーは headless 層 doc 記載済みの
//!   既存スコープ外を継承する。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{SlotRecipe, StateCondition, VariantValue};
// `Avatar` 状態機械はあえて再エクスポートしない（本モジュール冒頭の rustdoc
// 「`Avatar` 型を再エクスポートしない理由」参照）。状態管理・hydration が
// 必要な呼び出し側は `fandhe_frontend_headless_ui::avatar::Avatar` を直接 import する。
pub use fandhe_frontend_headless_ui::avatar::{fallback, image, AvatarAction, ImageStatus};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

/// [`SlotRecipe::new`] に渡す slot 一覧（`crates/headless-ui/src/avatar.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`]
/// が一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &["root", "image", "fallback"];

/// Avatar の外形（chakra-ui Avatar の `shape` variant を最小構成へ縮約）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AvatarShape {
    /// 円形（既定）。
    #[default]
    Circle,
    /// 角丸四角形。
    Rounded,
    /// 直角四角形。
    Square,
}

impl VariantValue for AvatarShape {
    fn axis(self) -> &'static str {
        "shape"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Circle => "circle",
            Self::Rounded => "rounded",
            Self::Square => "square",
        }
    }
}

/// この styled Avatar の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("avatar", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("background", "var(--fandhe-color-bg-subtle)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("overflow", "hidden"),
                decl("flex-shrink", "0"),
                decl("user-select", "none"),
            ],
        )
        .base(
            "image",
            vec![
                decl("width", "100%"),
                decl("height", "100%"),
                decl("object-fit", "cover"),
            ],
        )
        .base(
            "fallback",
            vec![
                decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
                decl("line-height", "1"),
            ],
        )
        // headless 層の `hidden` 存在属性（UA 既定 `[hidden] { display: none }`）
        // による JS なし SSR の表示制御を、`data-state="hidden"` 一致時の
        // 明示的な `display: none` で多層防御する（本モジュール冒頭の rustdoc
        // 「`image`/`fallback` の base 規則が `display` を宣言しない理由」参照）。
        .state(
            "image",
            StateCondition::AttrEq("data-state", "hidden"),
            vec![decl("display", "none")],
        )
        .state(
            "fallback",
            StateCondition::AttrEq("data-state", "hidden"),
            vec![decl("display", "none")],
        )
        // イシュー #1681: Xs/Xl は Sm(2)→Md(3)→Lg(4) の 1rem 刻み等差進行を
        // 両端へ外挿。font-size はトークン名を Size と同名の段へ 1:1 対応。
        .variant(
            crate::recipe::Size::Xs,
            "root",
            vec![
                decl("width", "1rem"),
                decl("height", "1rem"),
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
            ],
        )
        .variant(
            crate::recipe::Size::Sm,
            "root",
            vec![
                decl("width", "2rem"),
                decl("height", "2rem"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
            ],
        )
        .variant(
            crate::recipe::Size::Md,
            "root",
            vec![
                decl("width", "3rem"),
                decl("height", "3rem"),
                decl("font-size", "var(--fandhe-font-font-size-md)"),
            ],
        )
        .variant(
            crate::recipe::Size::Lg,
            "root",
            vec![
                decl("width", "4rem"),
                decl("height", "4rem"),
                decl("font-size", "var(--fandhe-font-font-size-lg)"),
            ],
        )
        .variant(
            crate::recipe::Size::Xl,
            "root",
            vec![
                decl("width", "5rem"),
                decl("height", "5rem"),
                decl("font-size", "var(--fandhe-font-font-size-xl)"),
            ],
        )
        .default_variant(crate::recipe::Size::Md)
        .variant(
            AvatarShape::Circle,
            "root",
            vec![decl("border-radius", "var(--fandhe-radius-full)")],
        )
        .variant(
            AvatarShape::Rounded,
            "root",
            vec![decl("border-radius", "var(--fandhe-radius-lg)")],
        )
        .variant(
            AvatarShape::Square,
            "root",
            vec![decl("border-radius", "0")],
        )
        .default_variant(AvatarShape::Circle)
}

/// この styled Avatar が生成する静的 CSS 全量を返す（決定的。
/// [`crate::tooltip::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size`/`shape` に応じたクラスを付与する
/// 唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は除去して
/// から合成する）。実体は
/// [`fandhe_frontend_headless_ui::avatar::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarShape};
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = avatar::root(Size::Md, AvatarShape::default(), vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="avatar" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: crate::recipe::Size,
    shape: AvatarShape,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value()), ("shape", shape.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::avatar::root(merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

    // --- anatomy ---

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(
            crate::recipe::Size::Md,
            AvatarShape::Circle,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="avatar""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        // headless anatomy の fail-closed 偽装除去（`Anatomy::part`）を
        // styled root 経由でも継承していることの回帰。
        let html = render(&root(
            crate::recipe::Size::Md,
            AvatarShape::Circle,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="avatar""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- data-state 連動 ---

    #[test]
    fn stylesheet_links_hidden_state_to_display_none_for_image_and_fallback() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="avatar"][data-part="image"][data-state="hidden"] {
  display: none;
}
"#
        ));
        assert!(css.contains(
            r#"[data-scope="avatar"][data-part="fallback"][data-state="hidden"] {
  display: none;
}
"#
        ));
    }

    #[test]
    fn image_and_fallback_base_rules_do_not_declare_display() {
        // `[hidden]`（詳細度 (0,1,0)）に対し `[data-scope][data-part]`
        // （詳細度 (0,2,0)）が勝ってしまう回帰を防ぐ（本モジュール rustdoc
        // 「`image`/`fallback` の base 規則が `display` を宣言しない理由」）。
        let css = stylesheet();
        let image_base_start = css
            .find(r#"[data-scope="avatar"][data-part="image"] {"#)
            .expect("image base rule must exist");
        let image_base_end = css[image_base_start..]
            .find('}')
            .map(|i| image_base_start + i)
            .unwrap();
        assert!(!css[image_base_start..image_base_end].contains("display"));

        let fallback_base_start = css
            .find(r#"[data-scope="avatar"][data-part="fallback"] {"#)
            .expect("fallback base rule must exist");
        let fallback_base_end = css[fallback_base_start..]
            .find('}')
            .map(|i| fallback_base_start + i)
            .unwrap();
        assert!(!css[fallback_base_start..fallback_base_end].contains("display"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_avatar_state_machine() {
        // `Avatar` は本モジュールから再エクスポートしない（本モジュール冒頭の
        // rustdoc「`Avatar` 型を再エクスポートしない理由」参照）ため、
        // headless-ui から直接 import して state machine 契約のみ検証する。
        use fandhe_frontend_headless_ui::avatar::Avatar;

        let mut a = Avatar::default();
        assert_eq!(a.status(), ImageStatus::Loading);

        let ssr_html = render(&a.fallback(vec![], vec![text("NM")]));
        assert!(ssr_html.contains(r#"data-state="visible""#));

        assert!(dispatch(&mut a, "loaded", ""));
        let hydrate_html = render(&render_for_hydration(&a));
        assert!(hydrate_html.contains(r#"data-hydrate-status="loaded""#));

        let restored = Avatar::from_hydration_attrs(&a.hydration_attrs()).unwrap();
        assert_eq!(restored.status(), ImageStatus::Loaded);
    }

    // --- variant クラス ---

    #[test]
    fn default_variant_is_md_and_circle() {
        let html = render(&root(
            crate::recipe::Size::Md,
            AvatarShape::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-avatar--size-md"));
        assert!(html.contains("fd-avatar--shape-circle"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (crate::recipe::Size::Sm, "fd-avatar--size-sm"),
            (crate::recipe::Size::Md, "fd-avatar--size-md"),
            (crate::recipe::Size::Lg, "fd-avatar--size-lg"),
        ] {
            let html = render(&root(size, AvatarShape::Circle, vec![], vec![]));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn shape_enumeration_maps_to_expected_classes() {
        for (shape, class) in [
            (AvatarShape::Circle, "fd-avatar--shape-circle"),
            (AvatarShape::Rounded, "fd-avatar--shape-rounded"),
            (AvatarShape::Square, "fd-avatar--shape-square"),
        ] {
            let html = render(&root(crate::recipe::Size::Md, shape, vec![], vec![]));
            assert!(html.contains(class), "shape={shape:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            crate::recipe::Size::Md,
            AvatarShape::Circle,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn stylesheet_is_deterministic_and_contains_variant_selectors_and_radius_token() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains("--size-"));
        assert!(a.contains("--shape-"));
        assert!(a.contains("var(--fandhe-radius-full)"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_attrs_attribute_breakout_payload_is_escaped() {
        let html = render(&root(
            crate::recipe::Size::Md,
            AvatarShape::Circle,
            vec![("data-x", "\" onmouseover=\"alert(1)")],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn fallback_children_script_payload_is_escaped() {
        let html = render(&fallback(
            ImageStatus::Loading,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }
}
