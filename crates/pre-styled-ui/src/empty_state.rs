//! EmptyState（イシュー #765）: slot recipe styled 部品。indicator/title/
//! description/actions を持つ空状態レイアウトコンテナ。
//!
//! [`crate::card`] と同じく中立的なレイアウトコンテナであり、`role`/
//! `aria-*` は付与しない（`.claude/rules/coding-rust.md` 準拠のプレーンな
//! HTML を尊重する方針）。特定のセマンティック色を持つ意味論を持たないため
//! `color-palette` 軸は提供しない（[`crate::card`] rustdoc と同型の判断）。
//! `title` を見出し要素（`<h1>`〜`<h6>`）にせず `<div>` とするのは、
//! `fandhe-frontend-docs-site` の showcase が `.docs-content h3` 等の
//! セレクタで見出しを拾うテスト・スタイルを持ち、部品埋め込み位置に応じて
//! 見出しレベルが変わり得る呼び出し文脈では固定レベルの見出し要素を強制
//! しない方が安全という判断（[`crate::alert::title`] と同型の判断）。
//!
//! # 参考サイト基準への調整（イシュー #1560）
//!
//! 参照サイト（chakra-ui `EmptyState`。ark-ui / Radix Themes / Radix
//! Primitives には対応部品がないため比較対象は chakra-ui のみ）の
//! スクリーンショット
//! （`docs/design/reference-screenshots/chakra-empty-state-{1,2,3}.png`）と
//! `themes-empty-state.png` を比較した結果を記録する。
//!
//! - **サイズ**: 是正前は `size`（Xs〜Xl）が root の `padding` のみを
//!   `1rem`〜`5rem` の生リテラルで切り替え、`content` の gap・`indicator`
//!   の font-size・`title`/`description` の font-size は固定値だった。
//!   chakra の size スケール（`sm`/`md`（既定）/`lg`）は padding・content
//!   gap・indicator/title/description の文字サイズが一斉に連動するため、
//!   [`callout`](crate::callout) と同型の「root の
//!   `--fandhe-empty-state-*` custom property 一本化」パターンへ是正した
//!   （[`SlotRecipe::size_variants`] を使用）。chakra が持たない Xs/Xl は
//!   #1681 の外挿方針を踏襲し、gap（テキスト間）+ section-gap
//!   （indicator 下 / actions 上の区画間余白）の合計が chakra の content
//!   gap（sm 1rem / md 1.5rem / lg 2rem）に一致するよう等差で外挿した。
//! - **バリアント**: 変更なし。chakra `EmptyState` に `variant` prop は
//!   存在しない（`Root.size` のみ）ため、[`crate::card`] と同型の中立
//!   コンテナ判断を維持し `variant`/`color-palette` 軸を追加しない。
//! - **色**: `indicator` を `--fandhe-color-fg-muted` から
//!   `--fandhe-color-fg-subtle` へ変更した（chakra `indicator` の
//!   `color: fg.subtle` に整合。`description` は chakra `fg.muted` の
//!   ままで変更なし）。生の色リテラルは持ち込まない。
//! - **状態（`data-*`）**: 変更なし。`data-scope`/`data-part` のみ。
//! - **ダーク**: 追加した custom property はすべて既存トークン
//!   （`--fandhe-space-*`/`--fandhe-font-font-size-*`/
//!   `--fandhe-color-fg-subtle`）参照のため `write_dark_declarations` へ
//!   自動追従する。新規トークン追加はない。
//! - **フォーカス**: 非適用（変更なし）。表示専用の静的レイアウト
//!   コンテナであり
//!   `docs/design/pre-styled-ui-interaction-visual-language.md` §3
//!   「表示専用には付けない」に該当する（`actions` 内の button 等は
//!   button 自身のフォーカスリングを持つ）。
//! - **余白・角丸・影**: padding・gap を `--fandhe-space-*` トークン
//!   経由の custom property へ是正した（角丸・影は参照元同様なし）。
//!
//! ## 意図的に合わせない点（スコープ外）
//!
//! 1. **`_icon: { boxSize: 1em }`（chakra の indicator 内 svg 自動
//!    サイズ）**: [`SlotRecipe`] は子孫セレクタ（`svg`）を表現できない。
//!    `indicator` の `font-size` を size 連動させ
//!    `display: inline-flex; line-height: 1` で 1em 基準の整列を作ることで、
//!    呼び出し側が `1em` 指定のアイコンを渡せば同等の見た目になる。
//! 2. **title / description のグルーピング用の専用 slot 新設**: 参照元は
//!    title+description を別コンテナ（小 gap）で包み、indicator /
//!    テキスト群 / 操作群の間を content gap（大）で分ける。本部品の
//!    anatomy（title/description が `content` の直接の子）を変えずに
//!    同じ視覚リズムを得るため、`content` の gap を「テキスト間の小
//!    gap」とし、`indicator` の `margin-bottom` と `actions` の
//!    `margin-top` に「区画間の追加余白」を持たせた（anatomy 変更は
//!    破壊的変更であり見送る）。
//! 3. **indicator Lg/Xl の font-size**: chakra `6xl`（3.75rem）は
//!    タイポグラフィトークン上限 `4xl`（2.25rem）を超えるため、
//!    [`crate::heading`] と同型の判断でトークンは追加せず Lg/Xl のみ
//!    rem リテラルを使う。
//! 4. **`actions` slot の維持**: 参照元は操作要素を content に直接
//!    置くが、既存 API・anatomy を壊さないため `actions` slot は維持し
//!    size 連動の余白のみ調整する。
//! 5. タイポグラフィトークン `5xl`/`6xl` の追加、ブラウザ実機での
//!    スクリーンショット再取得は行わない（別 Phase の一括撮影運用に
//!    委ねる）。
//!
//! # イシュー #2047（shadcn/ui 突合）
//!
//! 参照サイト（shadcn/ui `Empty`。2026-09-07 改訂〔#2153〕で 3 者主基準の
//! 1 つ）のスクリーンショット
//! （`docs/design/reference-screenshots/shadcn-empty-{1,2,3}.png`）と
//! `themes-empty-state.png` を比較し、chakra-ui / Radix Themes 基準では
//! 拾えなかった欠落バリアント・状態を補完した。
//!
//! ## 補完した点（純追加）
//!
//! - **root `variant` 軸**（[`EmptyStateVariant`]）: 既定 `Plain`（従来
//!   どおり class を出力しない）に加え、`Outline`（shadcn の
//!   `border-dashed` 例相当。`1px dashed` + `radius-lg`）と `Subtle`
//!   （shadcn の `Background`〔`from-muted/50` グラデーション〕例相当。
//!   トークン体系で半透明グラデーションは表現できないため単色
//!   `--fandhe-color-bg-subtle` へ置換）を追加した。
//! - **indicator `variant` 軸**（[`EmptyStateIndicatorVariant`]、
//!   [`indicator_with`]）: 既定 `Plain`（[`indicator`] と同一出力）に
//!   加え、`Boxed`（shadcn `EmptyMedia variant="icon"` の `bg-muted`
//!   角丸タイル相当）を追加した。タイル寸法は shadcn の固定 `40px` では
//!   なく `padding: var(--fandhe-space-2)` とし、size 軸連動の
//!   `font-size` に追従させる（新規 custom property を size ブロックへ
//!   足すと既存 golden が変わるため）。
//!
//! いずれも [`EmptyStateProps::variant`] の既定値 `Plain`・`indicator()`
//! の出力は変更前とバイト一致（純追加原則）。
//!
//! ## 参照競合の判定
//!
//! - root 枠線（Outline）は shadcn-ui の値を採る（chakra-ui / Radix
//!   Themes に対応 variant 軸が存在せず競合しないため）。
//! - root 背景（Subtle）は chakra-ui / Radix Themes 系の単色トークン
//!   （[`crate::card`] の `Subtle` 相当）を採る（shadcn の半透明
//!   グラデーションはトークン体系で表現できないため）。
//! - indicator タイル（Boxed）は shadcn-ui の値を採る（chakra-ui /
//!   Radix Themes に対応表現がないため）。
//!
//! ## コントラスト
//!
//! Subtle root（`bg-subtle` 背景）上の description（`fg-muted`）は
//! [`crate::card`] の `Subtle` variant と同じ組み合わせであり、
//! `theme.rs` の `CARD_SUBTLE_VARIANT_PAIRS`（本文相当 4.5:1 契約）で
//! 既に検証済み。既定 `Plain` indicator（`fg-subtle`、大型装飾グリフ）が
//! Subtle root に乗る組（`fg-subtle`/`bg-subtle`）は `theme.rs` の
//! `LARGE_TEXT_UI_PAIRS`（3:1 契約）へ本イシューで追加した。Boxed
//! indicator（`fg`/`bg-muted`）は `theme.rs` の `BODY_TEXT_PAIRS`
//! （`("fg", "bg-muted")`、4.5:1 契約）で既に検証済み。
//!
//! ## 意図的に合わせなかった点
//!
//! - **`EmptyHeader` 相当の slot 新設**: #1560 の判断 2（`content` の
//!   gap + `indicator`/`actions` の section-gap で同じ視覚リズムを得る）
//!   を維持する。slot 追加は base ブロックが golden の中間に挿入され
//!   純追加原則と相性が悪い。
//! - **title の `font-weight: medium` 化**: chakra-ui 基準の `semibold`
//!   を維持する（変更すると既存 golden の視覚変更になるため）。
//! - **description 内 `<a>` の下線・hover 色**: [`crate::recipe::SlotRecipe`]
//!   は子孫セレクタを表現できない（#708 で不採用確定）ため非追随。
//! - **`max-w-sm`/`text-balance`**: base 変更（golden 変更）になるため
//!   非追随。呼び出し側が `attrs` の `style` で付与できる。
//! - **Outline での 1px 分のボックス高さ増**: base へ透明 `border` を
//!   置くと既存 golden が変わるため許容する（意図的な差分）。
//! - タイル寸法の shadcn 固定値（`40px`）・スクリーンショット再取得
//!   （一括撮影運用の管轄）は対象外。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

