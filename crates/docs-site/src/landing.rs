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
//! （#3613）」で、原稿 `site/index.md` の `## はじめる` がその後ろへ続く。
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
//! - リンク先は nav に実在する内部ページだけ（`#fragment` は改稿で壊れるので使わない）。
//!
//! # セキュリティ上の不変条件
//!
//! ノード木 API だけで組み、`raw_html()` と HTML 文字列の直接組み立ては使わない。
//! すべてのテキストは `text()` 経由で既定エスケープされる。外部リンクは
//! `LinkProps::external` により `rel="noopener noreferrer"` が付く。CSS は
//! `StyleSheet::push_css` の検証（`<` 等の拒否）を通る定数で、`--fandhe-*`
//! トークンだけを参照する。

use fandhe_frontend_core::{code, div, el, h1, h2, li, p, pre, section, text, ul, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

use crate::code_copy;
use crate::layout::{asset_href, REPOSITORY_URL};

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

/// トップページへ差し込む節列。後続イシューはここへ節を追記する。
#[must_use]
pub fn render(base_path: &str) -> Vec<Node> {
    vec![hero(base_path), features(base_path)]
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
  display: grid;\n\
  grid-template-columns: minmax(0, 1fr);\n\
  gap: 1.5rem;\n\
}\n\
\n\
@media (min-width: 768px) {\n\
  .docs-landing .docs-features-grid {\n\
    grid-template-columns: repeat(2, minmax(0, 1fr));\n\
  }\n\
}\n\
\n\
@media (min-width: 1024px) {\n\
  .docs-landing .docs-features-grid {\n\
    grid-template-columns: repeat(3, minmax(0, 1fr));\n\
  }\n\
}\n\
\n\
.docs-landing .docs-feature {\n\
  min-width: 0;\n\
  margin: 0;\n\
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
        assert_eq!(h.matches("<h3").count(), 5);
        let grid = render_node(&features("/fandhe-frontend"));
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
    fn features_are_not_duplicated_in_site_index_md() {
        let md = include_str!("../../../site/index.md");
        assert!(!md.contains("## 特徴"));
        for f in &FEATURES {
            assert!(!md.contains(f.title), "{}", f.title);
        }
    }
}
