//! セクショントップ（`/guides/`・`/api/`・`/examples/`）の索引カードグリッド
//! （イシュー #3616）。
//!
//! # 役割・呼び出し文脈
//!
//! 3 セクションのトップページはこれまで手書き Markdown のリンク集で、
//! `site/nav.toml` の配下ページ一覧と二重管理になっていた。本モジュールは
//! [`crate::page_sections`]（汎用生成節フック、#3598）へ登録する生成関数
//! （`render_*`）と専用 CSS（[`stylesheet`]）を提供し、配下ページを
//! pre-styled-ui の `card` / `heading` / `text` で組んだカードグリッドとして
//! 差し込む。呼び出し元は [`crate::build::build_site_with`] 内の
//! `insert_generated_sections_with`（`render_*`）と、書き出し時の
//! `PageStylesheet::build`（[`stylesheet`]）のみ。
//!
//! # 説明文の供給元（イシューの選択肢からの意図的な逸脱）
//!
//! イシューは説明文を「原稿の最初の段落」または「`nav.toml` の
//! `description`」から取る案を挙げたが、いずれも `PageSection::render`
//! （`fn(&str) -> Vec<Node>`）へ原稿・`Nav` を渡すシグネチャ変更が要り、
//! 同じフックを使う兄弟イシューとの stale base 衝突リスクが高い。そのため
//! フックの API は変えず、本モジュールの定数台帳（[`GUIDES`] /
//! [`API_GROUPS`] / [`EXAMPLES`]）に `path` / `title` / 1 行説明を持たせ、
//! `site/nav.toml` との完全一致を `tests/section_index_nav.rs` が fail-closed
//! で固定する（ドリフトは機械検知される）。説明文の正を `nav.toml` へ一本化
//! する案は後続課題とする。
//!
//! # 構造
//!
//! ```text
//! ul.docs-index-grid
//!   li.docs-index-card            … position: relative
//!     card::root > card::body
//!       heading > a.docs-index-card-link   … ::after で全面クリック化
//!       text(Muted, Sm)                     … 1 行説明（リンク外）
//! ```
//!
//! pre-styled-ui の root は呼び出し側の `class` を破棄するため、`docs-*` class
//! は常にラッパー要素（`ul` / `li` / `a`）へ付ける。カード内は
//! `data-scope="card"` 配下なので TOC・検索テキストから除外される
//! （[`crate::layout`] の既存規則）。API のグループ見出し `h2` は
//! `data-scope` の外に置き、TOC・検索に載せる。
//!
//! # セキュリティ上の不変条件
//!
//! ノード木 API のみで組み、文字列は既定エスケープを通る（`raw_html()` 不使用、
//! REQ-1）。href は [`crate::layout::asset_href`] によるサイト内の絶対パスだけで、
//! 台帳はコンパイル時定数（外部入力を含まない）。JS・`on*=` 属性・`id` は出さない。

use fandhe_frontend_core::{a, div, h2, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::text::{text as text_part, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{StyleSheet, StylesheetError};

use crate::layout::asset_href;

/// 生成節が配線する追加 CSS の出力先（[`crate::page_sections::PAGE_STYLESHEETS`]
/// の `rel_path`）。兄弟のセクション索引（Themes 等）も再利用してよい。
pub const STYLESHEET_REL_PATH: &str = "assets/section-index.css";

/// 索引カード 1 枚分。`path` / `title` は `site/nav.toml` の `page.path` /
/// `page.title` と一致させる（`tests/section_index_nav.rs` が固定）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexCard {
    /// nav のページパス。
    pub path: &'static str,
    /// nav のページタイトル（カード見出し）。
    pub title: &'static str,
    /// 1 行説明（改行を含まない）。
    pub description: &'static str,
}

/// API Reference 索引のグループ（クレート別の区分）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexGroup {
    /// グループ見出し。
    pub title: &'static str,
    /// グループ配下のカード。
    pub cards: &'static [IndexCard],
}