use crate::recipe::{Size, SlotRecipe, VariantValue};

/// `data-scope="empty-state"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("empty-state");

/// [`SlotRecipe::new`] に渡す slot 一覧（recipe とレンダリング関数の両方が
/// この配列を共有し、slot 名の乖離を防ぐ）。
const SLOTS: &[&str] = &[
    "root",
    "content",
    "indicator",
    "title",
    "description",
    "actions",
];

/// root の見た目 variant（イシュー #2047、shadcn/ui `Empty` 突合。
/// モジュール冒頭「イシュー #2047」節参照）。
///
/// 既定 `Plain` は class を出力しない（既存 `class="fd-empty-state--size-md"`
/// をバイト不変に保つ純追加。[`crate::avatar::root`] の `stacked`/
/// `with_badge` 等と同型の
/// 「条件付き selection 追加」パターン、[`root`] 参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EmptyStateVariant {
    /// 既定。枠線・背景を追加しない。
    #[default]
    Plain,
    /// 破線枠（shadcn `Empty` の `border-dashed` 例相当）。
    Outline,
    /// 淡色単色背景（shadcn `Empty` の `Background`〔グラデーション〕例を
    /// 単色トークンへ置換したもの。モジュール冒頭「参照競合の判定」参照）。
    Subtle,
}

