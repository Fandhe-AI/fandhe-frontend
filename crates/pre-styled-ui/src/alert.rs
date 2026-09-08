//! Alert（イシュー #550）: slot recipe styled 部品。root/indicator/content/
//! title/description/action の 6 パーツで構成する通知バナー。
//!
//! `root` に `role="alert"`（WAI-ARIA live region、ステータスに関わらず固定）
//! を付与する。chakra-ui v3 準拠でステータスごとに `role` を切り替える設計も
//! あり得るが、本イシューでは「注意を要する通知」という `alert` ロールの
//! 意味を全ステータス共通で固定する（`status`（緊急度の低い更新通知）との
//! 使い分けは呼び出し側が [`AlertStatus`] を見て判断する設計としない）。
//!
//! # 参考サイト基準への調整（イシュー #1553）
//!
//! chakra-ui `Alert` / Radix Themes `Callout` の視覚基準に照らし、以下を
//! 是正・追加した（詳細な対比は Issue #1553 参照）。
//!
//! - **色**: 生の中立色（`--fandhe-color-bg-subtle`）1 色だった背景を、
//!   [`crate::recipe::palette_scale_declarations`] 経由の 6 役割トークン
//!   （`--fandhe-palette-subtle`/`-fg-subtle`/`-muted` 等）へ移行した。
//!   [`AlertStatus`] → [`ColorPalette`] の対応は [`status_palette`] に固定する
//!   （公開 API は `ColorPalette` を露出しない、イシュー #606 の境界を維持）。
//! - **バリアント**: `variant` 軸（[`AlertVariant`]、既定 `Subtle`）を新設した。
//!   badge の `Solid`/`Subtle`/`Outline` + callout の `Surface` を踏襲する。
//! - **サイズ**: `size` 軸（[`crate::recipe::Size`]、既定 `Md`）を新設した。
//!   root の size variant が `--fandhe-alert-*` custom property を切り替え、
//!   各パーツはそれを `var(--fandhe-alert-*, <Md 相当>)` で参照する
//!   （[`crate::tab_nav`] と同型のスコープ接頭辞規約）。
//! - **余白・角丸**: `gap`/`padding` の生値をトークン（`--fandhe-space-*`）化。
//!   角丸は既存どおり `--fandhe-radius-md` を維持する。
//! - **意図的に追随しない点**:
//!   - hover / disabled / transition: 表示専用部品であり、参照サイトにも
//!     状態遷移がないため付けない
//!     （`docs/design/pre-styled-ui-interaction-visual-language.md` §hover）。
//!   - フォーカスリング: root は非フォーカス要素であり、参照サイトの
//!     `Alert`/`Callout` も自身にフォーカスリングを持たない。
//!   - chakra `inline` prop（title/description の横並び）: 既存 slot 構成
//!     （`content` が column flex 固定）を超える追加軸のため見送り。
//!   - Radix `highContrast`: トークン体系にない軸のため見送り。
//!
//! # イシュー #2043（shadcn/ui 突合、`docs/design/shadcn-reference-adoption-policy.md` §8）
//!
//! shadcn/ui Alert（Base UI 版 docs ページ。`new-york-v4` レジストリの
//! `alert.tsx` ソースには存在しない `AlertAction` を Base 版 docs ページのみが
//! 持つ。本節は Base 版 docs ページを突合対象とする）と突合した結果を記す。
//!
//! | 観点 | shadcn/ui | 当部品の現状 | 判定 |
//! |---|---|---|---|
//! | variant `default`（白背景+枠線） | あり | `AlertStatus::Neutral` + `AlertVariant::Surface`/`Outline` で再現可 | 既存軸で再現。`default` の名称は持ち込まない |
//! | variant `destructive` | あり | `AlertStatus::Error` + `AlertVariant::Outline` で再現可 | 既存軸で再現 |
//! | アイコン付き | `svg` を `has-[>svg]` で列化 | [`indicator`] slot | 既存 slot で再現 |
//! | title のみ / description のみ | 可 | [`content`] は column flex、title/description は任意 | 既存 anatomy で再現 |
//! | `AlertAction`（右上のアクション併記） | root 右上に absolute 配置 | 該当パートなし | **純追加**: [`action`] パートを新設（下記） |
//! | description の弱め文字色 | `text-muted-foreground` | title と同じ `--fandhe-palette-fg-subtle` | **非追随**（既存 golden の視覚変更になるため。§8 規則 3） |
//! | title `line-clamp-1` | 1 行に切り詰め | なし | **非追随**（`role="alert"` live region の本文欠落を避けるため） |
//!
//! ## 参照競合の判定
//!
//! - action の配置は shadcn-ui の値（右上配置）を採る。理由: chakra-ui /
//!   Radix Themes の Alert / Callout にはアクション slot 自体がなく競合相手が
//!   存在しないため。ただし実現手段は `position: absolute` ではなく root
//!   （`display: flex`）の末尾 flex 兄弟（`margin-inline-start: auto`）と
//!   する。理由: [`content`] と重ならず、[`crate::recipe::SlotRecipe`] が
//!   持たない `:has()` / 子孫セレクタに依存せず、既存 base/variant 出力を
//!   バイト同一に保てるため。
//! - description の文字色は chakra-ui / Radix Themes の値（title と同じ
//!   `fg-subtle`）を採る。理由: shadcn の `muted-foreground` へ変えると
//!   既存 golden の視覚変更（§8 規則 3 違反）になるため。
//! - title の行数は chakra-ui / Radix Themes の値（切り詰めなし）を採る。
//!   理由: `role="alert"` の live region 本文を `line-clamp` で隠すと通知
//!   内容が欠落するため。
//!
//! ## pre-styled-only `action` パート
//!
//! [`action`] は本モジュールが独自に追加する**レイアウト専用**パートで、
//! headless-ui には対応する anatomy を持たない（headless-ui に alert 自体が
//! 存在しない。`crates/docs-site/tests/wrap_state.rs` のバケット D
//! `PRE_STYLED_ONLY` に登録済み）。[`crate::dialog::footer`]（イシュー
//! #1690）・[`crate::dialog::body`]（イシュー #2030）と同型の「アプリケー
//! ションロジックを内包しない、レイアウト専用パート」として設計する
//! （`docs/policy/intentional-non-adoption.md` §3.25 規則 1・規則 2 に適合。
//! クリック配線・送信処理は持たず、内包する `button` 等の操作は呼び出し側が
//! 配線する）。
//!
//! [`action`] の CSS は `display: inline-flex` / `align-items: center` /
//! `flex-shrink: 0` / `margin-inline-start: auto`（論理プロパティ、RTL
//! 対応）/ `gap` の 5 宣言のみを持ち、[`content`] が既に `flex: 1;
//! min-width: 0` を持つため [`action`] を root の最後の子として置くだけで
//! 右上（RTL では左上）に収まり [`content`] と重ならない。素の `button` を
//! root の第 3 子に直接置いても位置は同じになるが、[`action`] が提供する
//! `flex-shrink: 0` と `data-part="action"`（利用者 CSS・テストの捕捉点）が
//! 本パートの存在理由である。既存の `root` `position: relative` は変更しない。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{palette_scale_declarations, ColorPalette, Size, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, role, Anatomy};

