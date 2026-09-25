//! `feature-split-image` block（イシュー #2768。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー」配下、Phase 1（マーケティング A）に
//! 属する。テキスト列 + 4:3 画像列の 2 列 feature セクションの合成例で、
//! `feature-accordion-image`/`feature-alternating-rows`/`feature-expand`/
//! `feature-image-cards` に続く Marketing / Feature カテゴリの 5 件目）。
//! 取得手段・ファイル名・内部コンポーネント識別子は記載しない（対応表 ID
//! は R0471（基準形）と R0485/R0474/R0940/R0482/R0098 のみを記す）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `button` / `image` / `blockquote` /
//! `avatar` / `icon` の 8 部品を合成する（[`BLOCK`] の `parts` に一致させる
//! 契約、`crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が
//! 検証する）。
//!
//! # 4 形を 1 つの Demo に並記する
//!
//! `feature-alternating-rows`/`cta-split-image` と同型に、集約元 6 件
//! （R0471 基準形 + 差分 5 件）を [`variant_label`] で見出しを付けながら
//! [`demo`] 1 つの中へ縦に並べる。
//!
//! | 形 | 対応 ID | 内容 |
//! |----|---------|------|
//! | A 基準形 | R0471 | badge/heading/text/ボタン 2 個（テキスト列）+ 4:3 画像（メディア列） |
//! | B チェックリスト | R0485 + R0482 | 説明文の下にチェック付き箇条書き。lg 以上で画像を左へ入れ替え |
//! | C 引用付き | R0474 + R0940 | ボタン列の下に発言者付き引用（画面画像を添える） |
//! | D 3 列一覧付き | R0098 | 2 列の下に 3 列の feature 一覧を続ける |
//!
//! # DOM 順はテキスト → 画像（左右入れ替えは grid 配置だけで行う）
//!
//! 各形の DOM 順は常にテキスト → 画像で固定する（読み上げ順を変えない）。
//! lg（64rem）未満は `.blocks-feature-split-image-row` が flex column の
//! ためテキストの下に画像が来る。lg 以上では 12 列 grid の
//! `grid-column` 指定だけで配置を決め、DOM 順自体は変えない（
//! `feature-alternating-rows` と同じ判断）。
//!
//! # 反転を `nth-child` ではなく data 属性で示す理由
//!
//! 形 B（R0482）の左右入れ替えは CSS の `:nth-child` ではなく、Rust 側で
//! 組み立てる時点で `data-blocks-feature-split-image-reverse` 属性を明示
//! 付与することで表す（`feature-alternating-rows` と同じ判断軸。本 block
//! は形ごとに構造が異なり `nth-child` による偶奇判定に対応する規則的な
//! 並びを持たないため、attribute セレクタの方が実装意図を HTML から機械的
//! に読み取れる）。
//!
//! # ブレークポイントに lg（64rem）を使う理由
//!
//! 他の Marketing / Feature block と同じ判断で、
//! [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`]（1024px =
//! 64rem）のリテラル値を [`LAYOUT_CSS`] へ直書きする（テーマの
//! breakpoint トークンは `@media` 条件式の中では解決できないため）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `badge::badge`/`heading::heading`/`styled_text::text`/`button::button`/
//! `image::image`/`blockquote::root`/`avatar::root`/`icon::icon` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-feature-split-image-*` 属性で渡し、[`LAYOUT_CSS`] 側も
//! 同じ属性セレクタで対応する。素の `div`/`ul`/`li` には `class` がそのまま
//! 効くため、それらは `.blocks-feature-split-image-*` クラスセレクタを
//! 使う。レイアウト root の class（`blocks-feature-split-image-layout`）は
//! [`Block::demo_class`]（`blocks-feature-split-image`）と意図的に別名に
//! する（既存 block と同じ Bugbot 教訓の回避）。
//!
//! # 詳細度の罠（`text`/`image`/`blockquote` recipe への勝ち方）
//!
//! `styled_text::text`・`image::image`・`blockquote::root` の recipe
//! （詳細度 (0,2,0)）に確実に勝つため、上書きは `[data-scope="…"]
//! [data-part="…"][data-blocks-feature-split-image-*]` の 3 セレクタ構成
//! （詳細度 (0,3,0)）で行う（`feature-alternating-rows`/`cta-split-image`
//! と同型の判断）。
//!
//! # 見出しレベルに `H3`/`H4` を使う理由
//!
//! ページ側が `## Demo` として `h2` を出すため、各形の見出しは
//! `HeadingLevel::H3` にする。形 D の feature 一覧項目はそれより 1 段
//! 下げて `HeadingLevel::H4` にする（`feature-alternating-rows` と同じ
//! 階層判断）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と `fandhe_frontend_core::text`
//! が同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # チェックリストのアイコン
//!
//! チェックマークは [`super::super::cta::cta_split_image::geo_icon`] と
//! 同型の自作幾何アイコン（lucide 等の著作物を複製しない単純な折れ線）を
//! 本ファイル内へ複製する（クレート内の private 関数のため共有できない）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない・id を使わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。ボタンは `button::button` の既定 `type="button"` のまま送信先を
//! 持たず、値は一切送信されない。文言はすべて架空のもの（実企業名・実
//! クレデンシャル・PII を含まない）。画像は [`crate::blocks::dummy_assets`]
//! のビルド時生成プレースホルダー SVG のみを使い、`alt=""`（装飾扱い）で
//! 出力する（`data:` URI は使わない）。Demo は複数インスタンスを持つため
//! `id` 属性は一切使わない（重複 id 検知テスト対策）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 自作の幾何アイコン（線画。モジュール doc「チェックリストのアイコン」
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