impl VariantValue for EmptyStateVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            EmptyStateVariant::Plain => "plain",
            EmptyStateVariant::Outline => "outline",
            EmptyStateVariant::Subtle => "subtle",
        }
    }
}

/// [`indicator_with`] の見た目 variant（イシュー #2047、shadcn/ui
/// `EmptyMedia` 突合）。
///
/// 既定 `Plain` は [`indicator`] とバイト一致の出力になる（純追加）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EmptyStateIndicatorVariant {
    /// 既定。[`indicator`] と同一（class を出力しない）。
    #[default]
    Plain,
    /// `bg-muted` の角丸タイル（shadcn `EmptyMedia variant="icon"` 相当）。
    Boxed,
}

impl VariantValue for EmptyStateIndicatorVariant {
    fn axis(self) -> &'static str {
        "indicator"
    }

    fn value(self) -> &'static str {
        match self {
            EmptyStateIndicatorVariant::Plain => "plain",
            EmptyStateIndicatorVariant::Boxed => "boxed",
        }
    }
}

/// [`root`] の設定。
#[derive(Debug, Clone, Copy)]
pub struct EmptyStateProps {
    /// サイズ variant（既定 `Md`）。root の `--fandhe-empty-state-*`
    /// custom property 経由で padding・`content` の gap・`indicator`/
    /// `title`/`description` の font-size を連動させる（イシュー #1560、
    /// モジュール冒頭「参考サイト基準への調整」参照）。
    pub size: Size,
    /// root の見た目 variant（既定 `Plain`、イシュー #2047）。
    pub variant: EmptyStateVariant,
}

