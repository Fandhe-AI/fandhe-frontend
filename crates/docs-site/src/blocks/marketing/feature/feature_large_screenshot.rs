//! `feature-large-screenshot` block（イシュー #2766。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー」配下、Phase 1（マーケティング A）に
//! 属する。中央寄せの見出し・リード文の下にアプリ画面の大きな画像を全幅で
//! 置き、その下に feature 一覧を並べる 1 列構成の feature セクションの
//! 合成例（`feature-accordion-image`/`feature-alternating-rows`/
//! `feature-expand` に続く Marketing / Feature カテゴリの 4 件目）。
//! 取得手段・ファイル名・内部コンポーネント識別子は記載しない（対応表 ID
//! は基準形 R0933 と、R0473 / R0488 / R0934 / R0938 / R1154 の集約元のみを
//! 記す）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `button` / `image` / `icon` の 6 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 3 形を 1 つの Demo へ並べる
//!
//! イシューが求める「画像の見せ方 3 通り（枠付き/下端フェード/暗色パネル）
//! と、一覧を省いた最小形」を、`content-split-image` のように別々の
//! block へ分けず、縦に並ぶ 3 つのセクションとして 1 つの Demo に収める。
//!
//! - **A（基準形・下端フェード、R0933 基準・R0473 集約）**: eyebrow badge +
//!   見出し + リード文 → 下端フェード付きの大画像 → feature 6 件を 3 列
//!   grid。
//! - **B（最小形・枠付き、R0488/R0934 集約）**: 見出し + リード文 +
//!   ボタン行 → 枠付きの大画像。**一覧は持たない**（イシューが挙げる
//!   最小形の要件）。
//! - **C（暗色パネル、R0938 集約）**: 反転配色のパネルの中に見出し + リード
//!   文 → 大画像 → feature 3 件を 3 列 grid。
//!
//! R1154（上部の帯画像 + 定義リスト 6 項目）は独立の Demo にせず、A の
//! 6 件 grid へ統合した（定義リストは使用部品 6 つに含まれないため
//! `<dl>` は使わない）。
//!
//! # フェードの色を面の色に合わせる理由
//!
//! `.blocks-demo` 枠の背景はテーマの `--fandhe-color-bg-subtle` のため、
//! フェードだけを `--fandhe-color-bg` へ向けると section の背景との境界が
//! 見えてしまう。A の section 自体に `background: var(--fandhe-color-bg)`
//! を明示し、フェードの `linear-gradient` の行き先も同じ
//! `var(--fandhe-color-bg)` にすることで、フェードが指すテーマの背景色と
//! 画像の背後にある面の色を一致させる（イシュー本文の「テーマの背景色へ
//! 向かう」という表現をそのまま満たす）。C（暗色パネル）でフェードを使う
//! 場合も同じ理由でパネル面の `var(--fandhe-color-fg)` へ向ける。フェードは
//! 空の装飾要素を DOM に追加する方式ではなく、`::after` 疑似要素で描く
//! （読み上げに影響を与えない）。
//!
//! # 負の margin によるはみ出しを採らない理由
//!
//! 参照元は画像を親コンテナ幅より広げるため負の `margin-inline` を使うが、
//! `.blocks-demo` は `overflow-x: auto` のため、はみ出しは横スクロール
//! バーの出現という形で Demo の可読性を損なう。本 Demo は画像を
//! `width: 100%` の全幅のみに留め、はみ出しは行わない。
//!
//! # 画像に `AspectRatio::Auto` を選ぶ理由
//!
//! [`dummy_assets::SCREENSHOT_SRC`] は 200×140 のプレースホルダー SVG
//! （ブラウザ枠を模した図形）であり、`AspectRatio::Video`（16:9）で
//! `Cover` すると上部のブラウザ枠が切れる。元の比率のまま全幅表示する
//! ため `AspectRatio::Auto` を使う（R0473 が使う 16:9 との差分）。
//!
//! # 見出しレベルに `H3`/`H4` を使う理由
//!
//! ページ側が `## Demo` として `h2` を出すため、各セクションの見出しは
//! `HeadingLevel::H3`、feature 項目見出しはそれより 1 段下げて
//! `HeadingLevel::H4` にする（他 block と同じ判断）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と
//! `fandhe_frontend_core::text` が同名のため、styled 側を `styled_text`
//! として取り込む（`crate::blocks` 内の他 block と同じ回避方法）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `badge::badge` / `heading::heading` / `button::button` /
//! `image::image` / `icon::icon` はいずれも `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、Demo
//! 固有のスタイルフックは `data-blocks-feature-large-screenshot-*` 属性で
//! 渡し、[`LAYOUT_CSS`] 側も同じ属性セレクタで対応する。素の `div` には
//! `class` がそのまま効くため、それらは
//! `.blocks-feature-large-screenshot-*` クラスセレクタを使う。レイアウト
//! root の class（`blocks-feature-large-screenshot-layout`）は
//! [`Block::demo_class`]（`blocks-feature-large-screenshot`）とは意図的に
//! 別名にする（`content-split-image` と同じ Bugbot 教訓の回避）。
//!
//! `styled_text::text`・`image::image` の recipe（詳細度 (0,2,0)）に確実に
//! 勝つため、上書きは `[data-scope="text"][data-part="root"][data-blocks-
//! feature-large-screenshot-*]` / `[data-scope="image"][data-part="root"]
//! [data-blocks-feature-large-screenshot-image]` の 3 セレクタ構成（詳細度
//! (0,3,0)）で行う。
//!
//! # 暗色パネルの配色継承と badge を置かない理由
//!
//! C（暗色パネル）は `background: var(--fandhe-color-fg); color:
//! var(--fandhe-color-bg);` を付与する div で包み、`heading`/`styled_text`
//! は `color: inherit` の恩恵をそのまま受ける（個別の色上書きを追加
//! しない）。`icon` は `fill="currentColor"` 固定のため同様に自動で
//! 追従する。`badge` は全 variant が自前の配色（`--fandhe-palette-*`
//! トークン参照）を持ち反転面での可読性を保証しないため、C には置かない
//! （`badge` は A で使用済みのため `parts` 契約は満たされる）。
//!
//! # md（48rem）未満で一覧を 1 列にする
//!
//! feature 一覧の grid は `grid-template-columns: minmax(0, 1fr)` を既定
//! とし、`@media (min-width: 48rem)` で 3 列（`repeat(3, minmax(0, 1fr))`）
//! へ切り替える。`@media` の条件式内ではテーマの breakpoint トークンが
//! 解決できないため、他 block と同じ判断でリテラル `48rem` を直書きする。
//!
//! # `<form>` を持たない・データ取得/送信を行わない・id を使わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言はすべて架空のもの（実企業名・実クレデンシャル・PII を
//! 含まない）。画像は [`dummy_assets::SCREENSHOT_SRC`]（ビルド時生成の
//! プレースホルダー SVG）のみを使い、装飾扱いの `alt=""` で出力する。
//! ボタンは既定の `type="button"` のまま送信先を持たない。`id` 属性は
//! 一切使わない（重複 id 検知テスト対策）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 装飾用の自作幾何アイコン（lucide 等の著作物を複製しないための単純図形、
/// `feature_expand::geo_icon` と同型の判断）。線画（ストローク）として
/// 描画するため `fill="none"` を明示し、`icon` の `<svg>` 側が固定で持つ
/// `fill="currentColor"`（塗り面）を上書きする。
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

