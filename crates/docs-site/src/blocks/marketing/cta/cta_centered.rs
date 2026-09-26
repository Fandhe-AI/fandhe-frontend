//! `cta-centered` block（イシュー #3224。親トラッキング #2730「Blocks
//! 目的別パーツ拡充ツリー」配下、対応表 ID R0878 を主参照とし、
//! R0068 / R0069 / R0446 / R0452 / R0876 / R0877 / R0879 / R0880 を配色・
//! 訴求点の差分として集約した合成例。中央寄せの CTA セクション（見出し・
//! 説明文・ボタン列を縦に積む）。取得手段・ファイル名・内部コンポーネント
//! 識別子は記載しない（[`super::cta_split_actions`] と同じライセンス上の
//! 転記制限、対応表 ID のみを記す）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `button` / `badge` / `card` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 5 インスタンスで配色差分を表現する（各対応表 ID との対応）
//!
//! 1 つの Demo に 5 インスタンスを縦積みし、tone ごとの配色差分を並べて
//! 見せる（[`super::cta_split_actions`] と同型の「1 block・複数
//! インスタンス」構成）。
//!
//! - `plain`（R0878 主参照、R0876）: 背景なし。見出し + リード文（`text`）
//!   + ボタン 2 個。R0876 の左寄せは配置差のみのため個別インスタンス化せず、
//!     `site/blocks/cta-centered.md` の「原案差分メモ」節へ記す。
//! - `badge`（R0068）: 淡色面。`badge`（[`BadgeVariant::Subtle`]）→ 見出し
//!   → リード → ボタン 1 個。
//! - `accent`（R0446、R0880、R0069）: アクセント色の全面背景。ロゴ枠
//!   （自作の抽象幾何、R0069 の代用）+ 2 行見出し + ボタン 2 個。R0880 は
//!   配色差のみで本 tone へ統合する。
//! - `dark`（R0877、R0879）: 前景色と背景色を反転したパネル + グラデ装飾。
//!   R0877 の放射グラデは持ち込まず、`linear-gradient` の面装飾のみ実装
//!   する（R0879 との差は装飾の有無のみ）。
//! - `card`（R0452）: [`card::root`]（[`CardVariant::Elevated`]）+
//!   [`card::body`] の中に中央寄せで見出し + リード + ボタン 2 個。
//!
//! # レイアウトとレスポンシブ（`40rem` を境にボタン列を反転）
//!
//! セクションは常に `flex-direction: column; align-items: center;
//! text-align: center` の中央寄せ。ボタン列（[`actions`]）は既定
//! （`40rem` 未満）で縦積み、`40rem` 以上で横並び + 中央寄せへ切り替える。
//! ブレークポイントは
//! [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Sm`]（640px =
//! 40rem）と一致するリテラル値を [`LAYOUT_CSS`] へ直書きする（`@media`
//! 条件式の中ではテーマの breakpoint トークンを解決できないため、
//! [`super::cta_split_actions`] と同じ判断）。
//!
//! # tone 上書きの詳細度 (0,4,0)（[`super::cta_split_actions`] と同じ判断軸）
//!
//! `plain`/`badge`/`accent`/`dark` の 4 tone は素の `div`（section）へ
//! `[data-blocks-cta-centered-tone="…"]`（属性 1 個をセクション自身の
//! `class="blocks-cta-centered-stack"` の子孫セレクタと組み合わせた
//! `.blocks-cta-centered-stack [data-blocks-cta-centered-tone="…"]`、詳細度
//! (0,2,0)）で背景・文字色を上書きする。`card` tone は [`card::root`] の
//! recipe base（詳細度 (0,3,0)）より高い詳細度が必要なため、
//! `.blocks-cta-centered [data-scope="card"][data-part="root"]
//! [data-blocks-cta-centered-tone="card"]`（1 クラス + 3 属性 = 詳細度
//! (0,4,0)）で上書きする。`accent`/`dark`/`card` の各面に置くボタンは
//! 既定の accent 配色のままだと背景と同化する（Primary）・前景色が読み
//! にくい（Secondary）ため、`data-blocks-cta-centered-primary`/
//! `-secondary` 属性 + tone 祖先セレクタ（詳細度 (0,4,0)）で反転・継承へ
//! 上書きする。`card` tone の内側 section は基底ルールの `padding` を
//! そのまま受けるが、[`card::body`] 自身も余白を持つため、`.blocks-
//! cta-centered [data-scope="card"][data-part="body"]
//! [data-blocks-cta-centered-tone="card"]`（詳細度 (0,4,0)）で
//! `padding: 0` に上書きし、二重適用を防ぐ（[`super::cta_split_actions`]
//! と同じ判断、Bugbot 指摘イシュー #2758 PR レビュー参照）。
//!
//! # `drop_class_attr` と CSS フックの選び方
//!
//! `heading::heading` / `text::text` / `button::button` / `badge::badge` /
//! `card::root` / `card::body` はいずれも `drop_class_attr` により呼び出し
//! 側 `attrs` の `class` を黙って除去する契約を持つため、Demo 固有の
//! スタイルフックは `data-blocks-cta-centered-*` 属性で渡し、
//! [`LAYOUT_CSS`] 側も同じ属性セレクタで対応する（[`super::
//! cta_split_actions`] と同じ判断）。素の `div`/`span` には `class` が
//! そのまま効くため、スタック・ロゴ枠・グラデ装飾は
//! `.blocks-cta-centered-*` クラスセレクタを使う。グリッド用のルート
//! class（`blocks-cta-centered-stack`）は [`Block::demo_class`]
//! （`blocks-cta-centered`）と意図的に別名にする（同名にすると
//! `crate::blocks::insert_generated_sections` が付与する Demo ラッパーへも
//! スタック規則が当たってしまう、イシュー #2809 で Bugbot が指摘した
//! 不具合と同型、[`super::cta_split_actions`] モジュール doc 参照）。
//!
//! # ロゴ枠は自作の抽象幾何図形
//!
//! 参照元のロゴ・実ブランドは使わず、角丸の枠と抽象幾何のみの CSS で
//! 描く（`aria-hidden="true"` を付けた素の `span`）。画像・SVG は取得しない。
//!
//! # `crate::blocks::dummy_assets` を使わない理由（コードフェンス自己完結）
//!
//! `crate::blocks::dummy_assets` は `pub(crate)` のため、Markdown 原稿の
//! Rust コードフェンス（`// blocks-code:begin`/`:end` マーカー内、クレート
//! 外から読めるコード例として提示される）がこれを参照すると単体では
//! コンパイルできなくなる（イシュー #2811 のレビュー指摘、[`super::
//! cta_split_actions`] と同じ判断軸）。文言は独自に書いた架空の日本語
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
use fandhe_frontend_core::{div, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// Primary/Secondary の 2 ボタン列（モジュール doc「tone 上書きの詳細度
/// (0,4,0)」節）。`data-blocks-cta-centered-primary`/`-secondary` 属性を
/// 付与し、tone 面上での配色上書きの対象を識別できるようにする。
/// `secondary` を `None` にすると Primary 1 個のみになる（`badge` tone）。
fn actions(primary_label: &'static str, secondary_label: Option<&'static str>) -> Node {
    let mut children = vec![button::button(
        &ButtonProps {
            size: Size::Lg,
            ..ButtonProps::default()
        },
        vec![("data-blocks-cta-centered-primary", "")],
        vec![text(primary_label)],
    )];
    if let Some(label) = secondary_label {
        children.push(button::button(
            &ButtonProps {
                variant: ButtonVariant::Outline,
                size: Size::Lg,
                ..ButtonProps::default()
            },
            vec![("data-blocks-cta-centered-secondary", "")],
            vec![text(label)],
        ));
    }
    div(vec![("class", "blocks-cta-centered-actions")], children)
}

/// リード文（[`TextVariant::Plain`]。`Muted` は自身に色を持つため、
/// `accent`/`dark` 面で継承色を上書きしコントラストが崩れる、
/// [`super::cta_split_actions`] と同じ判断）。
fn lead(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            variant: TextVariant::Plain,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// セクション共通ラッパー（モジュール doc「レイアウトとレスポンシブ」
/// 節）。子要素は `Vec<Node>` で直接受け取り、`gap` が効く flex コンテナの
/// 直接の子を複数にする（[`super::cta_split_actions::row`] と同じ判断）。
fn section(tone: &'static str, children: Vec<Node>) -> Node {
    div(vec![("data-blocks-cta-centered-tone", tone)], children)
}

/// 2 行構成の見出し（`accent` tone。`h3` の内容モデルは phrasing content
/// のため、行の区切りには `div` ではなく `span` + `display: block` を
/// 用いる）。
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
                vec![("class", "blocks-cta-centered-line")],
                vec![text(line1)],
            ),
            span(
                vec![("class", "blocks-cta-centered-line")],
                vec![text(line2)],
            ),
        ],
    )
}

