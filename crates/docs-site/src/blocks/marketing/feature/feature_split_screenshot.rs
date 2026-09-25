//! `feature-split-screenshot` block（イシュー #2770。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー」配下、Phase 1（マーケティング A）に
//! 属する。テキスト列（eyebrow badge + 見出し + リード文 + アイコン付き
//! インライン feature 3 件）と画像列（アプリ画面。列幅を超えてはみ出す）の
//! 2 列構成の feature セクションの合成例（`feature-large-screenshot`/
//! `feature-side-heading-grid` に続く Marketing / Feature カテゴリの 7 件
//! 目）。取得手段・ファイル名・内部コンポーネント識別子は記載しない
//! （対応表 ID は基準形 R0932 と、R0936（画像左の反転版）/ R0939
//! （アクセント色パネル）/ R0942（タブ付きコード枠）の集約元のみを記す）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `icon` / `image` / `code` の 6 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 3 形を 1 つの Demo へ並べる
//!
//! `feature-large-screenshot` と同じく、集約元 4 件の差分を別々の block へ
//! 分けず、縦に並ぶ 3 つのセクションとして 1 つの Demo に収める。DOM 順は
//! どのセクションでも「テキスト列 → メディア列」に固定する（後述）。
//!
//! - **A（基準形、R0932）**: テキスト列が左、画像列が右。画像は列幅を
//!   超えて右へはみ出す。
//! - **B（反転 + アクセント色パネル、R0936 + R0939 の統合）**: lg
//!   （64rem）以上で画像列が左、テキスト列が右。画像はアクセント色
//!   パネルに載り、パネルの外側（左方向）へはみ出す。R0936（左右反転）
//!   と R0939（アクセント色パネル）はどちらも画像列の見せ方の差分のため
//!   1 セクションへ統合した（意図的な判断、`site/blocks/
//!   feature-split-screenshot.md` の原案差分メモに明記する）。
//! - **C（タブ付きコード枠、R0942）**: テキスト列が左、右列は画像の代わり
//!   にタブ付きのコード表示枠。開発者向けの feature 3 件を使う。
//!
//! # DOM 順固定 + lg 以上でのみ反転する理由
//!
//! DOM 順を常に「テキスト → メディア」に固定することで、lg（64rem）未満
//! （`display: flex; flex-direction: column`）では自然に画像がテキストの
//! 下へ来る。lg 以上では `display: grid; grid-template-columns:
//! repeat(2, minmax(0, 1fr))` にし、B（反転）だけ
//! `data-blocks-feature-split-screenshot-reverse` 属性でテキスト列を
//! `grid-column: 2`、メディア列を `grid-column: 1` へ入れ替える
//! （`feature_alternating_rows` と同じ手法。`column-reverse`/`order` は
//! 使わない）。`@media` の条件式内ではテーマの breakpoint トークンが
//! 解決できないため、他 block と同じ判断でリテラル `64rem` を直書きする。
//!
//! # 負の margin を採らず `overflow: hidden` で切り取る理由
//!
//! `feature_large_screenshot` は「負の margin によるはみ出しは採らない」
//! と判断したが、本イシューは画像のはみ出しを明示的に要求するため、ここは
//! 意図的に逸脱する。`.blocks-demo` は `overflow-x: auto` のため、負の
//! margin でセクション外へはみ出させると横スクロールバーが出て Demo の
//! 可読性を損なう。本 Demo は各セクションのルートへ `overflow: hidden` を
//! 付け、画像を列幅より大きく（`width: 48rem`）して「セクションの境界で
//! 切り取られる」形ではみ出しを表現する（負の margin は使わない）。lg
//! 未満では画像を `width: 100%` に留め、はみ出させない。
//!
//! # アクセント色パネルにテキストを置かない理由
//!
//! B のアクセント色パネル（`var(--fandhe-color-accent)` 背景）には画像
//! だけを置き、テキストは置かない。前景色のコントラスト問題（
//! `feature_large_screenshot` の暗色パネルが直面した課題）がそもそも
//! 発生しない設計である。
//!
//! # `tabs` 部品を使わない理由（C）
//!
//! `tabs` は [`BLOCK`] の `parts` に含まれず、JS ハイドレーションを行わ
//! ない docs サイトでは操作もできない。タブ列は素の `span` 2 つで表し、
//! 選択中タブは `data-blocks-feature-split-screenshot-tab-active` 属性で
//! 静的に固定する。`role="tab"`/`aria-selected`/`aria-controls`/`id` は
//! 付けない（操作できない要素に操作可能を示す ARIA を付けると誤った案内
//! になり、aria 参照のぶら下がり検知テストにも触れるため）。
//!
//! # 見出しレベルに `H3` を使う理由
//!
//! ページ側が `## Demo` として `h2` を出すため、各セクションの見出しは
//! `HeadingLevel::H3` にする（他 block と同じ判断）。インライン feature
//! 項目は見出し要素にしない（1 行のインライン表記のため）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と
//! `fandhe_frontend_core::text` が同名のため、styled 側を `styled_text`
//! として取り込む（`crate::blocks` 内の他 block と同じ回避方法）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `badge::badge` / `heading::heading` / `image::image` / `icon::icon` /
//! `code::code` はいずれも `drop_class_attr` により呼び出し側 `attrs` の
//! `class` を黙って除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-feature-split-screenshot-*` 属性で渡し、[`LAYOUT_CSS`] 側も
//! 同じ属性セレクタで対応する。素の `div`/`span`/`pre` には `class` が
//! そのまま効くため、それらは `.blocks-feature-split-screenshot-*`
//! クラスセレクタを使う。レイアウト root の class
//! （`blocks-feature-split-screenshot-layout`）は [`Block::demo_class`]
//! （`blocks-feature-split-screenshot`）とは意図的に別名にする
//! （`feature_large_screenshot` と同じ Bugbot 教訓の回避）。
//!
//! `styled_text::text`・`image::image`・`code::code` の recipe（詳細度
//! (0,2,0)）に確実に勝つため、上書きは `[data-scope="..."][data-part=
//! "root"][data-blocks-feature-split-screenshot-*]` の 3 セレクタ構成
//! （詳細度 (0,3,0)）で行う。`code` recipe の inline 表示は本 Demo では
//! ブロック表示へ切り替える（`display: block; white-space: pre;
//! background: transparent; color: inherit; border: 0; padding: 0`）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない・id を使わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言はすべて架空のもの（実企業名・実クレデンシャル・PII・
//! トークン・URL・メールアドレスを含まない）。画像は
//! [`dummy_assets::SCREENSHOT_SRC`]（ビルド時生成のプレースホルダー SVG）
//! のみを使い、装飾扱いの `alt=""` で出力する。`id` 属性は一切使わない
//! （重複 id 検知テスト対策）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};

