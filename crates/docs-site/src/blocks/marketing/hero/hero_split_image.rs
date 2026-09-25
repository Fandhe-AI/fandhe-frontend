//! `hero-split-image` block（イシュー #2791。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー」配下、Phase 1（マーケティング A）に
//! 属する）。左にテキスト（badge/heading/text/button 群）・右に画像を置く、
//! 最も基本的な分割ヒーローを、主参照 R0129（+ R1010 の 7:5 分割）に
//! R0535/R0544/R0536/R0548/R0125/R0543 を構造として集約した 4 形として
//! 合成する。取得手段・ファイル名・内部コンポーネント識別子は記載しない
//! （`super::hero_parallax_layers` 等と同じライセンス上の転記制限、対応表
//! ID のみを記す）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `button` / `image` / `avatar` / `icon` の
//! 7 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 4 形を 1 つの Demo に並記する
//!
//! [`super::super::cta::cta_split_image`] と同型に、4 形を [`variant_label`]
//! で見出しを付けながら [`demo`] 1 つの中へ縦に並べる。
//!
//! | 形 | 対応 ID | 内容 |
//! |----|---------|------|
//! | A 基準形 | R0129/R1010 | badge → 見出し → リード文 → CTA 2 個（コピー列）+ 正方形画像 |
//! | B 箇条書き | R0535/R0544 | リード文と CTA の間にチェック付き箇条書き 3 件 + 縦長画像 |
//! | C 社会的証明 | R0536 | A に加え CTA 下へ重なりアバター 3 枚 + `+N` + 短い補足 |
//! | D 反転 | R0548/R0125/R0543 | `lg` 以上で画像を左へ（DOM 順は不変、`grid-column` のみで反転） |
//!
//! # DOM 順を固定し、`lg` 以上でのみ画像位置を反転する
//!
//! 4 形とも DOM 順は「コピー列 → 画像」で固定する（`lg` 未満の 1 列表示で
//! 画像がテキストの後に来る要件を満たすため）。形 D の画像左配置は
//! `[data-blocks-hero-split-image-reverse]` 属性を row へ付与し、`lg` 以上の
//! `grid-column` 指定のみで見た目上の左右を入れ替える（絶対配置・DOM 順の
//! 並べ替えは行わない）。
//!
//! # 詳細度: `[data-scope]` を含めた 3 セレクタ構成
//!
//! [`super::super::cta::cta_split_image`] と同じ判断軸で、`image::image`
//! の recipe に確実に勝つため、上書きは `[data-scope="image"][data-part=
//! "root"][data-blocks-hero-split-image-*]` のように `data-scope`/`data-part`
//! を含めた 3 セレクタ構成（詳細度 (0,3,0) 以上）で行う。
//!
//! # 箇条書きのアイコン
//!
//! チェックマークは [`super::super::cta::cta_split_image::geo_icon`] と
//! 同型の自作幾何アイコン（lucide 等の著作物を複製しない単純な折れ線）を
//! 本ファイル内へ複製する（クレート内の private 関数のため共有できない）。
//!
//! # 社会的証明: 重なりアバター + `+N`
//!
//! `super::super::banner::banner_announcement_pill::avatar_stack` と同型に
//! `avatar::root` の `stacked: true` + `avatar::group` で重なり表示する。
//! 3 枚の画像アバターに加え、`+N` の残数表示は画像を持たない `fallback`
//! のみの `avatar::root` で表現する（`<img>` を増やさないため）。
//!
//! # `id`/`aria-labelledby` を出力しない・複数インスタンス
//!
//! 形 A/B/C/D の 4 インスタンスを持つため、`id` 属性は一切使わない
//! （`demo_output_has_no_dangling_aria_references_or_duplicate_ids` 契約、
//! `crate::blocks` モジュール doc 参照）。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、全ての見出しは
//! `HeadingLevel::H3` を使う。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と `fandhe_frontend_core::text`
//! が同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # ブレークポイントをリテラルで直書きする理由
//!
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! （CSS custom property は宣言側でのみ有効）ため、
//! [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`]（1024px =
//! 64rem）と一致するリテラル値を [`LAYOUT_CSS`] へ直書きする（既存 block と
//! 同じ判断）。
//!
//! # `drop_class_attr` と CSS フックの選び方
//!
//! `badge::badge`/`heading::heading`/`text::text`/`button::button`/
//! `image::image`/`avatar::root`/`icon::icon` はいずれも `drop_class_attr`
//! により呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、
//! Demo 固有のスタイルフックは `data-blocks-hero-split-image-*` 属性で渡し、
//! [`LAYOUT_CSS`] 側も同じ属性セレクタで対応する。素の `div`/`ul`/`li` には
//! `class` がそのまま効くため、レイアウト骨格は従来どおり
//! `.blocks-hero-split-image-*` クラスセレクタを使う。レイアウト root の
//! class（`blocks-hero-split-image-layout`）は [`Block::demo_class`]
//! （`blocks-hero-split-image`）と意図的に別名にする（既存 block と同じ
//! Bugbot 教訓の回避）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。ボタンは `button::button` の既定 `type="button"` のまま送信先を
//! 持たず、値は一切送信されない。文言はすべて架空のもの（実企業名・実
//! クレデンシャル・PII を含まない）。画像は
//! [`crate::blocks::dummy_assets`] のビルド時生成プレースホルダー SVG のみ
//! を使い、`alt=""`（装飾扱い）で出力する（`data:` URI は使わない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 自作の幾何アイコン（線画。モジュール doc「箇条書きのアイコン」節
/// 参照）。`path` へ `fill="none"` + `stroke="currentColor"` を明示し、
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