/// `/guides/` 配下ページ（nav 宣言順）。
pub const GUIDES: &[IndexCard] = &[
    IndexCard {
        path: "/guides/component-authoring/",
        title: "コンポーネント記述ガイド",
        description:
            "マクロ DSL に依存せず、純粋な Rust のノード木 API でコンポーネントを記述する方法。",
    },
    IndexCard {
        path: "/guides/embedding-guide/",
        title: "最小埋め込みガイド",
        description: "既存の HTML ページの div へコンポーネントを部分的にマウントする手順。",
    },
    IndexCard {
        path: "/guides/view-transitions/",
        title: "View Transitions",
        description: "クロスドキュメントおよび SPA 内でのビュー遷移を有効化する方法。",
    },
    IndexCard {
        path: "/guides/npm-asset-build/",
        title: "NPM アセットビルド",
        description: "ignore-scripts を既定としたサプライチェーン対策付きの静的アセット取り込み。",
    },
    IndexCard {
        path: "/guides/no-js-ssg/",
        title: "JS ゼロ SSG での利用ガイド",
        description:
            "クライアント JavaScript を読み込まない静的サイトでの部品の振る舞いと代替パターン。",
    },
    IndexCard {
        path: "/guides/wasm-full-features/",
        title: "wasm-full feature 選択ガイド",
        description: "wasm-full の Cargo feature 一覧と、default-features を切る場合の移行手順。",
    },
    IndexCard {
        path: "/guides/pre-styled-ui-motion-feature/",
        title: "pre-styled-ui motion feature ガイド",
        description: "motion feature（既定 off）の有効化手順と、無効時のゼロコスト保証。",
    },
    IndexCard {
        path: "/guides/animation-core/",
        title: "fandhe-animation API ガイド",
        description: "演算コアと Web アダプタを Rust コードから直接呼び出す使い方。",
    },
    IndexCard {
        path: "/guides/animation/",
        title: "アニメーション機能ガイド",
        description: "data 属性を書くだけで動く宣言的アニメーション機能を機能別に解説。",
    },
    IndexCard {
        path: "/guides/deployment/",
        title: "デプロイガイド",
        description: "SSG と単一実行ファイル配布の使い分け、および Vercel へのデプロイ方法。",
    },
];

/// `/examples/` 配下ページ（nav 宣言順）。説明は `docs/guides/examples.md`
/// の比較表「目的」列と揃える。
pub const EXAMPLES: &[IndexCard] = &[
    IndexCard {
        path: "/examples/ssr-routing/",
        title: "ssr-routing",
        description: "Loader・respond_with・Router による SSR ページの構築。",
    },
    IndexCard {
        path: "/examples/ssg-blog/",
        title: "ssg-blog",
        description: "generate_pages による静的ブログ（sitemap.xml / robots.txt 付き）の書き出し。",
    },
    IndexCard {
        path: "/examples/dist-server-docker/",
        title: "dist-server-docker",
        description: "単一バイナリ配布と Docker イメージでのデプロイ。",
    },
    IndexCard {
        path: "/examples/interactive-view-transitions/",
        title: "interactive-view-transitions",
        description: "クライアント側状態管理と View Transitions の実演。",
    },
    IndexCard {
        path: "/examples/headless-pre-styled-ui/",
        title: "headless-pre-styled-ui",
        description: "Primitives / Themes 2 層 UI コンポーネントのショーケース。",
    },
    IndexCard {
        path: "/examples/wireframe-ui/",
        title: "wireframe-ui",
        description: "ローファイ・モノクロのワイヤーフレーム UI 全 49 部品のショーケース。",
    },
    IndexCard {
        path: "/examples/vercel-ssg/",
        title: "vercel-ssg",
        description: "SSG から Vercel Build Output API を経由した静的配置。",
    },
    IndexCard {
        path: "/examples/vercel-ssr/",
        title: "vercel-ssr",
        description: "Vercel Container Images（Beta）上でのリクエスト時 SSR。",
    },
];

