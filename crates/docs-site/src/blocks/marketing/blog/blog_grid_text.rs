//! `blog-grid-text` block（イシュー #2811。親トラッキング #2807「Blocks
//! マーケティング B」配下、対応表 ID R0772 の 1 件を主参照とし、R0016 /
//! R0417 / R0775 を構造だけ集約した合成例。画像を持たない記事カードの
//! グリッドレイアウト）。取得手段・ファイル名・内部コンポーネント識別子は
//! 記載しない（[`super::blog_featured_with_list`] モジュール doc と同じ
//! ライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `badge` / `card` / `avatar` / `link` / `link-overlay`
//! の 7 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 静的な 2 インスタンス併記（バリエーションの表現方法）
//!
//! JS ハイドレーションを行わない docs サイトでは「揃え（左寄せ/中央）」
//! 「上罫線の有無」の 2 軸バリエーションをトグルで切り替えられないため、
//! [`super::footer::footer_newsletter`] と同型の「キャプション + 静的な
//! 2 インスタンス併記」で両方を Demo 上に同時提示する
//! （`crates/docs-site/tests/blocks_contract.rs` が両インスタンスの属性を
//! 固定する）。
//!
//! - インスタンス A（対応表 R0772 が主参照、著者構成は R0417 を集約）:
//!   左寄せ見出し + 上罫線あり。
//! - インスタンス B（対応表 R0016 の簡素な導入を集約）: 中央揃え見出し +
//!   上罫線なし。
//!
//! 対応表 R0775（記事カード自体の構成は同一の 1 列縦積みレイアウト）は
//! 第 3 インスタンスとして個別に再現しない。狭い幅で
//! [`LAYOUT_CSS`] のグリッドが 1 列へ折り返すレスポンシブ挙動そのものが
//! 対応するため、原稿側の「原案差分メモ」節にその旨を明記する。
//!
//! # カード全面リンクと著者リンクを両立する 2 段構成
//!
//! [`super::blog_featured_with_list`] の記事リストと同じ判断軸で、
//! `link_overlay::overlay` は `z-index: 0` でカード全面に重なるため
//! （`fandhe_frontend_pre_styled_ui::link_overlay` モジュール doc「全面拡張
//! の CSS 実装」節）、著者リンクを `overlay` と同じ `link_overlay::root` の
//! 内側に置くとクリックできなくなる。本 block ではカード
//! （`card::root`）の直下へ「`link_overlay::root`（メタ行・タイトル・
//! 抜粋・overlay）」と「著者リンク」を**兄弟**として並べ、著者リンクは
//! [`LAYOUT_CSS`] の `[data-blocks-blog-grid-text-author]` セレクタで
//! `position: relative; z-index: 1` を与えて overlay より前面に出す
//! （`overlay` の外に置く著者リンクだけが独立してクリック可能なままになる
//! 契約は変えない）。
//!
//! # `href="#"` を使わない・`base_path` を受け取れない制約
//!
//! [`super::blog_featured_with_list`] モジュール doc「`href="#"` を
//! 使わない・`base_path` を受け取れない制約」節と同じ判断軸で、
//! [`Block::demo`] は `fn() -> Node` のため `base_path` を受け取れず、
//! 記事・著者・「すべての記事を見る」のリンク先はすべて [`REPO`]
//! （実在する GitHub リポジトリへの外部絶対 URL）に固定する。死リンク
//! `href="#"` は使わない。
//!
//! # `<time datetime>` と表示日付の一致
//!
//! [`super::blog_featured_with_list`] と同じく、各記事は機械可読な
//! ISO 8601 日付（`datetime` 属性）と表示用の日本語表記を別々の定数として
//! 持つが、常に同じ日を指す値を組にする。
//!
//! # `demo_class` とグリッド class を分ける理由
//!
//! [`Block::demo_class`]（`blocks-blog-grid-text`）と
//! `.blocks-blog-grid-text-grid`（記事カードのグリッド化）を意図的に
//! 別名にする。同名にすると `crate::blocks::insert_generated_sections` が
//! 付与する Demo ラッパー（子要素 1 個の `div`）にもグリッド class が
//! 当たり、ラッパー自身がグリッドの 1 トラックへ押し込まれて残りが空白に
//! なる（イシュー #2809 で Bugbot が指摘した不具合と同型、
//! [`super::blog_featured_with_list`] モジュール doc「レイアウト」節参照）。
//!
//! # ブレークポイントをリテラルで直書きする理由
//!
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! （CSS custom property は宣言側でのみ有効）ため、
//! `fandhe_frontend_pre_styled_ui::recipe::Breakpoint` の `Md`（768px =
//! 48rem）・`Lg`（1024px = 64rem）と一致するリテラル値を [`LAYOUT_CSS`] へ
//! 直書きする（[`super::blog_featured_with_list`] と同じ判断）。
//!
//! # `drop_class_attr` と CSS フックの選び方
//!
//! `heading::heading` / `badge::badge` / `card::root` / `avatar::root` /
//! `link::root` / `link_overlay::root` はいずれも `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、Demo 固有
//! のスタイルフックは `data-blocks-blog-grid-text-*` 属性で渡し、
//! [`LAYOUT_CSS`] 側も同じ属性セレクタで対応する（`crate::blocks`
//! モジュール doc「CSS フックが `class` と `[data-*]` で混在する理由」節
//! 参照）。素の `div`/`time` には `class` がそのまま効くため、それらは
//! 従来どおり `.blocks-blog-grid-text-*` クラスセレクタを使う。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、セクション見出しは
//! `HeadingLevel::H3`、記事タイトルは `HeadingLevel::H4` を使う（他 block
//! と同じ先例）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言・人名はすべて架空のもので `Post` 構造体の `&'static str`
//! フィールドとして直接持つ（実企業名・実クレデンシャル・PII を含まない）。
//!
//! # `crate::blocks::dummy_assets` を使わない理由（コードフェンス自己完結）
//!
//! `crate::blocks::dummy_assets` は `pub(crate)` のため、Markdown 原稿の
//! Rust コードフェンス（`// blocks-code:begin`/`:end` マーカー内、クレート
//! 外から読めるコード例として提示される）がこれを参照すると単体では
//! コンパイルできなくなる（イシュー #2811 のレビュー指摘）。
//! [`super::blog_featured_with_list`] が同じ理由で `dummy_assets` を
//! 避けているのと同じ判断軸により、著者名・役職・イニシャルは `Post` へ
//! 直接持たせるリテラルとし、アバターは `avatar::fallback` のイニシャル
//! 表示（`ImageStatus::Error`）のみを使う（画像アセットへ依存しない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, p, section, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク先の固定外部 URL（モジュール doc「`href="#"` を使わない」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 記事 1 件分のダミーデータ（架空、実在の人物・企業とは無関係）。著者
/// 情報は `crate::blocks::dummy_assets`（`pub(crate)`）を参照せず、
/// Markdown 原稿のコードフェンスが単体でコンパイルできるようリテラルで
/// 直接持つ（モジュール doc「`crate::blocks::dummy_assets` を使わない
/// 理由」節参照）。
struct Post {
    date_iso: &'static str,
    date_label: &'static str,
    category: &'static str,
    title: &'static str,
    excerpt: &'static str,
    author_name: &'static str,
    author_role: &'static str,
    author_initials: &'static str,
}