/// feature 項目 1 件分（見出し + 説明 + アイコン形状）。
struct FeatureItem {
    title: &'static str,
    body: &'static str,
    icon_path_d: &'static str,
}

/// A（基準形）向けの 6 件分の架空データ。C はこの先頭 3 件（`&ITEMS[..3]`）
/// を再利用する。
const ITEMS: [FeatureItem; 6] = [
    FeatureItem {
        title: "リアルタイム同期",
        body: "変更内容は即座に全メンバーへ反映され、手動更新は不要です。",
        icon_path_d: "M12 2a10 10 0 100 20 10 10 0 000-20zM12 6v6l4 2",
    },
    FeatureItem {
        title: "きめ細かな権限",
        body: "リソース単位で閲覧・編集・共有の範囲を割り当てられます。",
        icon_path_d: "M12 2l8 4v6c0 5-3.5 8-8 10-4.5-2-8-5-8-10V6z",
    },
    FeatureItem {
        title: "監査ログ",
        body: "誰が何をいつ変更したかを追跡し、後から検索できます。",
        icon_path_d: "M4 4h16v4H4zM4 10h10v4H4zM4 16h14v4H4z",
    },
    FeatureItem {
        title: "自動バックアップ",
        body: "日次でスナップショットを取得し、任意の時点へ復元できます。",
        icon_path_d: "M12 2v6l4-3M4 13a8 8 0 1016 0",
    },
    FeatureItem {
        title: "外部連携",
        body: "既存のツールとイベント連携し、通知や同期を自動化します。",
        icon_path_d: "M8 12h8M12 8v8M4 4h6v6H4zM14 14h6v6h-6z",
    },
    FeatureItem {
        title: "使用状況の可視化",
        body: "チームごとの利用傾向をダッシュボードで確認できます。",
        icon_path_d: "M4 20V10M10 20V4M16 20v-7M22 20V2",
    },
];