/// 形 B のチェック付き項目 4 件（架空文言）。
const CHECKLIST_ITEMS: [&str; 4] = [
    "既定エスケープ済みの HTML 出力",
    "外部依存ゼロの描画コア",
    "決定的なビルド成果物",
    "型で保証されたコンポーネント境界",
];

/// 形 D の feature 一覧 3 件（見出し + 説明）。
const MINI_FEATURES: [(&str, &str); 3] = [
    (
        "静的な表示のみ",
        "JS ハイドレーションを行わない、決定的な静的表示専用の構成です。",
    ),
    (
        "単一実行ファイル配布",
        "サーバー・アセットをひとまとめにし、Docker イメージ 1 枚で配布できます。",
    ),
    (
        "既定エスケープ",
        "テキスト補間は必ず既定エスケープを経由し、迂回経路を新設しません。",
    ),
];

/// チェック付き項目 1 件分（チェックマーク + 短い文言）。
fn checklist_item(label: &'static str) -> Node {
    li(
        vec![("class", "blocks-feature-split-image-checklist-item")],
        vec![check_icon(), text(label)],
    )
}

/// A/B/C 共通のボタン列（Solid + Outline の 2 個）。
fn actions() -> Node {
    div(
        vec![("class", "blocks-feature-split-image-actions")],
        vec![
            button::button(
                &ButtonProps {
                    size: Size::Lg,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("今すぐ試す")],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    size: Size::Lg,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("資料を見る")],
            ),
        ],
    )
}

/// テキスト列共通の先頭 3 要素（badge + 見出し + 説明文）。
fn copy_header(eyebrow: &'static str, title: &'static str, body: &'static str) -> Vec<Node> {
    vec![
        badge::badge(&BadgeProps::default(), vec![], vec![text(eyebrow)]),
        heading(
            HeadingLevel::H3,
            &HeadingProps {
                size: HeadingSize::Xl3,
                weight: HeadingWeight::Bold,
            },
            vec![],
            vec![text(title)],
        ),
        styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text(body)],
        ),
    ]
}

/// 4:3 画像列（[`AspectRatio::Landscape`]・角丸・装飾扱いの `alt=""`）。
fn media(src: &'static str) -> Node {
    div(
        vec![("class", "blocks-feature-split-image-media")],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                aspect_ratio: AspectRatio::Landscape,
                shape: ImageShape::Rounded,
                ..ImageProps::new(src, "")
            },
            vec![("data-blocks-feature-split-image-image", "")],
        )],
    )
}

/// 形 A（R0471 基準形）: テキスト列（badge + 見出し + 説明 + ボタン 2 個）+
/// 4:3 画像列。
fn variant_basic() -> Node {
    let copy = div(vec![("class", "blocks-feature-split-image-copy")], {
        let mut children = copy_header(
            "新機能",
            "テキストと画像を左右に並べて紹介する",
            "要点を短い説明文にまとめ、隣に画面を添えて価値を素早く伝えます。",
        );
        children.push(actions());
        children
    });

    div(
        vec![("class", "blocks-feature-split-image-row")],
        vec![copy, media(dummy_assets::PRODUCT_SRC)],
    )
}

