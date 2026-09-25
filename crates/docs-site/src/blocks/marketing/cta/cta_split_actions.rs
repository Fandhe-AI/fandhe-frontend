//! `cta-split-actions` block（イシュー #2758。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー」配下、対応表 ID R0881 を主参照とし、
//! R0067 / R0071 / R0447 / R0451 / R0882 を配色・訴求点の差分として集約
//! した合成例。見出しとボタン列を左右両端へ揃える CTA）。取得手段・
//! ファイル名・内部コンポーネント識別子は記載しない（[`super::
//! cta_feature_links`] と同じライセンス上の転記制限、対応表 ID のみを
//! 記す）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `button` / `card` / `icon` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 4 インスタンスで配色差分を表現する（各対応表 ID との対応）
//!
//! 1 つの Demo に 4 インスタンスを縦積みし、tone ごとの配色差分を並べて
//! 見せる（[`super::super::banner::banner_full_width_bar`] と同型の
//! 「1 block・複数インスタンス」構成）。
//!
//! - `plain`（R0881 主参照）: 背景なし。2 行構成の見出し（2 行目のみ
//!   アクセント色）+ ボタン 2 個。
//! - `subtle`（R0882 の淡色ブランド面 + R0067/R0071 の訴求ポイント）:
//!   `--fandhe-color-accent-subtle` の面に見出し 1 行 + 訴求ポイント 3 点
//!   （チェックアイコン + `text`）+ ボタン 2 個。R0067 の 2 点版・R0071 の
//!   2 列配置は個別インスタンス化せず `site/blocks/cta-split-actions.md`
//!   「原案差分メモ」節へ記す（Demo の肥大化を避ける判断）。
//! - `inverted`（R0447 の反転配色）: 前景色と背景色を入れ替えた面に見出し +
//!   補足 1 行 + ボタン 2 個を横並び。
//! - `card`（R0451 のアクセント色の浮いたカード）: [`card::root`]
//!   （`CardVariant::Elevated`）のカード面をアクセント色へ上書きし、内側に
//!   見出し + ボタン 2 個を横並び。
//!
//! # レイアウトとレスポンシブ（`64rem` を境に反転）
//!
//! 各行（`[data-blocks-cta-split-actions-row]`）は既定（`64rem` 未満）で
//! 縦積み（見出し領域の下にボタン列）、`64rem` 以上で
//! `justify-content: space-between` の横並びへ切り替え、見出しとボタン列を
//! 両端へ揃える。ブレークポイントは
//! [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`]（1024px =
//! 64rem）と一致するリテラル値を [`LAYOUT_CSS`] へ直書きする（テーマの
//! breakpoint トークンは `@media` 条件式の中では解決できないため、
//! [`super::super::blog::blog_grid_text`]・[`cta_feature_links`] と同じ
//! 判断）。
//!
//! # tone 上書きの詳細度 (0,4,0)（[`super::super::banner::
//! banner_full_width_bar`] と同じ判断軸）
//!
//! `plain`/`subtle`/`inverted` の 3 tone は素の `div`（row）へ
//! `[data-blocks-cta-split-actions-row][data-blocks-cta-split-actions-tone=
//! "…"]`（属性 2 個 = 詳細度 (0,2,0)）で背景・文字色を上書きする。`card`
//! tone は [`card::root`] の recipe base（`[data-scope="card"]
//! [data-part="root"].fd-card--…`、詳細度 (0,3,0)）より高い詳細度が必要
//! なため、`.blocks-cta-split-actions [data-scope="card"][data-part="root"]
//! [data-blocks-cta-split-actions-tone="card"]`（1 クラス + 3 属性 =
//! 詳細度 (0,4,0)）で上書きする。`inverted`/`card` の両 tone 面に置く
//! ボタンは、既定の accent 配色のままだと背景と同化する（Primary）・
//! 前景色が読みにくい（Secondary）ため、`data-blocks-cta-split-actions-
//! primary`/`-secondary` 属性 + tone 祖先セレクタ（`[data-blocks-cta-
//! split-actions-tone="…"] [data-scope="button"][data-part="root"]
//! [data-blocks-cta-split-actions-primary/-secondary]`、詳細度 (0,4,0)）で
//! 反転・継承へ上書きする。
//!
//! # `drop_class_attr` と CSS フックの選び方
//!
//! `heading::heading` / `text::text` / `button::button` / `card::root` /
//! `card::body` / `icon::icon` はいずれも `drop_class_attr` により呼び出し
//! 側 `attrs` の `class` を黙って除去する契約を持つため、Demo 固有の
//! スタイルフックは `data-blocks-cta-split-actions-*` 属性で渡し、
//! [`LAYOUT_CSS`] 側も同じ属性セレクタで対応する（[`cta_feature_links`]
//! と同じ判断）。素の `div`/`span` には `class` がそのまま効くため、
//! スタック・行・訴求ポイント一覧等は従来どおり
//! `.blocks-cta-split-actions-*` クラスセレクタを使う。グリッド用の
//! ルート class（`blocks-cta-split-actions-stack`）は [`Block::demo_class`]
//! （`blocks-cta-split-actions`）と意図的に別名にする（同名にすると
//! `crate::blocks::insert_generated_sections` が付与する Demo ラッパーへも
//! スタック規則が当たってしまう、イシュー #2809 で Bugbot が指摘した
//! 不具合と同型、[`cta_feature_links`] モジュール doc 参照）。
//!
//! # アイコンは自作の抽象幾何図形
//!
//! 参照元のアイコンは使わず、チェックマーク風の抽象線画（[`check_icon`]）
//! を自作する（[`cta_feature_links::geo_icon`] と同型のパターン）。
//!
//! # `crate::blocks::dummy_assets` を使わない理由（コードフェンス自己完結）
//!
//! `crate::blocks::dummy_assets` は `pub(crate)` のため、Markdown 原稿の
//! Rust コードフェンス（`// blocks-code:begin`/`:end` マーカー内、クレート
//! 外から読めるコード例として提示される）がこれを参照すると単体では
//! コンパイルできなくなる（イシュー #2811 のレビュー指摘、
//! [`cta_feature_links`] と同じ判断軸）。文言は独自に書いた架空の日本語
//! 文言をリテラルとして直接持つ（実企業名・実クレデンシャル・PII を
//! 含まない）。
//!
//! # リンクを持たない・`id`/`aria-labelledby` を出力しない
//!
//! ボタンのみで構成し `link`/`link-overlay` を使わないため `href="#"`
//! 問題は生じない。`id`/`aria-labelledby` は一切出力しない
//! （`demo_output_has_no_dangling_aria_references_or_duplicate_ids` 契約、
//! `crate::blocks` モジュール doc 参照）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。ボタンは `button::button` の既定 `type="button"` のまま送信先を
//! 持たず、値は一切送信されない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 自作の幾何アイコン（線画。モジュール doc「アイコンは自作の抽象幾何
/// 図形」節参照）。`path` へ `fill="none"` + `stroke="currentColor"` を
/// 明示し、`icon` の `<svg>` 側が固定で持つ `fill="currentColor"`
/// （塗り面）を上書きして線画（ストローク）として描画する。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                ("d", path_d),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// 訴求ポイント一覧で使うチェックマーク風アイコン。
fn check_icon() -> Node {
    geo_icon("M5 13l4 4L19 7")
}

