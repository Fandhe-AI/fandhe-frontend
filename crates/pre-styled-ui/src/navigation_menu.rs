//! styled Navigation Menu（headless ラッパー、イシュー #993、親 #932 Phase 8）。
//!
//! `fandhe_frontend_headless_ui::navigation_menu`（イシュー #993）の Root /
//! List / Item / Trigger / Content / Link 6 anatomy パーツと
//! [`fandhe_frontend_headless_ui::navigation_menu::NavigationMenu`]
//! （[`fandhe_frontend_headless_ui::state::SingleSelect`] を埋め込んだ
//! 「高々 1 個の Trigger だけが開く」状態機械）をそのまま再エクスポートし、
//! [`stylesheet`] で既定 CSS を追加提供する（[`crate::menubar`] と同型の
//! 薄い委譲）。
//!
//! # レイアウト（トリガー行の重なり・縦ずれ回帰の予防、PR #1000 の反省）
//!
//! `item` に `position: relative` を、`content` に `position: absolute;
//! top: 100%; left: 0;` を宣言する（一般的なナビゲーションドロップダウン。
//! [`crate::menu`] の `positioner`（`absolute; top: 100%`）と同型）。
//!
//! `list` の `align-items` は **`center` ではなく `flex-start` を既定にする**。
//! トリガーの高さが揃っている通常表示では `center` と視覚的に同一だが、
//! showcase で `content` をフロー内へ中和したときに 1 項目だけ縦に伸びて
//! 他項目が縦ずれする回帰
//!（`crates/docs-site/src/showcase.rs` の `SHOWCASE_LAYOUT_CSS` が
//! `[data-scope="navigation-menu"][data-part="content"] { position: static;
//! }` で `content` を中和した際に発生しうる、PR #1000 の Menubar showcase
//! 修正 3 コミット目と同型の障害）を、`flex-start` を既定にすることで
//! 構造的に発生させない（`center` のままだと、伸びた 1 項目の高さぶん
//! flexbox が全項目を中央合わせし直し、隣接する未展開項目が上下にずれる）。
//!
//! # focus-visible リング
//!
//! `trigger` はネイティブなフォーカス可能要素（`<button>`）であり、
//! キーボード操作時のみのフォーカスリングを
//! [`crate::recipe::StateCondition::FocusVisible`] 経由で登録する
//! （[`crate::menubar`]/[`crate::toolbar`] と同じ判断）。`link` はネイティブ
//! `<a>` であり同様にフォーカス可能だが、本モジュールでは強調は
//! `data-current`（アクティブリンク）側で表現し、`:focus-visible` は
//! `trigger` のみに登録する（headless 層が `link` へ独自の highlight
//! 状態を持たないため）。
//!
//! # 担当パートの是正（イシュー #1700、親 #1530 の 1/2 分割。外枠パート
//! `list`/`item`/`trigger`/`link` のみ担当。内部パート（`content`）と
//! `data-state` 開閉トランジションは兄弟 #1701（2/2）の担当のため一切
//! 触れていない）
//!
//! 親イシュー #1530 の実測（横断的な hover フィードバック欠落・Phase 0
//! ヘルパ未経由の直書き宣言）に対し、本イシューが担当する外枠パートで
//! 実施した是正・意図的に合わせなかった点を記録する。
//!
//! - **`trigger`**: `border-radius` の生リテラル（`0.25rem`）を
//!   `var(--fandhe-radius-sm)` へトークン化（値は同一、外観不変）。
//!   [`crate::recipe::hover_bg_muted`] +
//!   [`crate::recipe::StateCondition::HoverExcept`]`("data-state", "open")` +
//!   [`crate::recipe::hover_surface_declarations`] で hover 背景を追加した
//!   （`Hover`（無条件）ではなく `HoverExcept` を使う理由は
//!   `color_picker`（イシュー #1463/PR #1740）と同型: open 状態の
//!   `--fandhe-color-accent-subtle` 背景〔selector specificity
//!   (0,4,0) の hover 規則が末尾の `@media (hover: hover)` ブロックへ
//!   集約されるため必ず勝つ〕を hover の muted 背景で洗い流さないため）。
//!   `data-disabled` の直書き宣言（`opacity`/`cursor`）を
//!   [`crate::recipe::disabled_declarations`] へ置換（出力バイト同一）。
//!   `:focus-visible` の直書き outline 2 宣言を
//!   [`crate::recipe::focus_ring_declarations`]（`FocusRingColor::Token`。
//!   navigation-menu は `ColorPalette` 軸を持たないため menu 1/3 等と同じ
//!   選択）へ置換した。
//!   [`crate::recipe::transition_declarations`]（`background, color`、
//!   `MotionDuration::Fast`）で状態遷移のトランジションを追加した。
//! - **`link`**: `border-radius`（生 `0.25rem` →
//!   `var(--fandhe-radius-sm)`）をトークン化。
//!   [`crate::recipe::hover_bg_muted`] +
//!   [`crate::recipe::StateCondition::HoverExceptAttr`]`("data-current")` +
//!   [`crate::recipe::hover_surface_declarations`] で hover 背景を追加した
//!   （`data-current` は値なしの存在属性のため、値付き `HoverExcept`
//!   ではなく `HoverExceptAttr` を使う。combobox `data-highlighted`
//!   （イシュー #1468/PR #1745）と同型で、accent 背景 + accent-fg 文字色
//!   の現在地リンクへ muted 背景が重なりコントラストが崩れるのを防ぐ）。
//!   [`crate::recipe::transition_declarations`]（`background, color`、
//!   `MotionDuration::Fast`）を追加した。既存の `data-current` 表現
//!   （accent 背景 + accent-fg 文字色）はトークン経由済みのため維持した。
//! - **`item`**: `data-disabled` の直書き宣言を
//!   [`crate::recipe::disabled_declarations`] へ置換した（出力バイト
//!   同一）。`item` は非インタラクティブな `<li>` ラッパーのため hover・
//!   transition は付与しない。
//! - **`list` は是正対象なし（意図的な非対応）**: gap/margin/padding は
//!   既にトークン経由・リセット済みで、`align-items: flex-start` は
//!   モジュール冒頭 rustdoc「レイアウト」節の縦ずれ回帰予防の固定値
//!   （PR #1000 の反省）のため変更しない。表示専用の `<ul>` レイアウト
//!   コンテナであり hover 対象でもない。
//! - **indicator パートは実装しない（意図的な非対応）**: headless 層
//!   （`crates/headless-ui/src/navigation_menu.rs`）の anatomy は
//!   root/list/item/trigger/content/link の 6 パーツのみで Indicator は
//!   スコープ外として明示的に非実装（イシュー #993 継承）。本モジュールは
//!   headless anatomy の再エクスポート + stylesheet のみの薄い委譲（規約
//!   B-1）であり、pre-styled-ui 単独では indicator パートを追加できない
//!   （headless 層の anatomy 拡張が前提になる別スコープ）。
//! - **`:active`（押下）擬似クラスは追加しない（意図的な非対応）**:
//!   [`crate::recipe::StateCondition`] に `:active` に相当する variant が
//!   存在せず、新設は本イシューの 2h 粒度・recipe 契約変更の双方に対して
//!   過大であるため見送った。「active」は headless 層が実際に出す
//!   `data-current`（アクティブリンク）で表現する。
//!
//! # 担当パートの是正（イシュー #1701、親 #1530 の 2/2 分割。内部パート
//! `content` と `data-state` 開閉トランジションを担当。外枠パート
//! （`list`/`item`/`trigger`/`link`）は兄弟 #1700（PR #1806）が完了済み）
//!
//! ## スコープ解釈の注記
//!
//! headless 層（`crates/headless-ui/src/navigation_menu.rs`）の anatomy は
//! root/list/item/trigger/content/link の 6 パーツのみで、viewport /
//! positioner / arrow / indicator は存在しない（イシュー #993 継承、
//! モジュール冒頭 rustdoc「indicator パートは実装しない」節参照）。
//! pre-styled-ui 単独では headless anatomy の再エクスポート + stylesheet
//! のみの薄い委譲（規約 B-1）であるため単独でパート追加はできない。
//! イシュータイトルの「viewport 等」は、[`crate::menu`] 2/3・
//! [`crate::menubar`]（#1703）の「スコープ解釈の注記」先例に倣い、唯一の
//! 内部・オーバーレイパートである `content` へ読み替える。
//!
//! ## 是正内容
//!
//! - **`content`**: `border-radius` の生リテラル（`0.375rem`）を
//!   `var(--fandhe-radius-md)` へ、`box-shadow` の生リテラル
//!   （`0 4px 6px rgba(0, 0, 0, 0.15)`）を `var(--fandhe-shadow-md)` へ
//!   トークン化した（[`crate::menu`] の `content`・[`crate::menubar`] の
//!   `content`/`sub-content`〔#1703〕と同型の是正）。ライトの影は
//!   `0.15` → `0.1` へわずかに薄くなり、ダークは専用の影値
//!   （`0 4px 6px rgba(0, 0, 0, 0.3)`）がトークン再定義で自動成立する
//!   意匠変更（値意匠は同等の判断は #1703 と同型）。
//!
//! ## 意図的に合わせなかった点・開閉トランジション非対応
//!
//! - **開閉（entry/exit）トランジションは追加しない**: headless 層
//!   （`crates/headless-ui/src/navigation_menu.rs`）の `content` は
//!   closed 時に `hidden` 存在属性を同一フレームで即時付与・除去する
//!   契約であり、遷移前フレームが描画されないため CSS トランジションは
//!   開閉どちら向きも発火しない。dialog（PR #1795 codex-review P1
//!   指摘）→ [`crate::menu`] 1/3（PR #1800）→ [`crate::menubar`]
//!   内部パート（#1703）で確立した「意図的な非対応として rustdoc に
//!   記録する」判断を継承する。`@starting-style` 等による真の実現は
//!   recipe 基盤の横断設計変更（ユーザー承認事項）であり、本イシューでは
//!   行わない。
//! - **`prefers-reduced-motion` は新規対応不要**: 本イシューは新規
//!   transition を追加していない（上記のとおり開閉トランジション自体を
//!   追加しない）ため、`@media (prefers-reduced-motion: reduce)` の
//!   個別対応は不要。兄弟 #1700 が trigger/link へ追加した transition は
//!   すべて motion トークン（[`crate::recipe::transition_declarations`]）
//!   経由であり、`Theme::to_css` の一括 `0ms` 上書きで既に自動成立して
//!   いる。
//! - **`content` の `position`/`top`/`left`/`z-index`/`min-width` は現状
//!   維持**: `position: absolute; top: 100%; left: 0;` はモジュール冒頭
//!   rustdoc「レイアウト」節（PR #1000 の縦ずれ回帰予防）の位置ジオメトリ
//!   契約であり変更しない。`z-index: 10` はトークンが theme に無いため
//!   現状維持（[`crate::menu`] 1/3・[`crate::menubar`]（#1703）と同判断）。
//!   `min-width: 10rem` は navigation-menu が `--fandhe-reference-width`
//!   契約（[`crate::menu`] の `content` が使う
//!   `var(--fandhe-reference-width, 10rem)`）を持たないため、意匠を
//!   変えない生値のまま維持する（[`crate::menubar`]（#1703）の
//!   `content`/`sub-content` と同判断）。
//! - **indicator / viewport / positioner / arrow パートは追加しない**:
//!   上記「スコープ解釈の注記」のとおり headless anatomy に存在せず、
//!   追加は headless 層の変更（ユーザー承認事項）を伴うため本イシューの
//!   範囲外とする。
//!
//! # 本イシューのスコープ外
//!
//! headless 層（`crates/headless-ui/src/navigation_menu.rs`）のモジュール
//! doc「スコープ外」節をそのまま継承する（`data-motion`・viewport 寸法
//! 測定、Indicator/Viewport/Sub\* パーツ・`orientation` 引数、キーボード
//! 操作の実 DOM 配線）。

