//! `promo-image-tiles` block（イシュー #3081。Ecommerce/Promo カテゴリの
//! 2 件目）。
//!
//! # 出典に関する注記
//!
//! 参照素材（対応表 ID R1197/R1201）はローカル取り込み前の一時ディレクトリ
//! （`_/blocks-intake/`）を出典とするが、本実装セッションからは読めない
//! 状態だった。そのため文言・列配分・オフセット量はイシュー本文の
//! レイアウト仕様のみから設計した独自実装であり、参照元の文言・配色・
//! 装飾は一切持ち込んでいない（`hero_image_tiles` と同じ「着想のみ参照・
//! 実装は独自」の判断軸）。
//!
//! # 使用部品
//!
//! `heading`（見出し）/ `text`（リード文）/ `link`（CTA・外部リンク）/
//! `image`（タイル）の 4 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約）。新しい UI 部品は追加しない。
//! 当初 `base` 形の CTA は `button::button`（遷移先を持たない非操作要素）
//! だったが、クリック操作・遷移先のいずれも持たず実質的に操作不能という
//! レビュー指摘（PR #3528 codex 指摘）を受け、`dark` 形と同じ `link::root`
//! （[`REPO`] 固定リンク）へ統一した。
//!
//! # 2 つの形を縦に並べて差分を示す（R1201 基準形・R1197 暗色帯）
//!
//! Demo は `base`（R1201 相当: 見出し・リード文・CTA リンク 1 個・タイル
//! 7 枚）と `dark`（R1197 相当: 暗色帯・見出し・リード文・外部リンク・
//! タイル 6 枚）の 2 形を [`demo`] 内で縦に積んで並記する。列配分は
//! [`BASE_COLUMNS`]/[`DARK_COLUMNS`] の `const` 配列で持ち、
//! [`collage`] に渡す（`hero_image_tiles` の `COLUMNS` と同じ型、
//! 新しい抽象は作らない）。
//!
//! # タイルは `aria-hidden` + `alt=""` で支援技術から隠す
//!
//! コラージュのラッパ `div.blocks-promo-image-tiles-collage` へ
//! `aria-hidden="true"` を付与し、タイル群（フォーカス可能な要素を
//! 含まない装飾画像のみ）をまとめて支援技術から隠す。各タイルの
//! `image` は `alt=""` で装飾用途を明示する。
//!
//! # sm（640px）未満ではタイルを隠す
//!
//! 既定（モバイル）ではコラージュを `display: none` にし、`sm`
//! （40rem）以上で `display: flex` へ切り替える（イシューの明示要件）。
//! `lg`（64rem）以上でさらに横並びレイアウトへ切り替え、オフセットを
//! 拡大する。
//!
//! # 暗色帯は `cta_centered` の `dark` tone と同型
//!
//! `[data-blocks-promo-image-tiles-tone="dark"]` へ
//! `background: var(--fandhe-color-fg); color: var(--fandhe-color-bg);`
//! を当て、帯の中の `heading`/`text`/`link` には `color: inherit` を
//! 上書きする（`cta_centered` の tone 上書きと同じ判断）。補助テキスト
//! は `opacity: 0.8` 程度に留め、AA コントラストを確保する。
//!
//! リード文（[`copy_dark`]）は `cta_centered`（`lead` 関数 rustdoc 参照）
//! と同じ理由で [`TextVariant::Plain`] を使う。`Muted` は自身の色
//! （`--fandhe-color-fg-muted`）を持つため、上記 `color: inherit` 上書き
//! に対して詳細度で競合し得、`dark` 面でコントラストが崩れる
//! （Cursor Bugbot 指摘、PR #3528）。`base` 形（[`copy_base`]）は通常の
//! 前景色のままのため `Muted` を維持する。
//!
//! # リンク先の方針
//!
//! `footer_link_columns` 等の前例と同じく、外部の絶対 URL
//! `https://github.com/Fandhe-AI/fandhe-frontend` を固定のリンク先として
//! 使う（`external: true` で `rel="noopener noreferrer"` を付与）。
//! `href="#"` の死リンクは使わない。CTA 文言は `category_featured_banner`
//! と同じ判断軸で「GitHub で見る」とし、実際の遷移先と一致させる
//! （「特集を見る」「もっと見る」のように存在しない特集ページへ案内する
//! 文言は使わない。レビュー指摘、PR #3528 codex 指摘）。
//!
//! # `drop_class_attr` を踏まえた CSS フックの選び方
//!
//! `heading::heading`/`styled_text::text`/`link::root`/`image::image` は
//! いずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を
//! 黙って除去する契約を持つため、本 block 固有のフックは
//! `data-blocks-promo-image-tiles-*` 属性で渡す。素の `div` には
//! `class` がそのまま効くため `.blocks-promo-image-tiles-*`
//! クラスセレクタを使う。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>`
//! を出力しない。CTA は `link::root` のみで構成し `<button>` は使わない。
//! 文言はすべて架空のものであり、実企業名・実サービス
//! 名・実クレデンシャル・PII を含まない。画像は
//! [`crate::blocks::dummy_assets::PRODUCT_SRC`]（ビルド時生成の商品
//! プレースホルダー SVG）を使う。`id`・`aria-labelledby` は出力しない
//! （`demo_output_has_no_dangling_aria_references_or_duplicate_ids` 契約）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{
    self as styled_heading, HeadingLevel, HeadingProps, HeadingSize,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 固定の外部リンク先（モジュール doc「リンク先の方針」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// `base`（R1201 相当）の列ごとのタイル枚数配分（合計 7 枚）。