/// 中央寄せのセクション見出し（eyebrow badge + 見出し + リード文）。
/// `eyebrow` が `None` のとき badge は出力しない（B が使う）。
fn section_header(eyebrow: Option<&str>, title: &str, lead: &str) -> Node {
    let mut children: Vec<Node> = Vec::new();
    if let Some(eyebrow) = eyebrow {
        children.push(badge::badge(
            &BadgeProps::default(),
            vec![("data-blocks-feature-large-screenshot-eyebrow", "")],
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
        vec![("data-blocks-feature-large-screenshot-lead", "")],
        vec![text(lead)],
    ));

    div(
        vec![("class", "blocks-feature-large-screenshot-header")],
        children,
    )
}

/// 大きな画面画像（`variant` は `fade`/`bordered`/`panel` のいずれかで、
/// [`LAYOUT_CSS`] 側の見せ方の切り替えに使う）。装飾扱いのため `alt=""`。
fn screenshot(variant: &'static str) -> Node {
    div(
        vec![
            ("class", "blocks-feature-large-screenshot-media"),
            ("data-blocks-feature-large-screenshot-variant", variant),
        ],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                aspect_ratio: AspectRatio::Auto,
                shape: ImageShape::Rounded,
                ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
            },
            vec![("data-blocks-feature-large-screenshot-image", "")],
        )],
    )
}

/// feature 項目 1 件（アイコン + 見出し + 説明）。
fn feature_item(item: &FeatureItem) -> Node {
    div(
        vec![("class", "blocks-feature-large-screenshot-item")],
        vec![
            geo_icon(item.icon_path_d),
            heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text(item.title)],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-feature-large-screenshot-desc", "")],
                vec![text(item.body)],
            ),
        ],
    )
}

/// feature 一覧（3 列 grid、md 未満は 1 列。R1154 の帯画像 + 定義リストを
/// ここへ統合した）。
fn feature_grid(items: &[FeatureItem]) -> Node {
    div(
        vec![("class", "blocks-feature-large-screenshot-grid")],
        items.iter().map(feature_item).collect(),
    )
}

/// A: 基準形（eyebrow + 見出し + リード文 → 下端フェード付き画像 →
/// feature 6 件の grid）。
fn fade_section() -> Node {
    div(
        vec![
            ("class", "blocks-feature-large-screenshot-section"),
            (
                "data-blocks-feature-large-screenshot-section-variant",
                "fade",
            ),
        ],
        vec![
            section_header(
                Some("導入事例"),
                "現場の画面をそのまま見せる",
                "実際のダッシュボード画面と、そこから得られる主要な特長をあわせて紹介します。",
            ),
            screenshot("fade"),
            feature_grid(&ITEMS),
        ],
    )
}

/// B: 最小形（見出し + リード文 + ボタン行 → 枠付き画像。一覧は持たない）。
fn bordered_section() -> Node {
    div(
        vec![
            ("class", "blocks-feature-large-screenshot-section"),
            (
                "data-blocks-feature-large-screenshot-section-variant",
                "bordered",
            ),
        ],
        vec![
            section_header(
                None,
                "まずは画面を見てみる",
                "導入前に、実際の操作画面を確認できます。",
            ),
            div(
                vec![("class", "blocks-feature-large-screenshot-actions")],
                vec![
                    button::button(
                        &ButtonProps::default(),
                        vec![("type", "button")],
                        vec![text("デモを見る")],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![("type", "button")],
                        vec![text("資料をダウンロード")],
                    ),
                ],
            ),
            screenshot("bordered"),
        ],
    )
}

/// C: 暗色パネル（反転配色の中に見出し + リード文 → 画像 → feature 3 件の
/// grid）。
fn panel_section() -> Node {
    div(
        vec![
            ("class", "blocks-feature-large-screenshot-section"),
            (
                "data-blocks-feature-large-screenshot-section-variant",
                "panel",
            ),
        ],
        vec![
            section_header(
                None,
                "暗い画面でも見やすく",
                "ダークテーマのアプリでも、同じ構成で特長を伝えられます。",
            ),
            screenshot("panel"),
            feature_grid(&ITEMS[..3]),
        ],
    )
}

