//! `list-sticky-groups` block（イシュー #2924。親トラッキング #2892
//! 「Blocks アプリケーション A」配下、対応表 ID R1293（代表構成）を唯一の
//! 参照元とする合成例）。頭文字ごとにグループ化した人物ディレクトリで、
//! 各グループ見出しがスクロール中にビューポート上端へ貼り付く。取得手段・
//! ファイル名・内部コンポーネント識別子は記載しない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じライセンス
//! 上の転記制限）。
//!
//! # 使用部品
//!
//! `list` / `avatar` / `heading` / `scroll_area` の 4 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、`crates/docs-site/tests/
//! blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # sticky の実体（`position: sticky` のみ、JS 不使用）
//!
//! 各グループ見出し（[`LAYOUT_CSS`] の
//! `[data-blocks-list-sticky-groups-heading]`）へ `position: sticky; top:
//! 0;` を付与する。包含ブロックはグループの `div`（[`group_section`]）
//! なので、見出しはグループの末尾でスクロールアウトし次のグループの見出し
//! に押し出される。スクロールコンテナは `scroll_area::viewport`（
//! `overflow: auto`）で、Demo 枠自体を固定高（20rem）にすることで無 JS でも
//! その場でスクロールして確かめられる（イシュー本文の要件）。
//!
//! # 静的表示（無 JS、架空データ固定）
//!
//! [`GROUPS`] は呼び出しごとに同一の `Node` を生成する純関数（[`demo`]）が
//! 参照するだけの固定配列であり、状態機械・フォーム・データ取得を一切
//! 持たない。ソート・グループ化は行わず、あらかじめ頭文字順に並べた状態で
//! 保持する。
//!
//! # `<form>` を使わない・`id` 属性を持たない理由
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。ボタンも置かない。`id`/`aria-labelledby` を一切出力しないため
//! 宙に浮いた参照・重複は発生しない（グループへは `aria-labelledby` を
//! 使わず、見出しの直後にリストを置くだけで視覚的な関連付けとする）。
//! 人名は架空、メールは予約ドメイン `example.com`（RFC 2606）のみを使う
//! （実在の人物・企業・PII を含まない）。メールは `mailto:` リンクにせず
//! プレーンテキストとする（`link` 部品を増やさず使用部品を 4 つに保つ
//! ため）。
//!
//! # a11y: `viewport` の `role`/`aria-label`
//!
//! `scroll_area::viewport` は `tabindex="0"` が固定で付き、キーボードで
//! スクロールできる。素の `div` へ `aria-label` だけを付けるのは ARIA 上
//! 不適切なため、`role="region"` を併記してランドマーク名を持つ領域として
//! 公開する。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `list::root`/`heading::heading`/`avatar::root` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため（`scroll_area::root`/`viewport`/`content` は再エクスポート
//! のみの薄いラッパーで `class` をそのまま連結する）、Demo 固有のスタイル
//! フックは `class` ではなく `data-blocks-list-sticky-groups-*` 属性で渡す
//! （`list_narrow_activity` と同じ判断）。
//!
//! # `list` recipe の item 余白を上書きする理由
//!
//! `pre-styled-ui` の `list` recipe は item に `margin-block:
//! var(--fandhe-space-1)` を既定で与えるが、本 block は行間に区切り線を
//! 挟む密な一覧を志向するため、詳細度
//! `.blocks-list-sticky-groups [data-blocks-list-sticky-groups-row]`
//! （複合セレクタで recipe の単一 `[data-part="item"]` 宣言より詳細度を
//! 上げる）で上書きする（`list_narrow_activity` と同じ判断軸）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::scroll_area;
use fandhe_frontend_pre_styled_ui::Size;