const BASE_COLUMNS: [usize; 3] = [2, 3, 2];

/// `dark`（R1197 相当）の列ごとのタイル枚数配分（合計 6 枚）。
const DARK_COLUMNS: [usize; 3] = [2, 2, 2];

/// `base` 形のコピー列（見出し → リード文 → CTA リンク 1 個）。
fn copy_base() -> Node {
    div(
        vec![("class", "blocks-promo-image-tiles-copy")],
        vec![
            styled_heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![("data-blocks-promo-image-tiles-title", "")],
                vec![text("季節の新作、まとめてチェック")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-promo-image-tiles-lead", "")],
                vec![text(
                    "入荷したばかりのアイテムをタイルでまとめてご紹介します。",
                )],
            ),
            link::root(
                REPO,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![("data-blocks-promo-image-tiles-cta", "")],
                vec![text("GitHub で見る")],
            ),
        ],
    )
}

/// `dark` 形のコピー列（見出し → リード文 → 外部リンク）。
fn copy_dark() -> Node {
    div(
        vec![("class", "blocks-promo-image-tiles-copy")],
        vec![
            styled_heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![("data-blocks-promo-image-tiles-title", "")],
                vec![text("限定アイテム、数量限り")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    variant: TextVariant::Plain,
                    ..TextProps::default()
                },
                vec![("data-blocks-promo-image-tiles-lead", "")],
                vec![text(
                    "今季限定の生産本数で仕立てたアイテムを取り扱っています。",
                )],
            ),
            link::root(
                REPO,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![("data-blocks-promo-image-tiles-link", "")],
                vec![text("GitHub で見る")],
            ),
        ],
    )
}

/// タイル 1 枚分（角丸・影はラッパ `div` 側で付ける、`hero_image_tiles`
/// と同型）。
fn tile() -> Node {
    div(
        vec![("class", "blocks-promo-image-tiles-tile")],
        vec![image::image(
            &ImageProps {
                aspect_ratio: AspectRatio::Portrait,
                ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
            },
            vec![("data-blocks-promo-image-tiles-image", "")],
        )],
    )
}

/// 列 1 本分（`count` 枚のタイルを縦に並べる）。
fn column(index: usize, count: usize) -> Node {
    let column_number = (index + 1).to_string();
    div(
        vec![
            ("class", "blocks-promo-image-tiles-column"),
            (
                "data-blocks-promo-image-tiles-column",
                column_number.as_str(),
            ),
        ],
        (0..count).map(|_| tile()).collect(),
    )
}

