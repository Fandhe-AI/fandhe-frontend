//! トップページ（`/`）のヒーロー節とランディング用 CSS（イシュー #3612）。
//!
//! # 役割・呼び出し文脈
//!
//! [`crate::page_sections::PAGE_SECTIONS`] が `/` へ [`render`] を登録し、
//! [`crate::build::build_site_with`] が Markdown 本文の先頭（`Placement::Prepend`）へ
//! 差し込む。同じ登録が [`crate::layout::PageLayout::Landing`] も宣言するため、
//! トップだけが右目次なしの全幅骨格になる。CSS は [`CSS`] を
//! [`crate::site_theme`] が `STRUCTURAL_CSS` の直後に `site.css` へ積む
//! （ヒーローが使う badge / code / link の recipe が `site.css` にしか無いため、
//! 別ファイルにしない）。後続の #3613〜#3615（特徴グリッド・入口カード・
//! コード例と CTA）は `/` の登録を増やせない（`DuplicatePath`）ので、
//! [`render`] の返す節列へ追記して拡張する。
//!
//! # 設計判断
//!
//! - h1 とリード文は pre-styled-ui の `heading` / `text` を使わず core の素の
//!   要素に `docs-hero-*` class を付ける。両部品は `data-scope` を持ち、配下は
//!   検索インデックスと TOC から除外される（`crate::layout` / `crate::search_index`
//!   の既存規則）ため、使うと説明段落が検索から消える。
//! - インストールコマンドは [`crate::code_copy::copy_block`] で #3605 のコピー
//!   機構へ載せる（clipboard 部品は wasm 配線前提で無 JS サイトでは動かない）。
//! - CTA は `<a>` を出す `link::root`。`<a>` 内に `<button>` は置けない。
//!   ボタン風の見た目は `data-docs-hero-cta` 属性で `.docs-hero-actions` 配下から当てる。
//! - 入口カード（イシュー #3614）は `link_overlay` を使わない。設計文書 §4.1 が
//!   `link_overlay` を `SITE_RECIPES` から除外しているため、`a` の `::after` を
//!   カード全面へ伸ばす CSS だけで全面クリックを実現する（`<a>` 内に `<button>`
//!   を置かない）。カード内の heading / text は `data-scope` を持つが、
//!   `li.docs-landing-card` は `search_index` が全文を連結する特例（索引カードと同様）
//!   なので検索対象に残る。stat は検索対象外。節の h2 と導入文は素の要素にして検索対象を残す。
//! - 部品数の指標は `site/nav.toml` を `include_str!` して [`layer_counts`] で数える
//!   （[`crate::page_sections::PageSection`] の `render` は `Nav` を受け取らないため）。
//!   件数 = 層セクション配下の全ページ − 索引ページ。ページ追加へビルド時に追従する。
//!   依存上限は xtask の定数（`crates/xtask/src/check_deps.rs`）と同値を保持し、
//!   一致は `tests/landing_counts.rs` が固定する。
//! - badge のバージョンは CLI の `Cargo.toml` から取り出し（[`cli_version`]）、
//!   CLI のバンプへ自動追随させる。取れなければ badge を出さない。
//!
//! # セキュリティ上の不変条件
//!
//! ノード木 API だけで組み、`raw_html()` と HTML 文字列の直接組み立ては使わない。
//! すべてのテキストは `text()` 経由で既定エスケープされる。外部リンクは
//! `LinkProps::external` により `rel="noopener noreferrer"` が付く。CSS は
//! `StyleSheet::push_css` の検証（`<` 等の拒否）を通る定数で、`--fandhe-*`
//! トークンだけを参照する。

