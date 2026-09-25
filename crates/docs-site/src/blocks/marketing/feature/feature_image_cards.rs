//! `feature-image-cards` block（イシュー #2765。親トラッキング #2730/#2738
//! 「Blocks 目的別パーツ拡充ツリー、Phase 1 マーケティング A」配下）。
//! セクション見出し・説明文の下に、画像 → 短い見出し → 説明を縦に積んだ
//! カードを並べる feature セクションの合成例（`marketing/feature` の
//! 4 件目、`feature-alternating-rows` に続く）。取得手段・ファイル名・
//! 内部コンポーネント識別子は記載しない（対応表 ID は R0476（基準形）・
//! R1153（中央寄せ見出し）・R1160（正方形画像 4 列）のみを記す）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `card` / `image` の 5 部品のみを合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新規 UI 部品は追加しない。
//!
//! # 3 参照を 2 インスタンスへ統合する
//!
//! イシューが挙げる 3 件の参照（R0476: 画像 + 見出し + 説明の 3 列カード、
//! R1153: 中央寄せ見出し + 画像付き 2 列、R1160: 正方形画像の 4 列カード）
//! を、`blog-grid-image` と同じく **2 インスタンスを縦に並べる**構成へ統合
//! する。
//!
//! - **インスタンス A**（R0476 を基準形とし R1153 の中央寄せ見出しを統合）:
//!   中央寄せの eyebrow badge + 見出し + リード文の下へ、4:3
//!   （[`AspectRatio::Landscape`]）画像カードを 3 枚並べる。
//! - **インスタンス B**（R1160）: 左寄せの導入部の下へ、正方形
//!   （[`AspectRatio::Square`]）画像カードを 4 枚並べる。
//!
//! R1153 が持つ「lg で 2 列固定」と「背景色つきの帯」は Demo に持ち込まない
//! （md 幅でインスタンス A も 2 列表示になるため、2 列の見え方自体は Demo
//! で確認できる。差分は「原案差分メモ」節で説明する）。
//!
//! # レスポンシブ列数
//!
//! md 未満は両インスタンスとも 1 列。md（48rem）以上でインスタンス A は
//! 2 列、lg（64rem）以上で 3 列になる。インスタンス B は md 以上で常に
//! 2 列、lg 以上で `[data-blocks-feature-image-cards-grid][data-columns="4"]`
//! により 4 列へ切り替わる。`content-split-image`/`feature-alternating-rows`
//! と同じ判断で、[`fandhe_frontend_pre_styled_ui::recipe::Breakpoint`] の
//! Md（768px）/Lg（1024px）のリテラル値を [`LAYOUT_CSS`] へ直書きする
//! （テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! ため）。
//!
//! # 見出しレベルに `H3`/`H4` を使う理由
//!
//! ページ側が `## Demo` として `h2` を出すため、セクション見出しは
//! `HeadingLevel::H3` にする。カード見出しはそれより 1 段下げて
//! `HeadingLevel::H4` にする（[`card::title`] は `<h3>` 固定のため使わない、
//! `blog_grid_image` と同じ判断）。
//!
//! # alt を空文字列にする理由
//!
//! カードの見出しとテキストで内容が伝わるため、画像の `alt` は
//! `feature_alternating_rows` の前例に合わせて `alt=""`（装飾扱い）にする。
//! 記事サムネイルのように画像自体が識別情報を持つ `blog_grid_image` とは
//! 性質が異なるため、同じ汎用 alt を並べる状態の懸念には当たらない。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `badge::badge` / `heading::heading` / `styled_text::text` /
//! `card::root` / `image::image` はいずれも `drop_class_attr` により呼び
//! 出し側 `attrs` の `class` を黙って除去する契約を持つため、Demo 固有の
//! スタイルフックは `data-blocks-feature-image-cards-*` 属性で渡す。
//! `card::cover` / `card::body` は variant を持たず `attrs` をそのまま
//! 連結するパーツのため、従来どおり `class` が効く。レイアウト root の
//! class（`blocks-feature-image-cards-layout`）は [`Block::demo_class`]
//! （`blocks-feature-image-cards`）とは意図的に別名にする
//! （`content-split-image`/`feature-alternating-rows` と同じ Bugbot 教訓の
//! 回避）。
//!
//! # 詳細度の罠（`image`/`text` recipe への勝ち方）
//!
//! `image::image`・`styled_text::text` の recipe（詳細度 (0,2,0)）に確実に
//! 勝つため、上書きは `[data-scope="image"][data-part="root"][data-blocks-
//! feature-image-cards-image]` / `[data-scope="text"][data-part="root"]
//! [data-blocks-feature-image-cards-*]` の 3 セレクタ構成（詳細度 (0,3,0)）
//! で行う（`feature_alternating_rows` と同型の判断）。
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
//! 含まない）。画像は [`crate::blocks::dummy_assets`] の各定数（いずれも
//! ビルド時生成のプレースホルダー SVG）をカードごとに使い分け、`alt=""`
//! で出力する。`id` 属性は一切使わない（重複 id 検知テスト対策）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text as core_text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{
    self, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// カード 1 件分の架空データ（見出し・説明・使用画像）。