/// 形 B（R0485 + R0482）: 説明文の下にチェック付き箇条書きを追加し、
/// lg 以上で画像を左へ入れ替える（DOM 順はテキストが先のまま）。
fn variant_checklist() -> Node {
    let checklist = ul(
        vec![("class", "blocks-feature-split-image-checklist")],
        CHECKLIST_ITEMS
            .iter()
            .map(|label| checklist_item(label))
            .collect(),
    );

    let copy = div(vec![("class", "blocks-feature-split-image-copy")], {
        let mut children = copy_header(
            "チェックリスト",
            "満たすべき要件を、その場で確認できる",
            "説明文だけでなく要点を列挙することで、導入前に必要な条件を一目で把握できます。",
        );
        children.push(checklist);
        children
    });

    div(
        vec![
            ("class", "blocks-feature-split-image-row"),
            ("data-blocks-feature-split-image-reverse", ""),
        ],
        vec![copy, media(dummy_assets::BACKGROUND_SRC)],
    )
}

/// 形 C（R0474 + R0940）: ボタン列の下に発言者付き引用を置き、画像には
/// 画面画像（[`dummy_assets::SCREENSHOT_SRC`]）を使う。
fn variant_testimonial() -> Node {
    let quote = blockquote::root(
        BlockquoteVariant::default(),
        ColorPalette::default(),
        vec![("data-blocks-feature-split-image-quote", "")],
        vec![
            blockquote::content(vec![], vec![text(dummy_assets::TESTIMONIAL_QUOTES[0])]),
            blockquote::caption(
                vec![("class", "blocks-feature-split-image-meta")],
                vec![
                    avatar::root(
                        &AvatarProps::default(),
                        vec![("data-blocks-feature-split-image-avatar", "")],
                        vec![avatar::image(
                            ImageStatus::Loaded,
                            dummy_assets::AVATAR_SRC,
                            "",
                            vec![],
                        )],
                    ),
                    div(
                        vec![("class", "blocks-feature-split-image-byline")],
                        vec![
                            div(vec![], vec![text(dummy_assets::PERSON_NAMES[0])]),
                            div(vec![], vec![text(dummy_assets::JOB_TITLES[0])]),
                        ],
                    ),
                ],
            ),
        ],
    );

    let copy = div(vec![("class", "blocks-feature-split-image-copy")], {
        let mut children = copy_header(
            "導入事例",
            "実際に使うチームの声を添えて紹介する",
            "機能の説明だけでなく、利用しているチームの一言を添えることで説得力を高められます。",
        );
        children.push(actions());
        children.push(quote);
        children
    });

    div(
        vec![("class", "blocks-feature-split-image-row")],
        vec![copy, media(dummy_assets::SCREENSHOT_SRC)],
    )
}

/// 形 D（R0098）の feature 一覧 1 件分（幾何アイコン + 見出し + 説明）。
fn mini_feature((title, body): &(&'static str, &'static str)) -> Node {
    div(
        vec![("class", "blocks-feature-split-image-feature")],
        vec![
            geo_icon("M12 2l3 7h7l-5.5 4.5L18 21l-6-4-6 4 1.5-7.5L2 9h7z"),
            heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text(*title)],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(*body)],
            ),
        ],
    )
}

/// 形 D（R0098）: 形 A と同型の 2 列（ボタン 2 個）の下に、3 列の feature
/// 一覧を続ける。
fn variant_with_features() -> Node {
    let copy = div(vec![("class", "blocks-feature-split-image-copy")], {
        let mut children = copy_header(
            "まとめて紹介",
            "2 列の下に、関連する特長を並べて続ける",
            "主要な訴求の直後に、補足となる特長を 3 列で並べて理解を後押しします。",
        );
        children.push(actions());
        children
    });

    let row = div(
        vec![("class", "blocks-feature-split-image-row")],
        vec![copy, media(dummy_assets::LOGO_SRC)],
    );

    let features = div(
        vec![("class", "blocks-feature-split-image-features")],
        MINI_FEATURES.iter().map(mini_feature).collect(),
    );

    div(
        vec![("class", "blocks-feature-split-image-with-features")],
        vec![row, features],
    )
}