/// Primary/Secondary の 2 ボタン列（モジュール doc「tone 上書きの詳細度
/// (0,4,0)」節）。`data-blocks-cta-split-actions-primary`/`-secondary`
/// 属性を付与し、tone 面上での配色上書きの対象を識別できるようにする。
fn actions(primary_label: &'static str, secondary_label: &'static str) -> Node {
    div(
        vec![("class", "blocks-cta-split-actions-actions")],
        vec![
            button::button(
                &ButtonProps {
                    size: Size::Lg,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-cta-split-actions-primary", "")],
                vec![text(primary_label)],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    size: Size::Lg,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-cta-split-actions-secondary", "")],
                vec![text(secondary_label)],
            ),
        ],
    )
}

/// 2 行構成の見出し（`plain` tone、モジュール doc「4 インスタンスで配色
/// 差分を表現する」節）。2 行目のみアクセント色にする（`h3` の内容モデルは
/// phrasing content のため、行の区切りには `div` ではなく `span` +
/// `display: block` を用いる）。
fn two_line_heading(line1: &str, line2: &str) -> Node {
    heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl2,
            ..HeadingProps::default()
        },
        vec![],
        vec![
            span(
                vec![("class", "blocks-cta-split-actions-line")],
                vec![text(line1)],
            ),
            span(
                vec![
                    ("class", "blocks-cta-split-actions-line"),
                    ("data-blocks-cta-split-actions-line-accent", ""),
                ],
                vec![text(line2)],
            ),
        ],
    )
}

/// 訴求ポイント 1 件分（チェックアイコン + `text`）。
fn point(label: &str) -> Node {
    div(
        vec![("class", "blocks-cta-split-actions-point")],
        vec![
            check_icon(),
            styled_text::text(&TextProps::default(), vec![], vec![text(label)]),
        ],
    )
}

/// 行の共通ラッパー（見出し領域 + ボタン列、モジュール doc「レイアウトと
/// レスポンシブ」節）。`tone` は `plain`/`subtle`/`inverted` の 3 値のみを
/// 受け取る（`card` tone は [`card_row`] が別経路で組み立てる）。
fn row(tone: &'static str, copy: Node, actions: Node) -> Node {
    div(
        vec![
            ("data-blocks-cta-split-actions-row", ""),
            ("data-blocks-cta-split-actions-tone", tone),
        ],
        vec![
            div(vec![("class", "blocks-cta-split-actions-copy")], vec![copy]),
            actions,
        ],
    )
}

