//! `testimonial-masonry-grid` block（親 #2886「高さ不揃いの推薦文グリッド」
//! 配下、大規模な親 issue を 2 分割）。前半（#2887・PR #3314）が骨格
//! （CSS multi-column による真の masonry 配置・8 枚・先頭末尾 featured）を
//! 仕上げ、後半（本 #2888）が集約元 3 件（R0361/R1365/R1366）の並記と
//! 原稿の差分メモを追加した。
//!
//! # 使用部品
//!
//! `heading` / `text` / `card` / `blockquote` / `avatar` / `icon` の
//! 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 静的表示（無 JS、初期状態で固定）
//!
//! docs サイトは JS ハイドレーションを一切行わないため、本 Demo は状態
//! 機械を持たない。推薦文・氏名・役職・社名はすべて架空の固定値である。
//!
//! # レイアウトとブレークポイント
//!
//! 既定（狭い幅）は 1 列縦積み、`>= 40rem` で 2 列、`>= 64rem` で 3 列、
//! `>= 80rem` で 4 列へ切り替える（mobile-first の `min-width` メディア
//! クエリ。値は [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Sm`]/
//! `Lg`/`Xl` と一致するリテラル値、`content_with_testimonial` と同じ判断）。
//! CSS multi-column（`column-count`）で実現する（イシュー #2887 のレビュー
//! で 3 回指摘された根本原因: CSS Grid の `grid-auto-flow: row dense` は
//! 空きセルへの再配置のみを行い、各行の高さは行内の最大カードに揃って
//! しまうため、通常カードも既定でその行の高さまで伸び、高さ不揃いで
//! 敷き詰める表示契約を満たせなかった。multi-column は各列を独立した
//! 縦積みとして扱い、列内の各カードは自身の内容量ぶんの高さしか占有
//! しないため、列ごとに高さが異なる真の masonry 配置になる）。カードは
//! `break-inside: avoid`（1 枚のカードが列境界で分割されない）と
//! `margin-bottom`（multi-column コンテキストでは `gap` が列間隔にしか
//! 効かず行間隔を作れないため、縦方向の間隔はカード自身の下マージンで
//! 付ける）を持つ。
//!
//! # 3 案の並記（R0361/R1365/R1366、イシュー #2888）
//!
//! 集約元 3 件を [`variant_a`]/[`variant_b`]/[`variant_c`] として縦に
//! 並記する（`pricing_tiers_extra_row` の補足行 3 案並記と同じパターン）。
//! 各案の先頭に [`state_label`] で種別を示す。
//!
//! - **案 A**（[`variant_a`]、R0361・主参照）: 前半 #2887 の実装そのまま。
//!   8 枚を multi-column に敷き詰め、先頭・末尾を featured にする。
//! - **案 B**（[`variant_b`]、R1365）: featured カード 1 枚を
//!   multi-column 容器の**外**（直前）に全幅で置き、その下の
//!   multi-column に通常カード 10 枚を敷き詰める。featured を全幅にする
//!   には CSS Grid の 2×2 span や `column-span: all` が候補になるが、
//!   前者は行の高さが揃ってしまい masonry にならず（前半 #2887 と同じ
//!   理由）、後者は inline-block のカードには効かない。そのため featured
//!   を multi-column 容器の外に置く最も単純な方法を採る。
//! - **案 C**（[`variant_c`]、R1366）: featured なしの multi-column 9 枚。
//!   `>= 80rem` でも最大 3 列に留め（[`LAYOUT_CSS`] の案 C 専用オーバーライド
//!   セレクタ参照）、3 列 × 3 段で読める密度にする。
//!
//! 参照元（R0361/R1365/R1366）の文言・配色・装飾は持ち込まない。文言は
//! すべて架空（実在の企業名・PII を含まない）。
//!
//! # DOM 順と視覚順（multi-column の充填順）
//!
//! multi-column は既定（`column-fill: balance`）で列間の総高さを均等化
//! するよう配置するため、視覚上の充填順は厳密に「1 列目を上から詰め
//! きってから 2 列目」という単純な順にはならない場合がある（ブラウザの
//! バランス調整アルゴリズムに依存）。DOM 順（＝読み上げ順）は
//! [`QUOTES`] の宣言順のまま変わらない。静的な Demo であるため許容する。
//!
//! # アバターは架空・共通ダミー素材を再利用
//!
//! 人名・役職・社名は [`crate::blocks::dummy_assets`] の
//! `PERSON_NAMES`/`JOB_TITLES`/`COMPANY_NAMES` を index で引く
//! （`content_with_testimonial` と同型）。アバター画像は
//! `dummy_assets::AVATAR_SRC` を `alt=""`（装飾扱い。直後の氏名テキストと
//! 情報が重複するため）で出力する。
//!
//! # 装飾アイコン・チェックの a11y 判断
//!
//! 引用符の装飾アイコン（[`quote_icon`]）は `label: None` により
//! `aria-hidden="true"` になる（`pricing_tiers_extra_row::check_icon` と
//! 同型の判断）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading` / `text::text` / `card::root` / `icon::icon` /
//! `avatar::root` / `blockquote::root` はいずれも `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、Demo
//! 固有のスタイルフックは `data-blocks-testimonial-masonry-grid-*` 属性で
//! 渡す。素の `div`・`card::body`・`blockquote::caption`（`drop_class_attr`
//! を経由しない）には `.blocks-testimonial-masonry-grid-*` クラスを使う。
//! ルート class（`blocks-testimonial-masonry-grid-layout`）は
//! [`Block::demo_class`]（`blocks-testimonial-masonry-grid`）とは意図的に
//! 別名にする（`blog_list_image` 等と同じ Bugbot 教訓の回避）。
//!
//! # 詳細度の罠（`blockquote` recipe への勝ち方）
//!
//! `blockquote::caption` recipe（詳細度 (0,2,0)）に確実に勝つため、
//! caption 行の flex 化は `[data-scope="blockquote"][data-part="caption"]
//! .blocks-testimonial-masonry-grid-meta` の 3 セレクタ構成（詳細度
//! (0,3,0)）で行う（`testimonials_stack`/`content_with_testimonial` と
//! 同型の判断）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。人名・役職・社名・推薦文はすべて架空のもの（実在の企業名・
//! PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 11 件の推薦文（長さをわざと不揃いにし、multi-column の高さ不揃いを
/// Demo 上に出す。3 案（[`variant_a`]/[`variant_b`]/[`variant_c`]）が
/// それぞれ必要な件数だけスライスして使う）。
const QUOTES: [&str; 11] = [
    "導入して最初の週から、チーム全員の作業状況が一目で分かるようになりました。以前は週次の進捗確認会議に時間を取られていましたが、今ではダッシュボードを見るだけで十分です。ドキュメントも整備されていて、新しいメンバーの立ち上げも驚くほど早くなりました。",
    "サポートの反応が早く安心して使えます。",
    "既定のエスケープのおかげで、レビューで指摘される脆弱性がほぼゼロになりました。",
    "他のツールから乗り換えましたが、学習コストが低く、すぐに定着しました。",
    "設定ファイルが一目で分かるので、運用の引き継ぎが楽になりました。",
    "単一バイナリで配布できる点が、運用チームにとても好評です。",
    "細部まで作り込まれた操作感で、初めて触ったメンバーもすぐに馴染めました。",
    "料金プランの見直しを機に導入しましたが、機能面でも満足しています。特にレポート機能が充実していて、経営層への報告資料をそのまま出力できるのは大きな時短になりました。今後も長く使い続けたいツールです。",
    "導入前は複数ツールを併用していましたが、統合されたことで管理コストが大幅に減りました。",
    "ドキュメントが充実しているため、トラブル時も自己解決できることが多いです。",
    "チームの規模が大きくなっても、権限管理がシンプルなまま扱えています。",
];