/// `data-scope="alert"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("alert");

/// [`SlotRecipe::new`] に渡す slot 一覧。
const SLOTS: &[&str] = &[
    "root",
    "indicator",
    "content",
    "title",
    "description",
    "action",
];

/// Alert のステータス（既定 `Info`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlertStatus {
    /// 情報提供（既定）。
    #[default]
    Info,
    /// 成功。
    Success,
    /// 警告。
    Warning,
    /// エラー。
    Error,
    /// 中立（イシュー #1553 で追加、chakra `status="neutral"` 相当）。
    Neutral,
}

impl VariantValue for AlertStatus {
    fn axis(self) -> &'static str {
        "status"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Neutral => "neutral",
        }
    }
}

/// [`AlertStatus`] に対応する [`ColorPalette`]（イシュー #1553）。
///
/// `Error` のみ `ColorPalette::Danger` へ写像し、他は名前どおり 1:1 対応する。
/// 公開 API としては [`AlertStatus`] のみを露出し `ColorPalette` 軸自体は
/// 公開しない（イシュー #606 の境界、`root` はクラス
/// `fd-alert--status-*` のみを出力し `fd-alert--color-palette-*` は出さない）。
#[must_use]
fn status_palette(status: AlertStatus) -> ColorPalette {
    match status {
        AlertStatus::Info => ColorPalette::Info,
        AlertStatus::Success => ColorPalette::Success,
        AlertStatus::Warning => ColorPalette::Warning,
        AlertStatus::Error => ColorPalette::Danger,
        AlertStatus::Neutral => ColorPalette::Neutral,
    }
}