/// `/api/` 配下ページ（クレート別 6 グループ。`site/api.md` の旧区分と同じ）。
pub const API_GROUPS: &[IndexGroup] = &[
    IndexGroup {
        title: "core（描画コア）",
        cards: &[IndexCard {
            path: "/api/component-api/",
            title: "コンポーネント記述 API",
            description: "ノード木 API・既定エスケープなど描画コアの公開 API と呼び出し規約。",
        }],
    },
    IndexGroup {
        title: "app / server（アプリ構築・ルーティング）",
        cards: &[
            IndexCard {
                path: "/api/app-api/",
                title: "fandhe-frontend-app API",
                description: "Loader・束縛点など、モード非依存の共通コンポーネント層の API。",
            },
            IndexCard {
                path: "/api/router-path-matching/",
                title: "ルーター パスマッチング",
                description: "Router のパスマッチング規則とパスパラメータの扱い。",
            },
            IndexCard {
                path: "/api/server-api/",
                title: "fandhe-frontend-server SSG API",
                description: "generate_pages・generate_assets など SSG の書き出し API。",
            },
        ],
    },
    IndexGroup {
        title: "interactive（状態管理）",
        cards: &[IndexCard {
            path: "/api/interactive-api/",
            title: "状態管理 API",
            description: "明示的な状態管理コアの公開 API。",
        }],
    },
    IndexGroup {
        title: "wasm（CSR / ハイドレーション）",
        cards: &[
            IndexCard {
                path: "/api/hydration-api/",
                title: "hydrate() API",
                description: "SSR 出力へクライアント側の挙動を結び付ける hydrate の API。",
            },
            IndexCard {
                path: "/api/hydration-state-format/",
                title: "ハイドレーション状態フォーマット",
                description: "SSR からクライアントへ引き渡す状態の直列化フォーマット。",
            },
        ],
    },
    IndexGroup {
        title: "headless-ui",
        cards: &[IndexCard {
            path: "/api/headless-ui-api/",
            title: "fandhe-frontend-headless-ui API",
            description: "anatomy・data 属性・WAI-ARIA を担う headless UI 層の API。",
        }],
    },
    IndexGroup {
        title: "pre-styled-ui",
        cards: &[
            IndexCard {
                path: "/api/pre-styled-ui-api/",
                title: "fandhe-frontend-pre-styled-ui API",
                description: "headless 層の上に載るスタイル済み部品層の API。",
            },
            IndexCard {
                path: "/api/pre-styled-recipe-api/",
                title: "pre-styled-ui slot recipe API",
                description: "スタイル済み部品が使う slot recipe の API。",
            },
        ],
    },
];

/// カード 1 枚（`li.docs-index-card`）を組む。`level` は見出しレベル。
fn card_node(base_path: &str, c: &IndexCard, level: HeadingLevel) -> Node {
    let href = asset_href(base_path, c.path);
    let title = heading(
        level,
        &HeadingProps {
            size: HeadingSize::Md,
            ..HeadingProps::default()
        },
        vec![],
        vec![a(
            vec![("class", "docs-index-card-link"), ("href", href.as_str())],
            vec![text(c.title)],
        )],
    );
    let desc = text_part(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(c.description)],
    );
    li(
        vec![("class", "docs-index-card")],
        vec![card::root(
            CardProps::default(),
            vec![],
            vec![card::body(vec![], vec![title, desc])],
        )],
    )
}

fn grid(base_path: &str, cards: &[IndexCard], level: HeadingLevel) -> Node {
    ul(
        vec![("class", "docs-index-grid")],
        cards
            .iter()
            .map(|c| card_node(base_path, c, level))
            .collect(),
    )
}

/// `/guides/` 用: フラットなカードグリッド。
#[must_use]
pub fn render_guides(base_path: &str) -> Vec<Node> {
    vec![grid(base_path, GUIDES, HeadingLevel::H2)]
}

/// `/examples/` 用: フラットなカードグリッド。
#[must_use]
pub fn render_examples(base_path: &str) -> Vec<Node> {
    vec![grid(base_path, EXAMPLES, HeadingLevel::H2)]
}

/// `/api/` 用: クレート別グループ見出し（`h2`、TOC・検索に載る）+ カードグリッド。
#[must_use]
pub fn render_api(base_path: &str) -> Vec<Node> {
    API_GROUPS
        .iter()
        .map(|g| {
            div(
                vec![("class", "docs-index-group")],
                vec![
                    h2(vec![], vec![text(g.title)]),
                    grid(base_path, g.cards, HeadingLevel::H3),
                ],
            )
        })
        .collect()
}

