//! `contact-image-info` block（イシュー #2829。親トラッキング #2807
//! 「Blocks マーケティング B」、ルート #2730「Blocks 目的別パーツ拡充
//! ツリー」配下）。角丸画像 + タグライン・見出し・説明文・連絡先リンク
//! 3 件（電話・メール・住所）の 2 列レイアウトを、主参照 R0444 の構造
//! （領域配置・部品構成）だけを取り込んで合成する。取得手段・ファイル名・
//! 出典名・内部識別子は記載しない（`super::super::cta::cta_split_image`
//! と同じライセンス上の転記制限、対応表 ID のみを記す）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `image` / `icon` / `link` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言・連絡先はすべて架空のもの（実企業名・実クレデンシャル・
//! PII を含まない）。画像は [`crate::blocks::dummy_assets`] のビルド時生成
//! プレースホルダー SVG のみを使い、`alt=""`（装飾扱い）で出力する
//! （`data:` URI は使わない）。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、見出しは `HeadingLevel::H3`
//! を使う（既存 block と同じ判断）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と `fandhe_frontend_core::text`
//! が同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # `drop_class_attr` と CSS フックの選び方
//!
//! `heading::heading`/`text::text`/`image::image`/`icon::icon`/`link::root`
//! はいずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を
//! 黙って除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-contact-image-info-*` 属性で渡し、[`LAYOUT_CSS`] 側も同じ
//! 属性セレクタで対応する。素の `div`/`ul`/`li` には `class` がそのまま
//! 効くため、レイアウトは従来どおり `.blocks-contact-image-info-*` クラス
//! セレクタを使う。レイアウト root の class（`blocks-contact-image-info-
//! layout`）は [`Block::demo_class`]（`blocks-contact-image-info`）と
//! 意図的に別名にする（既存 block と同じ Bugbot 教訓の回避）。
//!
//! # ブレークポイントをリテラルで直書きする理由
//!
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! （CSS custom property は宣言側でのみ有効）ため、
//! [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md`]（768px =
//! 48rem）と一致するリテラル値を [`LAYOUT_CSS`] へ直書きする（既存 block と
//! 同じ判断）。イシュー本文の「md 幅以上では 2 列」要件に対応する。
//!
//! # 連絡先リンクの `href` 方針
//!
//! 電話・メールは実プロトコルリンク（`tel:`/`mailto:`）として出力する
//! （`fandhe_frontend_core::url::is_safe_url` は両スキームを許可する）。
//! 番号・メールアドレスはいずれも架空値: 電話は北米の架空番号用予約域
//! `555-01xx`（RFC 3092 の `example.com` と同種の予約域）、メールは
//! RFC 2606 の予約ドメイン `example.com` を使う。住所リンクは実在の地図
//! サービスへは接続せず、[`super::super::cta::cta_split_image::REPO`] と
//! 同型の固定外部 URL（本リポジトリ）へ `external: true` で遷移させる
//! （死リンク `href="#"` は使わない）。電話・メールは `external: false`
//! （ダイヤラー/メーラー起動に `target="_blank"` を付けない）。
//!
//! # アイコンは自作の単純図形
//!
//! 電話・メール・住所の 3 アイコンは、[`super::super::cta::cta_split_image::
//! geo_icon`] と同型の私有ヘルパ（`path` に `fill="none"`・
//! `stroke="currentColor"`・`stroke-width="2"`・丸い cap/join）を本ファイル
//! 内へ複製する（private 関数のため共有できない）。形は単純な図形のみで、
//! 参照元や外部アイコンセットのパスデータは複製しない。
//!
//! # `id` 属性を出力しない
//!
//! `crate::blocks` モジュール doc「宙に浮いた ARIA 参照・id 重複を避ける」
//! 契約に従い、本 Demo は `id` 属性を一切使わない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 住所リンクの固定外部 URL（モジュール doc「連絡先リンクの `href` 方針」
/// 節参照）。実在の地図サービスへは接続しない。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 自作の幾何アイコン（線画。モジュール doc「アイコンは自作の単純図形」
/// 節参照）。`path` へ `fill="none"` + `stroke="currentColor"` を明示し、
/// `icon` の `<svg>` 側が固定で持つ `fill="currentColor"`（塗り面）を
/// 上書きして線画（ストローク）として描画する。
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

/// 電話アイコン（角丸長方形の端末 + 上端の短い線）。
fn phone_icon() -> Node {
    geo_icon("M6 4h6v2H8v12h4v2H6z M9 6h1 M4 10c0 6 4 10 10 10")
}

/// メールアイコン（封筒 + V 字の折り返し線）。
fn mail_icon() -> Node {
    geo_icon("M3 6h18v12H3z M3 7l9 6 9-6")
}