/// 装飾用の自作幾何アイコン（lucide 等の著作物を複製しないための単純図形、
/// `feature_large_screenshot::geo_icon` と同型の判断）。線画（ストローク）
/// として描画するため `fill="none"` を明示し、`icon` の `<svg>` 側が固定で
/// 持つ `fill="currentColor"`（塗り面）を上書きする。
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

/// インライン feature 項目 1 件分（用語 + 説明 + アイコン形状）。
struct FeatureItem {
    term: &'static str,
    body: &'static str,
    icon_path_d: &'static str,
}

/// A/B 向けの 3 件分の架空データ。
const ITEMS: [FeatureItem; 3] = [
    FeatureItem {
        term: "自動同期",
        body: "端末間の差分を検出し、操作を待たずに反映します。",
        icon_path_d: "M12 2a10 10 0 100 20 10 10 0 000-20zM12 6v6l4 2",
    },
    FeatureItem {
        term: "権限管理",
        body: "役割ごとに閲覧・編集の範囲を割り当てられます。",
        icon_path_d: "M12 2l8 4v6c0 5-3.5 8-8 10-4.5-2-8-5-8-10V6z",
    },
    FeatureItem {
        term: "利用状況",
        body: "チームごとの利用傾向を継続的に記録します。",
        icon_path_d: "M4 20V10M10 20V4M16 20v-7M22 20V2",
    },
];

