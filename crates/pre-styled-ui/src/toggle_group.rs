//! styled ToggleGroup（headless ラッパー、イシュー #746、親 #520/#545）。
//!
//! `fandhe_frontend_headless_ui::toggle_group`（イシュー #746）の Item
//! anatomy パーツをそのまま再エクスポートし、[`stylesheet`] で既定 CSS を
//! 追加提供する。薄い委譲の根拠は [`crate::radio_group`]/[`crate::toggle`]
//! の rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由）
//!
//! 本モジュールは `size`/`palette` variant クラス付与のため styled `root`
//! （[`crate::toggle::root`] と同型）を本モジュールで再定義する。headless
//! 自由関数 `root` と名前衝突するため、`pub use ...::*` ではなく必要な
//! 識別子（[`item`]/[`ToggleGroup`]/[`MultiToggleGroup`]）のみを選択的に
//! 再エクスポートする。
//!
//! [`ToggleGroup`]/[`MultiToggleGroup`] 状態機械は inherent `root()` を
//! 持たない（item 系メソッドのみ、`crates/headless-ui/src/toggle_group.rs`
//! 参照）ため、[`crate::radio_group`] の `RadioGroup` と同じく、そのまま
//! 再エクスポートしても未スタイル root の静かな適用漏れは発生しない。
//!
//! # 複合部品の variant 統一方針（root のみへクラス付与）
//!
//! `size`（[`Size`]）/`palette`（[`ColorPalette`]）はいずれも [`root`] へ
//! のみクラスを付与する。[`recipe`] が root スコープへ登録する custom
//! property（`--fandhe-toggle-group-item-padding-y`/`-item-padding-x`/
//! `-item-font-size`）は CSS の通常のプロパティ継承により `item` へ伝わる
//! ため、`item` 自身へ variant クラスを付ける必要がない（`root` が `item`
//! を内包する祖先要素であるため成立する。[`crate::radio_group`] の
//! `item-control`/`item-text` と同じ設計、`crate::lib` rustdoc
//! 「複合部品の variant 統一方針」節参照）。
//!
//! # `data-state`/`aria-pressed` 語彙について
//!
//! headless 層の `item` は [`crate::toggle::root`] と同じ `"on"`/`"off"`
//! 語彙（[`crate::state::pressed_data_state`]）を使う
//! （`crates/headless-ui/src/toggle_group.rs` 参照）。[`recipe`] の状態規則
//! もこの語彙に合わせて `data-state="on"` を条件とする。
//!
//! # フォーカスリング（hidden-input パターン非該当）
//!
//! `item` はネイティブ `<button>` 自身であり実フォーカスを直接受けるため、
//! [`crate::toggle`]/[`crate::select`] の `trigger` と同じ
//! [`StateCondition::FocusVisible`] で足りる。`data-focus-visible` 配線は
//! 不要（[`crate::toggle`] モジュール rustdoc と同じ判断）。
//!
//! # セキュリティ不変条件
//!
//! 本モジュールは headless 層の再エクスポートと静的 CSS 生成のみで構成され、
//! `raw_html()` を使用しない。CSS 宣言値はすべてコンパイル時静的リテラルで
//! あり、動的値（`value`/`labelled_by`/属性/children）へ CSS 値として流し
//! 込む経路を持たない（動的値は headless 層経由で
//! `fandhe_frontend_core::render` の既定エスケープを必ず通る、REQ-1）。
//! styled `root` は [`drop_class_attr`] により呼び出し側の `class` を除去
//! してから合成するため、`class` 属性は常に単一。
//!
//! # 参考サイト基準のスタイル調整（イシュー #1513）
//!
//! Radix Primitives（`radixp-toggle-group-1`）/ ark-ui
//! （`ark-toggle-group-1〜3`）のスクリーンショット比較を基に、
//! [`crate::recipe`] の Phase 0 共通ビジュアル言語
//! （[`crate::recipe::focus_ring_declarations`]/
//! [`crate::recipe::transition_declarations`]/
//! [`crate::recipe::disabled_declarations`]/
//! [`crate::recipe::hover_bg_muted`]）へ載せ替え、item が隣接ボーダーを
//! 共有する連結セグメント状の外観（詳細は [`stylesheet`] rustdoc）を追加した。
//!
//! 是正しない点（意図的な判断）:
//!
//! - **pressed の palette solid 塗りを維持する**: 参照サイトの淡い soft
//!   背景ではなく、[`crate::toggle`] と共有する既存の `data-state="on"`
//!   表現語彙（[`ColorPalette`] 軸の存在意義）をそのまま使う。
//! - **variant 軸（solid/outline 等）は追加しない**: `crate::listbox`
//!   （イシュー #1483）と同じ、Forms 家族横断の設計判断を要するため本
//!   イシュー単体では追加しない。
//! - **roving focus の実 DOM 配線 / loopFocus はスコープ外**: headless 層
//!   （`crates/headless-ui/src/toggle_group.rs`）と同じく wasm keynav 層の
//!   責務（下記「本イシューのスコープ外」節と同一事項）。SSR 側の
//!   roving tabindex 初期値は [`fandhe_frontend_headless_ui::toggle_group::ToggleGroupProps::roving_focus`]
//!   （既定 `false`）で opt-in できる（イシュー #1630、[`root_with_props`]
//!   参照）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - roving focus の実 DOM 配線 / loopFocus は headless 層
//!   （`crates/headless-ui/src/toggle_group.rs`）と同じくスコープ外
//!   （wasm keynav 層の責務）。