/// インスタンス A（左寄せ見出し・上罫線あり）用の記事 3 件（架空）。
const INSTANCE_A: [Post; 3] = [
    Post {
        date_iso: "2026-09-18",
        date_label: "2026年9月18日",
        category: "アーキテクチャ",
        title: "ノード木 API だけで組み立てる合成例の作り方",
        excerpt: "HTML 文字列を直接組み立てず、既存部品を合成するときに気を付けている判断軸を振り返ります。",
        author_name: "高橋 美咲",
        author_role: "フロントエンドエンジニア",
        author_initials: "MT",
    },
    Post {
        date_iso: "2026-09-11",
        date_label: "2026年9月11日",
        category: "セキュリティ",
        title: "既定エスケープだけで守れる範囲を広げる",
        excerpt: "テキスト補間を必ずエスケープ経由にする設計判断が、レビューの負荷をどう下げたかをまとめました。",
        author_name: "中村 悠斗",
        author_role: "セキュリティエンジニア",
        author_initials: "YN",
    },
    Post {
        date_iso: "2026-09-04",
        date_label: "2026年9月4日",
        category: "配布",
        title: "単一バイナリ配布までの最短ルート",
        excerpt: "SSR から単一実行ファイルへ至る構成を、最小手順で振り返ります。",
        author_name: "小林 彩花",
        author_role: "SRE",
        author_initials: "AK",
    },
];

