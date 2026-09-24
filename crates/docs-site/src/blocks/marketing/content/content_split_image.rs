//! `content-split-image` block（イシュー #2755。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー」配下、Phase 1（#2738、マーケティング A）
//! に属する。見出しと本文を片側の列に、画像または画面画像を反対側の列に
//! 置く 2 列コンテンツの合成例で、`content-columns-screenshot` に続く
//! Marketing / Content カテゴリの 2 件目）。取得手段・ファイル名・内部
//! コンポーネント識別子は記載しない（`docs/design/motion-reference-
//! adoption-policy.md` §9 と同じライセンス上の転記制限、対応表 ID は
//! R0867（基準形）と R0872 のみを記す）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `image` / `icon` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! `button` は使用しない（イシュー本文の使用部品一覧に含まれないため）。
//!
//! # 2 形を 1 つの Demo に並記する
//!
//! イシューが要求する 2 形（R0867 基準形の sticky 配置・R0872 の左半分
//! 全高画像）を、`sidebar-07`/`footer-newsletter` と同型に 1 つの Demo
//! （[`demo`]）内へ縦に並記する。各形の直前に短いラベル（[`variant_label`]）
//! を置き、Demo 上でどちらがどちらかを読み取れるようにする。
//!
//! # ブレークポイントに lg（64rem）を使う理由
//!
//! `.docs-content` の `max-width` は `--fandhe-space-docs-max-content-width`
//! （46rem）であり、Demo の実効幅はそれよりわずかに狭い。イシュー仕様の
//! lg（[`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`]、
//! 1024px = 64rem）はビューポート幅で判定されるため、ビューポートが
//! 64rem 以上であれば Demo 内の 2 列化は実際に発火する（`content-columns-
//! screenshot` が md（47.99rem）へ逸脱したのとは異なり、本 block は逸脱
//! しない）。テーマの breakpoint トークンは `@media` 条件式の中では解決
//! できないため（CSS custom property は宣言側でのみ有効）、64rem の
//! リテラル値を [`LAYOUT_CSS`] へ直書きする（`content-columns-screenshot`
//! と同じ判断）。
//!
//! # sticky を実際に見せるためのスクロール枠（R0867 基準形との差分）
//!
//! `.blocks-demo` は横スクロール用の `overflow-x: auto` を持ち、CSS
//! Overflow 仕様上 `overflow-y` も省略時は同じ値へ強制されるため
//! `.blocks-demo` 自体がスクロールコンテナになるが、高さの上限が無いため
//! そのままでは sticky が一切効かない。本 block は形 A（sticky 配置）
//! 全体を専用のスクロール枠（[`variant_sticky`] が返す
//! `.blocks-content-split-image-scroller`）で包み、`max-height` +
//! `overflow-y: auto` を明示することで sticky を発火させる
//! （`footer-sticky-reveal` が Demo 枠自体を固定高スクロールコンテナに
//! した前例とは異なり、本 block は形 A だけを包む点が差分。形 B は
//! スクロール枠を持たない）。スクロール枠はキーボード操作者もスクロール
//! できるよう `tabindex="0"` + `role="region"` + `aria-label` を持つ
//! （原案差分メモ参照）。
//!
//! # R0872（全高画像）の簡略化
//!
//! 参照元が用いる画面左半分への絶対配置・`display: contents`・負の
//! マージンは持ち込まず、[`variant_full_height`] はレイアウト root 内の
//! 素直な 2 列 grid（`align-items: stretch`）へ簡略化する。画像は
//! `object-fit: cover` + lg 以上で `height: 100%` により右列の本文と
//! 同じ高さへ伸ばし、lg 未満では固定高で本文の上に置く（原案差分メモ
//! 参照）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `badge::badge` / `heading::heading` / `text::text` / `image::image` /
//! `icon::icon` はいずれも `drop_class_attr` により呼び出し側 `attrs` の
//! `class` を黙って除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-content-split-image-*` 属性で渡し、[`LAYOUT_CSS`] 側も
//! 同じ属性セレクタで対応する。素の `div`/`ul`/`li` には `class` が
//! そのまま効くため、それらは従来どおり `.blocks-content-split-image-*`
//! クラスセレクタを使う。レイアウト root の class
//! （`blocks-content-split-image-layout`）は [`Block::demo_class`]
//! （`blocks-content-split-image`）とは意図的に別名にする
//! （`content-columns-screenshot` と同じ Bugbot 教訓の回避）。
//!
//! # 詳細度の罠（`text`/`image` recipe への勝ち方）
//!
//! `text::text`・`image::image` の recipe（詳細度 (0,2,0)）に確実に勝つ
//! ため、上書きは `[data-scope="text"][data-part="root"][data-blocks-
//! content-split-image-*]` / `[data-scope="image"][data-part="root"]
//! [data-blocks-content-split-image-*]` の 3 セレクタ構成（詳細度
//! (0,3,0)）で行う（`content-columns-screenshot` と同型の判断）。
//!
//! # 見出しレベルに `H3` を使う理由
//!
//! ページ側が `## Demo` として `h2` を出すため、見出しは
//! `HeadingLevel::H3` にする（`content-columns-screenshot` と同じ判断）。
//! 参照元にある小見出しの `h2` は出さない。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と
//! `fandhe_frontend_core::text` が同名のため、styled 側を `styled_text`
//! として取り込む（`crate::blocks` 内の他 block と同じ回避方法）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない・id を使わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言はすべて架空のもの（実企業名・実クレデンシャル・PII を
//! 含まない）。画像は [`crate::blocks::dummy_assets::SCREENSHOT_SRC`]
//! （形 A）・[`crate::blocks::dummy_assets::BACKGROUND_SRC`]（形 B）
//! （いずれもビルド時生成のプレースホルダー SVG）を使い、`alt=""`
//! （装飾扱い）で出力する。Demo は 2 インスタンスを持つため `id` 属性は
//! 一切使わない（重複 id 検知テスト対策）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};