/// C（開発者向け）の 3 件分の架空データ。
const DEV_ITEMS: [FeatureItem; 3] = [
    FeatureItem {
        term: "型安全な API",
        body: "呼び出し側の型情報から補完と検証を行います。",
        icon_path_d: "M4 4h16v4H4zM4 10h10v4H4zM4 16h14v4H4z",
    },
    FeatureItem {
        term: "設定不要",
        body: "既定値のまま導入でき、追加の初期設定を要しません。",
        icon_path_d: "M12 2v6l4-3M4 13a8 8 0 1016 0",
    },
    FeatureItem {
        term: "拡張ポイント",
        body: "既存の処理を差し替えずに振る舞いを追加できます。",
        icon_path_d: "M8 12h8M12 8v8M4 4h6v6H4zM14 14h6v6h-6z",
    },
];

/// インライン feature 項目 1 件（アイコン + 太字の用語 + 説明を 1 行の
/// 流れで並べる）。
fn feature_row(item: &FeatureItem) -> Node {
    div(
        vec![("class", "blocks-feature-split-screenshot-feature")],
        vec![
            geo_icon(item.icon_path_d),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-feature-split-screenshot-feature-text", "")],
                vec![
                    el("strong", vec![], vec![text(item.term)]),
                    text(" "),
                    text(item.body),
                ],
            ),
        ],
    )
}

/// テキスト列（eyebrow badge + 見出し + リード文 + インライン feature
/// 一覧）。`eyebrow` が `None` のとき badge は出力しない。
fn copy_column(eyebrow: Option<&str>, title: &str, lead: &str, features: &[FeatureItem]) -> Node {
    let mut children: Vec<Node> = Vec::new();
    if let Some(eyebrow) = eyebrow {
        children.push(badge::badge(
            &BadgeProps::default(),
            vec![("data-blocks-feature-split-screenshot-eyebrow", "")],
            vec![text(eyebrow)],
        ));
    }
    children.push(heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl3,
            weight: HeadingWeight::Bold,
        },
        vec![],
        vec![text(title)],
    ));
    children.push(styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![("data-blocks-feature-split-screenshot-lead", "")],
        vec![text(lead)],
    ));
    children.push(div(
        vec![("class", "blocks-feature-split-screenshot-features")],
        features.iter().map(feature_row).collect(),
    ));

    div(
        vec![("class", "blocks-feature-split-screenshot-text")],
        children,
    )
}

/// アプリ画面のプレースホルダー画像（列幅より大きい固定幅で、
/// [`LAYOUT_CSS`] 側がセクション境界での切り取りを行う）。装飾扱いのため
/// `alt=""`。
fn screenshot() -> Node {
    image::image(
        &ImageProps {
            fit: ImageFit::Cover,
            aspect_ratio: AspectRatio::Auto,
            shape: ImageShape::Rounded,
            ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
        },
        vec![("data-blocks-feature-split-screenshot-image", "")],
    )
}

/// A: 基準形（テキスト列 → 画像列。画像は右へはみ出す）。
fn section_a() -> Node {
    div(
        vec![("class", "blocks-feature-split-screenshot-section")],
        vec![
            copy_column(
                Some("プラットフォーム"),
                "画面のまま特長を伝える",
                "実際の操作画面と、そこから得られる主要な特長をあわせて紹介します。",
                &ITEMS,
            ),
            div(
                vec![("class", "blocks-feature-split-screenshot-media")],
                vec![screenshot()],
            ),
        ],
    )
}

/// B: 反転 + アクセント色パネル（lg 以上で画像列が左。パネルの外側へ
/// はみ出す）。
fn section_b() -> Node {
    div(
        vec![
            ("class", "blocks-feature-split-screenshot-section"),
            ("data-blocks-feature-split-screenshot-reverse", ""),
        ],
        vec![
            copy_column(
                None,
                "反転レイアウトでも読みやすく",
                "画像とテキストの位置を入れ替えても、同じ構成のまま伝えられます。",
                &ITEMS,
            ),
            div(
                vec![("class", "blocks-feature-split-screenshot-media")],
                vec![div(
                    vec![("class", "blocks-feature-split-screenshot-panel")],
                    vec![screenshot()],
                )],
            ),
        ],
    )
}