impl Default for EmptyStateProps {
    fn default() -> Self {
        EmptyStateProps {
            size: Size::Md,
            variant: EmptyStateVariant::Plain,
        }
    }
}

/// EmptyState の recipe（scope `"empty-state"`、[`SLOTS`] の 6 パーツ）。
///
/// [`callout`](crate::callout) と同型の「root の custom property 一本化」
/// パターンを採用する（イシュー #1560）。padding・テキスト間 gap・区画間
/// 余白（indicator 下 / actions 上）・indicator/title/description の
/// font-size をそれぞれ `--fandhe-empty-state-*` custom property として
/// [`SlotRecipe::size_variants`] で一括登録し、各 slot の base 宣言は
/// Md 値をフォールバックにして参照する。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("empty-state", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("box-sizing", "border-box"),
                decl("width", "100%"),
                decl(
                    "padding",
                    "var(--fandhe-empty-state-padding, var(--fandhe-space-12) var(--fandhe-space-8))",
                ),
            ],
        )
        .base(
            "content",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl(
                    "gap",
                    "var(--fandhe-empty-state-gap, var(--fandhe-space-2))",
                ),
                decl("text-align", "center"),
            ],
        )
        .base(
            "indicator",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("line-height", "1"),
                decl(
                    "font-size",
                    "var(--fandhe-empty-state-indicator-size, var(--fandhe-font-font-size-4xl))",
                ),
                decl("color", "var(--fandhe-color-fg-subtle)"),
                decl(
                    "margin-bottom",
                    "var(--fandhe-empty-state-section-gap, var(--fandhe-space-4))",
                ),
            ],
        )
        .base(
            "title",
            vec![
                decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
                decl(
                    "font-size",
                    "var(--fandhe-empty-state-title-size, var(--fandhe-font-font-size-lg))",
                ),
            ],
        )
        .base(
            "description",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl(
                    "font-size",
                    "var(--fandhe-empty-state-description-size, var(--fandhe-font-font-size-sm))",
                ),
            ],
        )
        .base(
            "actions",
            vec![
                decl("display", "flex"),
                decl("flex-wrap", "wrap"),
                decl("justify-content", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl(
                    "margin-top",
                    "var(--fandhe-empty-state-section-gap, var(--fandhe-space-4))",
                ),
            ],
        )
        // イシュー #1560: chakra-ui EmptyState の size スケール（sm/md/lg）に
        // 揃え、Xs/Xl は #1681 の外挿方針を踏襲する。indicator の Lg/Xl は
        // タイポグラフィトークン上限（4xl = 2.25rem）を超えるため、
        // heading 同型の判断でリテラル値を使う（モジュール冒頭「意図的に
        // 合わせない点」3 参照）。
        .size_variants(
            "root",
            &[
                (
                    Size::Xs,
                    vec![
                        decl(
                            "--fandhe-empty-state-padding",
                            "var(--fandhe-space-4) var(--fandhe-space-3)",
                        ),
                        decl("--fandhe-empty-state-gap", "var(--fandhe-space-1)"),
                        decl("--fandhe-empty-state-section-gap", "var(--fandhe-space-2)"),
                        decl(
                            "--fandhe-empty-state-indicator-size",
                            "var(--fandhe-font-font-size-xl)",
                        ),
                        decl(
                            "--fandhe-empty-state-title-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                        decl(
                            "--fandhe-empty-state-description-size",
                            "var(--fandhe-font-font-size-xs)",
                        ),
                    ],
                ),
                (
                    Size::Sm,
                    vec![
                        decl(
                            "--fandhe-empty-state-padding",
                            "var(--fandhe-space-6) var(--fandhe-space-4)",
                        ),
                        decl("--fandhe-empty-state-gap", "var(--fandhe-space-1-5)"),
                        decl(
                            "--fandhe-empty-state-section-gap",
                            "var(--fandhe-space-2-5)",
                        ),
                        decl(
                            "--fandhe-empty-state-indicator-size",
                            "var(--fandhe-font-font-size-2xl)",
                        ),
                        decl(
                            "--fandhe-empty-state-title-size",
                            "var(--fandhe-font-font-size-md)",
                        ),
                        decl(
                            "--fandhe-empty-state-description-size",
                            "var(--fandhe-font-font-size-xs)",
                        ),
                    ],
                ),
                (
                    Size::Md,
                    vec![
                        decl(
                            "--fandhe-empty-state-padding",
                            "var(--fandhe-space-12) var(--fandhe-space-8)",
                        ),
                        decl("--fandhe-empty-state-gap", "var(--fandhe-space-2)"),
                        decl("--fandhe-empty-state-section-gap", "var(--fandhe-space-4)"),
                        decl(
                            "--fandhe-empty-state-indicator-size",
                            "var(--fandhe-font-font-size-4xl)",
                        ),
                        decl(
                            "--fandhe-empty-state-title-size",
                            "var(--fandhe-font-font-size-lg)",
                        ),
                        decl(
                            "--fandhe-empty-state-description-size",
                            "var(--fandhe-font-font-size-sm)",
                        ),
                    ],
                ),
                (
                    Size::Lg,
                    vec![
                        decl(
                            "--fandhe-empty-state-padding",
                            "var(--fandhe-space-16) var(--fandhe-space-12)",
                        ),
                        decl("--fandhe-empty-state-gap", "var(--fandhe-space-3)"),
                        decl("--fandhe-empty-state-section-gap", "var(--fandhe-space-5)"),
                        // chakra の 6xl（3.75rem）はトークン上限 4xl を超える
                        // ため、heading と同型の判断でリテラル値を使う。
                        decl("--fandhe-empty-state-indicator-size", "3.75rem"),
                        decl(
                            "--fandhe-empty-state-title-size",
                            "var(--fandhe-font-font-size-xl)",
                        ),
                        decl(
                            "--fandhe-empty-state-description-size",
                            "var(--fandhe-font-font-size-md)",
                        ),
                    ],
                ),
                (
                    Size::Xl,
                    vec![
                        decl(
                            "--fandhe-empty-state-padding",
                            "var(--fandhe-space-20) var(--fandhe-space-16)",
                        ),
                        decl("--fandhe-empty-state-gap", "var(--fandhe-space-4)"),
                        decl("--fandhe-empty-state-section-gap", "var(--fandhe-space-6)"),
                        decl("--fandhe-empty-state-indicator-size", "4.5rem"),
                        decl(
                            "--fandhe-empty-state-title-size",
                            "var(--fandhe-font-font-size-2xl)",
                        ),
                        decl(
                            "--fandhe-empty-state-description-size",
                            "var(--fandhe-font-font-size-lg)",
                        ),
                    ],
                ),
            ],
        )
        // イシュー #2047: shadcn/ui `Empty` 突合で純追加した root/indicator
        // の variant 軸。`size_variants` より後ろに登録することで、golden
        // CSS の既存ブロック（base → size variants）を変更せず末尾へ追記
        // する（モジュール冒頭「イシュー #2047」節参照）。
        .variant(
            EmptyStateVariant::Outline,
            "root",
            vec![
                decl("border", "1px dashed var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-lg)"),
            ],
        )
        .variant(
            EmptyStateVariant::Subtle,
            "root",
            vec![
                decl("background", "var(--fandhe-color-bg-subtle)"),
                decl("border-radius", "var(--fandhe-radius-lg)"),
            ],
        )
        .variant(
            EmptyStateIndicatorVariant::Boxed,
            "indicator",
            vec![
                decl("padding", "var(--fandhe-space-2)"),
                decl("border-radius", "var(--fandhe-radius-lg)"),
                decl("background", "var(--fandhe-color-bg-muted)"),
                decl("color", "var(--fandhe-color-fg)"),
            ],
        )
}