/// Alert の見た目 variant（イシュー #1553 で新設、既定 `Subtle`）。
///
/// chakra-ui `Alert` の `subtle`/`surface`/`solid`/`outline` に対応する
/// （`inline` は不採用、モジュール doc 参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlertVariant {
    /// 淡色背景（既定）。
    #[default]
    Subtle,
    /// 淡色背景 + 枠線（[`crate::callout::CalloutVariant::Surface`] と同型）。
    Surface,
    /// 塗りつぶし。
    Solid,
    /// 輪郭のみ。
    Outline,
}

impl VariantValue for AlertVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Subtle => "subtle",
            Self::Surface => "surface",
            Self::Solid => "solid",
            Self::Outline => "outline",
        }
    }
}

/// [`root`] の設定（イシュー #1553 で `status` 単独引数から拡張）。
#[derive(Debug, Clone, Copy)]
pub struct AlertProps {
    /// 見た目の状態色（既定 `Info`）。
    pub status: AlertStatus,
    /// 見た目 variant（既定 `Subtle`）。
    pub variant: AlertVariant,
    /// サイズ variant（既定 `Md`）。
    pub size: Size,
}

impl Default for AlertProps {
    fn default() -> Self {
        AlertProps {
            status: AlertStatus::Info,
            variant: AlertVariant::Subtle,
            size: Size::Md,
        }
    }
}