use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, SlotRecipe,
    StateCondition,
};

// REEXPORT-GLOB-REVIEWED: 本モジュールが定義する pub 項目は stylesheet() の
// みで styled パーツ関数を再定義しない（規約 B-1）。variant 軸も提供せず
// （規約 B-2）、CSS 到達は [data-scope]/[data-part] 属性セレクタのみに依存
// する（規約 B-3、イシュー #1062 規約参照）。
pub use fandhe_frontend_headless_ui::navigation_menu::*;
// `OpenState` は本モジュールの再エクスポート対象パーツ関数（`item`/
// `trigger`/`content`）の引数型として呼び出し側が組み立てる必要があるが、
// `navigation_menu` モジュールの glob 再エクスポートでは到達しない
// （`state` モジュール由来のため）。呼び出し側が
// `fandhe-frontend-pre-styled-ui` のみに依存して呼び出せることを保証する
// ための明示再エクスポート（[`crate::menubar`] の `Orientation`/`OpenState`
// と同型のパターン）。
pub use fandhe_frontend_headless_ui::state::OpenState;

/// headless `navigation_menu` anatomy の `data-part` 一覧
/// （`crates/headless-ui/src/navigation_menu.rs` の `ANATOMY.part(...)`
/// 呼び出しと同期させる契約。ずれると [`stylesheet`] が一部パーツの CSS を
/// 出力しない fail-closed 側の不具合として現れるため、変更時は両ファイルを
/// 合わせて確認する）。
const SLOTS: &[&str] = &["root", "list", "item", "trigger", "content", "link"];