/// EmptyState の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// root パーツ（`<div>`）を組み立てる。`size`/`variant` に応じたクラスを
/// 付与する唯一のパーツ（[`crate::class_attr::drop_class_attr`] により
/// 呼び出し側の `class` は除去してから合成する）。
///
/// イシュー #2047: `props.variant` が既定 `Plain` のときは `selection` へ
/// `variant` 軸を追加しない（[`crate::avatar::root`] の `stacked`/
/// `with_badge` と同型の条件付き push）。このため既定 `class` 出力
/// （`fd-empty-state--size-md`）は変更前とバイト一致のまま保たれる。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::empty_state::{self, EmptyStateProps};
///
/// let node = empty_state::root(&EmptyStateProps::default(), vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="empty-state" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    props: &EmptyStateProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let mut selection: Vec<(&str, &str)> = vec![("size", props.size.value())];
    if props.variant != EmptyStateVariant::Plain {
        selection.push((props.variant.axis(), props.variant.value()));
    }
    let class = recipe.variant_classes(&selection);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "div", merged, children)
}

/// content パーツ（`<div>`）を組み立てる。variant を持たないため `class` は
/// 付与せず、呼び出し側 `attrs` をそのまま連結する。
#[must_use]
pub fn content<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("content", "div", attrs, children)
}