/// Alert の recipe（scope `"alert"`、[`SLOTS`] の 5 パーツ）。
///
/// axis 登録順を status → variant → size に固定する（[`crate::badge`] の
/// recipe と同型。[`SlotRecipe::variant_classes`] は axis の登録順でクラスを
/// 連結するため、この順序が既定出力
/// `"fd-alert--status-info fd-alert--variant-subtle fd-alert--size-md"` を
/// 決定する）。
///
/// `border: 1px solid transparent` を base 側に置き、`Surface`/`Outline`
/// variant は `border-color` のみを上書きする（variant 切替でボックス高さが
/// ±1px ぶれないようにするため。イシュー #1787 の button box-sizing 是正と
/// 同じ動機）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("alert", SLOTS);

    // status 軸を最初に登録する（axis 登録順が variant_classes のクラス出力順を
    // 決めるため。`.variant()` の最初の呼び出しが status であることが
    // `"fd-alert--status-* fd-alert--variant-* fd-alert--size-*"` の順序を
    // 決定する契約）。
    for status in [
        AlertStatus::Info,
        AlertStatus::Success,
        AlertStatus::Warning,
        AlertStatus::Error,
        AlertStatus::Neutral,
    ] {
        recipe = recipe.variant(
            status,
            "root",
            palette_scale_declarations(status_palette(status)),
        );
    }
    recipe = recipe
        .default_variant(AlertStatus::Info)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("align-items", "flex-start"),
                decl("width", "100%"),
                // codex-review P1 指摘（PR #1825）: `width: 100%` を
                // `box-sizing: border-box` なしで指定すると content-box の
                // ため padding/border 分が親幅に加算され、狭いコンテナで
                // 横スクロール・はみ出しが発生する（`crate::button`/
                // `crate::dialog` 等と同型の是正、イシュー #1787 系統）。
                decl("box-sizing", "border-box"),
                decl("position", "relative"),
                decl("gap", "var(--fandhe-alert-gap, var(--fandhe-space-3))"),
                decl(
                    "padding",
                    "var(--fandhe-alert-padding, var(--fandhe-space-4))",
                ),
                decl("border", "1px solid transparent"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl(
                    "font-size",
                    "var(--fandhe-alert-font-size, var(--fandhe-font-font-size-sm))",
                ),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
            ],
        )
        .base(
            "indicator",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("flex-shrink", "0"),
                decl(
                    "width",
                    "var(--fandhe-alert-indicator-size, var(--fandhe-font-font-size-xl))",
                ),
                decl(
                    "height",
                    "var(--fandhe-alert-indicator-size, var(--fandhe-font-font-size-xl))",
                ),
            ],
        )
        .base(
            "content",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("flex", "1"),
                decl("gap", "var(--fandhe-space-1)"),
                decl("min-width", "0"),
            ],
        )
        .base(
            "title",
            vec![decl("font-weight", "var(--fandhe-font-font-weight-medium)")],
        )
        // イシュー #2043: pre-styled-only `action` パート（モジュール doc
        // 「pre-styled-only `action` パート」節参照）。root は既に
        // `display: flex` なので、`margin-inline-start: auto` のみで右側
        // （RTL では左側）へ押し出せる。`position: absolute` は採らない
        // （参照競合の判定、モジュール doc 参照）。
        .base(
            "action",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("flex-shrink", "0"),
                decl("margin-inline-start", "auto"),
                decl("gap", "var(--fandhe-space-2)"),
            ],
        )
        // イシュー #1553: `size` 軸新設に伴い、description 固有の
        // `font-size-sm` 直書きは削除し root の `font-size` へ一本化した
        // （継承のみで実現するため description 自体は base 宣言を持たない）。
        .variant(
            AlertVariant::Subtle,
            "root",
            vec![
                decl("background", "var(--fandhe-palette-subtle)"),
                decl("color", "var(--fandhe-palette-fg-subtle)"),
            ],
        )
        .variant(
            AlertVariant::Surface,
            "root",
            vec![
                decl("background", "var(--fandhe-palette-subtle)"),
                decl("color", "var(--fandhe-palette-fg-subtle)"),
                decl("border-color", "var(--fandhe-palette-muted)"),
            ],
        )
        .variant(
            AlertVariant::Solid,
            "root",
            vec![
                // codex-review P1 指摘（PR #1825）: `--fandhe-palette`
                // （素の status 色）+ `--fandhe-palette-fg` は
                // `LARGE_TEXT_UI_PAIRS`（3:1 保証、`crate::theme` 参照）
                // でのみ検証済みで、Alert 本文の既定サイズ（Md =
                // font-size-sm）には 4.5:1 の本文コントラスト契約が必要
                // なため満たさない。`--fandhe-palette-emphasized`
                // （`Surface`/`Outline` の border-color で既に使用済みの
                // 同一トークン）を背景に採用し、`-fg` との組を 4.5:1 以上へ
                // 引き上げる（実測は `crate::theme` の
                // `alert_solid_variant_pairs_meet_wcag_4_5_to_1_in_light_and_dark`
                // が固定する）。
                decl("background", "var(--fandhe-palette-emphasized)"),
                decl("color", "var(--fandhe-palette-fg)"),
            ],
        )
        .variant(
            AlertVariant::Outline,
            "root",
            vec![
                decl("background", "transparent"),
                decl("color", "var(--fandhe-palette-fg-subtle)"),
                decl("border-color", "var(--fandhe-palette-muted)"),
            ],
        )
        .default_variant(AlertVariant::Subtle)
        // イシュー #1553: size 軸（Xs〜Xl、既定 Md）。indicator-size は
        // font-size の 1 段上（Md=font-size-xl）を既定基準に、他段も同じ
        // 「本文サイズより 1 段大きい」比率で外挿する。
        .size_variants(
            "root",
            &[
                (
                    Size::Xs,
                    vec![
                        decl("--fandhe-alert-padding", "var(--fandhe-space-2)"),
                        decl("--fandhe-alert-gap", "var(--fandhe-space-2)"),
                        decl(
                            "--fandhe-alert-font-size",
                            "var(--fandhe-font-font-size-xs)",
                        ),
                        decl(
                            "--fandhe-alert-indicator-size",
                            "var(--fandhe-font-font-size-md)",
                        ),
                    ],
                ),
                (
                    Size::Sm,
                    vec![
                        decl("--fandhe-alert-padding", "var(--fandhe-space-3)"),
                        decl("--fandhe-alert-gap", "var(--fandhe-space-2)"),
                        decl(
                            "--fandhe-alert-font-size",
                            "var(--fandhe-font-font-size-xs)",
                        ),
                        decl(
                            "--fandhe-alert-indicator-size",
                            "var(--fandhe-font-font-size-lg)",
                        ),
                    ],
                ),
                (
                    Size::Md,
                    vec![
                        decl("--fandhe-alert-padding", "var(--fandhe-space-4)"),
                        decl("--fandhe-alert-gap", "var(--fandhe-space-3)"),
                        decl(
                            "--fandhe-alert-font-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                        decl(
                            "--fandhe-alert-indicator-size",
                            "var(--fandhe-font-font-size-xl)",
                        ),
                    ],
                ),
                (
                    Size::Lg,
                    vec![
                        // chakra 準拠: lg の padding は md と同値。
                        decl("--fandhe-alert-padding", "var(--fandhe-space-4)"),
                        decl("--fandhe-alert-gap", "var(--fandhe-space-3)"),
                        decl(
                            "--fandhe-alert-font-size",
                            "var(--fandhe-font-font-size-md)",
                        ),
                        decl(
                            "--fandhe-alert-indicator-size",
                            "var(--fandhe-font-font-size-2xl)",
                        ),
                    ],
                ),
                (
                    Size::Xl,
                    vec![
                        decl("--fandhe-alert-padding", "var(--fandhe-space-5)"),
                        decl("--fandhe-alert-gap", "var(--fandhe-space-4)"),
                        decl(
                            "--fandhe-alert-font-size",
                            "var(--fandhe-font-font-size-lg)",
                        ),
                        decl(
                            "--fandhe-alert-indicator-size",
                            "var(--fandhe-font-font-size-3xl)",
                        ),
                    ],
                ),
            ],
        );
    recipe
}

