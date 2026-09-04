//! styled RatingGroup（headless ラッパー、イシュー #742、親 #520/#736）。
//!
//! `fandhe_frontend_headless_ui::rating_group`（イシュー #742）の Label /
//! Control / Item / HiddenInput 4 anatomy パーツと
//! [`fandhe_frontend_headless_ui::rating_group::RatingGroup`] 状態機械を
//! そのまま再エクスポートし、[`stylesheet`] で既定 CSS（星形 indicator）を
//! 追加提供する。薄い委譲の根拠・スコープ外事項は [`crate::radio_group`] の
//! rustdoc と同じ方針に従う。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由、[`crate::radio_group`]
//! と同型）
//!
//! 本モジュールは `size`/`palette` variant クラス付与のため styled `root`
//! を本モジュールで再定義する。headless 自由関数 `root` と名前衝突するため、
//! `pub use ...::*` ではなく必要な識別子（[`label`]/[`control`]/[`item`]/
//! [`hidden_input`]/[`RatingItemFlags`]/[`RatingGroup`]/[`RatingGroupAction`]）のみ
//! を選択的に再エクスポートする。
//!
//! [`RatingGroup`] 状態機械は inherent `root()` を持たない
//! （`item`/`hidden_input` 系メソッドのみ、`crates/headless-ui/src/rating_group.rs`
//! 参照）ため、[`crate::radio_group::RadioGroup`] 非再エクスポート回避と
//! 同じ判断で、そのまま再エクスポートしても未スタイル root の静かな適用漏れ
//! は発生しない。
//!
//! # 星形 indicator（外部リソース非参照、CSS `clip-path` によるインライン表現）
//!
//! `item` は SVG ファイル・icon font・画像 URL を一切参照せず、`clip-path:
//! polygon(...)`（5 角星の固定座標リテラル）+ 正方形寸法
//! （`--fandhe-rating-group-item-size`）+ 既定塗り色（未点灯:
//! `var(--fandhe-color-border)`）で星形を描く。塗り色は `data-highlighted`
//! （headless 層が出力する「index <= display_value」の存在属性、
//! `crates/headless-ui/src/rating_group.rs` 参照）が付いたときのみ
//! `var(--fandhe-palette, var(--fandhe-color-accent))` へ切り替わる
//! （選択状態そのもの＝`data-checked` ではなく、hover プレビューを含む
//! 表示上の塗り判定＝`data-highlighted` を CSS 側の切替条件にする。これは
//! headless 層の「`display_value = hover.or(value)`」契約と一致させるため）。
//!
//! # `size`/`palette` variant
//!
//! `size`（[`Size`]）は `root` へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-rating-group-item-size` の root スコープ custom property
//! （通常の CSS 継承により `item` へ伝わる。`root` はこれを内包する祖先
//! 要素であるため、[`crate::recipe::SlotRecipe`] へ子孫セレクタ機構を追加
//! せずに実現できる、[`crate::radio_group`] と同型）経由で星の寸法を切り
//! 替える。`palette`（[`ColorPalette`]）は既存の
//! [`crate::recipe::palette_scale_declarations`]（chakra-ui virtual token 方式、
//! #606）を `root` へ登録し、点灯時の星の塗り色を `var(--fandhe-palette,
//! ...)` 経由で切り替える。`base`/`state` 規則の `var()` にはいずれも Md
//! サイズ・Accent パレット相当のフォールバック値を書き、styled `root` を
//! 経由しない headless 直接利用マークアップでも現行外観を維持する
//! （fail-safe、`crate::lib` rustdoc「複合部品の variant 統一方針」節参照）。
//!
//! # スタイル調整（イシュー #1496）
//!
//! 参照サイト（chakra-ui `rating` / ark-ui `rating-group`）基準との視覚差分を
//! `docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` /
//! `docs/design/pre-styled-ui-interaction-visual-language.md` の共通ビジュアル
//! 言語へ合わせて是正した。
//!
//! - **フォーカスリング**: `item` は `clip-path: polygon(...)` の星形であり、
//!   CSS 仕様上 `clip-path` は `outline` を含む要素描画全体をクリップするため、
//!   canonical 形（outline ベース）のリングを `item` へ付けても星形に切り
//!   取られて視認できない。roving focus（wasm-full 配線）でどの `item` に
//!   フォーカスがあっても `:focus-within` は祖先 `control` で成立するため、
//!   `control` へ [`crate::recipe::focus_ring_declarations`]（`palette` 軸を
//!   公開する部品のため `FocusRingColor::Palette`）を登録した。
//! - **per-star の CSS `:hover` は意図的に追加しない**: rating の hover
//!   フィードバックの正体は「hover した星番号までを塗るプレビュー」であり、
//!   headless 層の `display_value = hover.or(value)` 契約 +
//!   `RatingGroupAction::Hover`/`ClearHover` + `data-highlighted` が既に担う
//!   （参照サイトも JS 駆動で、単一星だけ暗くする CSS hover はいずれの挙動
//!   でもない）。固定セレクタ `:hover:not([data-disabled])`
//!   （[`crate::recipe::hover_surface_declarations`]）を追加すると
//!   `data-readonly`（平均評価表示等の常用形）へ hover が漏れる問題もあり
//!   採用しない。代わりに `item` へ
//!   [`crate::recipe::transition_declarations`]（`background`）を追加し、
//!   JS 駆動プレビューの塗り替わりを滑らかにする。
//! - **`data-disabled`**: `item` の規則を [`crate::recipe::disabled_declarations`]
//!   （canonical 順: opacity → cursor）へ揃えた（値そのものは既存 ad-hoc
//!   実装と同値）。
//! - **label の型階層**: `font-size` を size 連動 custom property
//!   （`--fandhe-rating-group-font-size`）へ変更し、`font-weight: medium` /
//!   `line-height: normal` を追加した（`radio_group`/`checkbox_group` と
//!   同型の 2 段階型階層）。
//! - **未塗り星の表現は現状維持**: ark-ui のアウトライン（輪郭線のみ）星は
//!   `clip-path` 塗りつぶし方式では表現できず、chakra-ui の「未塗り = 淡い
//!   グレー塗り」（`var(--fandhe-color-border)`）に既に一致しているため
//!   変更しない（意図的差分）。
//! - **`data-invalid` は追加しない**: headless `rating_group` は未出力で
//!   あり、chakra `rating` にも invalid 視覚言語がないため（`radio_group`
//!   が装飾したのは form control 系の同型視覚言語があるため。rating は
//!   対象外）。
//! - **サイズ・バリアント・色トークンは変更不要**: 星寸法（`Size` 5 段）・
//!   `ColorPalette` 6 色・全宣言のトークン参照は既に参照サイト水準を満たす
//!   （variant 軸は chakra `rating`/ark `rating-group` も持たない）。
//!
//! # セキュリティ不変条件
//!
//! 本モジュールは headless 層の再エクスポートと静的 CSS 生成のみで構成され、
//! `raw_html()` を使用しない。CSS 宣言値はすべてコンパイル時静的リテラルで
//! あり、動的値（`aria_label`/`name`/`value_text`/属性/children）へ CSS 値
//! として流し込む経路を持たない（動的値は headless 層経由で
//! `fandhe_frontend_core::render` の既定エスケープを必ず通る、REQ-1）。
//! styled `root` は [`drop_class_attr`] により呼び出し側の `class` を除去
//! してから合成するため、`class` 属性は常に単一（[`crate::radio_group::root`]
//! と同型）。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `allow_half`（0.5 刻み、ark-ui `allowHalf`）の半星 CSS 表現は headless
//!   層のスコープ外（`crates/headless-ui/src/rating_group.rs` doc 参照）に
//!   伴い、本モジュールでも未提供。
//! - hover/クリック/キーボードナビゲーションの DOM 配線は
//!   `fandhe-frontend-wasm-full` の後続責務。
//! - `examples/headless-pre-styled-ui` への追随は crates.io 公開後に別途
//!   行う（[`crate::number_input`] の先例と同じ判断）。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, palette_scale_declarations,
    transition_declarations, ColorPalette, FocusRingColor, FocusRingOffset, MotionDuration, Size,
    SlotRecipe, StateCondition, VariantValue,
};