/// `feature-split-image` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（モジュール doc「4 形を 1 つの Demo に並記する」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-feature-split-image-layout")],
        vec![
            variant_label("画像 1 枚 + テキスト列（対応表 ID R0471 基準形）"),
            variant_basic(),
            variant_label("チェックリスト + 左右入れ替え（対応表 ID R0485/R0482）"),
            variant_checklist(),
            variant_label("発言者付き引用（対応表 ID R0474/R0940）"),
            variant_testimonial(),
            variant_label("3 列 feature 一覧付き（対応表 ID R0098）"),
            variant_with_features(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/feature-split-image/",
    title: "feature-split-image",
    category: BlockCategory::Feature,
    rust_source: "crates/docs-site/src/blocks/marketing/feature/feature_split_image.rs",
    demo_class: "blocks-feature-split-image",
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
            label: "Blockquote",
            path: "/themes/blockquote/",
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

/// `feature_split_image` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。セレクタは
/// `.blocks-feature-split-image-*` と `[data-blocks-feature-split-image-*]`
/// のみを用い、他 block や部品の素のセレクタへ影響させない
/// （`feature-alternating-rows` と同じ名前空間分離）。
const LAYOUT_CSS: &str = "\
.blocks-feature-split-image-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-feature-split-image-row {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-feature-split-image-copy {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  align-items: start;\n  min-width: 0;\n}\n\
.blocks-feature-split-image-actions {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-feature-split-image-image] {\n  display: block;\n  width: 100%;\n}\n\
.blocks-feature-split-image-checklist {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  margin: 0;\n  padding: 0;\n  list-style: none;\n}\n\
.blocks-feature-split-image-checklist-item {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"blockquote\"][data-part=\"root\"][data-blocks-feature-split-image-quote] {\n  margin: 0;\n}\n\
[data-scope=\"blockquote\"][data-part=\"caption\"].blocks-feature-split-image-meta {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-feature-split-image-byline {\n  display: flex;\n  flex-direction: column;\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
.blocks-feature-split-image-with-features {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-feature-split-image-features {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-feature-split-image-feature {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
@media (min-width: 64rem) {\n  \
.blocks-feature-split-image-row {\n    display: grid;\n    grid-template-columns: repeat(12, minmax(0, 1fr));\n    column-gap: var(--fandhe-space-10);\n    align-items: center;\n  }\n  \
.blocks-feature-split-image-row > .blocks-feature-split-image-copy {\n    grid-column: 1 / span 5;\n    grid-row: 1;\n  }\n  \
.blocks-feature-split-image-row > .blocks-feature-split-image-media {\n    grid-column: 7 / span 6;\n    grid-row: 1;\n  }\n  \
.blocks-feature-split-image-row[data-blocks-feature-split-image-reverse] > .blocks-feature-split-image-copy {\n    grid-column: 8 / span 5;\n  }\n  \
.blocks-feature-split-image-row[data-blocks-feature-split-image-reverse] > .blocks-feature-split-image-media {\n    grid-column: 1 / span 6;\n  }\n  \
.blocks-feature-split-image-features {\n    display: grid;\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n    gap: var(--fandhe-space-8);\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, dummy_assets, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 8 部品を出力すること、非対話制約（`<form>` 不在・
    /// `data:` URI 不在・`id`/`href="#"` 不在）を満たすことの単体回帰
    /// （`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複し
    /// 過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"button\"",
            "data-scope=\"image\"",
            "data-scope=\"blockquote\"",
            "data-scope=\"avatar\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        // A(1) + B(1) + C(画像 1 + avatar 1) + D(1) = 5 枚。
        assert_eq!(html.matches("<img").count(), 5);
        assert_eq!(
            html.matches("data-blocks-feature-split-image-reverse")
                .count(),
            1
        );
        assert!(html.contains("type=\"button\""));
        for absent in ["<form", "src=\"data:", "href=\"#\"", " id=\""] {
            assert!(
                !html.contains(absent),
                "demo output should never contain {absent}"
            );
        }
        assert!(html.contains(dummy_assets::PRODUCT_SRC));
        assert!(html.contains(dummy_assets::BACKGROUND_SRC));
        assert!(html.contains(dummy_assets::SCREENSHOT_SRC));
        assert!(html.contains(dummy_assets::LOGO_SRC));
        assert!(html.contains(dummy_assets::AVATAR_SRC));
    }

    /// [`LAYOUT_CSS`] が想定する lg ブレークポイント・12 列 grid・反転規則・
    /// 3 列 feature 一覧を持つこと。
    #[test]
    fn layout_css_declares_lg_grid_and_reverse_rules() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("repeat(12, minmax(0, 1fr))"));
        assert!(LAYOUT_CSS.contains("[data-blocks-feature-split-image-reverse]"));
        assert!(LAYOUT_CSS.contains("repeat(3, minmax(0, 1fr))"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（既存 block と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-feature-split-image-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-feature-split-image-layout");
    }

    /// 4:3 画像（[`fandhe_frontend_pre_styled_ui::image::AspectRatio::
    /// Landscape`]）の variant class が出力されること。
    #[test]
    fn aspect_ratio_is_landscape() {
        let html = render(&demo());
        assert!(html.contains("fd-image--aspect-ratio-landscape"));
    }
}