/// Alert の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// root パーツを組み立てる。`role="alert"` + `status`/`variant`/`size` に
/// 応じたクラスを付与する唯一のパーツ（`class_attr::drop_class_attr` により
/// 呼び出し側の `class` は除去してから合成する）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::alert::{self, AlertProps, AlertStatus};
///
/// let props = AlertProps {
///     status: AlertStatus::Error,
///     ..AlertProps::default()
/// };
/// let node = alert::root(&props, vec![], vec![]);
/// let html = render(&node);
/// assert!(html.contains(r#"role="alert""#));
/// assert!(html.contains("fd-alert--status-error"));
/// ```
#[must_use]
pub fn root<'a>(props: &AlertProps, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[
        ("status", props.status.value()),
        ("variant", props.variant.value()),
        ("size", props.size.value()),
    ]);
    let mut merged: Vec<(&str, &str)> = vec![role("alert"), ("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "div", merged, children)
}

/// indicator パーツ（`<span>`。アイコン等の装飾要素）を組み立てる。
#[must_use]
pub fn indicator<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("indicator", "span", attrs, children)
}

/// content パーツ（`<div>`。title/description をまとめる）を組み立てる。
#[must_use]
pub fn content<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("content", "div", attrs, children)
}

/// title パーツ（`<div>`）を組み立てる。
#[must_use]
pub fn title<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("title", "div", attrs, children)
}