// headless 自由関数 `root` はあえて再エクスポートしない（本モジュール冒頭
// の rustdoc「選択的 re-export」節参照、`root` は本モジュールで styled 版
// として再定義する）。
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::rating_group::{
    control, hidden_input, item, label, RatingGroup, RatingGroupAction, RatingGroupProps,
    RatingItemFlags,
};

/// headless `rating_group` anatomy の `data-part` 一覧（`crates/headless-ui/src/rating_group.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約）。
const SLOTS: &[&str] = &["root", "label", "control", "item", "hidden-input"];

/// この styled RatingGroup の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("rating-group", SLOTS)
        .base(
            "root",
            vec![decl("display", "inline-flex"), decl("flex-direction", "column")],
        )
        .base(
            "label",
            vec![
                decl("display", "block"),
                decl("color", "var(--fandhe-color-fg)"),
                decl(
                    "font-size",
                    "var(--fandhe-rating-group-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                decl("margin-bottom", "var(--fandhe-space-1)"),
            ],
        )
        .base(
            "control",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-1)"),
            ],
        )
        // 星形は SVG/icon font/画像 URL を一切参照しない `clip-path` による
        // インライン表現（モジュール doc「星形 indicator」参照）。5 角星の
        // 座標はコンパイル時静的リテラルであり、動的値は一切流れない。
        .base(
            "item",
            vec![
                decl(
                    "clip-path",
                    "polygon(50% 0%, 61% 35%, 98% 35%, 68% 57%, 79% 91%, 50% 70%, 21% 91%, 32% 57%, 2% 35%, 39% 35%)",
                ),
                decl(
                    "width",
                    "var(--fandhe-rating-group-item-size, 1.25rem)",
                ),
                decl(
                    "height",
                    "var(--fandhe-rating-group-item-size, 1.25rem)",
                ),
                decl("display", "inline-block"),
                decl("background", "var(--fandhe-color-border)"),
                decl("cursor", "pointer"),
                decl("flex-shrink", "0"),
            ],
        )
        // `base` は同一 slot への複数回登録が許され出力順で連結される
        // （`radio_group.rs`/`checkbox.rs` の transition 追加と同型のパターン）。
        // hover プレビュー（`data-highlighted` の付け外し、モジュール doc
        // 「星形 indicator」節参照）の塗り替わりを滑らかにする。per-star の
        // CSS `:hover` 自体は意図的に追加しない（イシュー #1496 rustdoc
        // 「スタイル調整」節参照）。
        .base(
            "item",
            transition_declarations("background", MotionDuration::Fast),
        )
        // hidden_input はフォーム送信専用のネイティブ input であり、視覚上は
        // 不要（`type="hidden"` のためブラウザが元々描画しないが、既定 CSS の
        // 一貫性のため明示的に display: none を与える）。
        .base("hidden-input", vec![decl("display", "none")])
        // `data-highlighted`（headless 層が「index <= display_value（hover
        // 優先）」で出力、モジュール doc 参照）が付いた星を点灯色で塗る。
        .state(
            "item",
            StateCondition::Attr("data-highlighted"),
            vec![decl(
                "background",
                "var(--fandhe-palette, var(--fandhe-color-accent))",
            )],
        )
        // `data-disabled`/`data-readonly`（headless 層が `data_disabled`/
        // `data_readonly` 経由で `item` へ出力）時の操作不能な見た目。
        .state(
            "item",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "item",
            StateCondition::Attr("data-readonly"),
            vec![decl("cursor", "default")],
        )
        // イシュー #1496: フォーカスリング皆無の是正。`item` は
        // `clip-path: polygon(...)` の星形（モジュール doc「星形
        // indicator」節参照）であり、CSS 仕様上 `clip-path` は `outline` を
        // 含む要素描画全体をクリップするため、canonical 形（outline
        // ベース、`crate::recipe::focus_ring_declarations` 参照）のリングが
        // 星形に切り取られて視認できない。roving focus（wasm-full 配線）で
        // どの `item` にフォーカスがあっても `:focus-within` は祖先
        // `control` で成立するため、`item` ではなく `control` へリングを
        // 登録し、星の外周にリングを出す。`palette` 軸を公開する部品の
        // ため `docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`
        // §6 の規約に従い `FocusRingColor::Palette` を使う。
        .state(
            "control",
            StateCondition::FocusWithin,
            focus_ring_declarations(FocusRingColor::Palette, FocusRingOffset::Outside),
        )
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-rating-group-item-size", "0.75rem"),
                decl(
                    "--fandhe-rating-group-font-size",
                    "var(--fandhe-font-font-size-xs)",
                ),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-rating-group-item-size", "1rem"),
                decl(
                    "--fandhe-rating-group-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-rating-group-item-size", "1.25rem"),
                decl(
                    "--fandhe-rating-group-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-rating-group-item-size", "1.5rem"),
                decl(
                    "--fandhe-rating-group-font-size",
                    "var(--fandhe-font-font-size-md)",
                ),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-rating-group-item-size", "1.75rem"),
                decl(
                    "--fandhe-rating-group-font-size",
                    "var(--fandhe-font-font-size-lg)",
                ),
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