/// 推薦文の装飾アイコン（引用符。参照元の形状は持ち込まない独自図形。
/// 常に `aria-hidden`、モジュール doc「装飾アイコン・チェックの a11y
/// 判断」節参照）。
fn quote_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Md,
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                (
                    "d",
                    "M7 8c-1.7 0-3 1.3-3 3v5h5v-5H7c0-1.1.9-2 2-2V8zm9 0c-1.7 0-3 1.3-3 3v5h5v-5h-2c0-1.1.9-2 2-2V8z",
                ),
                ("fill", "currentColor"),
            ],
            vec![],
        )],
    )
}

/// セクション見出し（H3 見出し + リード文）。
fn section_header() -> Node {
    div(
        vec![("class", "blocks-testimonial-masonry-grid-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("利用者の声")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "長さの異なる推薦文を高さを揃えずに敷き詰める 3 案を並記しています。",
                )],
            ),
        ],
    )
}

/// 案の種別を示す状態並記の見出し（モジュール doc「3 案の並記」節、
/// `pricing_tiers_extra_row::state_label` と同型）。
fn state_label(label: &str) -> Node {
    span(
        vec![("class", "blocks-testimonial-masonry-grid-state-label")],
        vec![text(label)],
    )
}

/// 推薦文カード 1 枚（`index` は [`dummy_assets`] の人名・役職・社名を
/// 引くためのオフセット、`quote` は表示する推薦文、`featured` は強調
/// 表示の有無）。
fn testimonial_card(index: usize, quote: &str, featured: bool) -> Node {
    let (variant, card_state) = if featured {
        (CardVariant::Elevated, "featured")
    } else {
        (CardVariant::Outline, "default")
    };
    let name = dummy_assets::PERSON_NAMES[index % dummy_assets::PERSON_NAMES.len()];
    let job_title = dummy_assets::JOB_TITLES[index % dummy_assets::JOB_TITLES.len()];
    let company = dummy_assets::COMPANY_NAMES[index % dummy_assets::COMPANY_NAMES.len()];

    card::root(
        CardProps {
            variant,
            ..CardProps::default()
        },
        vec![("data-blocks-testimonial-masonry-grid-card", card_state)],
        vec![card::body(
            vec![],
            vec![
                quote_icon(),
                blockquote::root(
                    BlockquoteVariant::default(),
                    ColorPalette::default(),
                    vec![("data-blocks-testimonial-masonry-grid-quote", "")],
                    vec![
                        blockquote::content(vec![], vec![text(quote)]),
                        blockquote::caption(
                            vec![("class", "blocks-testimonial-masonry-grid-meta")],
                            vec![
                                avatar::root(
                                    &AvatarProps::default(),
                                    vec![("data-blocks-testimonial-masonry-grid-avatar", "")],
                                    vec![avatar::image(
                                        ImageStatus::Loaded,
                                        dummy_assets::AVATAR_SRC,
                                        "",
                                        vec![],
                                    )],
                                ),
                                div(
                                    vec![("class", "blocks-testimonial-masonry-grid-byline")],
                                    vec![
                                        span(vec![], vec![text(name)]),
                                        span(
                                            vec![("class", "blocks-testimonial-masonry-grid-role")],
                                            vec![text(format!("{job_title}, {company}"))],
                                        ),
                                    ],
                                ),
                            ],
                        ),
                    ],
                ),
            ],
        )],
    )
}