/// indicator パーツ（`<span>`）を組み立てる。アイコン等を子ノードとして
/// 受け取る（本クレートは外部リソース・アイコンフォントを参照しない方針の
/// ため、具体的な意匠は呼び出し側が children として渡す）。
#[must_use]
pub fn indicator<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("indicator", "span", attrs, children)
}

/// indicator パーツを `variant` 付きで組み立てる（イシュー #2047、shadcn
/// `EmptyMedia` 突合）。
///
/// `Plain` は [`indicator`] とバイト一致の出力になる（純追加）。`Boxed` は
/// [`crate::avatar::badge`] と同型に [`SlotRecipe::variant_class`] を単独で
/// 呼び、`indicator` slot が持つ唯一の軸（`indicator`）のクラスのみを
/// 付与する（[`SlotRecipe::variant_classes`] は使わない。`size`/`variant`
/// 等 root 専用軸の既定値補完が indicator slot へ誤って波及しないため）。
/// [`crate::class_attr::drop_class_attr`] により呼び出し側の `class` は
/// 除去してから合成する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::empty_state::{self, EmptyStateIndicatorVariant};
///
/// let node = empty_state::indicator_with(EmptyStateIndicatorVariant::Boxed, vec![], vec![]);
/// assert!(render(&node).contains("fd-empty-state--indicator-boxed"));
/// ```
#[must_use]
pub fn indicator_with<'a>(
    variant: EmptyStateIndicatorVariant,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    if variant == EmptyStateIndicatorVariant::Plain {
        return indicator(attrs, children);
    }
    let recipe = recipe();
    let class = recipe.variant_class(variant);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("indicator", "span", merged, children)
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