/// 索引カード専用 CSS（`assets/section-index.css`）。色・余白・角丸は既存の
/// `--fandhe-*` トークンだけを参照し、新しいトークンは作らない。
/// `.docs-content` 配下の typography ミラー（`.docs-content ul/li/a/h2/p`）に
/// 詳細度で勝つため、セレクタは `.docs-content` を前置する。
///
/// # Errors
///
/// [`StyleSheet::push_css`] の検証（`<`・制御文字の拒否）に落ちた場合。
/// 定数のため通常は到達しないが、黙って欠けた CSS を公開しない fail-closed。
pub fn stylesheet() -> Result<StyleSheet, StylesheetError> {
    let mut sheet = StyleSheet::new();
    sheet.push_css(SECTION_INDEX_CSS)?;
    Ok(sheet)
}

const SECTION_INDEX_CSS: &str = "\
.docs-content ul.docs-index-grid {
  list-style: none;
  margin: var(--fandhe-space-6) 0;
  padding: 0;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(min(100%, 15rem), 1fr));
  gap: var(--fandhe-space-4, 1rem);
}
.docs-content .docs-index-group {
  margin-block-end: var(--fandhe-space-6);
}
.docs-content .docs-index-group ul.docs-index-grid {
  margin-block-start: var(--fandhe-space-4, 1rem);
}
.docs-content li.docs-index-card {
  position: relative;
  margin: 0;
  min-width: 0;
}
.docs-content .docs-index-card > [data-scope=\"card\"] {
  height: 100%;
  box-sizing: border-box;
  transition: border-color 0.15s ease, background-color 0.15s ease;
}
.docs-content .docs-index-card:hover > [data-scope=\"card\"] {
  border-color: var(--fandhe-color-accent);
  background-color: var(--fandhe-color-bg-subtle);
}
.docs-content .docs-index-card h2,
.docs-content .docs-index-card h3 {
  margin: 0 0 var(--fandhe-space-1);
  padding: 0;
  border: 0;
  font-size: var(--fandhe-font-font-size-md);
}
.docs-content .docs-index-card p {
  margin: 0;
}
.docs-content a.docs-index-card-link,
.docs-content a.docs-index-card-link:hover {
  color: var(--fandhe-color-fg);
  text-decoration: none;
}
.docs-content a.docs-index-card-link::after {
  content: \"\";
  position: absolute;
  inset: 0;
  border-radius: var(--fandhe-radius-sm);
}
.docs-content a.docs-index-card-link:focus-visible {
  outline: none;
}
.docs-content a.docs-index-card-link:focus-visible::after {
  outline: 2px solid var(--fandhe-color-accent);
  outline-offset: 2px;
}
@media (prefers-reduced-motion: reduce) {
  .docs-content .docs-index-card > [data-scope=\"card\"] {
    transition: none;
  }
}
";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn titles_and_descriptions_are_escaped_by_default() {
        let c = IndexCard {
            path: "/x/",
            title: "<b>&t</b>",
            description: "<script>d</script>",
        };
        let html = render(&card_node("/b", &c, HeadingLevel::H2));
        assert!(!html.contains("<b>") && !html.contains("<script>"));
        assert!(html.contains("&lt;b&gt;&amp;t&lt;/b&gt;"));
        assert!(html.contains("href=\"/b/x/\""));
    }

    #[test]
    fn description_is_a_single_short_line() {
        for c in GUIDES
            .iter()
            .chain(EXAMPLES)
            .chain(API_GROUPS.iter().flat_map(|g| g.cards))
        {
            assert!(!c.description.is_empty() && !c.description.contains('\n'));
            assert!(c.description.chars().count() <= 80, "{}", c.path);
        }
    }

    #[test]
    fn api_groups_use_h2_headings_and_h3_cards() {
        let html: String = render_api("").iter().map(render).collect();
        assert_eq!(
            html.matches("<h2>").count() + html.matches("<h2 ").count(),
            API_GROUPS.len()
        );
        assert!(html.contains("<h3"));
    }

    #[test]
    fn stylesheet_assembles() {
        assert!(stylesheet().unwrap().as_css().contains("docs-index-grid"));
    }
}