/// 自作のロゴ枠（モジュール doc「ロゴ枠は自作の抽象幾何図形」節）。
/// 実ブランドを模さない角丸枠 + 抽象幾何のみを CSS で描く。
fn logo_mark() -> Node {
    span(
        vec![
            ("class", "blocks-cta-centered-logo"),
            ("aria-hidden", "true"),
        ],
        vec![],
    )
}

/// 1 個目: 基準形（背景なし、モジュール doc「5 インスタンスで配色差分を
/// 表現する」節）。対応: R0878（主参照）、R0876（左寄せは配置差のみのため
/// 統合せず、原案差分メモへ記す）。
fn instance_plain() -> Node {
    section(
        "plain",
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("新しい働き方を、今日から始めましょう")],
            ),
            lead("チームの規模を問わず、導入したその日から成果を実感できます。"),
            actions("今すぐ始める", Some("詳しく見る")),
        ],
    )
}

/// 2 個目: 淡色面 + バッジ。対応: R0068。
fn instance_badge() -> Node {
    section(
        "badge",
        vec![
            badge::badge(
                &BadgeProps {
                    variant: BadgeVariant::Subtle,
                    ..BadgeProps::default()
                },
                vec![],
                vec![text("新機能")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("β版のご利用登録を受け付けています")],
            ),
            lead("先着枠のご案内は登録いただいた方から順にお送りします。"),
            actions("登録する", None),
        ],
    )
}