/// 頭文字グループ順の人物ディレクトリ（架空。頭文字と (名前, メール) の組の
/// 配列）。あらかじめ頭文字順に並べた状態で持ち、実行時にソート・グループ
/// 化は行わない。スクロールが確実に起きる件数（6 グループ・合計 16 人）に
/// している。
const GROUPS: &[(&str, &[(&str, &str)])] = &[
    (
        "A",
        &[
            ("安藤 明", "akira.ando@example.com"),
            ("荒木 愛子", "aiko.araki@example.com"),
        ],
    ),
    (
        "C",
        &[
            ("千葉 智也", "tomoya.chiba@example.com"),
            ("近藤 千夏", "chinatsu.kondo@example.com"),
            ("千田 治郎", "jiro.chida@example.com"),
        ],
    ),
    (
        "F",
        &[
            ("藤原 文子", "fumiko.fujiwara@example.com"),
            ("福田 太郎", "taro.fukuda@example.com"),
        ],
    ),
    (
        "M",
        &[
            ("松本 まどか", "madoka.matsumoto@example.com"),
            ("三浦 実", "minoru.miura@example.com"),
            ("森田 美咲", "misaki.morita@example.com"),
            ("宮本 学", "manabu.miyamoto@example.com"),
        ],
    ),
    (
        "S",
        &[
            ("佐々木 進", "susumu.sasaki@example.com"),
            ("清水 幸子", "sachiko.shimizu@example.com"),
            ("杉山 聡", "satoshi.sugiyama@example.com"),
        ],
    ),
    (
        "Y",
        &[
            ("山口 洋子", "yoko.yamaguchi@example.com"),
            ("吉田 裕也", "yuya.yoshida@example.com"),
        ],
    ),
];

/// 名前の先頭 1 文字をアバターのフォールバック表示に使う
/// （`content_article`/`list_narrow_activity` と同じ判断）。
fn initial_avatar(name: &str) -> Node {
    let initial: String = name
        .split_whitespace()
        .filter_map(|part| part.chars().next())
        .collect();
    avatar::root(
        &AvatarProps {
            size: Size::Sm,
            ..AvatarProps::default()
        },
        vec![],
        vec![avatar::fallback(
            ImageStatus::Error,
            vec![],
            vec![text(initial)],
        )],
    )
}

/// 1 行（アバター + 名前 + メール）を組み立てる。
fn person_row(name: &str, email: &str) -> Node {
    list::item(
        vec![("data-blocks-list-sticky-groups-row", "")],
        vec![
            initial_avatar(name),
            div(
                vec![],
                vec![
                    div(
                        vec![("data-blocks-list-sticky-groups-name", "")],
                        vec![text(name)],
                    ),
                    div(
                        vec![("data-blocks-list-sticky-groups-email", "")],
                        vec![text(email)],
                    ),
                ],
            ),
        ],
    )
}

/// 1 グループ（sticky 見出し + 人物一覧）を組み立てる。
fn group_section(initial: &str, people: &[(&str, &str)]) -> Node {
    div(
        vec![("data-blocks-list-sticky-groups-group", "")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Sm,
                    ..HeadingProps::default()
                },
                vec![("data-blocks-list-sticky-groups-heading", "")],
                vec![text(initial)],
            ),
            list::root(
                ListType::Unordered,
                ListVariant::Plain,
                vec![],
                people
                    .iter()
                    .map(|(name, email)| person_row(name, email))
                    .collect(),
            ),
        ],
    )
}

