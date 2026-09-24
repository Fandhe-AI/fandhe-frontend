//! `careers-card-grid` block（イシュー #2815。親トラッキング #2807
//! 「Blocks マーケティング B」配下、対応表 ID R0033 の 1 件のみを構造の
//! 参照元とする合成例。求人カードグリッド）。取得手段・ファイル名・
//! 内部コンポーネント識別子は記載しない（`docs/design/motion-reference-
//! adoption-policy.md` §9 と同じライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `badge` / `card` / `icon` / `button` の 6 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # レイアウト（md = 48rem をブレークポイントとする理由）
//!
//! `< 48rem` はカード 1 列、`>= 48rem` で 2 列へ切り替える。テーマの
//! breakpoint トークンは `@media` 条件式の中では解決できないため（CSS
//! custom property は宣言側でのみ有効）、
//! `fandhe_frontend_pre_styled_ui::recipe::Breakpoint` の `Md`（768px =
//! 48rem）と一致するリテラル値を [`LAYOUT_CSS`] へ直書きする
//! （`blog_list_image` と同じ判断）。ルート grid class は [`demo`] 直下の
//! `blocks-careers-card-grid-layout` へ適用し、[`Block::demo_class`]
//! （`blocks-careers-card-grid`）とは意図的に別名にする（`blog_list_image`
//! モジュール doc「レイアウト」節と同じ Bugbot 教訓の回避）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # 見出しレベルに `H3`/`H4` を使う理由・`card::title` を使わない理由
//!
//! ページ側が `## Demo` として `h2` を出すため、セクション見出しは
//! `HeadingLevel::H3` にする（`blog_list_image` と同じ判断）。カード内の
//! 職種名は `card::title`（`<h3>` 固定）ではなく `heading` の
//! `HeadingLevel::H4` で表す。`card::title` を使うとセクション見出し
//! （H3）と同じ見出しレベルが 1 ページに重複するためである（原稿「原案
//! 差分メモ」参照）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading`/`text::text`/`badge::badge`/`card::root`/
//! `button::button`/`icon::icon` はいずれも `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、Demo
//! 固有のスタイルフックは `data-blocks-careers-card-grid-*` 属性で渡し、
//! [`LAYOUT_CSS`] 側も同じ属性セレクタで対応する（`crate::blocks`
//! モジュール doc「CSS フックが `class` と `[data-*]` で混在する理由」
//! 節参照）。`card::header`/`card::body`/`card::footer`/`card::description`
//! （variant を持たず `attrs` をそのまま連結する）と素の `div` には
//! `class` がそのまま効くため、それらは従来どおりクラスセレクタを使う。
//!
//! # 自作幾何アイコン（位置ピン・時計・右矢印）
//!
//! lucide 等の既存アイコンセットの path を複製しないため、
//! `feature_expand::geo_icon`/`bento_staggered::geo_icon` と同型の自作
//! ヘルパ [`geo_icon`] で描く。`path` へ `fill="none"` +
//! `stroke="currentColor"` を明示し、`icon::icon` の `<svg>` 側が固定で
//! 持つ `fill="currentColor"`（塗り面）を上書きして線画として描画する。
//! いずれも隣に可視テキストがあるため装飾扱い（`IconProps::label` は
//! `None` のまま、`aria-hidden="true"`）とする。
//!
//! # 同名ボタンが 4 つ並ぶ問題への対処
//!
//! 各求人カードの「詳細を見る」ボタンへ `aria-label="詳細を見る
//! （<職種名>）"` を付与して区別する（可視テキストをアクセシブルネームに
//! 含める形なので WCAG 2.5.3 label-in-name に適合、`blog_list_image` の
//! 著者リンクとは異なる手段だが目的は同じ）。`id`/`aria-describedby` は
//! 出力しない（宙に浮いた ARIA 参照・id 重複を構造的に避けるための
//! `blog_list_image` と同じ判断）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言・部署名・勤務地はすべて架空のもの（実企業名・実クレデン
//! シャル・PII を含まない）。詳細への導線は使用部品として指定された
//! `button`（`type="button"` 固定、送信先を持たない）で表す静的な例であり、
//! 実アプリでページ遷移させる場合は `link` 部品を使うべきである
//! （原稿「原案差分メモ」参照）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::Size;