/// インスタンス B（中央揃え見出し・上罫線なし）用の記事 3 件（架空、
/// インスタンス A とは別内容にして区別しやすくする）。
const INSTANCE_B: [Post; 3] = [
    Post {
        date_iso: "2026-08-28",
        date_label: "2026年8月28日",
        category: "テスト",
        title: "XSS 回帰テストを削除せずに保つための工夫",
        excerpt: "SSR/SSG/CSR/WASM の各経路で回帰テストを弱体化させない運用について書きました。",
        author_name: "山本 拓海",
        author_role: "QA エンジニア",
        author_initials: "TY",
    },
    Post {
        date_iso: "2026-08-21",
        date_label: "2026年8月21日",
        category: "CI",
        title: "壁時計時間を優先した CI 並列化の考え方",
        excerpt: "資源の無駄を許容してでも所要時間を優先するときの判断基準を整理します。",
        author_name: "渡辺 さくら",
        author_role: "CI/CD エンジニア",
        author_initials: "SW",
    },
    Post {
        date_iso: "2026-08-14",
        date_label: "2026年8月14日",
        category: "設計",
        title: "依存グラフの上限を機械で守る",
        excerpt: "60 件・深さ 6 という上限を、レビューではなく機械検証で保つ仕組みを紹介します。",
        author_name: "佐々木 陸",
        author_role: "アーキテクト",
        author_initials: "RS",
    },
];

/// `<time datetime>` を組み立てる（モジュール doc「`<time datetime>` と
/// 表示日付の一致」節）。
fn post_date(iso: &str, label: &str) -> Node {
    el(
        "time",
        vec![("class", "blocks-blog-grid-text-date"), ("datetime", iso)],
        vec![text(label)],
    )
}

/// メタ行（日付 + カテゴリ badge）を組み立てる。
fn post_meta(post: &Post) -> Node {
    div(
        vec![("class", "blocks-blog-grid-text-meta")],
        vec![
            post_date(post.date_iso, post.date_label),
            badge::badge(
                &BadgeProps {
                    size: Size::Sm,
                    ..BadgeProps::default()
                },
                vec![("data-blocks-blog-grid-text-badge", "")],
                vec![text(post.category)],
            ),
        ],
    )
}

/// 著者リンク（アバターのイニシャル fallback + 氏名・役職）。`overlay`
/// の外へ兄弟として置くことでクリック可能なまま保つ（モジュール doc
/// 「カード全面リンクと著者リンクを両立する 2 段構成」節参照）。画像
/// アセットへは依存しない（モジュール doc「`crate::blocks::dummy_assets`
/// を使わない理由」節参照）。
fn author(name: &str, role: &str, initials: &str) -> Node {
    link::root(
        REPO,
        &LinkProps::default(),
        vec![("data-blocks-blog-grid-text-author", "")],
        vec![
            avatar::root(
                &AvatarProps {
                    size: Size::Sm,
                    ..AvatarProps::default()
                },
                vec![],
                vec![avatar::fallback(
                    ImageStatus::Error,
                    vec![],
                    vec![text(initials)],
                )],
            ),
            div(
                vec![("class", "blocks-blog-grid-text-author-info")],
                vec![
                    div(
                        vec![("class", "blocks-blog-grid-text-author-name")],
                        vec![text(name)],
                    ),
                    div(
                        vec![("class", "blocks-blog-grid-text-author-role")],
                        vec![text(role)],
                    ),
                ],
            ),
        ],
    )
}