/// 装飾用の自作幾何アイコン（lucide 等の著作物を複製しないための単純図形、
/// `feature_expand::geo_icon` と同型の判断）。`path` へ `fill="none"` +
/// `stroke="currentColor"` を明示し、`icon` の `<svg>` 側が固定で持つ
/// `fill="currentColor"`（塗り面）を上書きして線画（ストローク）として
/// 描画する。
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

/// 特長リスト 1 項目分（架空の特長名 + 短い説明 + 幾何アイコン）。
struct Feature {
    icon_path_d: &'static str,
    label: &'static str,
    description: &'static str,
}

/// 形 A（sticky）の本文が持つ特長リスト（3 項目、架空文言）。
const FEATURES: [Feature; 3] = [
    Feature {
        icon_path_d: "M5 13l4 4L19 7",
        label: "決定的な出力",
        description: "同じ入力からは常に同じ HTML を生成し、差分を追跡しやすくします。",
    },
    Feature {
        icon_path_d: "M12 3l8 4.5v9L12 21l-8-4.5v-9L12 3z",
        label: "型で保証する構造",
        description: "スロットと props は Rust の型で表現され、不整合をコンパイル時に検出します。",
    },
    Feature {
        icon_path_d: "M12 8v4l3 3",
        label: "静的な表示のみ",
        description: "JS ハイドレーションを行わない docs サイト向けの、決定的な静的表示です。",
    },
];

/// 本文段落（架空文言、2 形で使い回して検索インデックスの増分を抑える）。
const PARAGRAPHS: &[&str] = &[
    "本文は node 木 API で組み立てた text ノードのみで構成し、文字列結合による HTML 生成は行いません。",
    "見出し・本文・画像はいずれも Themes の既存部品からのみ合成しています。",
    "この Demo はデータ取得や送信を行わない、静的な表示専用の合成例です。",
];

/// 各形の直前に置く短い形ラベル（`styled_text::text` の `Sm`/`Muted`）。
fn variant_label(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// 本文段落 1 個（`margin: 0` 上書きフックを内蔵する。呼び出し箇所ごとに
/// 属性リテラルを重複させないため、フック名はここへ 1 箇所だけ書く）。
fn paragraph(body: &'static str) -> Node {
    styled_text::text(
        &TextProps::default(),
        vec![("data-blocks-content-split-image-paragraph", "")],
        vec![text(body)],
    )
}

/// eyebrow badge + 見出しの組（形 A・形 B で共通の頭部、
/// `content-columns-screenshot::demo` のヘッダー構成と同型）。
fn header(eyebrow: &'static str, title: &'static str) -> Node {
    div(
        vec![("class", "blocks-content-split-image-header")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-content-split-image-eyebrow", "")],
                vec![text(eyebrow)],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text(title)],
            ),
        ],
    )
}