use fandhe_frontend_core::{a, code, div, h1, h2, li, p, pre, section, text, ul, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::text::{text as text_part, TextProps, TextSize, TextVariant};

use crate::code_copy;
use crate::layout::{asset_href, REPOSITORY_URL};
use crate::nav::{parse_nav, Nav};

/// 登録先ページパス（`site/nav.toml` の「はじめに」）。
pub const PATH: &str = "/";

/// インストールコマンド（コピー対象。プロンプト記号は CSS の `::before` で出す）。
pub const INSTALL_COMMAND: &str = "cargo install fandhe-frontend-cli";

/// ランディングが出力する `docs-*` class の一覧。`tests/site_css_contract.rs` が
/// 「全件が `site.css` にセレクタとして存在」「ランディング以外の HTML には出ない」を固定する。
pub const CLASSES: &[&str] = &[
    "docs-landing",
    "docs-hero",
    "docs-hero-meta",
    "docs-hero-title",
    "docs-hero-lead",
    "docs-hero-install",
    "docs-hero-actions",
    "docs-landing-section",
    "docs-landing-section-title",
    "docs-landing-section-lead",
    "docs-landing-cards",
    "docs-landing-card",
    "docs-landing-card-link",
    "docs-landing-stats-section",
    "docs-landing-stats",
    "docs-landing-stat",
];

/// CTA の見た目指定に使う属性名（値は `primary` / `secondary`）。
pub const CTA_ATTR: &str = "data-docs-hero-cta";

const CLI_CARGO_TOML: &str = include_str!("../../cli/Cargo.toml");

/// `fw` CLI の現行バージョンを `[package]` 節から取り出す。
fn cli_version() -> Option<&'static str> {
    cargo_package_version(CLI_CARGO_TOML)
}

/// `Cargo.toml` 本文の `[package]` 節にある `version = "x.y.z"` を返す。
fn cargo_package_version(toml: &'static str) -> Option<&'static str> {
    let mut in_package = false;
    for line in toml.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        if let Some(rest) = line.strip_prefix("version") {
            let value = rest.trim_start().strip_prefix('=')?.trim();
            let value = value.strip_prefix('"')?;
            let end = value.find('"')?;
            let v = &value[..end];
            let ok = !v.is_empty()
                && v.chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '+'));
            return ok.then_some(v);
        }
    }
    None
}

const SITE_NAV_TOML: &str = include_str!("../../../site/nav.toml");

/// 依存パッケージ数の上限（REQ-3）。正本は `crates/xtask/src/check_deps.rs` の
/// `MAX_PACKAGES`（docs-site は xtask に依存できないので値を保持し、一致をテストで固定する）。
pub const DEP_MAX_PACKAGES: usize = 60;

/// 依存の深さの上限（REQ-3）。正本は `crates/xtask/src/check_deps.rs` の `MAX_DEPTH`。
pub const DEP_MAX_DEPTH: usize = 6;

/// 部品数を数える層の台帳（表示名・セクションの `index_path`）。
const LAYERS: &[(&str, &str)] = &[
    ("Primitives", "/primitives/"),
    ("Themes", "/themes/"),
    ("Blocks", "/blocks/"),
    ("Wireframes", "/wireframes/"),
];

/// 入口カード 1 枚分の台帳エントリ。
struct EntryCard {
    title: &'static str,
    description: &'static str,
    /// サイト内パス（先頭 `/`、`asset_href` へ渡すときに先頭の `/` を外す）。
    path: &'static str,
}

/// 「はじめる」の 3 入口（説明は旧 `site/index.md` から移した）。
const ENTRY_CARDS: &[EntryCard] = &[
    EntryCard {
        title: "Getting Started",
        description: "`fw new` でのプロジェクト作成からビルド・ブラウザ確認までを最短経路でたどる入門ガイドです。",
        path: "/getting-started/quickstart/",
    },
    EntryCard {
        title: "Guides",
        description: "コンポーネントの作成方法・既存ページへの部分埋め込み・ビュー遷移など、目的別の実践ガイド群です。",
        path: "/guides/",
    },
    EntryCard {
        title: "API Reference",
        description: "コンポーネント API・ハイドレーション API など、公開 API の仕様リファレンスです。",
        path: "/api/",
    },
];