/// 3 個目: アクセント色の全面背景 + ロゴ枠 + 2 行見出し。対応: R0446
/// （反転配色）、R0880（配色差のみで統合）、R0069（ロゴ枠を代用）。
fn instance_accent() -> Node {
    section(
        "accent",
        vec![
            logo_mark(),
            two_line_heading("導入をご検討の方へ、", "個別相談を承ります"),
            lead("担当チームが要件をヒアリングし、最適な構成をご提案します。"),
            actions("相談する", Some("資料をもらう")),
        ],
    )
}

/// 4 個目: 反転配色パネル + グラデ装飾。対応: R0877（放射グラデは除外し
/// 面装飾のみ実装）、R0879（装飾の有無の差のみ）。
fn instance_dark() -> Node {
    section(
        "dark",
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
            lead("導入事例をもとに、貴社に合わせた進め方をご提案します。"),
            actions("事例を見る", Some("問い合わせる")),
        ],
    )
}

/// 5 個目: アクセント色の浮いたカード。対応: R0452。カード面の余白は
/// [`card::body`] へ委ね、内側 section の padding は CSS 側で 0 に上書き
/// する（モジュール doc「tone 上書きの詳細度 (0,4,0)」節）。
fn instance_card() -> Node {
    let inner = section(
        "card",
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("年間プランなら 2 か月分お得です")],
            ),
            lead("月額プランからいつでも切り替えられます。"),
            actions("年間プランを見る", Some("月額プランのまま続ける")),
        ],
    );
    card::root(
        CardProps {
            variant: CardVariant::Elevated,
            ..CardProps::default()
        },
        vec![],
        vec![card::body(vec![], vec![inner])],
    )
}