/// 1 個目: 基準形（背景なし、モジュール doc「4 インスタンスで配色差分を
/// 表現する」節）。対応: R0881（主参照）。
fn instance_plain() -> Node {
    row(
        "plain",
        two_line_heading("新しいワークフローを、", "今日から始めましょう"),
        actions("今すぐ始める", "詳しく見る"),
    )
}

/// 2 個目: 淡色ブランド面 + 訴求ポイント 3 点。対応: R0882（淡色面）、
/// R0067 / R0071（訴求ポイント。2 点版・2 列配置は「原案差分メモ」節へ
/// 記す）。
fn instance_subtle() -> Node {
    let copy = div(
        vec![],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("チームの導入を、まるごとサポートします")],
            ),
            div(
                vec![("class", "blocks-cta-split-actions-points")],
                vec![
                    point("初期設定は担当者が同席してご案内します"),
                    point("既存データの移行を無償でお手伝いします"),
                    point("導入後 30 日間はいつでも解約できます"),
                ],
            ),
        ],
    );
    row("subtle", copy, actions("プランを選ぶ", "資料をもらう"))
}

/// 3 個目: 反転配色。対応: R0447。
fn instance_inverted() -> Node {
    let copy = div(
        vec![],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("次の四半期の計画を、いま固めませんか")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "担当チームが要件をヒアリングし、導入案をご提示します。",
                )],
            ),
        ],
    );
    row("inverted", copy, actions("相談する", "事例を見る"))
}

/// 4 個目: アクセント色の浮いたカード。対応: R0451。カード面の余白は
/// [`card::body`] へ委ね、行のレイアウトは [`row`] と同じ属性構成
/// （`tone` は付けず、tone 上書きはカード側の
/// `data-blocks-cta-split-actions-tone="card"` から祖先セレクタで届かせる、
/// モジュール doc「tone 上書きの詳細度 (0,4,0)」節）。
fn instance_card() -> Node {
    let copy = div(
        vec![("class", "blocks-cta-split-actions-copy")],
        vec![heading(
            HeadingLevel::H3,
            &HeadingProps {
                size: HeadingSize::Xl2,
                ..HeadingProps::default()
            },
            vec![],
            vec![text("年間プランなら 2 か月分お得です")],
        )],
    );
    let row_inner = div(
        vec![("data-blocks-cta-split-actions-row", "")],
        vec![copy, actions("年間プランを見る", "月額プランのまま続ける")],
    );
    card::root(
        CardProps {
            variant: CardVariant::Elevated,
            ..CardProps::default()
        },
        vec![("data-blocks-cta-split-actions-tone", "card")],
        vec![card::body(vec![], vec![row_inner])],
    )
}