/// チェックマークの幾何アイコン。
fn check_icon() -> Node {
    geo_icon("M5 13l4 4L19 7")
}

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

/// CTA ボタン 2 個（Solid + Outline）。
fn actions(primary: &'static str, secondary: &'static str) -> Node {
    div(
        vec![("class", "blocks-hero-split-image-actions")],
        vec![
            button::button(
                &ButtonProps {
                    size: Size::Lg,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text(primary)],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    size: Size::Lg,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text(secondary)],
            ),
        ],
    )
}

/// 右列（または反転時は左列）に置く画像。`data-blocks-hero-split-image-
/// image` フックへ `aspect_ratio`/`shape` を差し替えて渡す（形ごとに
/// 正方形/縦長/全面高さを使い分ける、モジュール doc §「詳細度」節参照）。
fn media(aspect_ratio: AspectRatio, shape: ImageShape) -> Node {
    image::image(
        &ImageProps {
            fit: ImageFit::Cover,
            aspect_ratio,
            shape,
            ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
        },
        vec![("data-blocks-hero-split-image-image", "")],
    )
}

/// 形 A（R0129 基準形 + R1010 の 7:5 分割）: badge → 見出し → リード文 →
/// CTA 2 個（コピー列）+ 正方形画像。
fn variant_basic() -> Node {
    let copy = div(
        vec![("class", "blocks-hero-split-image-copy")],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text("新規リリース")]),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("公開までの導線を、迷わず一本化する")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "テンプレートと雛形を組み合わせ、初期構築から公開までの手間を大きく減らします。",
                )],
            ),
            actions("今すぐ試す", "詳しく見る"),
        ],
    );
    div(
        vec![("class", "blocks-hero-split-image-row")],
        vec![copy, media(AspectRatio::Square, ImageShape::Rounded)],
    )
}

/// チェック付き項目 1 件分（チェックマーク + 短い文言）。
fn checklist_item(label: &'static str) -> Node {
    li(
        vec![("class", "blocks-hero-split-image-checklist-item")],
        vec![check_icon(), text(label)],
    )
}

/// チェック付き項目 3 件（架空文言）。
const CHECKLIST_ITEMS: [&str; 3] = [
    "既定エスケープ済みの HTML 出力",
    "外部依存ゼロの描画コア",
    "単一実行ファイルでの配布",
];