/// 装飾用の自作幾何アイコン（lucide 等の既存アイコンセットの path を
/// 複製しないための単純図形、`feature_expand::geo_icon` と同型の判断）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
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

/// 位置ピンの幾何アイコン（勤務地メタ行）。
fn location_icon() -> Node {
    geo_icon("M12 21s-7-6.5-7-11a7 7 0 0114 0c0 4.5-7 11-7 11z M12 12a2 2 0 100-4 2 2 0 000 4z")
}

/// 時計の幾何アイコン（雇用形態メタ行）。
fn clock_icon() -> Node {
    geo_icon("M12 3a9 9 0 100 18 9 9 0 000-18z M12 7v5l4 2")
}

/// 右矢印の幾何アイコン（「詳細を見る」ボタン末尾）。
fn arrow_icon() -> Node {
    geo_icon("M4 12h16 M13 5l7 7-7 7")
}

/// 求人 1 件分のダミーデータ（架空、実在の企業・人物とは無関係）。
struct Job {
    dept: &'static str,
    title: &'static str,
    summary: &'static str,
    location: &'static str,
    employment: &'static str,
}

/// 求人一覧（架空、4 件。参照元は 5 件だったが 2 列で割り切れる枚数へ
/// 減らした、原稿「原案差分メモ」参照）。
const JOBS: [Job; 4] = [
    Job {
        dept: "エンジニアリング",
        title: "フロントエンドエンジニア",
        summary: "描画コアと UI コンポーネント層の設計・実装を担当します。",
        location: "リモート",
        employment: "正社員",
    },
    Job {
        dept: "エンジニアリング",
        title: "SRE",
        summary: "配信基盤の信頼性向上と CI/CD の運用改善を担当します。",
        location: "東京（ハイブリッド）",
        employment: "正社員",
    },
    Job {
        dept: "デザイン",
        title: "プロダクトデザイナー",
        summary: "コンポーネントライブラリのビジュアル言語を設計します。",
        location: "リモート",
        employment: "業務委託",
    },
    Job {
        dept: "ドキュメント",
        title: "テクニカルライター",
        summary: "利用者向けガイドと API リファレンスを執筆します。",
        location: "リモート",
        employment: "正社員",
    },
];

/// メタ行 1 件（アイコン + テキストの組。勤務地・雇用形態で共用する）。
fn meta_item(icon_node: Node, label: &str) -> Node {
    div(
        vec![("data-blocks-careers-card-grid-meta-item", "")],
        vec![
            icon_node,
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(label)],
            ),
        ],
    )
}

/// 求人カード 1 枚（部署 badge + 職種名 + 短い説明 + メタ行 + 詳細ボタン）。
fn job_card(job: &Job) -> Node {
    let aria_label = format!("詳細を見る（{}）", job.title);
    card::root(
        CardProps::default(),
        vec![("data-blocks-careers-card-grid-card", "")],
        vec![
            card::header(
                vec![("data-blocks-careers-card-grid-card-header", "")],
                vec![
                    badge::badge(
                        &BadgeProps {
                            palette: ColorPalette::Neutral,
                            ..BadgeProps::default()
                        },
                        vec![("data-blocks-careers-card-grid-dept", "")],
                        vec![text(job.dept)],
                    ),
                    heading(
                        HeadingLevel::H4,
                        &HeadingProps {
                            size: HeadingSize::Md,
                            weight: HeadingWeight::Semibold,
                        },
                        vec![],
                        vec![text(job.title)],
                    ),
                ],
            ),
            card::body(
                vec![("class", "blocks-careers-card-grid-card-body")],
                vec![
                    card::description(vec![], vec![text(job.summary)]),
                    div(
                        vec![("class", "blocks-careers-card-grid-meta")],
                        vec![
                            meta_item(location_icon(), job.location),
                            meta_item(clock_icon(), job.employment),
                        ],
                    ),
                ],
            ),
            card::footer(
                vec![("class", "blocks-careers-card-grid-card-footer")],
                vec![button::button(
                    &ButtonProps {
                        variant: ButtonVariant::Ghost,
                        palette: ColorPalette::Neutral,
                        ..ButtonProps::default()
                    },
                    vec![("aria-label", aria_label.as_str())],
                    vec![text("詳細を見る"), arrow_icon()],
                )],
            ),
        ],
    )
}