/// `feature-large-screenshot` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（他 block と同じ状態を持たない設計）。ルート class は
/// `demo_class`（`blocks-feature-large-screenshot`）とは別名にする。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-feature-large-screenshot-layout")],
        vec![fade_section(), bordered_section(), panel_section()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/feature-large-screenshot/",
    title: "feature-large-screenshot",
    category: BlockCategory::Feature,
    rust_source: "crates/docs-site/src/blocks/marketing/feature/feature_large_screenshot.rs",
    demo_class: "blocks-feature-large-screenshot",
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
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `feature_large_screenshot` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節）。セレクタは
/// `.blocks-feature-large-screenshot-*` と
/// `[data-blocks-feature-large-screenshot-*]` のみを用い、他 block や部品の
/// 素のセレクタへ影響させない（`content-split-image` と同じ名前空間分離）。
const LAYOUT_CSS: &str = "\
.blocks-feature-large-screenshot-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}\n\
.blocks-feature-large-screenshot-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n  padding: var(--fandhe-space-8);\n  border-radius: var(--fandhe-radius-lg);\n}\n\
[data-blocks-feature-large-screenshot-section-variant=\"fade\"] {\n  background: var(--fandhe-color-bg);\n}\n\
[data-blocks-feature-large-screenshot-section-variant=\"panel\"] {\n  background: var(--fandhe-color-fg);\n  color: var(--fandhe-color-bg);\n}\n\
.blocks-feature-large-screenshot-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  align-items: center;\n  text-align: center;\n  max-width: 48rem;\n  margin: 0 auto;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-feature-large-screenshot-lead] {\n  margin: 0;\n}\n\
.blocks-feature-large-screenshot-actions {\n  display: flex;\n  flex-wrap: wrap;\n  justify-content: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-feature-large-screenshot-media {\n  position: relative;\n  overflow: hidden;\n  border-radius: var(--fandhe-radius-lg);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-feature-large-screenshot-image] {\n  display: block;\n  width: 100%;\n}\n\
[data-blocks-feature-large-screenshot-variant=\"bordered\"] [data-scope=\"image\"][data-part=\"root\"][data-blocks-feature-large-screenshot-image] {\n  border: 1px solid var(--fandhe-color-border);\n  box-shadow: var(--fandhe-shadow-lg);\n}\n\
[data-blocks-feature-large-screenshot-variant=\"fade\"]::after,\n\
[data-blocks-feature-large-screenshot-variant=\"panel\"]::after {\n  content: \"\";\n  position: absolute;\n  inset: auto 0 0 0;\n  height: 35%;\n  pointer-events: none;\n}\n\
[data-blocks-feature-large-screenshot-variant=\"fade\"]::after {\n  background: linear-gradient(to top, var(--fandhe-color-bg), transparent);\n}\n\
[data-blocks-feature-large-screenshot-variant=\"panel\"]::after {\n  background: linear-gradient(to top, var(--fandhe-color-fg), transparent);\n}\n\
.blocks-feature-large-screenshot-grid {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-6);\n}\n\
@media (min-width: 48rem) {\n  \
.blocks-feature-large-screenshot-grid {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n\
}\n\
.blocks-feature-large-screenshot-item {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  min-width: 0;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-feature-large-screenshot-desc] {\n  margin: 0;\n}\n";

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
            "data-scope=\"button\"",
            "data-scope=\"image\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(html.matches("<img").count(), 3);
        assert_eq!(html.matches("<button").count(), 2);
        // アイコンは A（6 件）+ C（3 件）の合計 9 個。
        assert_eq!(html.matches("data-scope=\"icon\"").count(), 9);
        for variant in ["fade", "bordered", "panel"] {
            assert_eq!(
                html.matches(&format!(
                    "data-blocks-feature-large-screenshot-section-variant=\"{variant}\""
                ))
                .count(),
                1
            );
        }
        assert!(html.contains(dummy_assets::SCREENSHOT_SRC));
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("id=\""));
    }

    /// [`LAYOUT_CSS`] が想定する md グリッド切り替え・下端フェードを持つこと。
    #[test]
    fn layout_css_declares_md_grid_and_fade() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("repeat(3, minmax(0, 1fr))"));
        assert!(LAYOUT_CSS.contains("linear-gradient(to top, var(--fandhe-color-bg), transparent)"));
        assert!(LAYOUT_CSS.contains("::after"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（モジュール doc「CSS フックの選び方」節の Bugbot 教訓の
    /// 固定、`content-split-image` と同じ回帰）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-feature-large-screenshot-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-feature-large-screenshot-layout"
        );
    }
}