/// 形 B（R0535/R0544 集約）: リード文と CTA の間にチェック付き箇条書き 3
/// 件 + 縦長画像。
fn variant_checklist() -> Node {
    let copy = div(
        vec![("class", "blocks-hero-split-image-copy")],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text("導入ガイド")]),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("既存のチームでもそのまま採用できる")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "導入前に確認したいポイントを、あらかじめまとめました。",
                )],
            ),
            ul(
                vec![("class", "blocks-hero-split-image-checklist")],
                CHECKLIST_ITEMS
                    .iter()
                    .map(|label| checklist_item(label))
                    .collect(),
            ),
            actions("導入を始める", "資料を見る"),
        ],
    );
    div(
        vec![("class", "blocks-hero-split-image-row")],
        vec![copy, media(AspectRatio::Portrait, ImageShape::Rounded)],
    )
}

/// 重なりアバター 3 枚（社会的証明の装飾）。[`dummy_assets::AVATAR_SRC`]
/// （ビルド時生成 SVG、`data:` URI ではない）を画像に使う
/// （`super::banner_announcement_pill::avatar_stack` と同型）。
fn avatar_trio() -> Vec<Node> {
    ["HF", "EV", "KB"]
        .iter()
        .map(|initials| {
            avatar::root(
                &AvatarProps {
                    size: Size::Sm,
                    stacked: true,
                    ..AvatarProps::default()
                },
                vec![("data-blocks-hero-split-image-avatar", "")],
                vec![
                    avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, "", vec![]),
                    avatar::fallback(ImageStatus::Loaded, vec![], vec![text(*initials)]),
                ],
            )
        })
        .collect()
}

/// `+N` の残数表示（画像を持たない fallback のみの avatar、`<img>` を
/// 増やさないための選択、モジュール doc「社会的証明」節参照）。
fn avatar_overflow(label: &'static str) -> Node {
    avatar::root(
        &AvatarProps {
            size: Size::Sm,
            stacked: true,
            ..AvatarProps::default()
        },
        vec![("data-blocks-hero-split-image-avatar-overflow", "")],
        vec![avatar::fallback(
            ImageStatus::Error,
            vec![],
            vec![text(label)],
        )],
    )
}

/// 形 C（R0536）: A に加え CTA 下へ重なりアバター 3 枚 + `+N` + 短い補足
/// テキスト（社会的証明）。
fn variant_social_proof() -> Node {
    let mut avatars = avatar_trio();
    avatars.push(avatar_overflow("+12"));
    let proof = div(
        vec![("class", "blocks-hero-split-image-proof")],
        vec![
            avatar::group(vec![("aria-hidden", "true")], avatars),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("すでに 500 チーム以上が利用しています")],
            ),
        ],
    );
    let copy = div(
        vec![("class", "blocks-hero-split-image-copy")],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text("導入実績")]),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("チームの合言葉を、そのまま画面へ")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "配色トークンを差し替えるだけで、複数ブランドの画面を作り分けられます。",
                )],
            ),
            actions("始める", "事例を見る"),
            proof,
        ],
    );
    div(
        vec![("class", "blocks-hero-split-image-row")],
        vec![copy, media(AspectRatio::Square, ImageShape::Rounded)],
    )
}

/// 形 D（R0548/R0125/R0543 集約）: `lg` 以上で画像を左へ反転（DOM 順は
/// 「コピー列 → 画像」のまま、`[data-blocks-hero-split-image-reverse]` +
/// `grid-column` のみで見た目を入れ替える、モジュール doc「DOM 順を固定し」
/// 節参照）。画像はセルいっぱいに広がる（全面画像）。
fn variant_reverse_full() -> Node {
    let copy = div(
        vec![("class", "blocks-hero-split-image-copy")],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text("新しい体験")]),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("画像を主役にした、もう一つの見せ方")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "同じ構成のまま、画像とテキストの左右だけを入れ替えて印象を変えられます。",
                )],
            ),
            actions("詳しく見る", "デモを試す"),
        ],
    );
    let media = div(
        vec![("class", "blocks-hero-split-image-reverse-media")],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
            },
            vec![("data-blocks-hero-split-image-reverse-image", "")],
        )],
    );
    div(
        vec![
            ("class", "blocks-hero-split-image-row"),
            ("data-blocks-hero-split-image-reverse", ""),
        ],
        vec![copy, media],
    )
}