/// actions パーツ（`<div>`）を組み立てる。ボタン等の操作導線を並べる。
#[must_use]
pub fn actions<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("actions", "div", attrs, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_variant_is_md() {
        let html = render(&root(&EmptyStateProps::default(), vec![], vec![]));
        assert!(html.contains("fd-empty-state--size-md"));
    }

    #[test]
    fn size_variants_map_to_expected_classes() {
        for (size, class) in [
            (Size::Xs, "fd-empty-state--size-xs"),
            (Size::Sm, "fd-empty-state--size-sm"),
            (Size::Md, "fd-empty-state--size-md"),
            (Size::Lg, "fd-empty-state--size-lg"),
            (Size::Xl, "fd-empty-state--size-xl"),
        ] {
            let props = EmptyStateProps {
                size,
                ..Default::default()
            };
            let html = render(&root(&props, vec![], vec![]));
            assert!(
                html.contains(&format!("class=\"{class}\"")),
                "size={size:?} -> {html}"
            );
        }
    }

    #[test]
    fn parts_use_expected_tags_and_data_part() {
        assert!(render(&content(vec![], vec![]))
            .starts_with(r#"<div data-scope="empty-state" data-part="content""#));
        assert!(render(&indicator(vec![], vec![]))
            .starts_with(r#"<span data-scope="empty-state" data-part="indicator""#));
        assert!(render(&title(vec![], vec![]))
            .starts_with(r#"<div data-scope="empty-state" data-part="title""#));
        assert!(render(&description(vec![], vec![]))
            .starts_with(r#"<div data-scope="empty-state" data-part="description""#));
        assert!(render(&actions(vec![], vec![]))
            .starts_with(r#"<div data-scope="empty-state" data-part="actions""#));
    }

    #[test]
    fn composed_empty_state_snapshot() {
        let node = root(
            &EmptyStateProps::default(),
            vec![],
            vec![content(
                vec![],
                vec![
                    indicator(vec![], vec![]),
                    title(vec![], vec![text("No results")]),
                    description(vec![], vec![text("Try a different search.")]),
                    actions(vec![], vec![]),
                ],
            )],
        );
        let html = render(&node);
        assert_eq!(
            html,
            concat!(
                r#"<div data-scope="empty-state" data-part="root" class="fd-empty-state--size-md">"#,
                r#"<div data-scope="empty-state" data-part="content">"#,
                r#"<span data-scope="empty-state" data-part="indicator"></span>"#,
                r#"<div data-scope="empty-state" data-part="title">No results</div>"#,
                r#"<div data-scope="empty-state" data-part="description">Try a different search.</div>"#,
                r#"<div data-scope="empty-state" data-part="actions"></div>"#,
                r#"</div>"#,
                r#"</div>"#,
            )
        );
    }

    #[test]
    fn root_has_no_role_attribute() {
        let html = render(&root(&EmptyStateProps::default(), vec![], vec![]));
        assert!(!html.contains("role="));
    }

    #[test]
    fn caller_class_attr_on_root_is_dropped_not_duplicated() {
        let html = render(&root(
            &EmptyStateProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_title_and_description_children_is_escaped() {
        let html = render(&title(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));

        let html = render(&description(
            vec![],
            vec![text("<script>alert(2)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(2)&lt;/script&gt;"));
    }

    #[test]
    fn css_output_declares_padding_and_subtle_fg_tokens() {
        let out = css();
        assert!(out.contains(
            "--fandhe-empty-state-padding: var(--fandhe-space-12) var(--fandhe-space-8);"
        ));
        assert!(out.contains("color: var(--fandhe-color-fg-subtle);"));
    }

    #[test]
    fn css_output_is_deterministic() {
        assert_eq!(css(), css());
    }

    // イシュー #2047: shadcn/ui `Empty` 突合で追加した root/indicator variant
    // 軸の回帰テスト。既定 `Plain` がバイト不変であることと、`Outline`/
    // `Subtle`/`Boxed` のクラス付与・エスケープを固定する。

    #[test]
    fn default_props_class_is_unchanged() {
        let html = render(&root(&EmptyStateProps::default(), vec![], vec![]));
        assert!(html.contains(r#"class="fd-empty-state--size-md""#));
        assert!(!html.contains("variant-plain"));
    }

    #[test]
    fn root_variant_classes() {
        let html = render(&root(
            &EmptyStateProps {
                size: Size::Md,
                variant: EmptyStateVariant::Outline,
            },
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"class="fd-empty-state--size-md fd-empty-state--variant-outline""#));

        let html = render(&root(
            &EmptyStateProps {
                size: Size::Md,
                variant: EmptyStateVariant::Subtle,
            },
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"class="fd-empty-state--size-md fd-empty-state--variant-subtle""#));
    }

    #[test]
    fn indicator_with_plain_equals_indicator() {
        let a = render(&indicator_with(
            EmptyStateIndicatorVariant::Plain,
            vec![("data-testid", "x")],
            vec![text("icon")],
        ));
        let b = render(&indicator(vec![("data-testid", "x")], vec![text("icon")]));
        assert_eq!(a, b);
    }

    #[test]
    fn indicator_with_boxed_adds_class_and_drops_caller_class() {
        let html = render(&indicator_with(
            EmptyStateIndicatorVariant::Boxed,
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(html.contains(r#"class="fd-empty-state--indicator-boxed""#));
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_indicator_with_children_and_attrs_is_escaped() {
        let html = render(&indicator_with(
            EmptyStateIndicatorVariant::Boxed,
            vec![("data-testid", "\"><script>alert(1)</script>")],
            vec![text("<script>alert(2)</script>")],
        ));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(2)&lt;/script&gt;"));
    }

    #[test]
    fn css_output_declares_variant_blocks() {
        let out = css();
        assert!(out.contains(
            r#"[data-scope="empty-state"][data-part="root"].fd-empty-state--variant-outline"#
        ));
        assert!(out.contains(
            r#"[data-scope="empty-state"][data-part="root"].fd-empty-state--variant-subtle"#
        ));
        assert!(out.contains(
            r#"[data-scope="empty-state"][data-part="indicator"].fd-empty-state--indicator-boxed"#
        ));
    }
}