/// `cta-split-actions` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（グリッド用ラッパーの class は [`Block::demo_class`] と別名にする
/// 理由をモジュール doc「`drop_class_attr` と CSS フックの選び方」節参照）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-cta-split-actions-stack")],
        vec![
            instance_plain(),
            instance_subtle(),
            instance_inverted(),
            instance_card(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/cta-split-actions/",
    title: "cta-split-actions",
    category: BlockCategory::Cta,
    rust_source: "crates/docs-site/src/blocks/marketing/cta/cta_split_actions.rs",
    demo_class: "blocks-cta-split-actions",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `cta_split_actions` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` では
/// なく本ファイル内 private 定数として `super::stylesheet` 経由の
/// `push_css` で連結される）。
///
/// セレクタは `.blocks-cta-split-actions*` と
/// `[data-blocks-cta-split-actions-*]` のみを用い、他 block や部品の素の
/// セレクタへ影響させない（[`cta_feature_links`] と同じ名前空間分離）。
/// tone 上書きの詳細度についてはモジュール doc「tone 上書きの詳細度
/// (0,4,0)」節を参照。
const LAYOUT_CSS: &str = "\
.blocks-cta-split-actions-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
[data-blocks-cta-split-actions-row] {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-start;\n  gap: var(--fandhe-space-6);\n  padding: var(--fandhe-space-6);\n  border-radius: var(--fandhe-radius-lg);\n}\n\
[data-blocks-cta-split-actions-row][data-blocks-cta-split-actions-tone=\"plain\"] {\n  padding: 0;\n}\n\
[data-blocks-cta-split-actions-row][data-blocks-cta-split-actions-tone=\"subtle\"] {\n  background: var(--fandhe-color-accent-subtle);\n  color: var(--fandhe-color-accent-fg-subtle);\n}\n\
[data-blocks-cta-split-actions-row][data-blocks-cta-split-actions-tone=\"inverted\"] {\n  background: var(--fandhe-color-fg);\n  color: var(--fandhe-color-bg);\n}\n\
.blocks-cta-split-actions-copy {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-cta-split-actions-line {\n  display: block;\n}\n\
[data-blocks-cta-split-actions-line-accent] {\n  color: var(--fandhe-color-accent);\n}\n\
.blocks-cta-split-actions-points {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-cta-split-actions-point {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-cta-split-actions-actions {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-cta-split-actions [data-scope=\"card\"][data-part=\"root\"][data-blocks-cta-split-actions-tone=\"card\"] {\n  background: var(--fandhe-color-accent);\n  color: var(--fandhe-color-accent-fg);\n}\n\
[data-blocks-cta-split-actions-tone=\"inverted\"] [data-scope=\"button\"][data-part=\"root\"][data-blocks-cta-split-actions-primary] {\n  background: var(--fandhe-color-bg);\n  color: var(--fandhe-color-fg);\n  border-color: var(--fandhe-color-bg);\n}\n\
.blocks-cta-split-actions [data-scope=\"card\"][data-part=\"root\"][data-blocks-cta-split-actions-tone=\"card\"] [data-scope=\"button\"][data-part=\"root\"][data-blocks-cta-split-actions-primary] {\n  background: var(--fandhe-color-accent-fg);\n  color: var(--fandhe-color-accent);\n  border-color: var(--fandhe-color-accent-fg);\n}\n\
[data-blocks-cta-split-actions-tone=\"inverted\"] [data-scope=\"button\"][data-part=\"root\"][data-blocks-cta-split-actions-secondary],\n\
.blocks-cta-split-actions [data-scope=\"card\"][data-part=\"root\"][data-blocks-cta-split-actions-tone=\"card\"] [data-scope=\"button\"][data-part=\"root\"][data-blocks-cta-split-actions-secondary] {\n  color: inherit;\n  border-color: currentColor;\n}\n\
@media (min-width: 64rem) {\n  [data-blocks-cta-split-actions-row] {\n    flex-direction: row;\n    align-items: center;\n    justify-content: space-between;\n  }\n  .blocks-cta-split-actions-actions {\n    flex-shrink: 0;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていることの単体
    /// 回帰（`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        assert!(html.contains(r#"data-scope="heading""#));
        assert!(html.contains(r#"data-scope="text""#));
        assert!(html.contains(r#"data-scope="button""#));
        assert!(html.contains(r#"data-scope="card""#));
        assert!(html.contains(r#"data-scope="icon""#));
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains(" id=\""));
    }

    /// 4 tone・8 ボタン（各インスタンス 2 個）が揃っていることを固定する。
    #[test]
    fn demo_renders_four_tones_and_eight_buttons() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"data-blocks-cta-split-actions-tone="plain""#)
                .count(),
            1
        );
        assert_eq!(
            html.matches(r#"data-blocks-cta-split-actions-tone="subtle""#)
                .count(),
            1
        );
        assert_eq!(
            html.matches(r#"data-blocks-cta-split-actions-tone="inverted""#)
                .count(),
            1
        );
        assert_eq!(
            html.matches(r#"data-blocks-cta-split-actions-tone="card""#)
                .count(),
            1
        );
        assert_eq!(html.matches(r#"data-scope="button""#).count(), 8);
        // card::root（data-part="root"）+ card::body（data-part="body"）の
        // 2 パーツ分。
        assert_eq!(html.matches(r#"data-scope="card""#).count(), 2);
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件を持つこと。
    #[test]
    fn layout_css_declares_breakpoint() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
    }

    /// tone 上書きセレクタが recipe base（詳細度 (0,3,0)）を上回る詳細度
    /// (0,4,0) で宣言されていることを固定する（モジュール doc「tone 上書き
    /// の詳細度 (0,4,0)」節、[`super::super::banner::
    /// banner_full_width_bar`] と同型の回帰）。
    #[test]
    fn tone_selectors_outweigh_recipe_base() {
        assert!(LAYOUT_CSS.contains(
            r#".blocks-cta-split-actions [data-scope="card"][data-part="root"][data-blocks-cta-split-actions-tone="card"]"#
        ));
        assert!(LAYOUT_CSS.contains(
            r#".blocks-cta-split-actions [data-scope="card"][data-part="root"][data-blocks-cta-split-actions-tone="card"] [data-scope="button"][data-part="root"][data-blocks-cta-split-actions-primary]"#
        ));
        assert!(LAYOUT_CSS.contains(
            r#".blocks-cta-split-actions [data-scope="card"][data-part="root"][data-blocks-cta-split-actions-tone="card"] [data-scope="button"][data-part="root"][data-blocks-cta-split-actions-secondary]"#
        ));
    }
}