/// `hero-split-image` の Demo 本体。呼び出しごとに同一の `Node` を返す純
/// 関数（モジュール doc「4 形を 1 つの Demo に並記する」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-hero-split-image-layout")],
        vec![
            variant_label("基準形（R0129/R1010）"),
            variant_basic(),
            variant_label("チェック付き箇条書き + 縦長画像（R0535/R0544 集約）"),
            variant_checklist(),
            variant_label("社会的証明（R0536）"),
            variant_social_proof(),
            variant_label("反転 + 全面画像（R0548/R0125/R0543 集約）"),
            variant_reverse_full(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/hero-split-image/",
    title: "hero-split-image",
    category: BlockCategory::Hero,
    rust_source: "crates/docs-site/src/blocks/marketing/hero/hero_split_image.rs",
    demo_class: "blocks-hero-split-image",
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
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `hero_split_image` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` ではなく
/// 本ファイル内 private 定数として `super::stylesheet` 経由の `push_css`
/// で連結される）。
///
/// セレクタは `.blocks-hero-split-image-*` と
/// `[data-blocks-hero-split-image-*]` のみを用い、他 block や部品の素の
/// セレクタへ影響させない（既存 block と同じ名前空間分離）。
const LAYOUT_CSS: &str = "\
.blocks-hero-split-image-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-hero-split-image-row {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-hero-split-image-copy {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  align-items: start;\n  min-width: 0;\n}\n\
.blocks-hero-split-image-actions {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-hero-split-image-image] {\n  display: block;\n  width: 100%;\n}\n\
.blocks-hero-split-image-checklist {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  margin: 0;\n  padding: 0;\n  list-style: none;\n}\n\
.blocks-hero-split-image-checklist-item {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-hero-split-image-proof {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-hero-split-image-reverse-media {\n  height: 14rem;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-hero-split-image-reverse-image] {\n  display: block;\n  width: 100%;\n  height: 100%;\n}\n\
@media (min-width: 64rem) {\n  \
.blocks-hero-split-image-row {\n    display: grid;\n    grid-template-columns: repeat(12, minmax(0, 1fr));\n    gap: var(--fandhe-space-8);\n    align-items: center;\n  }\n  \
.blocks-hero-split-image-row .blocks-hero-split-image-copy {\n    grid-column: 1 / span 7;\n    grid-row: 1;\n  }\n  \
.blocks-hero-split-image-row > [data-scope=\"image\"][data-part=\"root\"] {\n    grid-column: 8 / span 5;\n    grid-row: 1;\n  }\n  \
.blocks-hero-split-image-row > .blocks-hero-split-image-reverse-media {\n    grid-column: 8 / span 5;\n    grid-row: 1;\n  }\n  \
.blocks-hero-split-image-reverse-media {\n    height: 100%;\n  }\n  \
[data-blocks-hero-split-image-reverse] .blocks-hero-split-image-copy {\n    grid-column: 6 / span 7;\n  }\n  \
[data-blocks-hero-split-image-reverse] > .blocks-hero-split-image-reverse-media {\n    grid-column: 1 / span 5;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::demo;
    use fandhe_frontend_core::render;

    /// Demo が期待する 7 scope・4 行・非対話制約を満たしていることの単体
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
            "data-scope=\"avatar\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(
            html.matches("class=\"blocks-hero-split-image-row\"")
                .count(),
            4
        );
        assert_eq!(
            html.matches("blocks-hero-split-image-checklist-item")
                .count(),
            3
        );
        assert!(html.contains("data-blocks-hero-split-image-reverse"));
        assert!(html.contains("type=\"button\""));
        for absent in ["<form", "src=\"data:", "href=\"#\"", " id=\""] {
            assert!(
                !html.contains(absent),
                "demo output should never contain {absent}"
            );
        }
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件・反転セレクタを持つ
    /// こと。
    #[test]
    fn layout_css_declares_breakpoint_and_reverse_selector() {
        assert!(super::LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(super::LAYOUT_CSS.contains("data-blocks-hero-split-image-reverse"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（既存 block と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-hero-split-image-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-hero-split-image-layout");
    }
}