struct FeatureCard {
    title: &'static str,
    body: &'static str,
    src: &'static str,
}

/// インスタンス A（4:3 画像・3 枚）のカードデータ。
const CARDS_LANDSCAPE: [FeatureCard; 3] = [
    FeatureCard {
        title: "決定的なビルド",
        body: "同じ入力からは常に同じ静的ファイルを生成し、差分レビューを容易にします。",
        src: dummy_assets::PRODUCT_SRC,
    },
    FeatureCard {
        title: "型で表現する構造",
        body: "スロットと props は Rust の型で表現され、不整合はコンパイル時に検出されます。",
        src: dummy_assets::SCREENSHOT_SRC,
    },
    FeatureCard {
        title: "静的な表示のみ",
        body: "JS ハイドレーションを行わない、決定的な静的表示専用の合成例です。",
        src: dummy_assets::BACKGROUND_SRC,
    },
];

/// インスタンス B（正方形画像・4 枚）のカードデータ。
const CARDS_SQUARE: [FeatureCard; 4] = [
    FeatureCard {
        title: "既定エスケープ",
        body: "テキスト補間は既定でエスケープされ、迂回経路は限定されています。",
        src: dummy_assets::PRODUCT_SRC,
    },
    FeatureCard {
        title: "外部依存ゼロの描画コア",
        body: "描画コアは外部クレートに依存せず、サプライチェーンの露出面を抑えます。",
        src: dummy_assets::SCREENSHOT_SRC,
    },
    FeatureCard {
        title: "単一実行ファイル配布",
        body: "SSR/SSG のいずれも単一バイナリへまとめてデプロイできます。",
        src: dummy_assets::LOGO_SRC,
    },
    FeatureCard {
        title: "無 JS のドキュメント",
        body: "docs サイト自体は JS ハイドレーションを行わない静的表示です。",
        src: dummy_assets::BACKGROUND_SRC,
    },
];

/// 中央寄せのヘッダー（eyebrow badge + 見出し + リード文、インスタンス A）。
fn header_centered() -> Node {
    div(
        vec![("class", "blocks-feature-image-cards-header")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-feature-image-cards-eyebrow", "")],
                vec![core_text("特長")],
            ),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![core_text("画像付きカードで特長を紹介する")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-feature-image-cards-lead", "")],
                vec![core_text(
                    "各カードは画像・見出し・説明の順に積み、画面幅に応じて列数を切り替えます。",
                )],
            ),
        ],
    )
}

/// 左寄せのヘッダー（badge + 見出し + リード文、インスタンス B）。
fn header_start() -> Node {
    div(
        vec![
            ("class", "blocks-feature-image-cards-header"),
            ("data-align", "start"),
        ],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-feature-image-cards-eyebrow", "")],
                vec![core_text("できること")],
            ),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![core_text("正方形画像で並べる 4 つの特長")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-feature-image-cards-lead", "")],
                vec![core_text(
                    "同じ縦横比の画像で揃えることで、密度の高いグリッドを組めます。",
                )],
            ),
        ],
    )
}

/// カード 1 枚分（画像 → 見出し → 説明）を組み立てる。`aspect_ratio` で
/// インスタンス A（4:3）/B（正方形）を切り替える。
fn feature_card(card: &FeatureCard, aspect_ratio: AspectRatio) -> Node {
    let image_node = image::image(
        &ImageProps {
            fit: ImageFit::Cover,
            aspect_ratio,
            ..ImageProps::new(card.src, "")
        },
        vec![("data-blocks-feature-image-cards-image", "")],
    );
    let title = heading::heading(
        HeadingLevel::H4,
        &HeadingProps::default(),
        vec![],
        vec![core_text(card.title)],
    );
    let body = styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![("data-blocks-feature-image-cards-desc", "")],
        vec![core_text(card.body)],
    );

    card::root(
        CardProps::default(),
        vec![("data-blocks-feature-image-cards-card", "")],
        vec![
            card::cover(vec![], vec![image_node]),
            card::body(
                vec![("class", "blocks-feature-image-cards-body")],
                vec![title, body],
            ),
        ],
    )
}