/// 案 A（主参照 R0361、モジュール doc「3 案の並記」節参照）。前半 #2887
/// の実装そのまま: 8 枚を multi-column に敷き詰め、先頭・末尾を
/// featured にする。
fn variant_a() -> Node {
    let cards: Vec<Node> = QUOTES[0..8]
        .iter()
        .enumerate()
        .map(|(index, quote)| testimonial_card(index, quote, index == 0 || index == 7))
        .collect();
    div(
        vec![
            ("class", "blocks-testimonial-masonry-grid-variant"),
            ("data-blocks-testimonial-masonry-grid-variant", "a"),
        ],
        vec![
            state_label("案 A: 先頭と末尾を強調"),
            div(
                vec![("class", "blocks-testimonial-masonry-grid-grid")],
                cards,
            ),
        ],
    )
}

/// 案 B（R1365、モジュール doc「3 案の並記」節参照）。featured カード
/// 1 枚を multi-column 容器の外（直前）に全幅で置き、その下の
/// multi-column に通常カード 10 枚を敷き詰める。
fn variant_b() -> Node {
    let featured = testimonial_card(0, QUOTES[0], true);
    let cards: Vec<Node> = QUOTES[1..11]
        .iter()
        .enumerate()
        .map(|(offset, quote)| testimonial_card(offset + 1, quote, false))
        .collect();
    div(
        vec![
            ("class", "blocks-testimonial-masonry-grid-variant"),
            ("data-blocks-testimonial-masonry-grid-variant", "b"),
        ],
        vec![
            state_label("案 B: featured 1 枚 + 通常 10 枚"),
            div(
                vec![(
                    "class",
                    "blocks-testimonial-masonry-grid-featured-standalone",
                )],
                vec![featured],
            ),
            div(
                vec![("class", "blocks-testimonial-masonry-grid-grid")],
                cards,
            ),
        ],
    )
}