/// この styled RatingGroup が生成する静的 CSS 全量を返す（決定的。
/// [`crate::radio_group::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size`/`palette` に応じたクラスを付与
/// する唯一のパーツ（[`drop_class_attr`] により呼び出し側の `class` は除去
/// してから合成する）。実体は
/// [`fandhe_frontend_headless_ui::rating_group::root`] へ委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::rating_group;
/// use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
///
/// let node = rating_group::root(
///     Size::Md,
///     ColorPalette::Accent,
///     &rating_group::RatingGroupProps::default(),
///     vec![],
///     vec![],
/// );
/// assert!(render(&node).contains(r#"data-scope="rating-group" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    palette: ColorPalette,
    props: &RatingGroupProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class =
        recipe.variant_classes(&[("size", size.value()), ("color-palette", palette.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::rating_group::root(props, merged, children)
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
        assert!(a.contains(r#"[data-scope="rating-group"][data-part="item"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn item_uses_clip_path_star_and_no_external_resource() {
        let css = stylesheet();
        assert!(css.contains("clip-path: polygon("));
        // 外部リソース参照（url(...)・SVG ファイル・font 参照）を一切含まない。
        assert!(!css.contains("url("));
    }

    #[test]
    fn highlighted_item_switches_to_palette_fill() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="rating-group"][data-part="item"][data-highlighted]"#));
        assert!(css.contains("background: var(--fandhe-palette, var(--fandhe-color-accent));"));
    }

    #[test]
    fn disabled_item_gets_not_allowed_cursor() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="rating-group"][data-part="item"][data-disabled]"#));
        assert!(css.contains("cursor: not-allowed;"));
    }

    #[test]
    fn readonly_item_gets_default_cursor() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="rating-group"][data-part="item"][data-readonly]"#));
        assert!(css.contains("cursor: default;"));
    }

    /// イシュー #1496: `control` の `:focus-within` にフォーカスリングが
    /// 登録されていることを固定する（`item` は clip-path 星形のため
    /// outline が視認できず、祖先 `control` へ登録する設計。モジュール
    /// doc「スタイル調整」節参照）。
    #[test]
    fn control_focus_within_gets_focus_ring() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="rating-group"][data-part="control"]:focus-within"#));
        assert!(css.contains("outline:"));
        assert!(css.contains("var(--fandhe-palette, var(--fandhe-color-focus-ring"));
    }

    /// イシュー #1496: `item` の hover プレビュー塗り替わりを滑らかにする
    /// transition が base 規則として登録されていることを固定する。
    #[test]
    fn item_has_background_transition() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="rating-group"][data-part="item"] {"#));
        assert!(css.contains("transition-property: background;"));
    }

    /// イシュー #1496: `data-disabled` の宣言順が canonical
    /// （opacity → cursor）へ揃っていることを固定する（是正前は
    /// cursor → opacity の逆順だった）。
    #[test]
    fn disabled_item_declares_opacity_before_cursor_canonical_order() {
        let css = stylesheet();
        let marker = r#"[data-scope="rating-group"][data-part="item"][data-disabled]"#;
        let start = css.find(marker).expect("data-disabled block missing");
        let block_start = css[start..].find('{').expect("block start missing") + start;
        let block_end = css[block_start..].find('}').expect("block end missing") + block_start;
        let block = &css[block_start..block_end];
        let opacity_pos = block.find("opacity:").expect("opacity declaration missing");
        let cursor_pos = block.find("cursor:").expect("cursor declaration missing");
        assert!(
            opacity_pos < cursor_pos,
            "expected opacity before cursor (canonical order): {block}"
        );
    }

    #[test]
    fn hidden_input_is_display_none() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="rating-group"][data-part="hidden-input"]"#));
        assert!(css.contains("display: none;"));
    }

    // --- variant クラス ---

    #[test]
    fn root_outputs_scope_and_part() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &RatingGroupProps::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="rating-group""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn default_variant_is_md_and_accent() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &RatingGroupProps::default(),
            vec![],
            vec![],
        ));
        assert!(html.contains("fd-rating-group--size-md"));
        assert!(html.contains("fd-rating-group--color-palette-accent"));
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-rating-group--size-xs"),
            (Size::Sm, "fd-rating-group--size-sm"),
            (Size::Md, "fd-rating-group--size-md"),
            (Size::Lg, "fd-rating-group--size-lg"),
            (Size::Xl, "fd-rating-group--size-xl"),
        ] {
            let html = render(&root(
                size,
                ColorPalette::Accent,
                &RatingGroupProps::default(),
                vec![],
                vec![],
            ));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    /// イシュー #1496: `label` の型階層で追加した size 連動 custom property
    /// （`--fandhe-rating-group-font-size`）が各 `Size` variant ブロックへ
    /// 登録されていることを固定する（`checkbox_group`/`radio_group` と
    /// 同型の写像: Xs->xs / Sm->sm / Md->sm / Lg->md / Xl->lg）。
    #[test]
    fn size_variant_registers_label_font_size_custom_property() {
        let css = stylesheet();
        for (class, expected_token) in [
            ("fd-rating-group--size-xs", "--fandhe-font-font-size-xs"),
            ("fd-rating-group--size-sm", "--fandhe-font-font-size-sm"),
            ("fd-rating-group--size-md", "--fandhe-font-font-size-sm"),
            ("fd-rating-group--size-lg", "--fandhe-font-font-size-md"),
            ("fd-rating-group--size-xl", "--fandhe-font-font-size-lg"),
        ] {
            let selector = format!(r#".{class}"#);
            let start = css
                .find(&selector)
                .unwrap_or_else(|| panic!("variant selector missing: {selector} in {css}"));
            let block_start = css[start..].find('{').expect("block start missing") + start;
            let block_end = css[block_start..].find('}').expect("block end missing") + block_start;
            let block = &css[block_start..block_end];
            assert!(
                block.contains("--fandhe-rating-group-font-size")
                    && block.contains(expected_token),
                "class={class} missing font-size custom property mapping to {expected_token}: {block}"
            );
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (
                ColorPalette::Accent,
                "fd-rating-group--color-palette-accent",
            ),
            (ColorPalette::Info, "fd-rating-group--color-palette-info"),
            (
                ColorPalette::Success,
                "fd-rating-group--color-palette-success",
            ),
            (
                ColorPalette::Warning,
                "fd-rating-group--color-palette-warning",
            ),
            (
                ColorPalette::Danger,
                "fd-rating-group--color-palette-danger",
            ),
            (
                ColorPalette::Neutral,
                "fd-rating-group--color-palette-neutral",
            ),
        ] {
            let html = render(&root(
                Size::Md,
                palette,
                &RatingGroupProps::default(),
                vec![],
                vec![],
            ));
            assert!(html.contains(class), "palette={palette:?} -> {html}");
        }
    }

    #[test]
    fn class_attr_is_single_and_caller_class_is_dropped() {
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &RatingGroupProps::default(),
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
            &RatingGroupProps::default(),
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="rating-group""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn root_disabled_and_readonly_reflected() {
        let props = RatingGroupProps {
            disabled: true,
            readonly: true,
            required: false,
        };
        let html = render(&root(
            Size::Md,
            ColorPalette::Accent,
            &props,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-disabled=""#));
        assert!(html.contains(r#"data-readonly=""#));
    }

    // --- エスケープ回帰 ---

    #[test]
    fn xss_payload_in_item_aria_label_is_escaped_by_render() {
        // REQ-1 回帰: `aria_label`（動的値）へ与えた XSS ペイロードが
        // `render()` の既定エスケープを経由することを固定する。
        let payload = "\"><script>alert(1)</script>";
        let html = render(&item(
            1,
            RatingItemFlags::default(),
            payload,
            vec![],
            vec![],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn xss_payload_in_label_children_is_escaped_by_render() {
        let payload = "\"><img src=x onerror=alert(1)>";
        let html = render(&label(
            &RatingGroupProps::default(),
            None,
            vec![],
            vec![text(payload)],
        ));
        assert!(!html.contains("<img"));
        assert!(html.contains("&lt;img"));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_rating_group_state_machine() {
        // 再エクスポートされた `RatingGroup`（headless の Component/Hydrate
        // 実装をそのまま継承）経由で SSR/hydration 往復を固定する
        // （[`crate::radio_group`] の同型テストに準拠）。
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut g = RatingGroup::new(5, None, false);
        assert_eq!(g.value(), None);

        assert!(dispatch(&mut g, "set", "4"));
        assert_eq!(g.value(), Some(4));

        let ssr_html = render(&g.item(4, false, "4 stars", vec![], vec![]));
        assert!(ssr_html.contains(r#"data-checked="""#));

        let hydrate_html = render(&render_for_hydration(&g));
        assert!(hydrate_html.contains("data-hydrate-"));

        let restored = RatingGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
        assert_eq!(restored.value(), Some(4));
    }
}