/// 1 インスタンス分（導入部 + カードグリッド）を組み立てる。`four_columns`
/// が `true` のとき lg で 4 列（インスタンス B）、`false` のとき lg で 3 列
/// （インスタンス A）になる。
fn instance(
    header: Node,
    cards: &[FeatureCard],
    aspect_ratio: AspectRatio,
    four_columns: bool,
) -> Node {
    let mut grid_attrs = vec![("class", "blocks-feature-image-cards-grid")];
    if four_columns {
        grid_attrs.push(("data-columns", "4"));
    }
    let card_nodes: Vec<Node> = cards
        .iter()
        .map(|card| feature_card(card, aspect_ratio))
        .collect();

    div(
        vec![("class", "blocks-feature-image-cards-instance")],
        vec![header, div(grid_attrs, card_nodes)],
    )
}

/// `feature-image-cards` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（他 block と同じ状態を持たない設計）。
#[must_use]
pub fn demo() -> Node {
    let note_a = styled_text::text(
        &TextProps {
            size: TextSize::Xs,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![core_text(
            "中央寄せの導入部・4:3 画像・3 列 → 2 列 → 1 列の構成例。",
        )],
    );
    let note_b = styled_text::text(
        &TextProps {
            size: TextSize::Xs,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![core_text(
            "左寄せの導入部・正方形画像・4 列 → 2 列 → 1 列の構成例。",
        )],
    );

    div(
        vec![("class", "blocks-feature-image-cards-layout")],
        vec![
            note_a,
            instance(
                header_centered(),
                &CARDS_LANDSCAPE,
                AspectRatio::Landscape,
                false,
            ),
            note_b,
            instance(header_start(), &CARDS_SQUARE, AspectRatio::Square, true),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/feature-image-cards/",
    title: "feature-image-cards",
    category: BlockCategory::Feature,
    rust_source: "crates/docs-site/src/blocks/marketing/feature/feature_image_cards.rs",
    demo_class: "blocks-feature-image-cards",
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
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `feature_image_cards` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。セレクタは
/// `.blocks-feature-image-cards-*` と `[data-blocks-feature-image-cards-*]`
/// のみを用い、他 block や部品の素のセレクタへ影響させない
/// （`feature_alternating_rows`/`blog_grid_image` と同じ名前空間分離）。
const LAYOUT_CSS: &str = "\
.blocks-feature-image-cards-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-feature-image-cards-instance {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-feature-image-cards-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  align-items: center;\n  text-align: center;\n  max-width: 40rem;\n  margin-inline: auto;\n}\n\
.blocks-feature-image-cards-header[data-align=\"start\"] {\n  align-items: flex-start;\n  text-align: left;\n  margin-inline: 0;\n  max-width: none;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-feature-image-cards-lead] {\n  margin: 0;\n}\n\
.blocks-feature-image-cards-grid {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-6);\n}\n\
[data-blocks-feature-image-cards-card] {\n  height: 100%;\n  display: flex;\n  flex-direction: column;\n}\n\
.blocks-feature-image-cards-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-feature-image-cards-desc] {\n  margin: 0;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-feature-image-cards-image] {\n  display: block;\n  width: 100%;\n}\n\
@media (min-width: 48rem) {\n  \
.blocks-feature-image-cards-grid {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n\
}\n\
@media (min-width: 64rem) {\n  \
.blocks-feature-image-cards-grid {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n  \
.blocks-feature-image-cards-grid[data-columns=\"4\"] {\n    grid-template-columns: repeat(4, minmax(0, 1fr));\n  }\n\
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
            "data-scope=\"card\"",
            "data-scope=\"image\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(html.matches("<img").count(), 7);
        assert_eq!(html.matches("data-part=\"cover\"").count(), 7);
        assert!(!html.contains("<form"));
        assert!(!html.contains("<button"));
        assert!(!html.contains("id=\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 画像の src がすべて `dummy_assets` のプレースホルダーであること。
    #[test]
    fn demo_uses_dummy_asset_images_only() {
        let html = render(&demo());
        assert!(html.contains(dummy_assets::PRODUCT_SRC));
        assert!(html.contains(dummy_assets::SCREENSHOT_SRC));
        assert!(html.contains(dummy_assets::BACKGROUND_SRC));
        assert!(html.contains(dummy_assets::LOGO_SRC));
    }

    /// [`LAYOUT_CSS`] が想定する md/lg ブレークポイント・3 列/4 列 grid を
    /// 持つこと。
    #[test]
    fn layout_css_declares_md_lg_grid_rules() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("repeat(3, minmax(0, 1fr))"));
        assert!(LAYOUT_CSS.contains("repeat(4, minmax(0, 1fr))"));
        assert!(LAYOUT_CSS.contains("[data-columns=\"4\"]"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（モジュール doc「CSS フックの選び方」節の Bugbot 教訓の
    /// 固定、`feature_alternating_rows` と同じ回帰）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-feature-image-cards-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-feature-image-cards-layout");
    }
}