/// `list-sticky-groups` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（モジュール doc「静的表示」節）。
pub fn demo() -> Node {
    div(
        vec![("data-blocks-list-sticky-groups-frame", "")],
        vec![scroll_area::root(
            vec![("data-blocks-list-sticky-groups-scroll", "")],
            vec![scroll_area::viewport(
                vec![("role", "region"), ("aria-label", "People directory")],
                vec![scroll_area::content(
                    vec![],
                    GROUPS
                        .iter()
                        .map(|(initial, people)| group_section(initial, people))
                        .collect(),
                )],
            )],
        )],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/list-sticky-groups/",
    title: "list-sticky-groups",
    category: BlockCategory::List,
    rust_source: "crates/docs-site/src/blocks/application/list/list_sticky_groups.rs",
    demo_class: "blocks-list-sticky-groups",
    parts: &[
        Part {
            label: "List",
            path: "/themes/list/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Scroll Area",
            path: "/themes/scroll-area/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `list_sticky_groups` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。使うトークンは既存 block で使用実績
/// のあるものに限る（space-*/color-border/bg/bg-subtle/fg-muted/radius-lg/
/// z-index-docked/font-weight-medium）。
const LAYOUT_CSS: &str = "\
.blocks-list-sticky-groups {\n  padding: var(--fandhe-space-6) var(--fandhe-space-4);\n}\n\
[data-blocks-list-sticky-groups-frame] {\n  max-width: 28rem;\n  margin-inline: auto;\n}\n\
[data-blocks-list-sticky-groups-scroll] {\n  height: 20rem;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n  background: var(--fandhe-color-bg);\n}\n\
[data-blocks-list-sticky-groups-heading] {\n  position: sticky;\n  top: 0;\n  z-index: var(--fandhe-z-index-docked, 10);\n  background: var(--fandhe-color-bg-subtle);\n  border-bottom: 1px solid var(--fandhe-color-border);\n  padding: var(--fandhe-space-1-5) var(--fandhe-space-4);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-list-sticky-groups [data-blocks-list-sticky-groups-row] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  margin-block: 0;\n  padding: var(--fandhe-space-3) var(--fandhe-space-4);\n  border-top: 1px solid var(--fandhe-color-border);\n}\n\
[data-blocks-list-sticky-groups-name] {\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n\
[data-blocks-list-sticky-groups-email] {\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, GROUPS, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 4 種の部品を出力すること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"list\"",
            "data-scope=\"avatar\"",
            "data-scope=\"heading\"",
            "data-scope=\"scroll-area\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// 全グループの見出し（頭文字）が H3 として出力されること。
    #[test]
    fn demo_renders_all_group_headings_as_h3() {
        let html = render(&demo());
        assert_eq!(html.matches("<h3").count(), GROUPS.len(), "html={html}");
        for (initial, _) in GROUPS {
            assert!(html.contains(&format!(">{initial}<")), "html={html}");
        }
    }

    /// 全人物の行が出力されていること（アバター・名前・メールの件数一致）。
    #[test]
    fn demo_renders_all_people_rows() {
        let html = render(&demo());
        let expected: usize = GROUPS.iter().map(|(_, people)| people.len()).sum();
        assert_eq!(
            html.matches("data-blocks-list-sticky-groups-row").count(),
            expected,
            "html={html}"
        );
        for (_, people) in GROUPS {
            for (name, email) in *people {
                assert!(html.contains(name), "html={html}");
                assert!(html.contains(email), "html={html}");
            }
        }
    }

    /// 非対話・安全性の不変条件（`<form>`・`id="..."`・`<button` を持たない
    /// こと、`mailto:` リンクにしないこと）。
    #[test]
    fn demo_never_contains_forbidden_markup() {
        let html = render(&demo());
        assert!(!html.contains("<form"));
        assert!(!html.contains("id=\""));
        assert!(!html.contains("<button"));
        assert!(!html.contains("mailto:"));
    }

    /// [`LAYOUT_CSS`] が sticky 宣言を持つこと。
    #[test]
    fn layout_css_has_sticky_heading_rule() {
        assert!(LAYOUT_CSS.contains("position: sticky;"));
        assert!(LAYOUT_CSS.contains("top: 0;"));
    }

    /// `viewport` の `role="region"`/`aria-label` が出力されること。
    #[test]
    fn demo_wires_viewport_landmark() {
        let html = render(&demo());
        assert!(html.contains(r#"role="region""#));
        assert!(html.contains(r#"aria-label="People directory""#));
    }
}