/// 住所アイコン（ピン: 円 + 下向きの雫形）。
fn address_icon() -> Node {
    geo_icon(
        "M12 21s-7-6.5-7-11a7 7 0 0 1 14 0c0 4.5-7 11-7 11z M12 12a2 2 0 1 0 0-4 2 2 0 0 0 0 4z",
    )
}

/// 画像領域（角丸の正方形画像。モジュール doc「使用部品」節）。
fn media() -> Node {
    div(
        vec![("class", "blocks-contact-image-info-media")],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                aspect_ratio: AspectRatio::Square,
                shape: ImageShape::Rounded,
                ..ImageProps::new(dummy_assets::BACKGROUND_SRC, "")
            },
            vec![("data-blocks-contact-image-info-image", "")],
        )],
    )
}

/// 連絡先リンク 1 行（モジュール doc「連絡先リンクの `href` 方針」節）。
fn contact_link(href: &'static str, external: bool, glyph: Node, label: &'static str) -> Node {
    li(
        vec![],
        vec![link::root(
            href,
            &LinkProps {
                external,
                ..LinkProps::default()
            },
            vec![("data-blocks-contact-image-info-item", "")],
            vec![glyph, text(label)],
        )],
    )
}

/// テキスト領域（タグライン + 見出し + 説明文 + 連絡先リンク 3 件）。
fn info() -> Node {
    div(
        vec![("class", "blocks-contact-image-info-info")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-contact-image-info-tagline", "")],
                vec![text("お問い合わせ")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("お気軽にご連絡ください")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("平日 2 営業日以内にご返信します。")],
            ),
            ul(
                vec![("class", "blocks-contact-image-info-list")],
                vec![
                    contact_link("tel:+15550100", false, phone_icon(), "+1 555-0100"),
                    contact_link(
                        "mailto:hello@example.com",
                        false,
                        mail_icon(),
                        "hello@example.com",
                    ),
                    contact_link(REPO, true, address_icon(), "架空通り 1-2-3、サンプル区"),
                ],
            ),
        ],
    )
}

/// `contact-image-info` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。DOM 順は画像 → テキスト領域（狭幅の 1 列表示で画像が上に
/// 来るようにするため）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-contact-image-info-layout")],
        vec![media(), info()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/contact-image-info/",
    title: "contact-image-info",
    category: BlockCategory::Contact,
    rust_source: "crates/docs-site/src/blocks/marketing/contact/contact_image_info.rs",
    demo_class: "blocks-contact-image-info",
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
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `contact_image_info` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` では
/// なく本ファイル内 private 定数として `super::stylesheet` 経由の
/// `push_css` で連結される）。
///
/// セレクタは `.blocks-contact-image-info-*` と
/// `[data-blocks-contact-image-info-*]` のみを用い、他 block や部品の素の
/// セレクタへ影響させない（既存 block と同じ名前空間分離）。
const LAYOUT_CSS: &str = "\
.blocks-contact-image-info-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-contact-image-info-image] {\n  display: block;\n  width: 100%;\n}\n\
.blocks-contact-image-info-info {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  min-width: 0;\n}\n\
.blocks-contact-image-info-list {\n  list-style: none;\n  margin: 0;\n  padding: 0;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"link\"][data-part=\"root\"][data-blocks-contact-image-info-item] {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
@media (min-width: 48rem) {\n  \
.blocks-contact-image-info-layout {\n    display: grid;\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n    gap: var(--fandhe-space-8);\n    align-items: center;\n  }\n\
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
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"image\"",
            "data-scope=\"icon\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(html.matches("<img").count(), 1);
        assert!(html.contains("href=\"tel:"));
        assert!(html.contains("href=\"mailto:"));
        assert_eq!(
            html.matches("data-blocks-contact-image-info-item").count(),
            3
        );
        for absent in [
            "<form",
            "href=\"#\"",
            "src=\"data:",
            " id=\"",
            "type=\"submit\"",
        ] {
            assert!(
                !html.contains(absent),
                "demo output should never contain {absent}"
            );
        }
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件・2 列 grid を持つこと。
    #[test]
    fn layout_css_declares_breakpoint_and_grid() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("repeat(2, minmax(0, 1fr))"));
        assert!(!LAYOUT_CSS.contains('<'));
    }

    /// ルート grid class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（既存 block と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-contact-image-info-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-contact-image-info-layout");
    }

    /// dummy_assets の src を参照していること。
    #[test]
    fn media_uses_dummy_asset_src() {
        let html = render(&demo());
        assert!(html.contains(dummy_assets::BACKGROUND_SRC));
    }
}