/// 部品ギャラリー 4 層への入口。
const GALLERY_CARDS: &[EntryCard] = &[
    EntryCard {
        title: "Primitives",
        description: "anatomy・WAI-ARIA・data 属性だけを持つ headless UI 部品です。",
        path: "/primitives/",
    },
    EntryCard {
        title: "Themes",
        description: "Primitives の上に recipe を載せたスタイル済み部品です。",
        path: "/themes/",
    },
    EntryCard {
        title: "Blocks",
        description: "既存部品を合成した画面単位の例です。",
        path: "/blocks/",
    },
    EntryCard {
        title: "Wireframes",
        description: "ローファイ・モノクロの SSR 専用ワイヤーフレーム部品です。",
        path: "/wireframes/",
    },
];

/// ランディングが出す内部リンクのサイト内パス一覧（ヒーローの CTA 込み）。
/// 実サイトのリンク検証・テスト fixture の遷移先生成が参照する。
pub fn internal_link_paths() -> impl Iterator<Item = &'static str> {
    // ヒーローの CTA（クイックスタート）は入口カードの先頭と同じ遷移先なので重複させない。
    ENTRY_CARDS
        .iter()
        .chain(GALLERY_CARDS.iter())
        .map(|c| c.path)
}

/// 層ごとの部品数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerCount {
    /// 表示名。
    pub label: &'static str,
    /// 層セクションの索引パス。
    pub index_path: &'static str,
    /// 索引ページを除いたページ数。
    pub count: usize,
}

/// `nav` から層ごとの部品数（セクション配下の全ページ − 索引ページ）を数える。
/// `nav` に存在しない層は結果から外す。
#[must_use]
pub fn layer_counts(nav: &Nav) -> Vec<LayerCount> {
    LAYERS
        .iter()
        .filter_map(|(label, index_path)| {
            let section = nav.sections.iter().find(|s| s.index_path == *index_path)?;
            let count = section
                .all_pages()
                .filter(|pg| pg.path != *index_path)
                .count();
            Some(LayerCount {
                label,
                index_path,
                count,
            })
        })
        .collect()
}

/// 同梱の `site/nav.toml` から数えた層ごとの部品数。パース失敗は `None`。
#[must_use]
pub fn site_layer_counts() -> Option<Vec<LayerCount>> {
    parse_nav(SITE_NAV_TOML).ok().map(|nav| layer_counts(&nav))
}

/// トップページへ差し込む節列。後続イシューはここへ節を追記する。
#[must_use]
pub fn render(base_path: &str) -> Vec<Node> {
    let mut nodes = vec![hero(base_path)];
    // #3613 の特徴グリッドはここ（ヒーローの直後）へ挿入する。
    nodes.extend(entry_and_stats(base_path));
    nodes
}

/// 入口カード 2 節と数値指標節。
fn entry_and_stats(base_path: &str) -> Vec<Node> {
    let mut out = vec![
        cards_section(
            base_path,
            "はじめる",
            "目的に応じて、以下の 3 つの入口から進んでください。",
            ENTRY_CARDS,
        ),
        cards_section(
            base_path,
            "部品ギャラリー",
            "3 層の UI 部品とワイヤーフレームを、動くデモ付きで確認できます。",
            GALLERY_CARDS,
        ),
    ];
    if let Some(counts) = site_layer_counts() {
        out.push(stats_section(&counts));
    }
    out
}

/// 見出し・導入文・カード列からなる節。h2 と導入文は素の要素で検索対象に残す。
fn cards_section(base_path: &str, title: &str, lead: &str, cards: &[EntryCard]) -> Node {
    let items: Vec<Node> = cards.iter().map(|c| card_node(base_path, c)).collect();
    section(
        vec![("class", "docs-landing-section")],
        vec![
            h2(
                vec![("class", "docs-landing-section-title")],
                vec![text(title)],
            ),
            p(
                vec![("class", "docs-landing-section-lead")],
                vec![text(lead)],
            ),
            ul(vec![("class", "docs-landing-cards")], items),
        ],
    )
}