use crate::class_attr::drop_class_attr;
use crate::css::{decl, serialize_rule};
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    palette_scale_declarations, transition_declarations, ColorPalette, FocusRingColor,
    FocusRingOffset, MotionDuration, Size, SlotRecipe, StateCondition, VariantValue,
};

// headless 自由関数 `root` はあえて再エクスポートしない（本モジュール冒頭
// の rustdoc「選択的 re-export」節参照、`root` は本モジュールで styled 版
// として再定義する）。
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::toggle_group::{
    item, MultiToggleGroup, ToggleGroup, ToggleGroupProps,
};

/// headless `toggle-group` anatomy の `data-part` 一覧
/// (`crates/headless-ui/src/toggle_group.rs` の `ANATOMY.part(...)` 呼び出し
/// と同期させる契約)。
const SLOTS: &[&str] = &["root", "item"];

/// この styled ToggleGroup の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("toggle-group", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                // 参照サイト（Radix Primitives / ark-ui）の Toggle Group は
                // item 同士が隣接ボーダーを共有する連結セグメント状の外観
                // であり、item 間に可視の隙間を持たない。`gap: 0` とし、
                // 連結表現（隣接ボーダーの重ね合わせ・外端のみの角丸）は
                // [`stylesheet`] の raw CSS 追記（`SlotRecipe` では表現
                // できない `:first-child`/`:last-child` 構造擬似クラスと
                // orientation 別セレクタを要するため）で行う。
                decl("gap", "0"),
            ],
        )
        // headless 層が `data_orientation` 経由で出力する
        // `data-orientation="vertical"` では縦積みへ切り替える
        // （`crate::radio_group` の `data-orientation="horizontal"` と対称。
        // 既定は横並びのため、本コンポーネントは逆側の値のみを分岐する）。
        .state(
            "root",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![decl("flex-direction", "column")],
        )
        .state(
            "root",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .base(
            "item",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("box-sizing", "border-box"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl(
                    "padding",
                    "var(--fandhe-toggle-group-item-padding-y, 0.375rem) var(--fandhe-toggle-group-item-padding-x, 0.75rem)",
                ),
                decl(
                    "font-size",
                    "var(--fandhe-toggle-group-item-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("cursor", "pointer"),
                hover_bg_muted(),
            ]
            .into_iter()
            .chain(transition_declarations(
                "background, border-color, color",
                MotionDuration::Fast,
            ))
            .collect::<Vec<_>>(),
        )
        .state(
            "item",
            StateCondition::AttrEq("data-state", "on"),
            vec![
                decl(
                    "background",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl(
                    "border-color",
                    "var(--fandhe-palette, var(--fandhe-color-accent))",
                ),
                decl("color", "var(--fandhe-palette-fg)"),
                // 連結セグメント上で隣接ボーダー（下記 raw CSS の
                // `margin-inline-start`/`margin-block-start` による重ね
                // 合わせ）に押し潰されず on 状態の border-color が視認
                // できるよう最前面へ引き上げる（`:focus-visible` の
                // outline と同じ理由、[`stylesheet`] rustdoc 参照）。
                decl("position", "relative"),
                decl("z-index", "1"),
            ],
        )
        .state(
            "item",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // item はネイティブ button 自身が実フォーカスを受けるため、
        // hidden-input パターン（switch/radio_group）の data-focus-visible
        // 配線は不要（crate::toggle rustdoc と同じ判断）。canonical な
        // フォーカスリング（イシュー #1424）は palette 軸を持つ本部品では
        // `FocusRingColor::Palette` を使う（`crate::button` と同型）。
        .state(
            "item",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Palette, FocusRingOffset::Outside)
                .into_iter()
                .chain([decl("position", "relative"), decl("z-index", "1")])
                .collect::<Vec<_>>(),
        )
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-toggle-group-item-padding-y", "0.125rem"),
                decl("--fandhe-toggle-group-item-padding-x", "0.25rem"),
                decl("--fandhe-toggle-group-item-font-size", "var(--fandhe-font-font-size-xs)"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-toggle-group-item-padding-y", "0.25rem"),
                decl("--fandhe-toggle-group-item-padding-x", "0.5rem"),
                decl(
                    "--fandhe-toggle-group-item-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-toggle-group-item-padding-y", "0.375rem"),
                decl("--fandhe-toggle-group-item-padding-x", "0.75rem"),
                decl(
                    "--fandhe-toggle-group-item-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-toggle-group-item-padding-y", "0.5rem"),
                decl("--fandhe-toggle-group-item-padding-x", "1rem"),
                decl(
                    "--fandhe-toggle-group-item-font-size",
                    "var(--fandhe-font-font-size-md)",
                ),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-toggle-group-item-padding-y", "0.625rem"),
                decl("--fandhe-toggle-group-item-padding-x", "1.25rem"),
                decl("--fandhe-toggle-group-item-font-size", "var(--fandhe-font-font-size-lg)"),
            ],
        )
        .default_variant(Size::Md)
        .default_variant(ColorPalette::Accent);

    for palette in [
        ColorPalette::Accent,
        ColorPalette::Info,
        ColorPalette::Success,
        ColorPalette::Warning,
        ColorPalette::Danger,
        ColorPalette::Neutral,
    ] {
        recipe = recipe.variant(palette, "root", palette_scale_declarations(palette));
    }
    recipe
}

/// この styled ToggleGroup が生成する静的 CSS 全量を返す（決定的。
/// [`crate::toggle::stylesheet`] と同じ契約）。
///
/// # raw CSS 追記の理由（イシュー #1513、参照サイト基準の意匠是正）
///
/// 参照サイト（Radix Primitives `radixp-toggle-group-1` / ark-ui
/// `ark-toggle-group-1〜3` のスクリーンショット比較、`docs/design/
/// reference-screenshots/`）はいずれも item 同士が隣接ボーダーを共有する
/// 連結セグメント状の外観（中間 item は角丸なし・外端の item のみ角丸）を
/// 持つ。この表現は以下 2 点により [`SlotRecipe`] の型化された条件
/// （`base`/`state`/`variant`）だけでは組めない:
///
/// 1. **構造擬似クラス**（`:first-child`/`:last-child`）と
///    orientation 別の適用面（横並びは左右端、`[data-orientation="vertical"]`
///    配下は上下端）の組み合わせが必要（`SlotRecipe::state` の
///    `StateCondition` は自パーツの属性条件のみを表現し、構造擬似クラスを
///    持たない）。
/// 2. **hover** は祖先 `root` の `[data-disabled]` 不在を前提に含む必要が
///    ある。headless 層の `root(disabled: true, ...)` は root にのみ
///    `data-disabled` を付与し子 `item` へは伝播しない
///    （`crates/headless-ui/src/toggle_group.rs` 参照）ため、`item` 自身の
///    `data-disabled` だけを見る宣言では group 全体が disabled でも
///    個々の item に hover 背景が付いてしまう（[`crate::listbox::stylesheet`]
///    が同じ理由で raw CSS 追記している先例と同型の問題・同型の対処）。
///
/// いずれも [`marquee::css`](crate::marquee) / [`crate::listbox::stylesheet`]
/// と同型の raw CSS 追記パターンで、[`recipe().css()`](SlotRecipe::css) の
/// 出力へ後段追加する。
///
/// 参照サイトの pressed 表現（淡い soft 背景）に対し、本部品は palette
/// solid 塗りを [`crate::toggle`] と共有する既存の `data-state="on"`
/// 表現語彙として維持し、意図的に合わせない（`ColorPalette` 軸の存在意義。
/// モジュール冒頭 rustdoc 参照）。同様に on 状態の hover 色変化は参照サイト
/// でも僅少なため、hover 規則は `:not([data-state="on"])` を条件に含め on
/// 状態には適用しない。
#[must_use]
pub fn stylesheet() -> String {
    let mut out = recipe().css();

    const ROOT: &str = r#"[data-scope="toggle-group"][data-part="root"]"#;
    const ITEM: &str = r#"[data-scope="toggle-group"][data-part="item"]"#;

    // 連結セグメント化: 横並び（既定 orientation）は外端の item（最初/最後）
    // のみ角丸を残し、中間 item は角丸なしにする。隣接 item は 1px 分だけ
    // 重ねて二重ボーダーの太線化を避ける（`data-state="on"`/フォーカス時の
    // `z-index: 1` により重なった隣接ボーダーが視覚的に埋もれない、
    // recipe() 側の state 宣言参照）。
    let mut connected_rules = String::new();
    for (selector, decls) in [
        (
            format!("{ROOT} > {ITEM}:first-child"),
            vec![
                decl("border-start-start-radius", "var(--fandhe-radius-md)"),
                decl("border-end-start-radius", "var(--fandhe-radius-md)"),
                decl("border-start-end-radius", "0"),
                decl("border-end-end-radius", "0"),
            ],
        ),
        (
            format!("{ROOT} > {ITEM}:last-child"),
            vec![
                decl("border-start-end-radius", "var(--fandhe-radius-md)"),
                decl("border-end-end-radius", "var(--fandhe-radius-md)"),
                decl("border-start-start-radius", "0"),
                decl("border-end-start-radius", "0"),
            ],
        ),
        (
            format!("{ROOT} > {ITEM}:not(:first-child):not(:last-child)"),
            vec![decl("border-radius", "0")],
        ),
        // 単一 item（:first-child かつ :last-child）は上記 2 規則が競合し、
        // どちらが勝つかは同一特異度のため後勝ちのソース順に依存してしまう
        // （実際には後段の :last-child 規則が勝ち、始端側の角丸を失う）。
        // 二重擬似クラスで特異度を明示的に引き上げ、四隅とも角丸を保証する。
        (
            format!("{ROOT} > {ITEM}:first-child:last-child"),
            vec![
                decl("border-start-start-radius", "var(--fandhe-radius-md)"),
                decl("border-end-start-radius", "var(--fandhe-radius-md)"),
                decl("border-start-end-radius", "var(--fandhe-radius-md)"),
                decl("border-end-end-radius", "var(--fandhe-radius-md)"),
            ],
        ),
        (
            format!("{ROOT} > {ITEM} + {ITEM}"),
            vec![decl("margin-inline-start", "-1px")],
        ),
        (
            format!(r#"{ROOT}[data-orientation="vertical"] > {ITEM}:first-child"#),
            vec![
                decl("border-start-start-radius", "var(--fandhe-radius-md)"),
                decl("border-start-end-radius", "var(--fandhe-radius-md)"),
                decl("border-end-start-radius", "0"),
                decl("border-end-end-radius", "0"),
            ],
        ),
        (
            format!(r#"{ROOT}[data-orientation="vertical"] > {ITEM}:last-child"#),
            vec![
                decl("border-end-start-radius", "var(--fandhe-radius-md)"),
                decl("border-end-end-radius", "var(--fandhe-radius-md)"),
                decl("border-start-start-radius", "0"),
                decl("border-start-end-radius", "0"),
            ],
        ),
        // vertical 単一 item も同様に :first-child/:last-child の競合で
        // 始端側の角丸を失う（Cursor Bugbot 指摘）ため、二重擬似クラスで
        // 特異度を引き上げ四隅とも角丸を保証する。
        (
            format!(r#"{ROOT}[data-orientation="vertical"] > {ITEM}:first-child:last-child"#),
            vec![
                decl("border-start-start-radius", "var(--fandhe-radius-md)"),
                decl("border-start-end-radius", "var(--fandhe-radius-md)"),
                decl("border-end-start-radius", "var(--fandhe-radius-md)"),
                decl("border-end-end-radius", "var(--fandhe-radius-md)"),
            ],
        ),
        (
            format!(r#"{ROOT}[data-orientation="vertical"] > {ITEM} + {ITEM}"#),
            vec![
                decl("margin-inline-start", "0"),
                decl("margin-block-start", "-1px"),
            ],
        ),
    ] {
        if let Some(rule) = serialize_rule(&selector, &decls) {
            connected_rules.push_str(&rule);
        }
    }
    if !connected_rules.is_empty() {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&connected_rules);
    }

    // hover: root/item いずれも disabled ではなく、on 状態でもない item の
    // みへ適用する（本関数 rustdoc「raw CSS 追記の理由」節参照）。
    let hover_selector = format!(
        "{ROOT}:not([data-disabled]) > {ITEM}:hover:not([data-disabled]):not([data-state=\"on\"])"
    );
    if let Some(rule) = serialize_rule(&hover_selector, &hover_surface_declarations()) {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str("@media (hover: hover) {\n");
        for line in rule.lines() {
            out.push_str("  ");
            out.push_str(line);
            out.push('\n');
        }
        out.push_str("}\n");
    }

    out
}

/// styled root パーツを組み立てる。`size`/`palette` に応じたクラスを付与
/// する唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は除去
/// してから合成する）。実体は
/// [`fandhe_frontend_headless_ui::toggle_group::root`] へ委譲する。
///
/// 公開シグネチャは互換性のため `disabled: bool`/`orientation` のみを
/// 引数に取る形を維持し、内部で `roving_focus: false`（既定値）とした
/// [`ToggleGroupProps`] を組み立てて [`root_with_props`] へ委譲する
/// （[`crate::radio_group::root`]/[`RadioGroupProps`] と同型のパターン、
/// イシュー #1630）。`roving_focus` を有効にしたい場合は
/// [`root_with_props`] を使うこと。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::toggle_group;
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let node = toggle_group::root(
///     Size::Md,
///     ColorPalette::Accent,
///     false,
///     None,
///     None,
///     vec![],
///     vec![],
/// );
/// assert!(render(&node).contains(r#"data-scope="toggle-group" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    palette: ColorPalette,
    disabled: bool,
    orientation: Option<Orientation>,
    labelled_by: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let props = ToggleGroupProps {
        disabled,
        orientation,
        ..ToggleGroupProps::default()
    };
    root_with_props(size, palette, &props, labelled_by, attrs, children)
}

/// styled root パーツを、全 [`ToggleGroupProps`]（disabled/orientation/
/// roving_focus）を反映して組み立てる（[`crate::radio_group::root_with_props`]
/// と同型、イシュー #1630）。[`root`] と実体を共有するが、`roving_focus`
/// を既定値へ落とさず呼び出し側の `props` をそのまま headless
/// [`fandhe_frontend_headless_ui::toggle_group::root`] へ渡す。子パーツ
/// （[`item`]）へ渡す `props` と同一の値をここへも渡すことで、group 全体
/// （root/item）の `data-orientation`/`data-disabled`/`tabindex` の
/// 出力が一貫する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::toggle_group;
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
/// use fandhe_frontend_headless_ui::toggle_group::ToggleGroupProps;
///
/// let props = ToggleGroupProps {
///     roving_focus: true,
///     ..ToggleGroupProps::default()
/// };
/// let node = toggle_group::root_with_props(
///     Size::Md,
///     ColorPalette::Accent,
///     &props,
///     None,
///     vec![],
///     vec![],
/// );
/// assert!(render(&node).contains(r#"data-scope="toggle-group" data-part="root""#));
/// ```
#[must_use]
pub fn root_with_props<'a>(
    size: Size,
    palette: ColorPalette,
    props: &ToggleGroupProps,
    labelled_by: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class =
        recipe.variant_classes(&[("size", size.value()), ("color-palette", palette.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::toggle_group::root(props, labelled_by, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="toggle-group"][data-part="item"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn stylesheet_links_item_to_on_state() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="toggle-group"][data-part="item"][data-state="on"] {"#));
        assert!(css.contains("var(--fandhe-palette, var(--fandhe-color-accent))"));
    }

    #[test]
    fn stylesheet_links_root_to_vertical_orientation() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="toggle-group"][data-part="root"][data-orientation="vertical"] {"#
        ));
        assert!(css.contains("flex-direction: column;"));
    }

    #[test]
    fn stylesheet_links_item_to_focus_visible() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="toggle-group"][data-part="item"]:focus-visible {"#));
        // canonical フォーカスリングトークン（イシュー #1424）へ移行済み
        // であることの回帰（直書き `2px solid ...` へ後退させない）。
        assert!(css.contains("var(--fandhe-focus-ring-width, 2px)"));
    }

    #[test]
    fn stylesheet_uses_canonical_transition_and_disabled_helpers() {
        let css = stylesheet();
        // transition: Phase 0 共通ビジュアル言語（motion トークン）へ移行
        // 済みであることの回帰（shorthand 直書きへ後退させない）。
        assert!(css.contains("transition-duration: var(--fandhe-motion-duration-fast);"));
        assert!(css.contains("transition-property: background, border-color, color;"));
        // disabled: root/item ともヘルパ経由の宣言を維持する。
        assert!(css.contains("cursor: not-allowed;"));
        assert!(css.contains("opacity: 0.5;"));
    }

    #[test]
    fn stylesheet_links_item_hover_to_muted_background_excluding_disabled_and_on() {
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover) {"));
        assert!(css.contains(
            r#"[data-scope="toggle-group"][data-part="root"]:not([data-disabled]) > [data-scope="toggle-group"][data-part="item"]:hover:not([data-disabled]):not([data-state="on"]) {"#
        ));
        assert!(css.contains("background: var(--fandhe-hover-bg);"));
    }

    #[test]
    fn stylesheet_connects_items_into_a_segmented_group() {
        let css = stylesheet();
        // 横並び: 外端のみ角丸、中間は角丸なし、隣接ボーダーの重ね合わせ。
        assert!(css.contains(
            r#"[data-scope="toggle-group"][data-part="root"] > [data-scope="toggle-group"][data-part="item"]:first-child {"#
        ));
        assert!(css.contains(
            r#"[data-scope="toggle-group"][data-part="root"] > [data-scope="toggle-group"][data-part="item"]:last-child {"#
        ));
        assert!(css.contains(
            r#"[data-scope="toggle-group"][data-part="root"] > [data-scope="toggle-group"][data-part="item"]:not(:first-child):not(:last-child) {"#
        ));
        assert!(css.contains("margin-inline-start: -1px;"));
        // vertical: 上下端の角丸系統が横並びとは独立して存在する。
        assert!(css.contains(
            r#"[data-scope="toggle-group"][data-part="root"][data-orientation="vertical"] > [data-scope="toggle-group"][data-part="item"]:first-child {"#
        ));
        assert!(css.contains(
            r#"[data-scope="toggle-group"][data-part="root"][data-orientation="vertical"] > [data-scope="toggle-group"][data-part="item"] + [data-scope="toggle-group"][data-part="item"] {"#
        ));
        assert!(css.contains("margin-block-start: -1px;"));
    }

    #[test]
    fn stylesheet_zeroes_inner_corners_of_horizontal_end_items() {
        // codex-review P1 / Cursor Bugbot 指摘（イシュー #1513）: 横並びの
        // first-child/last-child が外側角丸を再設定するだけで内側の角丸を
        // 0 にしていなかったため、2 要素以上の group で内側の角が丸いまま
        // 重なり合う pill に見えてしまっていた。
        let css = stylesheet();
        let first_child_rule = extract_rule(
            &css,
            r#"[data-scope="toggle-group"][data-part="root"] > [data-scope="toggle-group"][data-part="item"]:first-child {"#,
        );
        assert!(first_child_rule.contains("border-start-end-radius: 0;"));
        assert!(first_child_rule.contains("border-end-end-radius: 0;"));

        let last_child_rule = extract_rule(
            &css,
            r#"[data-scope="toggle-group"][data-part="root"] > [data-scope="toggle-group"][data-part="item"]:last-child {"#,
        );
        assert!(last_child_rule.contains("border-start-start-radius: 0;"));
        assert!(last_child_rule.contains("border-end-start-radius: 0;"));
    }

    #[test]
    fn stylesheet_keeps_all_corners_rounded_for_a_single_item() {
        // Cursor Bugbot 追加指摘（イシュー #1513）: :first-child/:last-child
        // 双方が一致する単一 item では、上記の内側ゼロ化規則同士が競合し
        // 同一特異度のソース順（後勝ち）に依存して始端側の角丸を失って
        // しまう。二重擬似クラス（`:first-child:last-child`）による高特異度
        // 規則で四隅とも角丸を保証する。
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="toggle-group"][data-part="root"] > [data-scope="toggle-group"][data-part="item"]:first-child:last-child {"#
        ));
        let horizontal_only_child_rule = extract_rule(
            &css,
            r#"[data-scope="toggle-group"][data-part="root"] > [data-scope="toggle-group"][data-part="item"]:first-child:last-child {"#,
        );
        assert!(horizontal_only_child_rule
            .contains("border-start-start-radius: var(--fandhe-radius-md);"));
        assert!(horizontal_only_child_rule
            .contains("border-end-start-radius: var(--fandhe-radius-md);"));
        assert!(horizontal_only_child_rule
            .contains("border-start-end-radius: var(--fandhe-radius-md);"));
        assert!(
            horizontal_only_child_rule.contains("border-end-end-radius: var(--fandhe-radius-md);")
        );

        assert!(css.contains(
            r#"[data-scope="toggle-group"][data-part="root"][data-orientation="vertical"] > [data-scope="toggle-group"][data-part="item"]:first-child:last-child {"#
        ));
        let vertical_only_child_rule = extract_rule(
            &css,
            r#"[data-scope="toggle-group"][data-part="root"][data-orientation="vertical"] > [data-scope="toggle-group"][data-part="item"]:first-child:last-child {"#,
        );
        assert!(vertical_only_child_rule
            .contains("border-start-start-radius: var(--fandhe-radius-md);"));
        assert!(
            vertical_only_child_rule.contains("border-start-end-radius: var(--fandhe-radius-md);")
        );
        assert!(
            vertical_only_child_rule.contains("border-end-start-radius: var(--fandhe-radius-md);")
        );
        assert!(
            vertical_only_child_rule.contains("border-end-end-radius: var(--fandhe-radius-md);")
        );
    }

    /// 生成 CSS 文字列から、指定した `selector {` から対応する `}` までの
    /// 1 ルール分を抜き出す（本テストモジュール限定のヘルパ。`{`/`}` の
    /// ネストを持たない単純な宣言ブロックのみを対象とする素朴な実装で足りる）。
    fn extract_rule<'a>(css: &'a str, rule_start: &str) -> &'a str {
        let start = css
            .find(rule_start)
            .unwrap_or_else(|| panic!("rule not found: {rule_start}"));
        let end = css[start..]
            .find('}')
            .unwrap_or_else(|| panic!("unterminated rule: {rule_start}"));
        &css[start..start + end]
    }

    // --- variant クラス（root のみ） ---

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="toggle-group""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"role="group""#));
    }

    #[test]
    fn default_variant_is_md_and_accent() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-toggle-group--size-md"));
        assert!(html.contains("fd-toggle-group--color-palette-accent"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-toggle-group--size-xs"),
            (Size::Sm, "fd-toggle-group--size-sm"),
            (Size::Md, "fd-toggle-group--size-md"),
            (Size::Lg, "fd-toggle-group--size-lg"),
            (Size::Xl, "fd-toggle-group--size-xl"),
        ] {
            let html = render(&root(
                size,
                ColorPalette::Accent,
                false,
                None,
                None,
                vec![],
                vec![],
            ));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (
                ColorPalette::Accent,
                "fd-toggle-group--color-palette-accent",
            ),
            (ColorPalette::Info, "fd-toggle-group--color-palette-info"),
            (
                ColorPalette::Success,
                "fd-toggle-group--color-palette-success",
            ),
            (
                ColorPalette::Warning,
                "fd-toggle-group--color-palette-warning",
            ),
            (
                ColorPalette::Danger,
                "fd-toggle-group--color-palette-danger",
            ),
            (
                ColorPalette::Neutral,
                "fd-toggle-group--color-palette-neutral",
            ),
        ] {
            let html = render(&root(Size::Md, palette, false, None, None, vec![], vec![]));
            assert!(html.contains(class), "palette={palette:?} -> {html}");
        }
    }

    #[test]
    fn item_is_not_given_variant_classes() {
        // item は root のみへクラスが付く複合部品の variant 統一方針
        // （モジュール rustdoc 参照）。item 自体には class 属性がない。
        let html = render(&item(
            &ToggleGroupProps::default(),
            false,
            false,
            false,
            "bold",
            vec![],
            vec![],
        ));
        assert!(!html.contains("class="));
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            None,
            None,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            None,
            None,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="toggle-group""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn root_labelled_by_payload_is_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            false,
            None,
            Some(PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn reexported_item_value_and_children_are_escaped_on_render() {
        const PAYLOAD: &str = "\" onmouseover=\"alert(1)";
        let html = render(&item(
            &ToggleGroupProps::default(),
            false,
            false,
            false,
            PAYLOAD,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_toggle_group_state_machine() {
        use fandhe_frontend_headless_ui::fandhe_frontend_interactive::{
            dispatch, render_for_hydration, Hydrate,
        };

        let mut g = ToggleGroup::default();
        assert_eq!(g.value(), None);

        assert!(dispatch(&mut g, "toggle", "bold"));
        let hydrate_html = render(&render_for_hydration(&g));
        assert!(hydrate_html.contains("data-hydrate-selected="));
        assert!(hydrate_html.contains("bold"));

        let restored = ToggleGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
        assert_eq!(restored, g);
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_headless_multi_toggle_group_state_machine() {
        use fandhe_frontend_headless_ui::fandhe_frontend_interactive::{dispatch, Hydrate};

        let mut g = MultiToggleGroup::default();
        assert!(dispatch(&mut g, "toggle", "bold"));
        assert!(dispatch(&mut g, "toggle", "italic"));
        assert!(g.is_pressed("bold"));
        assert!(g.is_pressed("italic"));

        let restored = MultiToggleGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
        assert_eq!(restored, g);
    }
}