/// 案 C（R1366、モジュール doc「3 案の並記」節参照）。featured なしの
/// multi-column 9 枚。`>= 80rem` でも最大 3 列に留める（[`LAYOUT_CSS`] の
/// 案 C 専用オーバーライドセレクタ参照）。
fn variant_c() -> Node {
    let cards: Vec<Node> = QUOTES[0..9]
        .iter()
        .enumerate()
        .map(|(index, quote)| testimonial_card(index, quote, false))
        .collect();
    div(
        vec![
            ("class", "blocks-testimonial-masonry-grid-variant"),
            ("data-blocks-testimonial-masonry-grid-variant", "c"),
        ],
        vec![
            state_label("案 C: 強調なしの 9 枚"),
            div(
                vec![("class", "blocks-testimonial-masonry-grid-grid")],
                cards,
            ),
        ],
    )
}

/// `testimonial-masonry-grid` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数。見出しの後に 3 案（[`variant_a`]/[`variant_b`]/
/// [`variant_c`]）を縦に並べる。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-testimonial-masonry-grid-layout")],
        vec![section_header(), variant_a(), variant_b(), variant_c()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/testimonial-masonry-grid/",
    title: "testimonial-masonry-grid",
    category: BlockCategory::Testimonial,
    rust_source: "crates/docs-site/src/blocks/marketing/testimonial/testimonial_masonry_grid.rs",
    demo_class: "blocks-testimonial-masonry-grid",
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
            label: "Card",
            path: "/themes/card/",
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

/// `testimonial_masonry_grid` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節）。色・間隔はすべて既存
/// トークン（`--fandhe-*`）のみを使う。mobile-first（`min-width: 40rem`/
/// `64rem`/`80rem`）で列数（`column-count`）を切り替える CSS multi-column
/// レイアウト（R0361 準拠、モジュール doc「レイアウトとブレークポイント」
/// 節参照）。各カードは `break-inside: avoid` + `margin-bottom` で列内に
/// 縦積みされ、列ごとに独立した高さを持つことで「高さを揃えずに敷き
/// 詰める」契約を満たす。案 C（[`variant_c`]）専用のオーバーライド
/// セレクタ（詳細度 (0,1,1)、一般規則の (0,1,0) より確実に勝つ）が
/// `>= 80rem` でも最大 3 列に留める。
const LAYOUT_CSS: &str = "\
.blocks-testimonial-masonry-grid-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-testimonial-masonry-grid-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-testimonial-masonry-grid-variant {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-testimonial-masonry-grid-state-label {\n  display: block;\n  font-size: var(--fandhe-font-font-size-sm);\n  font-weight: 600;\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-testimonial-masonry-grid-featured-standalone {\n  margin-bottom: var(--fandhe-space-6);\n}\n\
.blocks-testimonial-masonry-grid-grid {\n  column-count: 1;\n  column-gap: var(--fandhe-space-6);\n}\n\
@media (min-width: 40rem) {\n  .blocks-testimonial-masonry-grid-grid {\n    column-count: 2;\n  }\n}\n\
@media (min-width: 64rem) {\n  .blocks-testimonial-masonry-grid-grid {\n    column-count: 3;\n  }\n}\n\
@media (min-width: 80rem) {\n  .blocks-testimonial-masonry-grid-grid {\n    column-count: 4;\n  }\n}\n\
@media (min-width: 80rem) {\n  [data-blocks-testimonial-masonry-grid-variant=\"c\"] .blocks-testimonial-masonry-grid-grid {\n    column-count: 3;\n  }\n}\n\
.blocks-testimonial-masonry-grid-grid > [data-blocks-testimonial-masonry-grid-card] {\n  display: inline-block;\n  width: 100%;\n  margin-bottom: var(--fandhe-space-6);\n  break-inside: avoid;\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-testimonial-masonry-grid-card=\"featured\"] {\n  border: 2px solid var(--fandhe-color-accent);\n}\n\
[data-blocks-testimonial-masonry-grid-card=\"featured\"] [data-scope=\"blockquote\"][data-part=\"content\"] {\n  font-size: var(--fandhe-font-font-size-lg);\n}\n\
[data-scope=\"blockquote\"][data-part=\"caption\"].blocks-testimonial-masonry-grid-meta {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-testimonial-masonry-grid-byline {\n  display: flex;\n  flex-direction: column;\n}\n\
.blocks-testimonial-masonry-grid-role {\n  color: var(--fandhe-color-fg-muted);\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 6 種の部品を出力し、`<form>`・`data:` を持たない
    /// こと。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"card\"",
            "data-scope=\"blockquote\"",
            "data-scope=\"avatar\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(!html.contains("<form"));
        assert!(!html.contains("data:"));
    }

    /// 3 案（A/B/C）が過不足なく 1 件ずつ、この順で並記されること
    /// （モジュール doc「3 案の並記」節参照）。
    #[test]
    fn demo_has_three_variants_in_order() {
        let html = render(&demo());
        for variant in ["a", "b", "c"] {
            assert_eq!(
                html.matches(&format!(
                    "data-blocks-testimonial-masonry-grid-variant=\"{variant}\""
                ))
                .count(),
                1,
                "demo output should contain exactly one variant {variant}"
            );
        }
        let pos_a = html
            .find("data-blocks-testimonial-masonry-grid-variant=\"a\"")
            .expect("variant a present");
        let pos_b = html
            .find("data-blocks-testimonial-masonry-grid-variant=\"b\"")
            .expect("variant b present");
        let pos_c = html
            .find("data-blocks-testimonial-masonry-grid-variant=\"c\"")
            .expect("variant c present");
        assert!(pos_a < pos_b, "variant a should come before variant b");
        assert!(pos_b < pos_c, "variant b should come before variant c");
    }

    /// 各案のカード数（A=8/B=11/C=9、合計 28）と featured 数
    /// （A=2/B=1/C=0、合計 3）が一致すること。
    #[test]
    fn demo_has_expected_card_and_featured_counts_per_variant() {
        let html = render(&demo());
        let variant_slice = |variant: &str| -> &str {
            let start = html
                .find(&format!(
                    "data-blocks-testimonial-masonry-grid-variant=\"{variant}\""
                ))
                .unwrap_or_else(|| panic!("variant {variant} present"));
            let next_variant = match variant {
                "a" => Some("data-blocks-testimonial-masonry-grid-variant=\"b\""),
                "b" => Some("data-blocks-testimonial-masonry-grid-variant=\"c\""),
                _ => None,
            };
            let end = next_variant
                .and_then(|marker| html[start..].find(marker).map(|p| start + p))
                .unwrap_or(html.len());
            &html[start..end]
        };

        let counts = [("a", 8usize, 2usize), ("b", 11, 1), ("c", 9, 0)];
        let mut total_cards = 0;
        let mut total_featured = 0;
        for (variant, expected_cards, expected_featured) in counts {
            let slice = variant_slice(variant);
            let cards = slice
                .matches("data-blocks-testimonial-masonry-grid-card=")
                .count();
            let featured = slice
                .matches("data-blocks-testimonial-masonry-grid-card=\"featured\"")
                .count();
            assert_eq!(cards, expected_cards, "variant {variant} card count");
            assert_eq!(
                featured, expected_featured,
                "variant {variant} featured count"
            );
            total_cards += cards;
            total_featured += featured;
        }
        assert_eq!(total_cards, 28);
        assert_eq!(total_featured, 3);
    }

    /// 案 A の先頭と末尾が featured であること（R0361 準拠）。
    #[test]
    fn variant_a_has_featured_at_ends() {
        let html = render(&demo());
        let start = html
            .find("data-blocks-testimonial-masonry-grid-variant=\"a\"")
            .expect("variant a present");
        let end = html
            .find("data-blocks-testimonial-masonry-grid-variant=\"b\"")
            .expect("variant b present");
        let slice = &html[start..end];
        let first_featured = slice
            .find("data-blocks-testimonial-masonry-grid-card=\"featured\"")
            .expect("at least one featured card");
        let first_default = slice
            .find("data-blocks-testimonial-masonry-grid-card=\"default\"")
            .expect("at least one default card");
        let last_featured = slice
            .rfind("data-blocks-testimonial-masonry-grid-card=\"featured\"")
            .expect("at least one featured card");
        let last_default = slice
            .rfind("data-blocks-testimonial-masonry-grid-card=\"default\"")
            .expect("at least one default card");
        assert!(
            first_featured < first_default,
            "first card should be featured"
        );
        assert!(last_featured > last_default, "last card should be featured");
    }

    /// 案 B の featured カードが multi-column 容器（`.blocks-testimonial-
    /// masonry-grid-grid`）より前に出力されること（モジュール doc「3 案の
    /// 並記」節の案 B 記述参照）。
    #[test]
    fn variant_b_featured_card_precedes_grid_container() {
        let html = render(&demo());
        let start = html
            .find("data-blocks-testimonial-masonry-grid-variant=\"b\"")
            .expect("variant b present");
        let end = html
            .find("data-blocks-testimonial-masonry-grid-variant=\"c\"")
            .expect("variant c present");
        let slice = &html[start..end];
        let featured_pos = slice
            .find("data-blocks-testimonial-masonry-grid-card=\"featured\"")
            .expect("featured card present");
        let grid_pos = slice
            .find("class=\"blocks-testimonial-masonry-grid-grid\"")
            .expect("grid container present");
        assert!(
            featured_pos < grid_pos,
            "featured card should precede the grid container"
        );
    }

    /// [`LAYOUT_CSS`] が 3 段のブレークポイント条件と、R0361 準拠の
    /// CSS multi-column（`column-count` + `break-inside: avoid`）を
    /// 持つこと（高さを揃えずに敷き詰める真の masonry 配置）。`grid-row`・
    /// `grid-auto-flow`・`column-span` は不採用のため不在。
    #[test]
    fn layout_css_has_breakpoints_and_multi_column_masonry() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 80rem)"));
        assert!(LAYOUT_CSS.contains("column-count: 2;"));
        assert!(LAYOUT_CSS.contains("column-count: 3;"));
        assert!(LAYOUT_CSS.contains("column-count: 4;"));
        assert!(LAYOUT_CSS.contains("break-inside: avoid;"));
        assert!(!LAYOUT_CSS.contains("grid-auto-flow"));
        assert!(!LAYOUT_CSS.contains("grid-row"));
        assert!(!LAYOUT_CSS.contains("column-span"));
    }

    /// 案 C（R1366）専用の 3 列上限オーバーライドセレクタが存在すること。
    #[test]
    fn layout_css_has_variant_c_three_column_cap() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-testimonial-masonry-grid-variant=\"c\"] .blocks-testimonial-masonry-grid-grid {\n    column-count: 3;"
        ));
    }

    /// caption 行の flex 化セレクタが詳細度 (0,3,0) で宣言されていること
    /// （モジュール doc「詳細度の罠」節参照）。
    #[test]
    fn layout_css_declares_caption_flex_with_expected_selector() {
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"blockquote\"][data-part=\"caption\"].blocks-testimonial-masonry-grid-meta {"
        ));
    }

    /// [`super::BLOCK`] の `parts` がモジュール doc「使用部品」節の 6 件と
    /// 一致すること。
    #[test]
    fn block_parts_has_six_entries() {
        assert_eq!(super::BLOCK.parts.len(), 6);
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（`blog_list_image` 等と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-testimonial-masonry-grid-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-testimonial-masonry-grid-layout"
        );
    }
}