/// 全面クリック可能なカード 1 枚（`a` の `::after` を CSS で伸ばす）。
fn card_node(base_path: &str, c: &EntryCard) -> Node {
    let href = asset_href(base_path, c.path.trim_start_matches('/'));
    li(
        vec![("class", "docs-landing-card")],
        vec![card::root(
            CardProps::default(),
            vec![],
            vec![card::body(
                vec![],
                vec![
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Md,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![a(
                            vec![("class", "docs-landing-card-link"), ("href", href.as_str())],
                            vec![text(c.title)],
                        )],
                    ),
                    text_part(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(c.description)],
                    ),
                ],
            )],
        )],
    )
}

/// 数値指標節（部品数 4 件と依存上限 2 件）。
fn stats_section(counts: &[LayerCount]) -> Node {
    let mut items: Vec<Node> = counts
        .iter()
        .map(|c| {
            stat_item(
                &format!("{} の部品数", c.label),
                &c.count.to_string(),
                "件",
                None,
            )
        })
        .collect();
    items.push(stat_item(
        "依存パッケージ数の上限",
        &DEP_MAX_PACKAGES.to_string(),
        "件",
        Some("標準サーバー構成"),
    ));
    items.push(stat_item(
        "依存の深さの上限",
        &DEP_MAX_DEPTH.to_string(),
        "段",
        Some("標準サーバー構成"),
    ));
    section(
        vec![("class", "docs-landing-stats-section")],
        vec![
            h2(
                vec![("class", "docs-landing-section-title")],
                vec![text("数字で見る fandhe-frontend")],
            ),
            ul(vec![("class", "docs-landing-stats")], items),
        ],
    )
}

fn stat_item(label: &str, value: &str, unit: &str, help: Option<&str>) -> Node {
    // dl 直下に置けるのは dt / dd のみなので、単位と補足は dd（value_text）の内側へ入れる。
    let mut value_children = vec![text(value), stat::value_unit(vec![], vec![text(unit)])];
    if let Some(h) = help {
        value_children.push(stat::help_text(vec![], vec![text(h)]));
    }
    let children = vec![
        stat::label(vec![], vec![text(label)]),
        stat::value_text(vec![], value_children),
    ];
    li(
        vec![("class", "docs-landing-stat")],
        vec![stat::root(Size::Md, vec![], children)],
    )
}

/// ヒーロー節（badge・h1・リード文・インストールコマンド・CTA 2 件）。
fn hero(base_path: &str) -> Node {
    let mut children: Vec<Node> = Vec::new();
    if let Some(version) = cli_version() {
        children.push(div(
            vec![("class", "docs-hero-meta")],
            vec![badge(
                &BadgeProps::default(),
                vec![],
                vec![text(format!("fandhe-frontend-cli v{version}"))],
            )],
        ));
    }
    children.push(h1(
        vec![("class", "docs-hero-title")],
        vec![text("既定で安全な、Rust 製フロントエンドフレームワーク")],
    ));
    children.push(p(vec![("class", "docs-hero-lead")], lead_children()));
    children.push(div(
        vec![("class", "docs-hero-install")],
        vec![code_copy::copy_block(pre(
            vec![],
            vec![code(vec![], vec![text(INSTALL_COMMAND)])],
        ))],
    ));
    let quickstart = asset_href(base_path, "getting-started/quickstart/");
    children.push(div(
        vec![("class", "docs-hero-actions")],
        vec![
            link::root(
                &quickstart,
                &LinkProps::default(),
                vec![(CTA_ATTR, "primary")],
                vec![text("クイックスタート")],
            ),
            link::root(
                REPOSITORY_URL,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![(CTA_ATTR, "secondary")],
                vec![text("GitHub")],
            ),
        ],
    ));
    section(vec![("class", "docs-hero")], children)
}

/// 旧 `site/index.md` 冒頭段落をそのまま移したリード文（二重に持たない）。
fn lead_children() -> Vec<Node> {
    vec![
        code(vec![], vec![text("fandhe-frontend")]),
        text(
            " は Rust 製のフロントエンドフレームワークです。AI 時代のセキュリティリスク低減を目的に、テキスト補間の既定エスケープ・",
        ),
        code(vec![], vec![text("unsafe")]),
        text(" の排除（"),
        code(vec![], vec![text("core")]),
        text(" / "),
        code(vec![], vec![text("interactive")]),
        text(" は "),
        code(vec![], vec![text("forbid(unsafe_code)")]),
        text(
            "）・依存クレート数の上限管理を製品仕様として固定しています。プレーンな HTML / JavaScript / CSS を尊重しながら、SSR・SPA・SSG・ビュー遷移といったモダンな機能を単一のフレームワークで網羅し、単一実行ファイルでのデプロイ（Docker 想定）まで見据えています。",
        ),
    ]
}