/// 特長リスト 1 項目（アイコン + 太字ラベル + 説明）。
fn feature_item(feature: &Feature) -> Node {
    li(
        vec![("class", "blocks-content-split-image-feature")],
        vec![
            geo_icon(feature.icon_path_d),
            div(
                vec![("class", "blocks-content-split-image-feature-copy")],
                vec![
                    styled_text::text(
                        &TextProps {
                            weight: TextWeight::Semibold,
                            ..TextProps::default()
                        },
                        vec![("data-blocks-content-split-image-feature-label", "")],
                        vec![text(feature.label)],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![("data-blocks-content-split-image-feature-desc", "")],
                        vec![text(feature.description)],
                    ),
                ],
            ),
        ],
    )
}

/// 特長リスト（形 A の本文にのみ現れる、`ul`/`li` を素の core タグで組む）。
fn feature_list() -> Node {
    ul(
        vec![("class", "blocks-content-split-image-features")],
        FEATURES.iter().map(feature_item).collect(),
    )
}

/// 形 A（R0867 基準形）: 本文をスクロールしている間、画面画像が上部に
/// 留まる sticky 配置。DOM 順は header → 画像 → body（lg 未満の 1 列
/// 表示で画像が本文より上に来るようにするため、モジュール doc「sticky を
/// 実際に見せるためのスクロール枠」節参照）。
fn variant_sticky() -> Node {
    let header_node = header("導入ガイド", "スクロールに追従する画面画像");

    let body = div(
        vec![("class", "blocks-content-split-image-body")],
        vec![
            paragraph(PARAGRAPHS[0]),
            paragraph(PARAGRAPHS[1]),
            paragraph(PARAGRAPHS[2]),
            feature_list(),
        ],
    );

    let media = div(
        vec![("class", "blocks-content-split-image-sticky-media")],
        vec![image::image(
            &ImageProps {
                shape: ImageShape::Rounded,
                ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
            },
            vec![("data-blocks-content-split-image-sticky-image", "")],
        )],
    );

    let grid = div(
        vec![("class", "blocks-content-split-image-sticky-grid")],
        vec![header_node, media, body],
    );

    div(
        vec![
            ("class", "blocks-content-split-image-scroller"),
            ("tabindex", "0"),
            ("role", "region"),
            ("aria-label", "本文をスクロールできる領域"),
        ],
        vec![grid],
    )
}

/// 形 B（R0872）: 左半分に全高の画像を張り、右に本文を置く形。DOM 順は
/// 画像 → テキスト列（lg 未満の 1 列表示で画像が上に来るようにするため）。
fn variant_full_height() -> Node {
    let media = div(
        vec![("class", "blocks-content-split-image-full-media")],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                shape: ImageShape::Rounded,
                ..ImageProps::new(dummy_assets::BACKGROUND_SRC, "")
            },
            vec![("data-blocks-content-split-image-full-image", "")],
        )],
    );

    let text_col = div(
        vec![("class", "blocks-content-split-image-text-col")],
        vec![
            header("導入ガイド", "左半分に全高の画像を敷く構成"),
            div(
                vec![("class", "blocks-content-split-image-body")],
                vec![paragraph(PARAGRAPHS[0]), paragraph(PARAGRAPHS[1])],
            ),
        ],
    );

    div(
        vec![("class", "blocks-content-split-image-full-grid")],
        vec![media, text_col],
    )
}

