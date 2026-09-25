//! `content-with-testimonial` block（イシュー #2756。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー」配下。本文の列と引用の列を横に並べる
//! 2 列コンテンツの合成例で、`content-article`/`content-article-toc`/
//! `content-columns-screenshot`/`content-image-tiles`/`content-split-image`
//! に続く Marketing / Content カテゴリの 6 件目）。
//! 取得手段・ファイル名・内部コンポーネント識別子は記載しない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じライセンス
//! 上の転記制限、対応表 ID は R0868（基準形）と R0871 のみを記す）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `blockquote` / `avatar` / `card` / `stat` /
//! `image` / `link` の 8 部品を合成する（[`BLOCK`] の `parts` に一致させる
//! 契約、`crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が
//! 検証する）。
//!
//! # レイアウトとブレークポイント（lg = 64rem を境に列数を切り替える）
//!
//! `>= 64rem` で本文 7 : 引用 5 の比率の 2 列、`< 64rem` で本文 → 引用の
//! 順の 1 列にする。DOM 順は両形とも「本文 → 引用」であり、1 列表示時の
//! 読み順は DOM 順とそのまま一致する（`order` プロパティは使わない）。
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できないため
//! （CSS custom property は宣言側でのみ有効）、
//! [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`]（1024px =
//! 64rem）と一致するリテラル値を [`LAYOUT_CSS`] へ直書きする
//! （`content_split_image` と同じ判断）。ルート grid の class
//! （`blocks-content-with-testimonial-layout`）は [`Block::demo_class`]
//! （`blocks-content-with-testimonial`）とは意図的に別名にする
//! （`content_columns_screenshot` と同じ Bugbot 教訓の回避）。
//!
//! # 2 形を 1 つの Demo に並記する理由
//!
//! イシューが要求する引用の見せ方 2 形（R0868 基準形の左罫線付き
//! blockquote・R0871 の写真カード + 統計）を、`content_split_image` と
//! 同型に 1 つの Demo（[`demo`]）内へ縦に並記する。各形の直前に短い
//! ラベル（[`variant_label`]）を置き、Demo 上でどちらの集約元由来かを
//! 読み取れるようにする。
//!
//! # 写真カードの重ね順（R0871 の引用列）
//!
//! `blog_overlay_cards` と同型に (1) 背景画像（`image::image`、
//! `position: absolute; inset: 0`）、(2) 下から上へのグラデーション
//! スクリム（`div.blocks-content-with-testimonial-scrim`、
//! `position: absolute; inset: 0`、`aria-hidden="true"`）、(3) 引用
//! （`card::body` に `position: relative` を与え、絶対配置の 2 層より
//! 手前に描画させる。CSS 2.1 の描画順ルール上、`position: static` な
//! 子孫は絶対配置の兄弟より**先に**描画されてしまうため、`card::body`
//! 自身を positioned にする必要がある）の順で重ねる。
//!
//! # 配色の反転ペア
//!
//! 写真カードの引用列は `background: var(--fandhe-color-fg)` の
//! スクリムの上に `color: var(--fandhe-color-bg)` の文字を乗せる、
//! `blog_overlay_cards` と同じトークン反転ペアで構成する（ライト/ダーク
//! 双方で継ぎ目なくコントラストを保つ）。基準形（罫線付き blockquote）は
//! 反転を行わず既定の前景色のまま表示する。スクリムは `blog_overlay_cards`
//! と同じく下端 35% を不透明度 0.9 の単色帯として保持し（`transparent` へ
//! 単純にグラデーションさせるだけでは上端付近のコントラストが不足する
//! ため）、引用（`card::body`）自身も `justify-content: flex-end` で
//! 下端の単色帯へ寄せる（`card::body` recipe の `flex: 1` により素の
//! ままだと上端へ揃い、単色帯の外に出てしまうため）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading` / `text::text` / `blockquote::root` / `avatar::root` /
//! `card::root` / `stat::root` / `image::image` / `link::root` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-content-with-testimonial-*` 属性で渡し、[`LAYOUT_CSS`]
//! 側も同じ属性セレクタで対応する。素の `div`・`card::body`・
//! `blockquote::content`/`caption`・`stat::label`/`value_text` には
//! `class` がそのまま効くため、それらは従来どおり
//! `.blocks-content-with-testimonial-*` クラスセレクタを使う。
//!
//! # 詳細度の罠（`text`/`blockquote` recipe への勝ち方）
//!
//! `text::text`・`blockquote::root`（root の左罫線）の recipe（詳細度
//! (0,2,0)）に確実に勝つため、上書きは `[data-scope="…"][data-part="…"]
//! [data-blocks-content-with-testimonial-*]` の 3 セレクタ構成
//! （詳細度 (0,3,0)）で行う（`content_columns_screenshot`/
//! `content_split_image` と同型の判断）。写真カード内の blockquote は
//! この構成で左罫線を消し、代わりに `color: inherit` +
//! `--fandhe-blockquote-caption-fg` の上書きでキャプションの文字色も
//! 白側へ寄せる。
//!
//! # 見出しレベルに `H3` を使う理由
//!
//! ページ側が `## Demo` として `h2` を出すため、見出しは
//! `HeadingLevel::H3` にする（`content_columns_screenshot`/
//! `content_split_image` と同じ判断）。
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
//! 含まない）。画像は [`crate::blocks::dummy_assets::AVATAR_SRC`]
//! （人物アバター）・[`crate::blocks::dummy_assets::BACKGROUND_SRC`]
//! （写真カードの背景）を使い、`alt=""`（装飾扱い）で出力する。リンクは
//! `Block::demo` が `base_path` を受け取らずサイト内絶対パスを持てない
//! ため、実在の自リポジトリへの絶対 URL（[`REPO`]）を使う（`href="#"` は
//! `linkcheck::check_links` が broken として検知するため使わない）。
//! Demo は 2 インスタンスを持つため `id` 属性は一切使わない（重複 id
//! 検知テスト対策）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// リンク先（`Block::demo` が `base_path` を受け取らずサイト内絶対パスを
/// 持てないため、実在の自リポジトリへの絶対 URL を使う。モジュール doc
/// 「`<form>` を持たない・データ取得/送信を行わない・id を使わない」節
/// 参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 本文段落（架空文言、2 形で使い回して検索インデックスの増分を抑える。
/// `content_split_image::PARAGRAPHS` と同型の判断）。
const PARAGRAPHS: &[&str] = &[
    "本文の列と引用の列を横に並べ、読み手に信頼のシグナルを一目で伝えます。",
    "既存の Themes 部品だけを組み合わせた、状態を持たない静的な合成例です。",
];