/// 記事カード 1 件（メタ行・タイトル・抜粋・全面 overlay + 独立した著者
/// リンク）を組み立てる。
fn article_card(post: &Post) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-blog-grid-text-card", "")],
        vec![card::body(
            vec![("data-blocks-blog-grid-text-body", "")],
            vec![
                link_overlay::root(
                    vec![("data-blocks-blog-grid-text-article", "")],
                    vec![
                        post_meta(post),
                        heading(
                            HeadingLevel::H4,
                            &HeadingProps::default(),
                            vec![],
                            vec![text(post.title)],
                        ),
                        styled_text::text(
                            &TextProps {
                                variant: TextVariant::Muted,
                                ..TextProps::default()
                            },
                            vec![("data-blocks-blog-grid-text-excerpt", "")],
                            vec![text(post.excerpt)],
                        ),
                        overlay(REPO, vec![("aria-label", post.title)], vec![]),
                    ],
                ),
                author(post.author_name, post.author_role, post.author_initials),
            ],
        )],
    )
}

/// セクション見出し・説明・「すべての記事を見る」リンクからなるヘッダを
/// 組み立てる。`align`/`rule` がそれぞれ「見出しの揃え」「上罫線の有無」
/// バリエーションを表す（モジュール doc「静的な 2 インスタンス併記」
/// 節参照）。
fn section_header(heading_text: &str, description: &str, align: &str, rule: bool) -> Node {
    let mut attrs: Vec<(&str, &str)> = vec![
        ("class", "blocks-blog-grid-text-header"),
        ("data-blocks-blog-grid-text-align", align),
    ];
    if rule {
        attrs.push(("data-blocks-blog-grid-text-rule", ""));
    }
    div(
        attrs,
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text(heading_text)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(description)],
            ),
            link::root(
                REPO,
                &LinkProps::default(),
                vec![],
                vec![text("すべての記事を見る")],
            ),
        ],
    )
}

/// インスタンス 1 件（ヘッダ + 記事グリッド）を組み立てる。
fn section_instance(
    heading_text: &str,
    description: &str,
    align: &str,
    rule: bool,
    posts: &[Post],
) -> Node {
    let cards: Vec<Node> = posts.iter().map(article_card).collect();
    section(
        vec![],
        vec![
            section_header(heading_text, description, align, rule),
            div(vec![("class", "blocks-blog-grid-text-grid")], cards),
        ],
    )
}

