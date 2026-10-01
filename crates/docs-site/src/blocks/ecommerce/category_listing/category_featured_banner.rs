//! `category-featured-banner` block（イシュー #3036。親トラッキング #3024
//! 「Blocks EC」配下、`crate::blocks::ecommerce::category_listing` カテゴリ）。
//!
//! 1 カテゴリだけを大きく扱う横長バナーを、主参照 R0823（全面画像 +
//! 半透明パネル）に集約元 R0608（画像 + テキストの左右分割カード）を
//! 集約した 2 形として合成する。取得手段・ファイル名・内部コンポーネント
//! 識別子は記載しない（対応表 ID のみを記す、`super::super::super::marketing::
//! cta::cta_split_image` と同じ転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `image` / `link` の 4 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、`blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。CTA はリンクとして描く節の理由により
//! `button::button`（`<button type="button">` のみを出力し href を持つ形を
//! 公開していない）は使わないため `parts` へは含めない
//! （`super::super::marketing::footer::footer_cta_columns` と同じ判断）。
//!
//! # 2 形を 1 つの Demo に並記する
//!
//! [`super::super::marketing::cta::cta_split_image`] と同型に、2 形を
//! [`variant_label`] で見出しを付けながら [`demo`] 1 つの中へ縦に並べる。
//!
//! | 形 | 対応 ID | 内容 |
//! |----|---------|------|
//! | A 主参照 | R0823 | 全面画像 + 重ねた半透明パネル |
//! | B 集約元 | R0608 | 画像 + テキストの左右分割 |
//!
//! # A: 絶対配置を使わず `grid-area` の重ね合わせで画像とパネルを重ねる
//!
//! 狭幅（通常フロー）では画像の下にパネルが来るだけでよいため、広幅
//! （`40rem` 以上）でのみ root を `display: grid` にし、画像・パネルの
//! 両方へ同じ `grid-area: 1 / 1` を与えて同一セルへ重ねる
//! （`position: absolute` を使わないため、狭幅での「画像の下へ回す」
//! 挙動が素の通常フローのままで成立する）。
//!
//! # A: パネルの反転配色
//!
//! [`super::super::marketing::testimonial::testimonial_background_image`]
//! と同じ反転ペアを採用する: パネル背景を `--fandhe-color-fg` ベースの
//! 半透明、文字を `--fandhe-color-bg` ベースにする。色リテラル
//! （`#…`/`white`/`black`）は使わず `--fandhe-*` トークンと `color-mix()`
//! のみで表現する。
//!
//! # A: パネル上の CTA・リンクの反転上書き
//!
//! [`link::root`] はどの `LinkVariant`/`ColorPalette` でも独自の `color` を
//! 持つため、CTA（ボタン風リンク）は `[data-scope="link"][data-part="root"]
//! [data-blocks-category-featured-banner-cta]
//! [data-blocks-category-featured-banner-cta-inverted]` へ `background:
//! var(--fandhe-color-bg); color: var(--fandhe-color-fg);` を明示上書きし、
//! パネルの暗い半透明面でもコントラストを確保する（`cta_split_image` の
//! B と同じ判断）。誘導リンクも同様に `[data-scope="link"][data-part="root"]
//! [data-blocks-category-featured-banner-link]` へ `color:
//! var(--fandhe-color-bg);` を明示上書きする。
//!
//! `styled_text::text` の [`TextVariant::Muted`] は `color:
//! var(--fandhe-color-fg-muted)` を明示宣言しており、パネルの
//! `color: var(--fandhe-color-bg)` 継承では上書きできない（Bugbot 指摘
//! #3496: 反転パネル上で eyebrow が暗い文字のまま残りコントラスト不足に
//! なる）。`copy_column` の eyebrow（[`variant_label`] 呼び出し）は
//! `inverted` のとき `data-blocks-category-featured-banner-eyebrow-inverted`
//! を付与し、`[data-scope="text"][data-part="root"]
//! [data-blocks-category-featured-banner-eyebrow-inverted]` へ `color:
//! var(--fandhe-color-bg);` を明示上書きする。
//!
//! # 詳細度: `[data-scope]` を含めた 3 セレクタ構成
//!
//! [`image::image`] の recipe（詳細度 (0,2,0) 程度）・[`link::root`] 自身の
//! `color` 宣言に確実に勝つため、上書きは `data-scope`/`data-part` を含めた
//! 3 セレクタ以上の構成（詳細度 (0,3,0) 以上）で行う（既存 block と同じ
//! 判断軸）。CTA のレイアウト（`padding`/`background`/`border-radius` 等）は
//! `link` 側に競合する既定宣言がないため、`data-blocks-category-featured-
//! banner-cta` 単体（詳細度 (0,1,0)）で足りる（`footer_cta_columns` と
//! 同じ判断）。
//!
//! # link の `href` を固定の外部絶対 URL にする・可視テキストとの整合
//!
//! [`Block::demo`] は `fn() -> Node` のため `base_path` を受け取れない
//! （`cta_split_image` と同じ制約）。CTA・誘導リンクともに実在する GitHub
//! リポジトリへの外部絶対 URL（[`REPO`] とその配下パス）に固定し、
//! `external: true` で `rel="noopener noreferrer"` を付与する。死リンク
//! `href="#"` は使わない。可視テキストは遷移先と矛盾しない文言にする
//! （`footer_cta_columns` と同じ判断）: CTA は repository ルートへの
//! 「GitHub で見る」、誘導リンクは `{REPO}/releases` への「リリースを見る」
//! とし、「カテゴリ一覧」を指すかのような文言（遷移先が実在しない架空の
//! カテゴリ一覧ページであるかのように誤認させる表現）は使わない。
//!
//! # ブレークポイントをリテラルで直書きする理由
//!
//! テーマの breakpoint トークンは `@container`/`@media` 条件式の中では
//! 解決できない（CSS custom property は宣言側でのみ有効）ため、`40rem` を
//! リテラルで直書きする（既存 block と同じ判断）。値は docs サイト上の
//! Demo 枠（`.docs-content` の `max-width: 46rem` から `.blocks-demo` 左右
//! padding `1.5rem` ずつを引いた実測コンテナ幅、上限約 `43rem`）でも
//! 閾値へ到達し、2 列レイアウトが docs サイト上で実際に有効化されるよう
//! 選んでいる（Bugbot 指摘 #3496: 旧 `48rem` では docs サイト上のどの
//! ビューポートでも実測幅がこの上限 `43rem` を超えず到達せず、オーバー
//! レイ・2 列レイアウトが一度も有効化されなかった）。
//!
//! # レイアウト切り替えに `@media` ではなく `@container` を使う理由
//!
//! `@media (min-width: …)` はビューポート幅を判定するため、本 block を
//! サイドバー付きレイアウト等の幅の狭いコンテンツ領域へ埋め込むと、表示
//! 領域が `40rem` 未満でも（ビューポート自体は広いため）形 A が重なり
//! 配置へ戻らず 2 列配置のままになり、狭幅時の挙動が成立しない（codex
//! 指摘 #3496）。[`crate::blocks::ecommerce::category_listing::
//! category_grid_overlay`]/`store_nav_mega_menu` と同じ判断で、Demo の
//! ルート（`.blocks-category-featured-banner-layout`）へ
//! `container-type: inline-size` を宣言し、実際の表示領域幅を基準に
//! 判定する `@container` クエリへ置き換える。
//!
//! # `drop_class_attr` と CSS フックの選び方
//!
//! `heading::heading`/`text::text`/`image::image`/`link::root` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、Demo 固有の
//! スタイルフックは `data-blocks-category-featured-banner-*` 属性で渡し、
//! [`LAYOUT_CSS`] 側も同じ属性セレクタで対応する。素の `div` には
//! `class` がそのまま効くため、レイアウト用の入れ子は従来どおり
//! `.blocks-category-featured-banner-*` クラスセレクタを使う。レイアウト
//! root の class（`blocks-category-featured-banner-layout`）は
//! [`Block::demo_class`]（`blocks-category-featured-banner`）と意図的に
//! 別名にする（既存 block と同じ Bugbot 教訓の回避）。
//!
//! # `<form>` を持たない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。CTA・誘導リンクはいずれも `link::root` の通常のナビゲーション
//! リンクであり、送信処理は持たない。文言はすべて架空のもの（実
//! 企業名・実クレデンシャル・PII を含まない）。画像は
//! [`crate::blocks::dummy_assets`] のビルド時生成プレースホルダー SVG の
//! みを使い、`alt=""`（装飾扱い）で出力する（`data:` URI は使わない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// リンク先の固定外部 URL（モジュール doc「link の `href` を固定の外部
/// 絶対 URL にする・可視テキストとの整合」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 各形の直前に置く短い形ラベル（`styled_text::text` の `Sm`/`Muted`）。
/// `copy_column` の eyebrow としても再利用するため、CSS フック用の
/// `attrs` を受け取る（反転パネル上でのコントラスト上書きに使う、
/// モジュール doc「A: パネル上の CTA・リンクの反転上書き」節参照）。
fn variant_label(label: &'static str, attrs: Vec<(&'static str, &'static str)>) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        attrs,
        vec![text(label)],
    )
}