/// ランディング骨格とヒーローの CSS（`STRUCTURAL_CSS` の直後に積む）。
///
/// `.docs-container.docs-landing`（詳細度 0,2,0）で標準骨格の grid 指定を後出しで解く。
/// サイドバーは DOM に残し、768px 以上でだけ隠す（768px 未満はヘッダーナビが
/// 非表示で、サイドバーの Menu トグルが唯一のナビ手段）。
pub const CSS: &str = "\
/*\n\
 * ---- トップのランディング骨格とヒーロー（イシュー #3612、`crate::landing`） ----\n\
 */\n\
\n\
.docs-container.docs-landing {\n\
  display: block;\n\
}\n\
\n\
@media (min-width: 768px) {\n\
  .docs-landing .docs-sidebar {\n\
    display: none;\n\
  }\n\
\n\
  .docs-landing .docs-main {\n\
    padding: 1.5rem 2rem 4rem;\n\
  }\n\
}\n\
\n\
.docs-landing .docs-content {\n\
  max-width: none;\n\
}\n\
\n\
.docs-hero {\n\
  max-width: 52rem;\n\
  margin: 0 auto;\n\
  padding: 1.5rem 0 2rem;\n\
  text-align: center;\n\
}\n\
\n\
.docs-hero-meta {\n\
  margin-bottom: 1rem;\n\
}\n\
\n\
.docs-landing .docs-hero-title {\n\
  margin: 0 0 1rem;\n\
  padding: 0;\n\
  border: 0;\n\
  font-size: var(--fandhe-font-font-size-2xl);\n\
  font-weight: var(--fandhe-font-font-weight-semibold);\n\
  line-height: 1.25;\n\
  text-wrap: balance;\n\
  letter-spacing: -0.01em;\n\
  color: var(--fandhe-color-fg);\n\
}\n\
\n\
.docs-landing .docs-hero-lead {\n\
  margin: 0 auto 1.5rem;\n\
  max-width: 44rem;\n\
  font-size: var(--fandhe-font-font-size-md);\n\
  line-height: 1.7;\n\
  color: var(--fandhe-color-fg-muted);\n\
  text-align: left;\n\
}\n\
\n\
.docs-hero-install {\n\
  max-width: 28rem;\n\
  margin: 0 auto 1.5rem;\n\
  text-align: left;\n\
}\n\
\n\
.docs-hero .docs-code-block pre {\n\
  margin: 0;\n\
  /* 狭幅でコピーボタンの下へ潜らないよう折り返す（コピー内容は不変）。 */\n\
  white-space: pre-wrap;\n\
  overflow-wrap: anywhere;\n\
}\n\
\n\
.docs-hero-install code::before {\n\
  content: \"$ \";\n\
  color: var(--fandhe-color-fg-muted);\n\
  user-select: none;\n\
}\n\
\n\
.docs-hero-actions {\n\
  display: flex;\n\
  flex-direction: column;\n\
  gap: 0.75rem;\n\
  align-items: stretch;\n\
}\n\
\n\
.docs-hero-actions [data-scope=\"link\"] {\n\
  display: inline-flex;\n\
  align-items: center;\n\
  justify-content: center;\n\
  padding: 0.6rem 1.25rem;\n\
  border: 1px solid var(--fandhe-color-border);\n\
  border-radius: var(--fandhe-radius-sm);\n\
  font-weight: var(--fandhe-font-font-weight-semibold);\n\
  text-decoration: none;\n\
  color: var(--fandhe-color-fg);\n\
  background: var(--fandhe-color-bg);\n\
}\n\
\n\
.docs-hero-actions [data-scope=\"link\"]:hover {\n\
  border-color: var(--fandhe-color-accent);\n\
}\n\
\n\
.docs-hero-actions [data-scope=\"link\"][data-docs-hero-cta=\"primary\"] {\n\
  color: var(--fandhe-color-bg);\n\
  background: var(--fandhe-color-accent);\n\
  border-color: var(--fandhe-color-accent);\n\
}\n\
\n\
.docs-hero-actions [data-scope=\"link\"]:focus-visible {\n\
  outline: 2px solid var(--fandhe-color-accent);\n\
  outline-offset: 2px;\n\
}\n\
\n\
@media (min-width: 640px) {\n\
  .docs-hero-actions {\n\
    flex-direction: row;\n\
    justify-content: center;\n\
  }\n\
}\n\
\n\
@media (min-width: 1024px) {\n\
  .docs-landing .docs-hero-title {\n\
    font-size: var(--fandhe-font-font-size-3xl);\n\
  }\n\
}\n\
/*\n\
 * ---- 入口カードと数値指標（イシュー #3614） ----\n\
 */\n\
\n\
.docs-landing .docs-landing-section,\n\
.docs-landing .docs-landing-stats-section {\n\
  max-width: 64rem;\n\
  margin: 0 auto;\n\
  padding: 1.5rem 0;\n\
}\n\
\n\
.docs-landing .docs-landing-section-title {\n\
  margin: 0 0 0.5rem;\n\
  padding: 0;\n\
  border: 0;\n\
  font-size: var(--fandhe-font-font-size-xl);\n\
  font-weight: var(--fandhe-font-font-weight-semibold);\n\
  color: var(--fandhe-color-fg);\n\
}\n\
\n\
.docs-landing .docs-landing-section-lead {\n\
  margin: 0 0 1rem;\n\
  color: var(--fandhe-color-fg-muted);\n\
}\n\
\n\
.docs-landing ul.docs-landing-cards,\n\
.docs-landing ul.docs-landing-stats {\n\
  display: grid;\n\
  gap: 1rem;\n\
  margin: 0;\n\
  padding: 0;\n\
  list-style: none;\n\
}\n\
\n\
.docs-landing ul.docs-landing-cards {\n\
  grid-template-columns: repeat(auto-fill, minmax(min(100%, 15rem), 1fr));\n\
}\n\
\n\
.docs-landing ul.docs-landing-stats {\n\
  grid-template-columns: repeat(2, minmax(0, 1fr));\n\
}\n\
\n\
.docs-landing .docs-landing-card,\n\
.docs-landing .docs-landing-stat {\n\
  position: relative;\n\
  min-width: 0;\n\
  margin: 0;\n\
  overflow-wrap: anywhere;\n\
}\n\
\n\
.docs-landing .docs-landing-card > [data-scope=\"card\"],\n\
.docs-landing .docs-landing-stat > dl {\n\
  height: 100%;\n\
  border: 1px solid var(--fandhe-color-border);\n\
  border-radius: var(--fandhe-radius-sm);\n\
  background: var(--fandhe-color-bg);\n\
  transition: border-color 0.15s ease;\n\
}\n\
\n\
.docs-landing .docs-landing-stat > dl {\n\
  padding: 1rem;\n\
}\n\
\n\
.docs-landing .docs-landing-card:hover > [data-scope=\"card\"] {\n\
  border-color: var(--fandhe-color-accent);\n\
}\n\
\n\
.docs-landing .docs-landing-card-link {\n\
  color: inherit;\n\
  text-decoration: none;\n\
}\n\
\n\
.docs-landing .docs-landing-card-link::after {\n\
  content: \"\";\n\
  position: absolute;\n\
  inset: 0;\n\
}\n\
\n\
.docs-landing .docs-landing-card-link:focus-visible {\n\
  outline: none;\n\
}\n\
\n\
.docs-landing .docs-landing-card-link:focus-visible::after {\n\
  outline: 2px solid var(--fandhe-color-accent);\n\
  outline-offset: 2px;\n\
  border-radius: var(--fandhe-radius-sm);\n\
}\n\
\n\
@media (min-width: 640px) {\n\
  .docs-landing ul.docs-landing-stats {\n\
    grid-template-columns: repeat(3, minmax(0, 1fr));\n\
  }\n\
}\n\
\n\
@media (min-width: 1024px) {\n\
  .docs-landing ul.docs-landing-stats {\n\
    grid-template-columns: repeat(6, minmax(0, 1fr));\n\
  }\n\
}\n\
\n\
@media (prefers-reduced-motion: reduce) {\n\
  .docs-landing .docs-landing-card > [data-scope=\"card\"] {\n\
    transition: none;\n\
  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render as render_node;

    fn html() -> String {
        render("/fandhe-frontend").iter().map(render_node).collect()
    }

    #[test]
    fn css_attribute_selectors_are_well_formed() {
        // 属性セレクタの `[` `]` と引用符の対応を固定する（不正なルールはブラウザに丸ごと捨てられる）。
        assert!(!CSS.contains("'\"") && !CSS.contains("\"'"), "stray quote");
        assert_eq!(CSS.matches('[').count(), CSS.matches(']').count());
        assert_eq!(CSS.matches('"').count() % 2, 0);
    }

    #[test]
    fn cli_version_matches_cli_manifest() {
        let v = cli_version().expect("cli version");
        assert!(CLI_CARGO_TOML.contains(&format!("\nversion = \"{v}\"")));
        assert!(v.chars().next().unwrap().is_ascii_digit());
    }

    #[test]
    fn version_parser_ignores_other_tables_and_rejects_odd_values() {
        assert_eq!(
            cargo_package_version(
                "[dependencies]\nversion = \"9\"\n[package]\nname = \"x\"\nversion = \"1.2.3\"\n"
            ),
            Some("1.2.3")
        );
        assert_eq!(
            cargo_package_version("[package]\nversion = \"a<b\"\n"),
            None
        );
        assert_eq!(
            cargo_package_version("[dependencies]\nversion = \"1\"\n"),
            None
        );
    }

    #[test]
    fn hero_has_all_parts() {
        let h = html();
        for class in CLASSES.iter().filter(|c| **c != "docs-landing") {
            assert!(h.contains(&format!("class=\"{class}\"")), "{class}");
        }
        assert!(h.contains("fandhe-frontend-cli v"));
        assert!(h.contains(INSTALL_COMMAND));
        assert!(h.contains("docs-code-copy"));
        assert!(h.contains("href=\"/fandhe-frontend/getting-started/quickstart/\""));
        assert!(h.contains("rel=\"noopener noreferrer\""));
        assert_eq!(h.matches("<h1").count(), 1);
    }

    fn entry_html() -> String {
        entry_and_stats("/fandhe-frontend")
            .iter()
            .map(render_node)
            .collect()
    }

    #[test]
    fn entry_cards_link_with_base_path_and_stats_present() {
        let h = html();
        for path in internal_link_paths() {
            assert!(
                h.contains(&format!("href=\"/fandhe-frontend{path}\"")),
                "{path}"
            );
        }
        assert_eq!(h.matches("class=\"docs-landing-stat\"").count(), 6);
        assert_eq!(h.matches("<h1").count(), 1);
        assert!(!entry_html().contains(" id=\""));
    }

    #[test]
    fn section_headings_live_outside_data_scope() {
        let h = entry_html();
        assert!(h.contains("<h2 class=\"docs-landing-section-title\">"));
    }

    #[test]
    fn stat_unit_lives_inside_value_dd() {
        let h = render_node(&stat_item("ラベル", "60", "件", Some("補足")));
        let dd_end = h.find("</dd>").expect("dd");
        assert!(h[..dd_end].contains("件"), "{h}");
        assert!(h[..dd_end].contains("補足"), "{h}");
        assert!(h.ends_with("</dd></dl></li>"), "{h}");
    }

    #[test]
    fn site_counts_cover_all_four_layers() {
        let counts = site_layer_counts().expect("nav parses");
        assert_eq!(counts.len(), 4);
        assert!(counts.iter().all(|c| c.count > 0));
    }
}