/// タイルのコラージュ（3 列、枚数配分は `columns`）。`aria-hidden` で
/// 支援技術から隠す（モジュール doc「タイルは `aria-hidden` + `alt=""`」
/// 節）。
fn collage(columns: &[usize]) -> Node {
    div(
        vec![
            ("class", "blocks-promo-image-tiles-collage"),
            ("aria-hidden", "true"),
        ],
        columns
            .iter()
            .enumerate()
            .map(|(i, &count)| column(i, count))
            .collect(),
    )
}

/// `base` 形（R1201 相当）1 件分。
fn instance_base() -> Node {
    div(
        vec![
            ("class", "blocks-promo-image-tiles-instance"),
            ("data-blocks-promo-image-tiles-tone", "base"),
        ],
        vec![copy_base(), collage(&BASE_COLUMNS)],
    )
}

/// `dark` 形（R1197 相当）1 件分。暗色帯は `data-blocks-promo-image-
/// tiles-tone="dark"` で判定する（モジュール doc「暗色帯」節）。
fn instance_dark() -> Node {
    div(
        vec![
            ("class", "blocks-promo-image-tiles-instance"),
            ("data-blocks-promo-image-tiles-tone", "dark"),
        ],
        vec![copy_dark(), collage(&DARK_COLUMNS)],
    )
}