/// `cta-centered` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数
/// （グリッド用ラッパーの class を [`Block::demo_class`] と別名にする
/// 理由をモジュール doc「`drop_class_attr` と CSS フックの選び方」節
/// 参照）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-cta-centered-stack")],
        vec![
            instance_plain(),
            instance_badge(),
            instance_accent(),
            instance_dark(),
            instance_card(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/cta-centered/",
    title: "cta-centered",
    category: BlockCategory::Cta,
    rust_source: "crates/docs-site/src/blocks/marketing/cta/cta_centered.rs",
    demo_class: "blocks-cta-centered",
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
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `cta_centered` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` ではなく
/// 本ファイル内 private 定数として `super::stylesheet` 経由の `push_css`
/// で連結される）。
///
/// セレクタは `.blocks-cta-centered*` と `[data-blocks-cta-centered-*]`
/// のみを用い、他 block や部品の素のセレクタへ影響させない（[`super::
/// cta_split_actions`] と同じ名前空間分離）。tone 上書きの詳細度について
/// はモジュール doc「tone 上書きの詳細度 (0,4,0)」節を参照。
const LAYOUT_CSS: &str = "\
.blocks-cta-centered-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
[data-blocks-cta-centered-tone] {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  text-align: center;\n  gap: var(--fandhe-space-4);\n  max-inline-size: 40rem;\n  margin-inline: auto;\n  padding: var(--fandhe-space-8);\n  border-radius: var(--fandhe-radius-lg);\n}\n\
[data-blocks-cta-centered-tone=\"plain\"] {\n  padding: 0;\n}\n\
[data-blocks-cta-centered-tone=\"badge\"] {\n  background: var(--fandhe-color-bg-subtle);\n}\n\
[data-blocks-cta-centered-tone=\"accent\"] {\n  background: var(--fandhe-color-accent);\n  color: var(--fandhe-color-accent-fg);\n}\n\
[data-blocks-cta-centered-tone=\"dark\"] {\n  position: relative;\n  overflow: hidden;\n  background: var(--fandhe-color-fg);\n  color: var(--fandhe-color-bg);\n}\n\
[data-blocks-cta-centered-tone=\"dark\"]::before {\n  content: \"\";\n  position: absolute;\n  inset: 0;\n  z-index: 0;\n  pointer-events: none;\n  background: linear-gradient(135deg, var(--fandhe-color-accent), transparent);\n  opacity: 0.35;\n}\n\
[data-blocks-cta-centered-tone=\"dark\"] > * {\n  position: relative;\n  z-index: 1;\n}\n\
.blocks-cta-centered-logo {\n  display: block;\n  inline-size: var(--fandhe-space-10);\n  block-size: var(--fandhe-space-10);\n  border-radius: var(--fandhe-radius-md);\n  border: 2px solid currentColor;\n  background: linear-gradient(135deg, currentColor, transparent);\n  opacity: 0.85;\n}\n\
.blocks-cta-centered-line {\n  display: block;\n}\n\
.blocks-cta-centered-actions {\n  display: flex;\n  flex-direction: column;\n  align-items: stretch;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-cta-centered [data-scope=\"card\"][data-part=\"root\"][data-blocks-cta-centered-tone=\"card\"] {\n  background: var(--fandhe-color-accent);\n  color: var(--fandhe-color-accent-fg);\n}\n\
.blocks-cta-centered [data-scope=\"card\"][data-part=\"body\"] [data-blocks-cta-centered-tone=\"card\"] {\n  padding: 0;\n}\n\
[data-blocks-cta-centered-tone=\"accent\"] [data-scope=\"button\"][data-part=\"root\"][data-blocks-cta-centered-primary],\n\
[data-blocks-cta-centered-tone=\"dark\"] [data-scope=\"button\"][data-part=\"root\"][data-blocks-cta-centered-primary],\n\
.blocks-cta-centered [data-scope=\"card\"][data-part=\"root\"][data-blocks-cta-centered-tone=\"card\"] [data-scope=\"button\"][data-part=\"root\"][data-blocks-cta-centered-primary] {\n  background: var(--fandhe-color-accent-fg);\n  color: var(--fandhe-color-accent);\n  border-color: var(--fandhe-color-accent-fg);\n}\n\
[data-blocks-cta-centered-tone=\"accent\"] [data-scope=\"button\"][data-part=\"root\"][data-blocks-cta-centered-secondary],\n\
[data-blocks-cta-centered-tone=\"dark\"] [data-scope=\"button\"][data-part=\"root\"][data-blocks-cta-centered-secondary],\n\
.blocks-cta-centered [data-scope=\"card\"][data-part=\"root\"][data-blocks-cta-centered-tone=\"card\"] [data-scope=\"button\"][data-part=\"root\"][data-blocks-cta-centered-secondary] {\n  color: inherit;\n  border-color: currentColor;\n}\n\
@media (min-width: 40rem) {\n  .blocks-cta-centered-actions {\n    flex-direction: row;\n    justify-content: center;\n  }\n}\n";

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
        assert!(html.contains(r#"data-scope="badge""#));
        assert!(html.contains(r#"data-scope="card""#));
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains(" id=\""));
    }

    /// 5 tone・9 ボタン（plain 2 + badge 1 + accent 2 + dark 2 + card 2）・
    /// badge 1 個・card 2 パーツ（root + body）が揃っていることを固定する。
    #[test]
    fn demo_renders_five_tones() {
        let html = render(&demo());
        for tone in ["plain", "badge", "accent", "dark", "card"] {
            let needle = format!(r#"data-blocks-cta-centered-tone="{tone}""#);
            assert_eq!(
                html.matches(&needle).count(),
                1,
                "tone {tone} should appear exactly once"
            );
        }
        assert_eq!(html.matches(r#"data-scope="button""#).count(), 9);
        assert_eq!(html.matches(r#"data-scope="badge""#).count(), 1);
        assert_eq!(html.matches(r#"data-scope="card""#).count(), 2);
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件を持つこと。
    #[test]
    fn layout_css_declares_breakpoint() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
    }

    /// tone 上書きセレクタが recipe base（詳細度 (0,3,0)）を上回る詳細度
    /// (0,4,0) で宣言されていることを固定する（モジュール doc「tone 上書き
    /// の詳細度 (0,4,0)」節）。
    #[test]
    fn tone_selectors_outweigh_recipe_base() {
        assert!(LAYOUT_CSS.contains(
            r#".blocks-cta-centered [data-scope="card"][data-part="root"][data-blocks-cta-centered-tone="card"]"#
        ));
        assert!(LAYOUT_CSS.contains(
            r#".blocks-cta-centered [data-scope="card"][data-part="root"][data-blocks-cta-centered-tone="card"] [data-scope="button"][data-part="root"][data-blocks-cta-centered-primary]"#
        ));
        assert!(LAYOUT_CSS.contains(
            r#".blocks-cta-centered [data-scope="card"][data-part="root"][data-blocks-cta-centered-tone="card"] [data-scope="button"][data-part="root"][data-blocks-cta-centered-secondary]"#
        ));
    }
}
