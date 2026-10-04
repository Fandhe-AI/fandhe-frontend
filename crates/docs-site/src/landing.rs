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
//! [`render`] の返す節列へ追記して拡張する。節の並びは「ヒーロー → 特徴グリッド
//! （#3613）→ コード例 → 締めの CTA（#3615）」で、原稿 `site/index.md` の
//! `## はじめる` がその後ろへ続く。
//!
//! - コード例（#3615）は `snippets/landing_ssr.rs` を [`SSR_SNIPPET`] として
//!   `include_str!` し、同じファイルを `tests/landing_snippet.rs` が実コンパイルする。
//!   CTA は `<a>` の 2 件（クイックスタート / ガイド一覧）で、ボタン風の見た目は
//!   ヒーローと同じ `data-docs-hero-cta` 属性で当てる。
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
//! - badge のバージョンは CLI の `Cargo.toml` から取り出し（[`cli_version`]）、
//!   CLI のバンプへ自動追随させる。取れなければ badge を出さない。
//!
//! - 特徴グリッド（#3613）の文言は [`FEATURES`] が唯一の正で、`site/index.md` には
//!   持たない（二重管理の再発は単体テストが検知する）。カードは `card::root` の
//!   中に icon・h3 見出し（`link::root` を内包）・説明文を置く。`link_overlay` の
//!   recipe は `site.css` に積まない規則のため、全面クリックは
//!   `[data-scope="link"]::after` の絶対配置で実現する（`card::root` が
//!   `position: relative`）。`card` は `data-scope` を持つので、カード内の見出しと
//!   説明文は検索インデックスから外れる。5 項目の要旨は索引されるヒーローの
//!   リード文に含まれるため許容する。節見出し h2 は `data-scope` の外に置く。
//! - 入口カード（イシュー #3614）は `link_overlay` を使わない。設計文書 §4.1 が
//!   `link_overlay` を `SITE_RECIPES` から除外しているため、`a` の `::after` を
//!   カード全面へ伸ばす CSS だけで全面クリックを実現する（`<a>` 内に `<button>`
//!   を置かない）。カード内の heading / text は `data-scope` を持つが、
//!   `li.docs-landing-card` は `search_index` が全文を連結する特例（索引カードと同様）
//!   なので検索対象に残る。stat は検索対象外。節の h2 と導入文は素の要素にして検索対象を残す。
//! - 部品数の指標は `site/nav.toml` を `include_str!` して [`layer_counts`] で数える
//!   （本関数の公開シグネチャが `base_path` のみを受け取るため。登録表側は #3700 で `Nav` を渡す形になったが、本関数は据え置き）。
//!   件数 = 層セクション配下の全ページ − 索引ページ。ページ追加へビルド時に追従する。
//!   依存上限は xtask の定数（`crates/xtask/src/check_deps.rs`）と同値を保持し、
//!   一致は `tests/landing_counts.rs` が固定する。
//! - リンク先は nav に実在する内部ページだけ（`#fragment` は改稿で壊れるので使わない）。
//!
//! # セキュリティ上の不変条件
//!
//! ノード木 API だけで組み、`raw_html()` と HTML 文字列の直接組み立ては使わない。
//! すべてのテキストは `text()` 経由で既定エスケープされる。外部リンクは
//! `LinkProps::external` により `rel="noopener noreferrer"` が付く。CSS は
//! `StyleSheet::push_css` の検証（`<` 等の拒否）を通る定数で、`--fandhe-*`
//! トークンだけを参照する。