/// 両形で共通のコピー列（小見出し・見出し・説明・CTA〔ボタン風リンク〕+
/// 誘導リンク）。`inverted` が `true` のとき（形 A のパネル上）は CTA・
/// リンクへ反転配色のフックを付与する。CTA・誘導リンクの可視テキストは
/// 遷移先（GitHub リポジトリ）と矛盾しない固定文言とする（モジュール doc
/// 「link の `href` を固定の外部絶対 URL にする・可視テキストとの整合」
/// 節参照）。
fn copy_column(
    eyebrow: &'static str,
    title: &'static str,
    body: &'static str,
    inverted: bool,
) -> Node {
    let cta_attrs = if inverted {
        vec![
            ("data-blocks-category-featured-banner-cta", ""),
            ("data-blocks-category-featured-banner-cta-inverted", ""),
        ]
    } else {
        vec![("data-blocks-category-featured-banner-cta", "")]
    };
    let link_attrs = if inverted {
        vec![("data-blocks-category-featured-banner-link", "")]
    } else {
        vec![]
    };
    let eyebrow_attrs = if inverted {
        vec![("data-blocks-category-featured-banner-eyebrow-inverted", "")]
    } else {
        vec![]
    };

    div(
        vec![("class", "blocks-category-featured-banner-copy")],
        vec![
            variant_label(eyebrow, eyebrow_attrs),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text(title)],
            ),
            styled_text::text(&TextProps::default(), vec![], vec![text(body)]),
            div(
                vec![("class", "blocks-category-featured-banner-actions")],
                vec![
                    link::root(
                        REPO,
                        &LinkProps {
                            external: true,
                            ..LinkProps::default()
                        },
                        cta_attrs,
                        vec![text("GitHub で見る")],
                    ),
                    link::root(
                        &format!("{REPO}/releases"),
                        &LinkProps {
                            external: true,
                            ..LinkProps::default()
                        },
                        link_attrs,
                        vec![text("リリースを見る")],
                    ),
                ],
            ),
        ],
    )
}