/// `blog-grid-text` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数
/// （モジュール doc「静的な 2 インスタンス併記」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-blog-grid-text-stack")],
        vec![
            p(
                vec![("class", "blocks-blog-grid-text-caption")],
                vec![text("左寄せ見出し・上罫線あり")],
            ),
            section_instance(
                "最近の記事",
                "チームが公開した記事から、まだ読んでいないものを見つけてください。",
                "start",
                true,
                &INSTANCE_A,
            ),
            p(
                vec![("class", "blocks-blog-grid-text-caption")],
                vec![text("中央揃え見出し・上罫線なし")],
            ),
            section_instance(
                "ブログ",
                "運用・設計・テストにまつわる記事を集めました。",
                "center",
                false,
                &INSTANCE_B,
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/blog-grid-text/",
    title: "blog-grid-text",
    category: BlockCategory::Blog,
    rust_source: "crates/docs-site/src/blocks/marketing/blog/blog_grid_text.rs",
    demo_class: "blocks-blog-grid-text",
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
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Link Overlay",
            path: "/themes/link-overlay/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `blog_grid_text` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` ではなく
/// 本ファイル内 private 定数として `super::stylesheet` 経由の `push_css` で
/// 連結される）。
///
/// セレクタは `.blocks-blog-grid-text*` と `[data-blocks-blog-grid-text-*]`
/// のみを用い、他 block や部品の素のセレクタへ影響させない
/// （`testimonials_stack` と同じ名前空間分離）。
const LAYOUT_CSS: &str = "\
.blocks-blog-grid-text-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}\n\
.blocks-blog-grid-text-caption {\n  margin: 0;\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n}\n\
.blocks-blog-grid-text-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  margin-block-end: var(--fandhe-space-6);\n}\n\
.blocks-blog-grid-text-header[data-blocks-blog-grid-text-align=\"center\"] {\n  text-align: center;\n  align-items: center;\n}\n\
.blocks-blog-grid-text-header[data-blocks-blog-grid-text-rule] {\n  border-block-start: 1px solid var(--fandhe-color-border);\n  padding-block-start: var(--fandhe-space-6);\n}\n\
.blocks-blog-grid-text-grid {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-8);\n}\n\
[data-blocks-blog-grid-text-card] {\n  height: 100%;\n}\n\
[data-blocks-blog-grid-text-body] {\n  display: flex;\n  flex-direction: column;\n  height: 100%;\n}\n\
[data-blocks-blog-grid-text-article] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  flex: 1 1 auto;\n  min-height: 0;\n}\n\
.blocks-blog-grid-text-meta {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-blog-grid-text-date {\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n}\n\
[data-blocks-blog-grid-text-excerpt] {\n  display: -webkit-box;\n  -webkit-box-orient: vertical;\n  -webkit-line-clamp: 3;\n  line-clamp: 3;\n  overflow: hidden;\n}\n\
[data-blocks-blog-grid-text-author] {\n  position: relative;\n  z-index: 1;\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  margin-top: auto;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n}\n\
.blocks-blog-grid-text-author-name {\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n\
.blocks-blog-grid-text-author-role {\n  color: var(--fandhe-color-fg-muted);\n}\n\
@media (min-width: 48rem) {\n  .blocks-blog-grid-text-grid {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n}\n\
@media (min-width: 64rem) {\n  .blocks-blog-grid-text-grid {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていることの単体
    /// 回帰（`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_dead_links() {
        let html = render(&demo());
        assert!(html.contains(r#"data-scope="heading""#));
        assert!(html.contains(r#"data-scope="text""#));
        assert!(html.contains(r#"data-scope="badge""#));
        assert!(html.contains(r#"data-scope="card""#));
        assert!(html.contains(r#"data-scope="avatar""#));
        assert!(html.contains(r#"data-scope="link""#));
        assert!(html.contains(r#"data-scope="link-overlay""#));
        assert!(html.contains("datetime=\"2026-09-18\""));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("<button"));
    }

    /// overlay の件数（記事 6 件分）とそれぞれの `aria-label` を固定する。
    #[test]
    fn demo_renders_six_overlays_with_aria_label() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"data-part="overlay""#).count(), 6);
        assert_eq!(html.matches("aria-label=").count(), 6);
    }

    /// 見出しの揃え・上罫線バリエーションがそれぞれ 1 インスタンスにのみ
    /// 現れることを固定する。
    #[test]
    fn demo_has_one_centered_instance_and_one_ruled_instance() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-blog-grid-text-align=\"center\"")
                .count(),
            1
        );
        assert_eq!(
            html.matches("data-blocks-blog-grid-text-rule=\"\"").count(),
            1
        );
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件・line-clamp を持つこと。
    #[test]
    fn layout_css_declares_breakpoints_and_line_clamp() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("-webkit-line-clamp: 3"));
    }
}