/// 写真カード側（形 B）の統計 4 件（ラベル, 値）。架空の数値。
const STATS: [(&str, &str); 4] = [
    ("設立", "2016"),
    ("メンバー", "48"),
    ("拠点", "6"),
    ("対応言語", "12"),
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
        vec![("data-blocks-content-with-testimonial-paragraph", "")],
        vec![text(body)],
    )
}

/// 形 A（R0868 基準形）の引用列: 左罫線付き blockquote + アバター付き
/// キャプション。
fn border_quote() -> Node {
    blockquote::root(
        BlockquoteVariant::default(),
        ColorPalette::default(),
        vec![("data-blocks-content-with-testimonial-quote", "")],
        vec![
            blockquote::content(vec![], vec![text(dummy_assets::TESTIMONIAL_QUOTES[0])]),
            blockquote::caption(
                vec![("class", "blocks-content-with-testimonial-meta")],
                vec![
                    avatar::root(
                        &AvatarProps::default(),
                        vec![("data-blocks-content-with-testimonial-avatar", "")],
                        vec![avatar::image(
                            ImageStatus::Loaded,
                            dummy_assets::AVATAR_SRC,
                            "",
                            vec![],
                        )],
                    ),
                    div(
                        vec![("class", "blocks-content-with-testimonial-byline")],
                        vec![
                            div(vec![], vec![text(dummy_assets::PERSON_NAMES[0])]),
                            div(vec![], vec![text(dummy_assets::JOB_TITLES[0])]),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// 形 A（R0868 基準形）: 本文の列 + 罫線付き blockquote の引用列。
fn variant_border() -> Node {
    let body = div(
        vec![("class", "blocks-content-with-testimonial-body")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("導入事例")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("顧客の声を届ける 2 列レイアウト")],
            ),
            paragraph(PARAGRAPHS[0]),
            paragraph(PARAGRAPHS[1]),
        ],
    );

    div(
        vec![("class", "blocks-content-with-testimonial-grid")],
        vec![body, border_quote()],
    )
}

/// 統計 1 件分（ラベル + 値、[`stat::root`] を `Md` サイズで組み立てる）。
fn stat_item((label, value): &(&'static str, &'static str)) -> Node {
    stat::root(
        Size::Md,
        vec![],
        vec![
            stat::label(vec![], vec![text(*label)]),
            stat::value_text(vec![], vec![text(*value)]),
        ],
    )
}

/// 形 B（R0871）の引用列: 写真カード + 暗幕 + 引用（罫線なし・反転配色）。
fn photo_quote() -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-content-with-testimonial-photo-card", "")],
        vec![
            image::image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    ..ImageProps::new(dummy_assets::BACKGROUND_SRC, "")
                },
                vec![("data-blocks-content-with-testimonial-photo", "")],
            ),
            div(
                vec![
                    ("class", "blocks-content-with-testimonial-scrim"),
                    ("aria-hidden", "true"),
                ],
                vec![],
            ),
            card::body(
                vec![("class", "blocks-content-with-testimonial-card-copy")],
                vec![blockquote::root(
                    BlockquoteVariant::default(),
                    ColorPalette::default(),
                    vec![("data-blocks-content-with-testimonial-card-quote", "")],
                    vec![
                        blockquote::content(
                            vec![],
                            vec![text(dummy_assets::TESTIMONIAL_QUOTES[1])],
                        ),
                        blockquote::caption(
                            vec![],
                            vec![
                                div(vec![], vec![text(dummy_assets::PERSON_NAMES[1])]),
                                div(vec![], vec![text(dummy_assets::COMPANY_NAMES[0])]),
                            ],
                        ),
                    ],
                )],
            ),
        ],
    )
}

/// 形 B（R0871）: 本文の列（見出し + 段落 + 統計 4 件 + テキストリンク）+
/// 写真カードの引用列。
fn variant_photo() -> Node {
    let stats = div(
        vec![("class", "blocks-content-with-testimonial-stats")],
        STATS.iter().map(stat_item).collect(),
    );

    let body = div(
        vec![("class", "blocks-content-with-testimonial-body")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("写真と数字で裏付ける導入実績")],
            ),
            paragraph(PARAGRAPHS[0]),
            stats,
            link::root(
                REPO,
                &LinkProps::default(),
                vec![("data-blocks-content-with-testimonial-link", "")],
                vec![text("導入事例をもっと見る")],
            ),
        ],
    );

    div(
        vec![("class", "blocks-content-with-testimonial-grid")],
        vec![body, photo_quote()],
    )
}