/// この styled Navigation Menu の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("navigation-menu", SLOTS)
        .base(
            "root",
            vec![decl("display", "flex"), decl("align-items", "center")],
        )
        .base(
            "list",
            vec![
                decl("display", "flex"),
                // §モジュール冒頭 rustdoc「レイアウト」節参照: showcase の
                // content 中和時の縦ずれ回帰を構造的に防ぐため center ではなく
                // flex-start を既定にする。
                decl("align-items", "flex-start"),
                decl("gap", "var(--fandhe-space-1)"),
                decl("list-style", "none"),
                decl("margin", "0"),
                decl("padding", "0"),
            ],
        )
        .base("item", vec![decl("position", "relative")])
        .base(
            "trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("cursor", "pointer"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("background", "transparent"),
                decl("border", "none"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("padding", "var(--fandhe-space-1) var(--fandhe-space-3)"),
                hover_bg_muted(),
            ],
        )
        // `base` は同一 slot への複数回登録が許され出力順で連結される
        // （menu 1/3 #1525 と同型のパターン）。
        .base(
            "trigger",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        .base(
            "content",
            vec![
                decl("position", "absolute"),
                decl("top", "100%"),
                decl("left", "0"),
                decl("z-index", "10"),
                decl("margin-top", "var(--fandhe-space-1)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("box-shadow", "var(--fandhe-shadow-md)"),
                decl("padding", "var(--fandhe-space-2)"),
                decl("min-width", "10rem"),
            ],
        )
        .base(
            "link",
            vec![
                decl("display", "block"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("text-decoration", "none"),
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                hover_bg_muted(),
            ],
        )
        .base(
            "link",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        // 開いている trigger を視覚的に強調する。
        .state(
            "trigger",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("background", "var(--fandhe-color-accent-subtle)")],
        )
        // アクティブリンク（現在地）を視覚的に強調する。
        .state(
            "link",
            StateCondition::Attr("data-current"),
            vec![
                decl("background", "var(--fandhe-color-accent)"),
                decl("color", "var(--fandhe-color-accent-fg)"),
            ],
        )
        // headless 層の navigation_menu trigger はネイティブ `disabled` 属性を
        // 付与する設計（[`crate::accordion`] の item_trigger と同型）であり、
        // disabled 項目もフォーカス順序に残す [`crate::menubar`]/[`crate::toolbar`]
        // （aria-disabled のみでネイティブ disabled を付与しない設計）とは逆に
        // フォーカス順序から除外される。ここでは視覚的にのみ操作不能を示す。
        .state(
            "trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "item",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // trigger はキーボード操作時のみのフォーカスリング。
        .state(
            "trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // イシュー #1700: trigger の hover 実適用。素の `Hover` ではなく
        // `HoverExcept("data-state", "open")` を使う理由はモジュール冒頭
        // rustdoc「担当パートの是正」節参照（open 状態の accent-subtle
        // 背景を hover の muted 背景で洗い流さないため）。
        .state(
            "trigger",
            StateCondition::HoverExcept("data-state", "open"),
            hover_surface_declarations(),
        )
        // イシュー #1700: link の hover 実適用。`data-current` は値なしの
        // 存在属性のため `HoverExceptAttr` を使う（モジュール冒頭 rustdoc
        // 「担当パートの是正」節参照）。
        .state(
            "link",
            StateCondition::HoverExceptAttr("data-current"),
            hover_surface_declarations(),
        )
}

/// この styled Navigation Menu が生成する静的 CSS 全量を返す（決定的。
/// [`crate::menubar::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        for part in SLOTS {
            let needle = format!(r#"[data-scope="navigation-menu"][data-part="{part}"]"#);
            assert!(a.contains(&needle), "missing selector for part={part}");
        }
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn trigger_open_state_is_visually_distinct() {
        let css = stylesheet();
        assert!(css
            .contains(r#"[data-scope="navigation-menu"][data-part="trigger"][data-state="open"]"#));
    }

    #[test]
    fn current_link_is_visually_distinct() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="navigation-menu"][data-part="link"][data-current]"#));
    }

    #[test]
    fn list_align_items_is_flex_start_not_center() {
        // モジュール冒頭 rustdoc「レイアウト」節参照: showcase の content
        // 中和時の縦ずれ回帰の予防策を固定する回帰テスト。
        let css = stylesheet();
        assert!(css.contains(
            "[data-scope=\"navigation-menu\"][data-part=\"list\"] {\n  display: flex;\n  align-items: flex-start;\n  "
        ));
        // root（トリガー行そのものの縦中央揃え、通常表示のみに関わる）は
        // center のままでよい。回帰対象は list（ドロップダウン展開時に
        // 縦ずれを起こしうるコンテナ）のみであるため、list パーツの
        // セレクタブロックに絞って center が使われていないことを確認する。
        assert!(!css.contains(
            "[data-scope=\"navigation-menu\"][data-part=\"list\"] {\n  display: flex;\n  align-items: center;\n"
        ));
    }

    #[test]
    fn trigger_declares_focus_visible_ring_but_link_does_not() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="navigation-menu"][data-part="trigger"]:focus-visible {"#)
        );
        assert!(
            !css.contains(r#"[data-scope="navigation-menu"][data-part="link"]:focus-visible {"#)
        );
        // イシュー #1700: focus_ring_declarations(FocusRingColor::Token, ...)
        // 経由のトークン参照形へ置換したことの確認（menu 1/3 #1525 と同型）。
        assert!(css.contains(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ));
    }

    #[test]
    fn trigger_and_link_declare_hover_feedback_excluding_active_state() {
        // イシュー #1700: trigger/link に hover フィードバックを追加した
        // ことの確認。open trigger（data-state="open"）と現在地リンク
        // （data-current）は accent 背景を維持するため hover から除外
        // される（モジュール冒頭 rustdoc「担当パートの是正」節参照）。
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="navigation-menu"][data-part="trigger"]:hover:not([data-disabled]):not([data-state="open"])"#
        ));
        assert!(css.contains(
            r#"[data-scope="navigation-menu"][data-part="link"]:hover:not([data-disabled]):not([data-current])"#
        ));
    }

    #[test]
    fn list_and_item_do_not_declare_hover_feedback() {
        // list（表示専用の <ul>）と item（<li> ラッパー）は非インタラクティブ
        // のため hover 規則を持たない（#1425 の判定基準）。
        let css = stylesheet();
        assert!(!css.contains(r#"[data-scope="navigation-menu"][data-part="list"]:hover"#));
        assert!(!css.contains(r#"[data-scope="navigation-menu"][data-part="item"]:hover"#));
    }

    #[test]
    fn trigger_and_link_border_radius_is_tokenized() {
        // イシュー #1700: 生リテラル `0.25rem` を `var(--fandhe-radius-sm)`
        // へトークン化した（値は同一、外観不変）。
        let css = stylesheet();
        assert!(css.contains("border-radius: var(--fandhe-radius-sm);"));
        assert!(!css.contains("border-radius: 0.25rem;"));
    }

    #[test]
    fn trigger_and_link_declare_state_transition() {
        // イシュー #1700: transition_declarations("background, color",
        // MotionDuration::Fast) による状態遷移を trigger/link へ追加した
        // （longhand 3 プロパティで構成、recipe.rs の
        // transition_declarations rustdoc 参照）。
        let css = stylesheet();
        assert!(css.contains("transition-property: background, color;"));
        assert!(css.contains("transition-duration: var(--fandhe-motion-duration-fast);"));
        assert!(css.contains("transition-timing-function: var(--fandhe-motion-easing-standard);"));
    }

    #[test]
    fn trigger_and_item_disabled_declarations_use_canonical_helper() {
        // イシュー #1700: 直書き宣言を disabled_declarations() へ置換
        // （出力バイト同一）。
        let css = stylesheet();
        let trigger_disabled = css
            .split(r#"[data-scope="navigation-menu"][data-part="trigger"][data-disabled] {"#)
            .nth(1)
            .and_then(|s| s.split('}').next())
            .expect("trigger data-disabled block must exist");
        let item_disabled = css
            .split(r#"[data-scope="navigation-menu"][data-part="item"][data-disabled] {"#)
            .nth(1)
            .and_then(|s| s.split('}').next())
            .expect("item data-disabled block must exist");
        assert_eq!(trigger_disabled, item_disabled);
        assert!(trigger_disabled.contains("opacity"));
        assert!(trigger_disabled.contains("cursor"));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root("Main", vec![], vec![]));
        assert!(html.contains(r#"data-scope="navigation-menu""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("role="));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_state_machine() {
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut m = NavigationMenu::default();
        assert_eq!(m.open_value(), None);

        let ssr_html = render(&m.trigger("products", false, None, None, vec![], vec![]));
        assert!(ssr_html.contains(r#"aria-expanded="false""#));

        assert!(dispatch(&mut m, "select", "products"));
        assert_eq!(m.open_value(), Some("products"));

        let hydrate_html = render(&render_for_hydration(&m));
        assert!(hydrate_html.contains("data-hydrate-selected="));

        let restored = NavigationMenu::from_hydration_attrs(&m.hydration_attrs()).unwrap();
        assert_eq!(restored, m);
    }
}