/// `content-split-image` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「2 形を 1 つの Demo に並記する」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-content-split-image-layout")],
        vec![
            variant_label("sticky な画面画像（R0867 基準形）"),
            variant_sticky(),
            variant_label("全高の画像（R0872）"),
            variant_full_height(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/content-split-image/",
    title: "content-split-image",
    category: BlockCategory::Content,
    rust_source: "crates/docs-site/src/blocks/marketing/content/content_split_image.rs",
    demo_class: "blocks-content-split-image",
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

/// `content_split_image` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` では
/// なく本ファイル内 private 定数として `super::stylesheet` 経由の
/// `push_css` で連結される）。
///
/// セレクタは `.blocks-content-split-image-*` と
/// `[data-blocks-content-split-image-*]` のみを用い、他 block や部品の
/// 素のセレクタへ影響させない（`content-columns-screenshot` と同じ
/// 名前空間分離）。
const LAYOUT_CSS: &str = "\
.blocks-content-split-image-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-content-split-image-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  align-items: flex-start;\n}\n\
.blocks-content-split-image-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-content-split-image-paragraph] {\n  margin: 0;\n}\n\
.blocks-content-split-image-features {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  margin: 0;\n  padding: 0;\n  list-style: none;\n}\n\
.blocks-content-split-image-feature {\n  display: flex;\n  gap: var(--fandhe-space-3);\n  align-items: flex-start;\n}\n\
.blocks-content-split-image-feature-copy {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-content-split-image-feature-label] {\n  margin: 0;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-content-split-image-feature-desc] {\n  margin: 0;\n}\n\
.blocks-content-split-image-scroller {\n  max-height: 24rem;\n  overflow-y: auto;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n  padding: var(--fandhe-space-6);\n}\n\
.blocks-content-split-image-sticky-grid {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-content-split-image-sticky-image] {\n  display: block;\n  width: 100%;\n  border: 1px solid var(--fandhe-color-border);\n  box-shadow: var(--fandhe-shadow-lg);\n}\n\
.blocks-content-split-image-full-grid {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  align-items: stretch;\n}\n\
.blocks-content-split-image-text-col {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  min-width: 0;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-content-split-image-full-image] {\n  display: block;\n  width: 100%;\n  height: 16rem;\n  border: 1px solid var(--fandhe-color-border);\n}\n\
@media (min-width: 64rem) {\n  \
.blocks-content-split-image-sticky-grid {\n    display: grid;\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n    gap: var(--fandhe-space-8);\n    align-items: start;\n  }\n  \
.blocks-content-split-image-sticky-grid > .blocks-content-split-image-header {\n    grid-column: 1;\n    grid-row: 1;\n  }\n  \
.blocks-content-split-image-sticky-grid > .blocks-content-split-image-sticky-media {\n    grid-column: 2;\n    grid-row: 1 / span 2;\n    position: sticky;\n    top: var(--fandhe-space-4);\n    align-self: start;\n  }\n  \
.blocks-content-split-image-sticky-grid > .blocks-content-split-image-body {\n    grid-column: 1;\n    grid-row: 2;\n  }\n  \
.blocks-content-split-image-full-grid {\n    display: grid;\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n    gap: var(--fandhe-space-8);\n  }\n  \
[data-scope=\"image\"][data-part=\"root\"][data-blocks-content-split-image-full-image] {\n    height: 100%;\n  }\n\
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
            "data-scope=\"image\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(html.matches("<img").count(), 2);
        assert_eq!(
            html.matches("data-blocks-content-split-image-paragraph")
                .count(),
            5
        );
        assert!(html.contains(dummy_assets::SCREENSHOT_SRC));
        assert!(html.contains(dummy_assets::BACKGROUND_SRC));
        assert!(!html.contains("<form"));
        assert!(!html.contains("<button"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("id=\""));
    }

    /// スクロール枠がキーボード操作者向けのフォーカス・ランドマーク属性を
    /// 持つこと（モジュール doc「sticky を実際に見せるためのスクロール枠」
    /// 節）。
    #[test]
    fn scroller_is_keyboard_focusable_region() {
        let html = render(&demo());
        assert!(html.contains("tabindex=\"0\""));
        assert!(html.contains("role=\"region\""));
        assert!(html.contains("aria-label=\""));
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件・sticky・全高画像の
    /// 規則を持つこと。
    #[test]
    fn layout_css_declares_lg_breakpoint_sticky_and_full_height_image() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("position: sticky"));
        assert!(LAYOUT_CSS.contains("height: 100%"));
        assert!(LAYOUT_CSS.contains("overflow-y: auto"));
    }

    /// ルート grid class（`demo_class` とは別名）が `demo()` の出力へ
    /// 実際に現れること（モジュール doc「CSS フックの選び方」節の
    /// Bugbot 教訓の固定、`content-columns-screenshot` と同じ回帰）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-content-split-image-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-content-split-image-layout");
    }
}