/// `promo-image-tiles` の Demo 本体。`base`/`dark` の 2 形を縦に積んで
/// 並記する（モジュール doc「2 つの形を縦に並べて差分を示す」節）。
/// 呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-promo-image-tiles-stack")],
        vec![instance_base(), instance_dark()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/promo-image-tiles/",
    title: "promo-image-tiles",
    category: BlockCategory::Promo,
    rust_source: "crates/docs-site/src/blocks/ecommerce/promo/promo_image_tiles.rs",
    demo_class: "blocks-promo-image-tiles",
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
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `promo_image_tiles` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。セレクタは `.blocks-promo-image-
/// tiles-*`/`[data-blocks-promo-image-tiles-*]` と、それらで絞り込んだ
/// `[data-scope="image"/"heading"/"text"/"link"]` のみを用いる。
const LAYOUT_CSS: &str = "\
.blocks-promo-image-tiles-stack {\n  display: grid;\n  gap: var(--fandhe-space-12);\n}\n\
.blocks-promo-image-tiles-instance {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-promo-image-tiles-copy {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-start;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-promo-image-tiles-collage {\n  display: none;\n}\n\
.blocks-promo-image-tiles-column {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  flex: 1 1 0;\n  min-width: 0;\n}\n\
.blocks-promo-image-tiles-tile {\n  border-radius: var(--fandhe-radius-xl);\n  box-shadow: var(--fandhe-shadow-lg);\n  overflow: hidden;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-promo-image-tiles-image] {\n  display: block;\n  width: 100%;\n}\n\
[data-blocks-promo-image-tiles-tone=\"dark\"] {\n  background: var(--fandhe-color-fg);\n  color: var(--fandhe-color-bg);\n  padding: var(--fandhe-space-8);\n  border-radius: var(--fandhe-radius-lg);\n}\n\
[data-blocks-promo-image-tiles-tone=\"dark\"] [data-scope=\"text\"],\n[data-blocks-promo-image-tiles-tone=\"dark\"] [data-scope=\"heading\"],\n[data-blocks-promo-image-tiles-tone=\"dark\"] [data-scope=\"link\"] {\n  color: inherit;\n}\n\
[data-blocks-promo-image-tiles-tone=\"dark\"] [data-scope=\"link\"][data-part=\"root\"]:hover {\n  color: inherit;\n}\n\
[data-blocks-promo-image-tiles-tone=\"dark\"] [data-blocks-promo-image-tiles-lead] {\n  opacity: 0.8;\n}\n\
@media (min-width: 40rem) {\n  \
.blocks-promo-image-tiles-collage {\n    display: flex;\n    gap: var(--fandhe-space-4);\n    overflow: hidden;\n    padding-inline: var(--fandhe-space-3);\n  }\n  \
.blocks-promo-image-tiles-column[data-blocks-promo-image-tiles-column=\"1\"] {\n    padding-top: var(--fandhe-space-6);\n  }\n  \
.blocks-promo-image-tiles-column[data-blocks-promo-image-tiles-column=\"3\"] {\n    padding-top: var(--fandhe-space-6);\n  }\n\
}\n\
@media (min-width: 64rem) {\n  \
.blocks-promo-image-tiles-instance {\n    flex-direction: row;\n    align-items: center;\n    gap: var(--fandhe-space-8);\n  }\n  \
.blocks-promo-image-tiles-copy {\n    flex: 1 1 0;\n    max-width: 32rem;\n  }\n  \
.blocks-promo-image-tiles-collage {\n    flex: 1 1 0;\n  }\n  \
.blocks-promo-image-tiles-column[data-blocks-promo-image-tiles-column=\"1\"] {\n    padding-top: var(--fandhe-space-10);\n  }\n  \
.blocks-promo-image-tiles-column[data-blocks-promo-image-tiles-column=\"3\"] {\n    padding-top: var(--fandhe-space-10);\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, BASE_COLUMNS, DARK_COLUMNS, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていることの単体
    /// 回帰（`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_tones() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"link\"",
            "data-scope=\"image\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        let total_tiles: usize =
            BASE_COLUMNS.iter().sum::<usize>() + DARK_COLUMNS.iter().sum::<usize>();
        assert_eq!(total_tiles, 13);
        assert_eq!(html.matches("<img").count(), total_tiles);
        assert_eq!(html.matches("alt=\"\"").count(), total_tiles);
        assert_eq!(html.matches("aria-hidden=\"true\"").count(), 2);
        assert!(html.contains("data-blocks-promo-image-tiles-tone=\"base\""));
        assert!(html.contains("data-blocks-promo-image-tiles-tone=\"dark\""));
        // 両 CTA（base/dark）が `REPO` への実リンクとして操作可能であること
        // （Codex 指摘 PR #3528: button::button は遷移先を持たず操作不能
        // だった）。
        assert_eq!(html.matches("rel=\"noopener noreferrer\"").count(), 2);
        for absent in ["<form", "<script", "src=\"data:", "id=\"", "href=\"#\""] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// `demo()` が決定的（呼び出しごとに同じ `Node`）であること。
    #[test]
    fn demo_is_deterministic() {
        assert_eq!(render(&demo()), render(&demo()));
    }

    /// [`LAYOUT_CSS`] が sm/lg breakpoint・モバイル非表示・暗色帯トークン・
    /// 角丸/影トークンを持つこと。
    #[test]
    fn layout_css_declares_breakpoints_and_tokens() {
        assert_eq!(
            fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Sm.min_width(),
            "640px"
        );
        assert_eq!(
            fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg.min_width(),
            "1024px"
        );
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("display: none"));
        assert!(LAYOUT_CSS.contains("var(--fandhe-color-fg)"));
        assert!(LAYOUT_CSS.contains("var(--fandhe-radius-xl)"));
        assert!(LAYOUT_CSS.contains("var(--fandhe-shadow-lg)"));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("white"));
        assert!(!LAYOUT_CSS.contains("animation"));
        assert!(!LAYOUT_CSS.contains("transition"));
    }

    /// Bugbot 指摘（PR #3528）回帰: dark 帯の `color: inherit` は属性
    /// セレクタ 2 つのみだと link の `:hover` 規則（`[data-scope="link"]
    /// [data-part="root"]:hover`、属性 2 つ + 疑似クラス 1 つ）に詳細度で
    /// 負け、反転配色帯で CTA リンクが accent-emphasized 色へ戻って
    /// コントラストを失っていた。`[data-part="root"]` を加えた専用
    /// `:hover` 上書き（属性 3 つ + 疑似クラス 1 つ）で明示的に勝たせる。
    #[test]
    fn dark_tone_link_hover_override_outranks_component_hover() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-promo-image-tiles-tone=\"dark\"] [data-scope=\"link\"][data-part=\"root\"]:hover {\n  color: inherit;\n}"
        ));
    }
}