/// `content-with-testimonial` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「2 形を 1 つの Demo に並記する理由」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-content-with-testimonial-layout")],
        vec![
            variant_label("罫線の引用（対応表 ID R0868 基準形）"),
            variant_border(),
            variant_label("写真カードの引用（対応表 ID R0871）"),
            variant_photo(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/content-with-testimonial/",
    title: "content-with-testimonial",
    category: BlockCategory::Content,
    rust_source: "crates/docs-site/src/blocks/marketing/content/content_with_testimonial.rs",
    demo_class: "blocks-content-with-testimonial",
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
            label: "Blockquote",
            path: "/themes/blockquote/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Stat",
            path: "/themes/stat/",
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

/// `content_with_testimonial` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節。他 block と同型で
/// `pub(super)` ではなく本ファイル内 private 定数として `super::stylesheet`
/// 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-content-with-testimonial-*` と
/// `[data-blocks-content-with-testimonial-*]` のみを用い、他 block や
/// 部品の素のセレクタへ影響させない（`content_columns_screenshot`/
/// `content_split_image` と同じ名前空間分離）。
const LAYOUT_CSS: &str = "\
.blocks-content-with-testimonial-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-content-with-testimonial-grid {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-content-with-testimonial-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  align-items: flex-start;\n  min-width: 0;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-content-with-testimonial-paragraph] {\n  margin: 0;\n}\n\
[data-scope=\"blockquote\"][data-part=\"caption\"].blocks-content-with-testimonial-meta {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-content-with-testimonial-byline {\n  display: flex;\n  flex-direction: column;\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
.blocks-content-with-testimonial-stats {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-4);\n  width: 100%;\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-content-with-testimonial-photo-card] {\n  position: relative;\n  overflow: hidden;\n  min-height: 20rem;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-content-with-testimonial-photo] {\n  position: absolute;\n  inset: 0;\n  width: 100%;\n  height: 100%;\n  max-width: none;\n  object-fit: cover;\n}\n\
.blocks-content-with-testimonial-scrim {\n  position: absolute;\n  inset: 0;\n  background: linear-gradient(to top, var(--fandhe-color-fg) 0%, var(--fandhe-color-fg) 35%, transparent 100%);\n  opacity: 0.9;\n}\n\
.blocks-content-with-testimonial-card-copy {\n  position: relative;\n  display: flex;\n  flex-direction: column;\n  justify-content: flex-end;\n  color: var(--fandhe-color-bg);\n}\n\
[data-scope=\"blockquote\"][data-part=\"root\"][data-blocks-content-with-testimonial-card-quote] {\n  border-inline-start: 0;\n  padding-inline-start: 0;\n  color: inherit;\n  --fandhe-blockquote-caption-fg: var(--fandhe-color-bg);\n}\n\
@media (min-width: 64rem) {\n  \
.blocks-content-with-testimonial-grid {\n    display: grid;\n    grid-template-columns: minmax(0, 7fr) minmax(0, 5fr);\n    gap: var(--fandhe-space-12);\n    align-items: start;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, dummy_assets, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 8 部品を出力すること、非対話制約（`<form>` 不在・
    /// `data:` URI 不在・`id` 不在）を満たすことの単体回帰
    /// （`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複し
    /// 過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"blockquote\"",
            "data-scope=\"avatar\"",
            "data-scope=\"card\"",
            "data-scope=\"stat\"",
            "data-scope=\"image\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        // avatar の image パート 1 個 + 写真カードの背景画像 1 個。
        assert_eq!(html.matches("<img").count(), 2);
        // 統計は 4 件（stat::root が 4 回出力される）。
        assert_eq!(
            html.matches("data-scope=\"stat\" data-part=\"root\"")
                .count(),
            4
        );
        assert!(html.contains(dummy_assets::AVATAR_SRC));
        assert!(html.contains(dummy_assets::BACKGROUND_SRC));
        assert!(!html.contains("<form"));
        assert!(!html.contains("<button"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("id=\""));
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件・7:5 比率・写真カード
    /// の重ね順規則を持つこと。
    #[test]
    fn layout_css_declares_lg_breakpoint_and_photo_card_overlay() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("minmax(0, 7fr) minmax(0, 5fr)"));
        assert!(LAYOUT_CSS.contains("position: absolute"));
        assert!(LAYOUT_CSS.contains(
            "linear-gradient(to top, var(--fandhe-color-fg) 0%, var(--fandhe-color-fg) 35%, transparent 100%)"
        ));
        assert!(LAYOUT_CSS.contains("justify-content: flex-end"));
    }

    /// ルート grid class（`demo_class` とは別名）が `demo()` の出力へ
    /// 実際に現れること（モジュール doc「レイアウトとブレークポイント」
    /// 節の Bugbot 教訓の固定、`content_columns_screenshot`/
    /// `content_split_image` と同じ回帰）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-content-with-testimonial-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-content-with-testimonial-layout"
        );
    }
}