/// `careers-card-grid` の Demo 本体（ページ見出し + 求人カードグリッド）。
/// 呼び出しごとに同一の `Node` を返す純関数（モジュール doc「レイアウト」
/// 節）。
pub fn demo() -> Node {
    let header = div(
        vec![("class", "blocks-careers-card-grid-header")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    weight: TextWeight::Medium,
                    ..TextProps::default()
                },
                vec![("data-blocks-careers-card-grid-tagline", "")],
                vec![text("採用情報")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("一緒に働く仲間を募集しています")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "現在募集中のポジションです。詳細は各カードからご確認ください。",
                )],
            ),
        ],
    );

    let grid = div(
        vec![("class", "blocks-careers-card-grid-grid")],
        JOBS.iter().map(job_card).collect(),
    );

    div(
        vec![("class", "blocks-careers-card-grid-layout")],
        vec![header, grid],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/careers-card-grid/",
    title: "careers-card-grid",
    category: BlockCategory::Careers,
    rust_source: "crates/docs-site/src/blocks/marketing/careers/careers_card_grid.rs",
    demo_class: "blocks-careers-card-grid",
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
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `careers_card_grid` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` ではなく
/// 本ファイル内 private 定数として `super::stylesheet` 経由の `push_css` で
/// 連結される）。
///
/// セレクタは `.blocks-careers-card-grid-*` と
/// `[data-blocks-careers-card-grid-*]` のみを用い、他 block や部品の素の
/// セレクタへ影響させない（`blog_list_image` と同じ名前空間分離）。
///
/// # `[data-blocks-careers-card-grid-card-header]` の詳細度を Card recipe 以上にする
///
/// `card::header` が出力する要素は `[data-scope="card"][data-part="header"]`
/// （詳細度 (0,2,0)）で `gap: var(--fandhe-space-1-5)` を既に宣言している
/// ため、block 側セレクタを単独の class（詳細度 (0,1,0)）のままにすると
/// Card recipe に負けてバッジと職種見出しの間隔が意図した値へ広がらない
/// （Cursor Bugbot 指摘、イシュー #2815）。`login_04`（イシュー #2093）と
/// 同型の対処として、呼び出し側の属性を `class` ではなく
/// `data-blocks-careers-card-grid-card-header` 属性へ変更し、セレクタを
/// `[data-scope="card"][data-part="header"][data-blocks-careers-card-grid-card-header]`
/// （詳細度 (0,3,0)）へ結合して Card recipe を確実に上書きする。
const LAYOUT_CSS: &str = "\
.blocks-careers-card-grid-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-careers-card-grid-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-careers-card-grid-tagline] {\n  color: var(--fandhe-color-accent);\n  text-transform: uppercase;\n  letter-spacing: 0.05em;\n}\n\
.blocks-careers-card-grid-grid {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-6);\n}\n\
[data-blocks-careers-card-grid-card] {\n  display: flex;\n  flex-direction: column;\n  height: 100%;\n}\n\
[data-scope=\"card\"][data-part=\"header\"][data-blocks-careers-card-grid-card-header] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-careers-card-grid-dept] {\n  align-self: flex-start;\n}\n\
.blocks-careers-card-grid-card-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  flex: 1;\n}\n\
.blocks-careers-card-grid-meta {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-careers-card-grid-meta-item] {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-careers-card-grid-card-footer {\n  margin-top: auto;\n}\n\
@media (min-width: 48rem) {\n  .blocks-careers-card-grid-grid {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n}\n";

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
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"badge\"",
            "data-scope=\"card\"",
            "data-scope=\"icon\"",
            "data-scope=\"button\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(
            html.matches("data-part=\"root\" class=\"fd-card").count(),
            4
        );
        assert_eq!(html.matches("type=\"button\"").count(), 4);
        assert!(html.contains("詳細を見る（フロントエンドエンジニア）"));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("id=\""));
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件を持つこと。
    #[test]
    fn layout_css_declares_md_breakpoint_and_two_columns() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("repeat(2"));
    }

    /// ルート grid class（`demo_class` とは別名）が `demo()` の出力へ
    /// 実際に現れること（モジュール doc「レイアウト」節の Bugbot 教訓の
    /// 固定、`blog_list_image` と同じ回帰）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-careers-card-grid-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-careers-card-grid-layout");
    }
}