/// C: タブ付きコード枠（テキスト列 → 開発者向けのコード表示枠）。
fn section_c() -> Node {
    let snippet = "let app = App::new();\napp.mount(\"#root\");\napp.run();\n";
    div(
        vec![("class", "blocks-feature-split-screenshot-section")],
        vec![
            copy_column(
                None,
                "コードで組み込む",
                "既存のアプリへ数行を追加するだけで導入できます。",
                &DEV_ITEMS,
            ),
            div(
                vec![("class", "blocks-feature-split-screenshot-media")],
                vec![div(
                    vec![("class", "blocks-feature-split-screenshot-code-frame")],
                    vec![
                        div(
                            vec![("class", "blocks-feature-split-screenshot-tabs")],
                            vec![
                                span(
                                    vec![("data-blocks-feature-split-screenshot-tab-active", "")],
                                    vec![text("main.rs")],
                                ),
                                span(vec![], vec![text("Cargo.toml")]),
                            ],
                        ),
                        el(
                            "pre",
                            vec![("class", "blocks-feature-split-screenshot-pre")],
                            vec![code::code(
                                &CodeProps::default(),
                                vec![("data-blocks-feature-split-screenshot-code", "")],
                                vec![text(snippet)],
                            )],
                        ),
                    ],
                )],
            ),
        ],
    )
}