use fandhe_frontend_core::{a, code, div, el, h1, h2, li, p, pre, section, text, ul, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

use crate::code_copy;
use crate::highlight;
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
    "docs-features",
    "docs-features-title",
    "docs-features-grid",
    "docs-feature",
    "docs-code-example",
    "docs-code-example-title",
    "docs-code-example-lead",
    "docs-code-example-body",
    "docs-code-example-source",
    "docs-cta",
    "docs-cta-title",
    "docs-cta-lead",
    "docs-cta-actions",
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
    let mut nodes = vec![hero(base_path), features(base_path)];
    nodes.extend(entry_and_stats(base_path));
    nodes.push(code_example(base_path));
    nodes.push(closing_cta(base_path));
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

/// トップに表示する最小 SSR コード例の本文（#3615）。
///
/// `tests/landing_snippet.rs` が同じファイルを `#[path]` で実コンパイル・実行するため、
/// 表示と検証は構造上ドリフトしない（API 変更はそのテストのコンパイルエラーで検知される）。
pub const SSR_SNIPPET: &str = include_str!("../snippets/landing_ssr.rs");

/// コード例節（h2 + 説明 + ハイライト済みコピー付きコード + 完全なサンプルへのリンク）。
///
/// 表示文字列は末尾改行だけを落とす（末尾の空行を出さないため。それ以外は不変）。
/// ハイライトのトークンは最終的にすべて `text()` を通るので既定エスケープを保つ。
fn code_example(base_path: &str) -> Node {
    let src = SSR_SNIPPET.trim_end_matches('\n');
    let children = highlight::highlight_children(src, "rust").unwrap_or_else(|| vec![text(src)]);
    let sample = asset_href(base_path, "examples/ssr-routing/");
    section(
        vec![("class", "docs-code-example")],
        vec![
            h2(
                vec![("class", "docs-code-example-title")],
                vec![text("最小の SSR")],
            ),
            p(
                vec![("class", "docs-code-example-lead")],
                vec![text(
                    "ノード木 API だけで HTML 文書を組み立てます。補間する値は既定でエスケープされます。",
                )],
            ),
            div(
                vec![("class", "docs-code-example-body")],
                vec![code_copy::copy_block(pre(
                    vec![],
                    vec![code(vec![("class", "language-rust")], children)],
                ))],
            ),
            p(
                vec![("class", "docs-code-example-source")],
                vec![link::root(
                    &sample,
                    &LinkProps::default(),
                    vec![],
                    vec![text("完全なサンプル（ssr-routing）")],
                )],
            ),
        ],
    )
}

/// 締めの CTA 節（クイックスタートとガイド一覧の 2 件。`<a>` のみで JS 不要）。
///
/// `card` で包まないのは、`data-scope` 配下が検索インデックスから外れるため（ヒーローと同じ判断）。
fn closing_cta(base_path: &str) -> Node {
    let quickstart = asset_href(base_path, "getting-started/quickstart/");
    let guides = asset_href(base_path, "guides/");
    section(
        vec![("class", "docs-cta")],
        vec![
            h2(
                vec![("class", "docs-cta-title")],
                vec![text("はじめましょう")],
            ),
            p(
                vec![("class", "docs-cta-lead")],
                vec![text("数分で最初のページを描画できます。")],
            ),
            div(
                vec![("class", "docs-cta-actions")],
                vec![
                    link::root(
                        &quickstart,
                        &LinkProps::default(),
                        vec![(CTA_ATTR, "primary")],
                        vec![text("クイックスタート")],
                    ),
                    link::root(
                        &guides,
                        &LinkProps::default(),
                        vec![(CTA_ATTR, "secondary")],
                        vec![text("ガイド一覧")],
                    ),
                ],
            ),
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
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        inline_code_nodes(c.description),
                    ),
                ],
            )],
        )],
    )
}