/// description パーツ（`<div>`）を組み立てる。
#[must_use]
pub fn description<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("description", "div", attrs, children)
}

/// pre-styled-only `action` パート（`<div>`、イシュー #2043）を組み立てる。
///
/// root の最後の子として置くとアクション（`button` 等）が右側（RTL では
/// 左側）へ押し出される。headless-ui に対応する anatomy を持たない
/// pre-styled 独自パート（モジュール doc「pre-styled-only `action` パート」
/// 節参照）。クリック配線・送信処理は内包せず、呼び出し側が children に
/// 渡すノードへ配線する。
#[must_use]
pub fn action<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("action", "div", attrs, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_props_render_info_subtle_md() {
        let html = render(&root(&AlertProps::default(), vec![], vec![]));
        assert!(html.contains(r#"role="alert""#));
        assert_eq!(
            html,
            r#"<div data-scope="alert" data-part="root" role="alert" class="fd-alert--status-info fd-alert--variant-subtle fd-alert--size-md"></div>"#
        );
    }

    #[test]
    fn status_enumeration_maps_to_expected_classes() {
        for (status, class) in [
            (AlertStatus::Info, "fd-alert--status-info"),
            (AlertStatus::Success, "fd-alert--status-success"),
            (AlertStatus::Warning, "fd-alert--status-warning"),
            (AlertStatus::Error, "fd-alert--status-error"),
            (AlertStatus::Neutral, "fd-alert--status-neutral"),
        ] {
            let props = AlertProps {
                status,
                ..AlertProps::default()
            };
            let html = render(&root(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"{class} fd-alert--variant-subtle fd-alert--size-md\""
                )),
                "status={status:?} -> {html}"
            );
        }
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (AlertVariant::Subtle, "fd-alert--variant-subtle"),
            (AlertVariant::Surface, "fd-alert--variant-surface"),
            (AlertVariant::Solid, "fd-alert--variant-solid"),
            (AlertVariant::Outline, "fd-alert--variant-outline"),
        ] {
            let props = AlertProps {
                variant,
                ..AlertProps::default()
            };
            let html = render(&root(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"fd-alert--status-info {class} fd-alert--size-md\""
                )),
                "variant={variant:?} -> {html}"
            );
        }
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-alert--size-xs"),
            (Size::Sm, "fd-alert--size-sm"),
            (Size::Md, "fd-alert--size-md"),
            (Size::Lg, "fd-alert--size-lg"),
            (Size::Xl, "fd-alert--size-xl"),
        ] {
            let props = AlertProps {
                size,
                ..AlertProps::default()
            };
            let html = render(&root(&props, vec![], vec![]));
            assert!(
                html.contains(&format!(
                    "class=\"fd-alert--status-info fd-alert--variant-subtle {class}\""
                )),
                "size={size:?} -> {html}"
            );
        }
    }

    #[test]
    fn parts_use_expected_tags_and_data_part() {
        assert!(render(&indicator(vec![], vec![]))
            .starts_with(r#"<span data-scope="alert" data-part="indicator""#));
        assert!(render(&content(vec![], vec![]))
            .starts_with(r#"<div data-scope="alert" data-part="content""#));
        assert!(render(&title(vec![], vec![]))
            .starts_with(r#"<div data-scope="alert" data-part="title""#));
        assert!(render(&description(vec![], vec![]))
            .starts_with(r#"<div data-scope="alert" data-part="description""#));
        assert!(render(&action(vec![], vec![]))
            .starts_with(r#"<div data-scope="alert" data-part="action""#));
    }

    #[test]
    fn composed_alert_snapshot() {
        let props = AlertProps {
            status: AlertStatus::Warning,
            ..AlertProps::default()
        };
        let node = root(
            &props,
            vec![],
            vec![content(
                vec![],
                vec![
                    title(vec![], vec![text("Heads up")]),
                    description(vec![], vec![text("Something needs attention")]),
                ],
            )],
        );
        let html = render(&node);
        assert_eq!(
            html,
            concat!(
                r#"<div data-scope="alert" data-part="root" role="alert" class="fd-alert--status-warning fd-alert--variant-subtle fd-alert--size-md">"#,
                r#"<div data-scope="alert" data-part="content">"#,
                r#"<div data-scope="alert" data-part="title">Heads up</div>"#,
                r#"<div data-scope="alert" data-part="description">Something needs attention</div>"#,
                r#"</div>"#,
                r#"</div>"#,
            )
        );
    }

    #[test]
    fn caller_class_attr_on_root_is_dropped_not_duplicated() {
        let html = render(&root(
            &AlertProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_title_children_is_escaped() {
        let html = render(&title(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    /// イシュー #2043: 新設 [`action`] も他パートと同じく `text()` を経由した
    /// children が既定エスケープを通ることを固定する。
    #[test]
    fn xss_payload_in_action_children_is_escaped() {
        let html = render(&action(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    /// イシュー #2043: root + indicator + content + action の合成
    /// （shadcn `AlertAction` 相当）が右端の兄弟パートとして描画されることを
    /// 固定する。
    #[test]
    fn composed_alert_with_action_snapshot() {
        let node = root(
            &AlertProps::default(),
            vec![],
            vec![
                indicator(vec![], vec![text("!")]),
                content(
                    vec![],
                    vec![
                        title(vec![], vec![text("Update available")]),
                        description(vec![], vec![text("A new version is ready.")]),
                    ],
                ),
                action(vec![], vec![text("Update")]),
            ],
        );
        let html = render(&node);
        assert_eq!(
            html,
            concat!(
                r#"<div data-scope="alert" data-part="root" role="alert" class="fd-alert--status-info fd-alert--variant-subtle fd-alert--size-md">"#,
                r#"<span data-scope="alert" data-part="indicator">!</span>"#,
                r#"<div data-scope="alert" data-part="content">"#,
                r#"<div data-scope="alert" data-part="title">Update available</div>"#,
                r#"<div data-scope="alert" data-part="description">A new version is ready.</div>"#,
                r#"</div>"#,
                r#"<div data-scope="alert" data-part="action">Update</div>"#,
                r#"</div>"#,
            )
        );
    }

    /// イシュー #1553: 公開 API（[`AlertStatus`]）のクラス出力は不変のまま、
    /// 内部で status ごとに 6 役割の `--fandhe-palette-*` を対応する
    /// セマンティック色へ束ね、radii トークンを参照することを固定する。
    #[test]
    fn css_output_declares_status_palette_mapping_and_radius_token() {
        let out = css();
        assert!(out.contains("border-radius: var(--fandhe-radius-md);"));
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-info)"));
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-success)"));
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-warning)"));
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-danger)"));
        assert!(out.contains("--fandhe-palette: var(--fandhe-color-neutral)"));
        assert!(out.contains("--fandhe-palette-subtle: var(--fandhe-color-info-subtle)"));
        assert!(out.contains("color: var(--fandhe-palette-fg-subtle);"));
    }

    /// イシュー #1553: size 軸が root へ `--fandhe-alert-*` custom property を
    /// 登録し、既定 `Md` が構造的に保証されることを固定する
    /// （[`crate::recipe::SlotRecipe::size_variants`] の契約）。
    #[test]
    fn css_output_declares_size_custom_properties() {
        let out = css();
        assert!(out.contains("--fandhe-alert-padding: var(--fandhe-space-2);"));
        assert!(out.contains("--fandhe-alert-padding: var(--fandhe-space-4);"));
        assert!(out.contains("--fandhe-alert-indicator-size: var(--fandhe-font-font-size-3xl);"));
    }
}