/// `feature-split-screenshot` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（他 block と同じ状態を持たない設計）。ルート class は
/// `demo_class`（`blocks-feature-split-screenshot`）とは別名にする。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-feature-split-screenshot-layout")],
        vec![section_a(), section_b(), section_c()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/feature-split-screenshot/",
    title: "feature-split-screenshot",
    category: BlockCategory::Feature,
    rust_source: "crates/docs-site/src/blocks/marketing/feature/feature_split_screenshot.rs",
    demo_class: "blocks-feature-split-screenshot",
    parts: &[
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Code",
            path: "/themes/code/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `feature_split_screenshot` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節）。セレクタは
/// `.blocks-feature-split-screenshot-*` と
/// `[data-blocks-feature-split-screenshot-*]` のみを用い、他 block や部品の
/// 素のセレクタへ影響させない（`feature_large_screenshot` と同じ名前空間
/// 分離）。
const LAYOUT_CSS: &str = "\
.blocks-feature-split-screenshot-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}\n\
.blocks-feature-split-screenshot-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  overflow: hidden;\n}\n\
.blocks-feature-split-screenshot-text {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  min-width: 0;\n}\n\
.blocks-feature-split-screenshot-features {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  margin-top: var(--fandhe-space-2);\n}\n\
.blocks-feature-split-screenshot-feature {\n  display: grid;\n  grid-template-columns: auto 1fr;\n  gap: var(--fandhe-space-3);\n  align-items: start;\n}\n\
.blocks-feature-split-screenshot-media {\n  min-width: 0;\n  display: flex;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-feature-split-screenshot-lead] {\n  margin: 0;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-feature-split-screenshot-feature-text] {\n  margin: 0;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-feature-split-screenshot-image] {\n  display: block;\n  width: 100%;\n  border: 1px solid var(--fandhe-color-border);\n  box-shadow: var(--fandhe-shadow-lg);\n}\n\
.blocks-feature-split-screenshot-panel {\n  background: var(--fandhe-color-accent);\n  border-radius: var(--fandhe-radius-lg);\n  padding: var(--fandhe-space-8) 0 0 var(--fandhe-space-8);\n  overflow: hidden;\n  width: 100%;\n}\n\
.blocks-feature-split-screenshot-panel [data-scope=\"image\"][data-part=\"root\"][data-blocks-feature-split-screenshot-image] {\n  box-shadow: none;\n}\n\
.blocks-feature-split-screenshot-code-frame {\n  width: 100%;\n  background: var(--fandhe-color-fg);\n  color: var(--fandhe-color-bg);\n  border-radius: var(--fandhe-radius-lg);\n  overflow: hidden;\n}\n\
.blocks-feature-split-screenshot-tabs {\n  display: flex;\n  gap: var(--fandhe-space-4);\n  padding: var(--fandhe-space-3) var(--fandhe-space-4);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-feature-split-screenshot-tabs span {\n  opacity: 0.6;\n}\n\
.blocks-feature-split-screenshot-tabs [data-blocks-feature-split-screenshot-tab-active] {\n  opacity: 1;\n  text-decoration: underline;\n}\n\
.blocks-feature-split-screenshot-pre {\n  margin: 0;\n  padding: var(--fandhe-space-6);\n  overflow-x: auto;\n}\n\
[data-scope=\"code\"][data-part=\"root\"][data-blocks-feature-split-screenshot-code] {\n  display: block;\n  white-space: pre;\n  background: transparent;\n  color: inherit;\n  border: 0;\n  padding: 0;\n}\n\
@media (min-width: 64rem) {\n  \
.blocks-feature-split-screenshot-section {\n    display: grid;\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n    align-items: center;\n    column-gap: var(--fandhe-space-12);\n  }\n  \
.blocks-feature-split-screenshot-media {\n    min-width: 0;\n  }\n  \
[data-scope=\"image\"][data-part=\"root\"][data-blocks-feature-split-screenshot-image] {\n    width: 48rem;\n    max-width: none;\n    flex: none;\n  }\n  \
.blocks-feature-split-screenshot-panel {\n    display: flex;\n    justify-content: flex-end;\n  }\n  \
.blocks-feature-split-screenshot-panel [data-scope=\"image\"][data-part=\"root\"][data-blocks-feature-split-screenshot-image] {\n    flex: none;\n  }\n  \
[data-blocks-feature-split-screenshot-reverse] > .blocks-feature-split-screenshot-text {\n    grid-column: 2;\n    grid-row: 1;\n  }\n  \
[data-blocks-feature-split-screenshot-reverse] > .blocks-feature-split-screenshot-media {\n    grid-column: 1;\n    grid-row: 1;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, dummy_assets, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていることの単体
    /// 回帰（`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"icon\"",
            "data-scope=\"image\"",
            "data-scope=\"code\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(html.matches("<img").count(), 2);
        assert_eq!(
            html.matches("data-blocks-feature-split-screenshot-reverse")
                .count(),
            1
        );
        assert_eq!(
            html.matches("data-blocks-feature-split-screenshot-tab-active")
                .count(),
            1
        );
        assert!(html.contains(dummy_assets::SCREENSHOT_SRC));
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("id=\""));
        assert!(!html.contains("role=\"tab\""));
        assert!(!html.contains("<button"));
    }

    /// [`LAYOUT_CSS`] が想定する lg グリッド切り替え・反転セレクタ・
    /// はみ出しのための切り取り宣言を持つこと。
    #[test]
    fn layout_css_declares_lg_grid_and_overflow_clip() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("repeat(2, minmax(0, 1fr))"));
        assert!(LAYOUT_CSS.contains("[data-blocks-feature-split-screenshot-reverse]"));
        assert!(LAYOUT_CSS.contains("overflow: hidden"));
        assert!(LAYOUT_CSS.contains("max-width: none"));
        assert!(LAYOUT_CSS.contains("var(--fandhe-color-accent)"));
    }

    /// Section A（基準形）の画像は `.blocks-feature-split-screenshot-media`
    /// が `display: flex` であるため、`flex: none`（`flex-shrink: 0` 相当）が
    /// 無いと flex アイテムとして縮小され、意図した `width: 48rem` の
    /// はみ出しが実現しない（レビュー指摘、Section B にのみ
    /// `flex: none` が付いていた非対称な実装漏れの回帰防止）。
    #[test]
    fn layout_css_base_image_selector_has_flex_none() {
        let base_image_rule_start = LAYOUT_CSS
            .find("[data-scope=\"image\"][data-part=\"root\"][data-blocks-feature-split-screenshot-image] {\n    width: 48rem;")
            .expect("lg breakpoint の基準 image セレクタ規則が存在すること");
        let base_image_rule_end = LAYOUT_CSS[base_image_rule_start..]
            .find("}\n  ")
            .map(|offset| base_image_rule_start + offset)
            .expect("基準 image セレクタ規則の閉じ括弧が存在すること");
        let base_image_rule = &LAYOUT_CSS[base_image_rule_start..base_image_rule_end];
        assert!(
            base_image_rule.contains("flex: none"),
            "Section A（基準形）の画像セレクタに flex: none が無いと、\
             media 列が display: flex のため画像が縮小されはみ出さない"
        );
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（モジュール doc「CSS フックの選び方」節の Bugbot 教訓の
    /// 固定、`feature_large_screenshot` と同じ回帰）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-feature-split-screenshot-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-feature-split-screenshot-layout"
        );
    }
}