/// 形 A（R0823 主参照）: 全面画像 + 重ねた半透明パネル。広幅では画像の
/// 上にパネルが重なり、狭幅では画像の下にパネルが回る（モジュール doc
/// 「A: 絶対配置を使わず `grid-area` の重ね合わせで画像とパネルを重ねる」
/// 節）。
fn variant_overlay() -> Node {
    let panel = div(
        vec![("class", "blocks-category-featured-banner-panel")],
        vec![copy_column(
            "今季の注目",
            "アウトドア用品",
            "軽量テントから調理器具まで、週末の遠出に必要な一式をまとめました。",
            true,
        )],
    );

    div(
        vec![("class", "blocks-category-featured-banner-overlay")],
        vec![
            image::image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    ..ImageProps::new(dummy_assets::BACKGROUND_SRC, "")
                },
                vec![("data-blocks-category-featured-banner-overlay-image", "")],
            ),
            panel,
        ],
    )
}

/// 形 B（R0608 集約）: 画像 + テキストの左右分割。狭幅では縦積み、広幅
/// （`40rem` 以上）では 2 列グリッドになる。
fn variant_split() -> Node {
    div(
        vec![("class", "blocks-category-featured-banner-split")],
        vec![
            image::image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
                },
                vec![("data-blocks-category-featured-banner-split-image", "")],
            ),
            copy_column(
                "人気急上昇",
                "キッチン家電",
                "毎日の調理をすこし楽にする定番アイテムを集めました。",
                false,
            ),
        ],
    )
}