/// 説明文中のバッククォート区間を `code` ノードへ、それ以外を `text` ノードへ変換する。
/// 旧 Markdown 由来の `` `fw new` `` がリテラルのバッククォートとして出ないようにする。
/// すべて `text()` 経由のためエスケープは保たれる。
fn inline_code_nodes(src: &str) -> Vec<Node> {
    src.split('`')
        .enumerate()
        .filter(|(_, seg)| !seg.is_empty())
        .map(|(i, seg)| {
            if i % 2 == 1 {
                code(vec![], vec![text(seg)])
            } else {
                text(seg)
            }
        })
        .collect()
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

/// 説明文の断片。`Code` はインラインコードとして描画する。
#[derive(Clone, Copy)]
enum Seg {
    Text(&'static str),
    Code(&'static str),
}

/// 特徴カード 1 枚の台帳項目。
struct Feature {
    title: &'static str,
    body: &'static [Seg],
    /// `base_path` 配下の内部リンク先（`site/nav.toml` に実在するページ）。
    href_rel: &'static str,
    /// アイコンの `d` 属性（自作の単純な幾何形状）。
    icon_path_d: &'static str,
}

/// 特徴グリッドの文言の唯一の正（`site/index.md` には持たない）。
const FEATURES: [Feature; 5] = [
    Feature {
        title: "既定エスケープ",
        body: &[
            Seg::Text("テキスト補間は必ずエスケープを経由し、迂回は "),
            Seg::Code("raw_html()"),
            Seg::Text(" 等の明示的なオプトイン API のみに限定"),
        ],
        href_rel: "api/component-api/",
        icon_path_d: "M12 3l8 3v6c0 5-3.5 8-8 9-4.5-1-8-4-8-9V6z",
    },
    Feature {
        title: "unsafe の排除",
        body: &[
            Seg::Text("描画コア・状態管理コアは "),
            Seg::Code("#![forbid(unsafe_code)]"),
            Seg::Text(" で "),
            Seg::Code("unsafe"),
            Seg::Text(" を一切使用しない"),
        ],
        href_rel: "api/interactive-api/",
        icon_path_d: "M6 11h12v10H6zM8 11V7a4 4 0 018 0v4",
    },
    Feature {
        title: "依存最小",
        body: &[Seg::Text(
            "標準サーバー構成で依存パッケージ 60 件以内・深さ 6 以内に収め、サプライチェーンの脅威面を抑制",
        )],
        href_rel: "guides/deployment/",
        icon_path_d: "M12 3l9 5-9 5-9-5zM3 16l9 5 9-5",
    },
    Feature {
        title: "プレーン HTML/JS/CSS の尊重",
        body: &[Seg::Text(
            "既存の静的ページへの部分埋め込みからフル機能構成までグラデーションを持つ",
        )],
        href_rel: "guides/embedding-guide/",
        icon_path_d: "M8 6l-6 6 6 6M16 6l6 6-6 6",
    },
    Feature {
        title: "SSR/SPA/SSG/ビュー遷移を網羅",
        body: &[Seg::Text(
            "モードを問わず同じコンポーネント実装を再利用できる",
        )],
        href_rel: "examples/",
        icon_path_d: "M4 8h14l-4-4M20 16H6l4 4",
    },
];

/// 説明文の断片列をノードへ変換する（文言は `text()` で既定エスケープされる）。
fn body_children(segs: &[Seg]) -> Vec<Node> {
    segs.iter()
        .map(|s| match s {
            Seg::Text(t) => text(*t),
            Seg::Code(c) => code(vec![], vec![text(*c)]),
        })
        .collect()
}

/// 装飾アイコン（`label` なしなので `aria-hidden="true"` が付く）。
fn feature_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps {
            size: Size::Md,
            ..IconProps::default()
        },
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

/// 特徴カード 1 枚。`docs-*` class は pre-styled-ui root が破棄するため `li` に付ける。
fn feature_card(base_path: &str, f: &Feature) -> Node {
    let href = asset_href(base_path, f.href_rel);
    li(
        vec![("class", "docs-feature")],
        vec![card::root(
            CardProps::default(),
            vec![],
            vec![card::body(
                vec![],
                vec![
                    feature_icon(f.icon_path_d),
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Md,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![link::root(
                            &href,
                            &LinkProps::default(),
                            vec![],
                            vec![text(f.title)],
                        )],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        body_children(f.body),
                    ),
                ],
            )],
        )],
    )
}

/// 特徴グリッド節（#3613。h2 + 5 枚のカード）。
fn features(base_path: &str) -> Node {
    section(
        vec![("class", "docs-features")],
        vec![
            h2(vec![("class", "docs-features-title")], vec![text("特徴")]),
            ul(
                vec![("class", "docs-features-grid")],
                FEATURES
                    .iter()
                    .map(|f| feature_card(base_path, f))
                    .collect(),
            ),
        ],
    )
}

/// ランディング骨格とヒーローの CSS（`STRUCTURAL_CSS` の直後に積む）。
///
/// `.docs-container.docs-landing`（詳細度 0,2,0）で標準骨格の grid 指定を後出しで解く。
/// サイドバーは DOM に残し、768px 以上でだけ隠す（768px 未満はヘッダーナビが
/// 非表示で、ヘッダーのナビ drawer が全セクションへの手段）。
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
.docs-hero-actions,\n\
.docs-cta-actions {\n\
  display: flex;\n\
  flex-direction: column;\n\
  gap: 0.75rem;\n\
  align-items: stretch;\n\
}\n\
\n\
.docs-hero-actions [data-scope=\"link\"],\n\
.docs-cta-actions [data-scope=\"link\"] {\n\
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
.docs-hero-actions [data-scope=\"link\"]:hover,\n\
.docs-cta-actions [data-scope=\"link\"]:hover {\n\
  border-color: var(--fandhe-color-accent);\n\
}\n\
\n\
.docs-hero-actions [data-scope=\"link\"][data-docs-hero-cta=\"primary\"],\n\
.docs-cta-actions [data-scope=\"link\"][data-docs-hero-cta=\"primary\"] {\n\
  color: var(--fandhe-color-bg);\n\
  background: var(--fandhe-color-accent);\n\
  border-color: var(--fandhe-color-accent);\n\
}\n\
\n\
.docs-hero-actions [data-scope=\"link\"]:focus-visible,\n\
.docs-cta-actions [data-scope=\"link\"]:focus-visible {\n\
  outline: 2px solid var(--fandhe-color-accent);\n\
  outline-offset: 2px;\n\
}\n\
\n\
@media (min-width: 640px) {\n\
  .docs-hero-actions,\n\
  .docs-cta-actions {\n\
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
\n\
/*\n\
 * ---- 特徴グリッド（イシュー #3613） ----\n\
 */\n\
\n\
.docs-features {\n\
  max-width: 72rem;\n\
  margin: 0 auto;\n\
  padding: 2rem 0;\n\
}\n\
\n\
.docs-landing .docs-features-title {\n\
  margin: 0 0 1.5rem;\n\
  padding: 0;\n\
  border: 0;\n\
  text-align: center;\n\
  font-size: var(--fandhe-font-font-size-xl);\n\
  font-weight: var(--fandhe-font-font-weight-semibold);\n\
  color: var(--fandhe-color-fg);\n\
}\n\
\n\
.docs-landing .docs-features-grid {\n\
  list-style: none;\n\
  margin: 0;\n\
  padding: 0;\n\
  display: flex;\n\
  flex-wrap: wrap;\n\
  justify-content: center;\n\
  gap: 1.5rem;\n\
}\n\
\n\
.docs-landing .docs-feature {\n\
  flex: 0 0 100%;\n\
  min-width: 0;\n\
  margin: 0;\n\
}\n\
\n\
@media (min-width: 768px) {\n\
  .docs-landing .docs-feature {\n\
    flex-basis: calc((100% - 1.5rem) / 2);\n\
  }\n\
}\n\
\n\
@media (min-width: 1024px) {\n\
  .docs-landing .docs-feature {\n\
    flex-basis: calc((100% - 3rem) / 3);\n\
  }\n\
}\n\
\n\
.docs-feature [data-scope=\"card\"][data-part=\"root\"] {\n\
  height: 100%;\n\
}\n\
\n\
.docs-feature [data-scope=\"card\"] > div {\n\
  display: flex;\n\
  flex-direction: column;\n\
  align-items: flex-start;\n\
  gap: 0.5rem;\n\
}\n\
\n\
.docs-feature [data-scope=\"heading\"] {\n\
  margin: 0;\n\
}\n\
\n\
/* `.docs-content p`（typography_css）の段落サイズ・下余白に負けないよう 2 クラスで上書きする。 */\n\
.docs-landing .docs-feature [data-scope=\"text\"] {\n\
  margin: 0;\n\
  font-size: var(--fandhe-font-font-size-sm);\n\
}\n\
\n\
.docs-feature [data-scope=\"link\"] {\n\
  text-decoration: none;\n\
  color: var(--fandhe-color-fg);\n\
}\n\
\n\
.docs-feature [data-scope=\"link\"]::after {\n\
  content: \"\";\n\
  position: absolute;\n\
  inset: 0;\n\
}\n\
\n\
.docs-feature:hover [data-scope=\"card\"][data-part=\"root\"] {\n\
  border-color: var(--fandhe-color-accent);\n\
}\n\
\n\
/* 全面オーバーレイの ::after へフォーカスリングを当て、カード全体を囲む。 */\n\
.docs-feature [data-scope=\"link\"]:focus-visible {\n\
  outline: none;\n\
}\n\
\n\
.docs-feature [data-scope=\"link\"]:focus-visible::after {\n\
  outline: 2px solid var(--fandhe-color-accent);\n\
  outline-offset: 2px;\n\
  border-radius: var(--fandhe-radius-sm);\n\
}\n\
\n\
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
  grid-template-columns: repeat(auto-fit, minmax(min(100%, 15rem), 1fr));\n\
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
/* 補足文は数値・単位の横ではなく次の行へ回す（3・6 列の狭い列で途中折り返しさせない）。 */\n\
.docs-landing .docs-landing-stat [data-part=\"value-text\"] {\n\
  flex-wrap: wrap;\n\
}\n\
\n\
.docs-landing .docs-landing-stat [data-part=\"help-text\"] {\n\
  flex: 0 0 100%;\n\
}\n\
\n\
/* `.docs-content h3` / `p` の装飾・段落間隔がカードへ漏れないよう 2 クラスで隔離する。 */\n\
.docs-landing .docs-landing-card [data-scope=\"heading\"] {\n\
  margin: 0 0 var(--fandhe-space-1, 0.25rem);\n\
  padding: 0;\n\
  border: 0;\n\
  font-size: var(--fandhe-font-font-size-md);\n\
  letter-spacing: normal;\n\
}\n\
\n\
.docs-landing .docs-landing-card [data-scope=\"text\"] {\n\
  margin: 0;\n\
  padding: 0;\n\
  border: 0;\n\
}\n\
\n\
@media (prefers-reduced-motion: reduce) {\n\
  .docs-landing .docs-landing-card > [data-scope=\"card\"] {\n\
    transition: none;\n\
  }\n\
}\n\
\n\
/*\n\
 * ---- コード例と締めの CTA（イシュー #3615） ----\n\
 */\n\
\n\
.docs-landing .docs-code-example,\n\
.docs-landing .docs-cta {\n\
  max-width: 52rem;\n\
  margin: 0 auto;\n\
  padding: 2rem 0;\n\
}\n\
\n\
.docs-landing .docs-code-example-title,\n\
.docs-landing .docs-cta-title {\n\
  margin: 0 0 0.75rem;\n\
  padding: 0;\n\
  border: 0;\n\
  text-align: center;\n\
  font-size: var(--fandhe-font-font-size-xl);\n\
  font-weight: var(--fandhe-font-font-weight-semibold);\n\
  color: var(--fandhe-color-fg);\n\
}\n\
\n\
.docs-landing .docs-code-example-lead,\n\
.docs-landing .docs-cta-lead {\n\
  margin: 0 0 1.25rem;\n\
  text-align: center;\n\
  color: var(--fandhe-color-fg-muted);\n\
}\n\
\n\
.docs-code-example-body {\n\
  min-width: 0;\n\
  text-align: left;\n\
}\n\
\n\
.docs-landing .docs-code-example-source {\n\
  margin: 0.75rem 0 0;\n\
  text-align: right;\n\
}\n\
\n\
.docs-landing .docs-cta {\n\
  margin-bottom: 2rem;\n\
  padding: 2rem 1rem;\n\
  text-align: center;\n\
  background: var(--fandhe-color-bg-muted);\n\
  border: 1px solid var(--fandhe-color-border);\n\
  border-radius: var(--fandhe-radius-sm);\n\
}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render as render_node;

    fn html() -> String {
        render("/fandhe-frontend").iter().map(render_node).collect()
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

    #[test]
    fn features_render_five_cards_with_internal_links() {
        let h = html();
        assert_eq!(h.matches("class=\"docs-feature\"").count(), 5);
        assert!(h.matches("data-scope=\"card\"").count() >= 5);
        let grid = render_node(&features("/fandhe-frontend"));
        assert_eq!(grid.matches("<h3").count(), 5);
        assert!(!grid.contains("target=\"_blank\""));
        let mut hrefs: Vec<&str> = FEATURES.iter().map(|f| f.href_rel).collect();
        hrefs.sort_unstable();
        hrefs.dedup();
        assert_eq!(hrefs.len(), 5);
        for f in &FEATURES {
            assert!(h.contains(&format!("href=\"/fandhe-frontend/{}\"", f.href_rel)));
        }
    }

    #[test]
    fn features_section_is_decorative_and_inert() {
        let h = render_node(&features("/fandhe-frontend"));
        assert_eq!(h.matches("aria-hidden=\"true\"").count(), 5);
        assert!(!h.contains("id=\""));
        assert!(!h.contains("<button"));
        assert!(!h.contains("href=\"#\""));
        assert!(!h.contains(" on"));
        assert!(h.contains("<code>raw_html()</code>"));
    }

    #[test]
    fn feature_hrefs_are_real_nav_pages() {
        let raw = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../site/nav.toml"),
        )
        .expect("read nav");
        let nav = crate::nav::parse_nav(&raw).expect("parse nav");
        for f in &FEATURES {
            let path = format!("/{}", f.href_rel);
            assert!(nav.all_pages().any(|p| p.path == path), "{path}");
        }
    }

    #[test]
    fn code_example_and_cta_close_the_page() {
        let nodes = render("/fandhe-frontend");
        let n = nodes.len();
        let last = render_node(&nodes[n - 1]);
        let prev = render_node(&nodes[n - 2]);
        assert!(last.starts_with("<section class=\"docs-cta\""));
        assert!(prev.starts_with("<section class=\"docs-code-example\""));
        for needle in [
            "language-rust",
            "<span class=\"docs-code-lang\">Rust</span>",
            "docs-code-copy\" hidden",
            "hello_page",
        ] {
            assert!(prev.contains(needle), "{needle}");
        }
        assert_eq!(last.matches(&format!("{CTA_ATTR}=\"primary\"")).count(), 1);
        assert_eq!(
            last.matches(&format!("{CTA_ATTR}=\"secondary\"")).count(),
            1
        );
        for h in [&last, &prev] {
            for bad in [
                "target=\"_blank\"",
                "<button type=\"submit",
                "href=\"#\"",
                " on",
                "id=\"",
            ] {
                assert!(!h.contains(bad), "{bad}");
            }
        }
        assert!(!last.contains("<button"));
    }

    #[test]
    fn cta_and_sample_hrefs_are_real_nav_pages() {
        let raw = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../site/nav.toml"),
        )
        .expect("read nav");
        let nav = crate::nav::parse_nav(&raw).expect("parse nav");
        for rel in [
            "getting-started/quickstart/",
            "guides/",
            "examples/ssr-routing/",
        ] {
            let path = format!("/{rel}");
            assert!(nav.all_pages().any(|p| p.path == path), "{path}");
        }
    }

    #[test]
    fn snippet_lines_are_short_and_tab_free() {
        for line in SSR_SNIPPET.lines() {
            assert!(line.chars().count() <= 80, "{line}");
            assert!(!line.contains('\t'));
        }
    }

    #[test]
    fn features_are_not_duplicated_in_site_index_md() {
        let md = include_str!("../../../site/index.md");
        assert!(!md.contains("## 特徴"));
        for f in &FEATURES {
            assert!(!md.contains(f.title), "{}", f.title);
        }
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