/// `category-featured-banner` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数（モジュール doc「2 形を 1 つの Demo に並記する」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-category-featured-banner-layout")],
        vec![
            variant_label("全面画像 + 半透明パネル（R0823 主参照）", vec![]),
            variant_overlay(),
            variant_label("画像 + テキストの左右分割（R0608 集約）", vec![]),
            variant_split(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/category-featured-banner/",
    title: "category-featured-banner",
    category: BlockCategory::CategoryListing,
    rust_source:
        "crates/docs-site/src/blocks/ecommerce/category_listing/category_featured_banner.rs",
    demo_class: "blocks-category-featured-banner",
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
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `category_featured_banner` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「CSS の置き場」節、他 block と同型で本ファイル内
/// `const` として [`super::blocks`] から `BLOCK.layout_css` 経由で連結
/// される）。
///
/// セレクタは `.blocks-category-featured-banner-*` と
/// `[data-blocks-category-featured-banner-*]` のみを用い、他 block や
/// 部品の素のセレクタへ影響させない（既存 block と同じ名前空間分離）。
/// 色リテラル（`#fff`/`white`/`black` 等）は使わず、可読性の確保は
/// すべて `--fandhe-color-*` トークンと `color-mix()` で行う。
const LAYOUT_CSS: &str = "\
.blocks-category-featured-banner-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n  container-type: inline-size;\n  container-name: blocks-category-featured-banner;\n}\n\
.blocks-category-featured-banner-copy {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  align-items: start;\n  min-width: 0;\n}\n\
.blocks-category-featured-banner-actions {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-4);\n  align-items: center;\n}\n\
.blocks-category-featured-banner-overlay {\n  display: flex;\n  flex-direction: column;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-category-featured-banner-overlay-image] {\n  display: block;\n  width: 100%;\n  height: 14rem;\n}\n\
.blocks-category-featured-banner-panel {\n  box-sizing: border-box;\n  padding: var(--fandhe-space-6);\n  background: color-mix(in srgb, var(--fandhe-color-fg) 80%, transparent);\n  color: var(--fandhe-color-bg);\n  border-radius: var(--fandhe-radius-lg);\n}\n\
[data-blocks-category-featured-banner-cta] {\n  display: inline-flex;\n  align-items: center;\n  justify-content: center;\n  padding: var(--fandhe-space-3) var(--fandhe-space-6);\n  border-radius: var(--fandhe-radius-md);\n  background: var(--fandhe-color-accent);\n  text-decoration: none;\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n\
[data-scope=\"link\"][data-part=\"root\"][data-blocks-category-featured-banner-cta] {\n  color: var(--fandhe-color-accent-fg);\n}\n\
[data-scope=\"link\"][data-part=\"root\"][data-blocks-category-featured-banner-cta][data-blocks-category-featured-banner-cta-inverted] {\n  background: var(--fandhe-color-bg);\n  color: var(--fandhe-color-fg);\n}\n\
[data-scope=\"link\"][data-part=\"root\"][data-blocks-category-featured-banner-link] {\n  color: var(--fandhe-color-bg);\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-category-featured-banner-eyebrow-inverted] {\n  color: var(--fandhe-color-bg);\n}\n\
.blocks-category-featured-banner-split {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-category-featured-banner-split-image] {\n  display: block;\n  width: 100%;\n  height: 14rem;\n}\n\
@container blocks-category-featured-banner (min-width: 40rem) {\n  \
.blocks-category-featured-banner-overlay {\n    display: grid;\n    min-height: 24rem;\n  }\n  \
[data-scope=\"image\"][data-part=\"root\"][data-blocks-category-featured-banner-overlay-image] {\n    grid-area: 1 / 1;\n    width: 100%;\n    height: 100%;\n  }\n  \
.blocks-category-featured-banner-panel {\n    grid-area: 1 / 1;\n    align-self: end;\n    justify-self: start;\n    max-width: 28rem;\n    margin: var(--fandhe-space-6);\n  }\n  \
.blocks-category-featured-banner-split {\n    display: grid;\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n    gap: var(--fandhe-space-8);\n    align-items: center;\n  }\n  \
[data-scope=\"image\"][data-part=\"root\"][data-blocks-category-featured-banner-split-image] {\n    height: 100%;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, REPO};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていることの単体
    /// 回帰（`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"image\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(html.matches("<img").count(), 2);
        // CTA（ボタン風リンク）・誘導リンクはいずれも実在する GitHub
        // リポジトリへ遷移する（codex 指摘 #3496 の回帰: 送信先のない
        // 無反応な `<button>` を使わない）。
        assert_eq!(
            html.matches("data-blocks-category-featured-banner-cta=\"\"")
                .count(),
            2,
            "both variants should carry the CTA hook on a navigable link"
        );
        assert_eq!(
            html.matches("data-blocks-category-featured-banner-cta-inverted")
                .count(),
            1,
            "only the overlay (inverted) variant should carry the inverted-color hook"
        );
        assert!(html.contains(&format!("href=\"{REPO}\"")));
        assert!(html.contains(&format!("href=\"{REPO}/releases\"")));
        // Bugbot 指摘 #3496 の回帰: 反転パネル上の eyebrow は
        // `TextVariant::Muted` の既定色のままにせず、専用フックで
        // コントラストを上書きする（CSS 側は
        // `layout_css_overrides_inverted_eyebrow_contrast` で固定）。
        assert_eq!(
            html.matches("data-blocks-category-featured-banner-eyebrow-inverted")
                .count(),
            1,
            "only the overlay (inverted) variant's eyebrow should carry the contrast hook"
        );
        for absent in [
            "<form",
            "src=\"data:",
            "href=\"#\"",
            " id=\"",
            "data-scope=\"button\"",
            "type=\"button\"",
        ] {
            assert!(
                !html.contains(absent),
                "demo output should never contain {absent}"
            );
        }
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件・重ね合わせ用の
    /// `grid-area` を持つこと（色リテラルは使わず `color-mix()` +
    /// トークン参照のみであること）。
    #[test]
    fn layout_css_declares_breakpoint_and_overlay_grid_area() {
        assert!(LAYOUT_CSS.contains("grid-area: 1 / 1"));
        assert!(LAYOUT_CSS.contains("color-mix("));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("white"));
        assert!(!LAYOUT_CSS.contains("black"));
    }

    /// codex 指摘 #3496 の回帰: レイアウト切り替えはビューポート幅判定の
    /// `@media` ではなく、Demo 枠自身の実測幅を基準にする `@container` を
    /// 使うこと（狭いコンテンツ領域へ埋め込んでも狭幅時の重なり配置が
    /// 成立するための固定）。
    #[test]
    fn layout_css_uses_container_query_not_viewport_media_query() {
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("container-name: blocks-category-featured-banner;"));
        assert!(
            LAYOUT_CSS.contains("@container blocks-category-featured-banner (min-width: 40rem)")
        );
        assert!(!LAYOUT_CSS.contains("@media"));
    }

    /// Bugbot 指摘 #3496 の回帰: 反転パネル上の eyebrow フックが
    /// `--fandhe-color-bg` へ明示上書きされること（`TextVariant::Muted`
    /// の既定色のまま残らないことの固定）。
    #[test]
    fn layout_css_overrides_inverted_eyebrow_contrast() {
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"text\"][data-part=\"root\"]\
[data-blocks-category-featured-banner-eyebrow-inverted] {\n  color: var(--fandhe-color-bg);\n}"
        ));
    }

    /// ルート grid class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（既存 block と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-category-featured-banner-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-category-featured-banner-layout"
        );
    }
}
