//! `empty-state-starter-grid` block（イシュー #2972。親トラッキング #2951。
//! Application / Empty State カテゴリ、3 件目）。開始候補（テンプレート等）
//! を「アイコン + 題名 + 説明 + 開く操作」のタイルとしてグリッドで並べる
//! 空状態。主参照 R0463（代表構成）を軸に、R1396（テンプレート選択の 2 列
//! グリッド・全面クリック）を集約する。`_/blocks-intake/` の対応ファイルは
//! 本イシュー着手時点で本 worktree に存在しないため、原稿・本コメントには
//! 対応表 ID のみを記す（`empty_state_invite_team` と同じ扱い）。
//!
//! # 使用部品
//!
//! `item` / `icon` / `heading` / `text` / `link` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # `button` を使わない（Issue の部品指示からの意図的逸脱）
//!
//! Issue は `button` を使用部品に含めるが、「タイル全体をリンク」と
//! 「開くボタン」を両立させようとすると `<a>` に `<button>` を入れ子にする
//! ことになり HTML 不正になる。`type="button"` の無 JS ボタンをリンクの
//! 下へ並べる案も、キーボード・支援技術に無意味な追加停止点を増やすため
//! 採らない。[`super::super::marketing::footer::footer_cta_columns`]
//! （`button` を `parts` から除外しボタン風外見の `link::root` へ置き換えた
//! 先例）と同型の判断で `link::root`（[`open_link`]）をボタン風に装飾し、
//! CSS の stretched-link（`::after { inset: 0 }`）でタイル全面へ拡張する。
//! 末尾の「Blocks 一覧に戻る」導線も `link::root`（`LinkVariant::Underline`）
//! にする。`href="../"` は Blocks 索引ページへ戻るリンクであり（空の
//! プロジェクトを開始する導線ではない）、レビュー指摘（イシュー #2972
//! PR #3424 の Codex 指摘）を受けてリンク文言を実遷移先に合わせている。
//!
//! # 構造（1 タイル = `<a>` 1 個）
//!
//! [`grid_list_action_tiles`](super::super::grid_list::grid_list_action_tiles)
//! と異なり `link_overlay` は使用部品に含めないため、タイル全体を包む外側
//! リンクを重ねる方式ではなく、タイル内の「開く」リンク（[`open_link`]）
//! 自体を `position: relative` の `item::root` いっぱいに広げる
//! stretched-link 方式を採る（`item::root` 自身は overlay を持たないため
//! `[data-blocks-empty-state-starter-grid-tile]` 側に `position: relative`
//! を宣言する）。アクセシブル名は `aria-label="<題名> を開く"`。
//!
//! # href の方針（`href="#"` を使わない）
//!
//! [`Block::demo`] は `fn() -> Node` のため `base_path` を受け取れず、
//! タイルはいずれもサイト内に実在する索引ページへの相対パス
//! （[`grid_list_action_tiles`](super::super::grid_list::grid_list_action_tiles)
//! と同じ先例）を指す。実利用時は自分のページ URL へ差し替えることを
//! `site/blocks/empty-state-starter-grid.md` の導入文で明記する。
//!
//! # `@container` によるレスポンシブ列数
//!
//! Demo 枠の幅はビューポート幅と一致しないため `@container`
//! （コンテナクエリ）で判定する（`empty_state_invite_team` と同型の
//! パターン）。[`LAYOUT_CSS`] のラッパー
//! `.blocks-empty-state-starter-grid-stack` へ `container-type: inline-size`
//! を宣言し、コンテナ幅が `36rem` 未満では常に 1 列、`36rem` 以上では
//! 2 列、`56rem` 以上かつ `data-columns="3"`（代表構成版）のみ 3 列へ
//! 切り替える。
//!
//! # `id` は出力しない
//!
//! 見出し・入力に紐付ける `id`/`aria-labelledby` を一切使わないため
//! （`demo_output_has_no_dangling_aria_references_or_duplicate_ids` 契約、
//! `crate::blocks` モジュール doc 参照）、2 版を同一 Demo 内へ並記しても
//! 衝突しない。
//!
//! # アイコンは自作の単純図形
//!
//! [`fandhe_frontend_pre_styled_ui::icon`] を使用部品に含めるため、装飾
//! アイコンは [`icon::icon`] へ [`el`] による線画（`stroke="currentColor"`,
//! `fill="none"`）を渡して組み立てる（`grid_list_action_tiles::geo_icon` と
//! 同型のパターン）。装飾用途のため `IconProps::label` は付けない。実在
//! ブランドのロゴ・商標は模さない。
//!
//! # ダミー素材について
//!
//! 題名・説明はすべて独自の架空文言であり、実企業名・実在人物・PII を
//! 含まない。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo はフォーム・
//! 状態機械を持たない静的な合成例である。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::item::{self, ItemMediaVariant, ItemRootProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

const TILE_ATTR: &str = "data-blocks-empty-state-starter-grid-tile";
const MEDIA_ATTR: &str = "data-blocks-empty-state-starter-grid-media";
const OPEN_ATTR: &str = "data-blocks-empty-state-starter-grid-open";

/// タイル 1 件分のデータ（モジュール doc「href の方針」節参照）。
struct Tile {
    href: &'static str,
    title: &'static str,
    description: &'static str,
    icon_path_d: &'static str,
}

/// 開始候補タイル 6 件（サイト内に実在する索引ページのみを指す）。
const TILES: [Tile; 6] = [
    Tile {
        href: "../../guides/",
        title: "ガイド集",
        description: "手順に沿って機能の使い方を学べるガイド一覧から始めます。",
        icon_path_d: "M4 4h16v16H4zM8 8h8M8 12h8M8 16h5",
    },
    Tile {
        href: "../../examples/",
        title: "サンプル集",
        description: "動く構成をそのまま確認できるサンプル一覧から始めます。",
        icon_path_d: "M4 4h7v7H4zM13 4h7v4h-7zM13 11h7v9h-7zM4 14h7v6H4z",
    },
    Tile {
        href: "../../primitives/",
        title: "Primitives 一覧",
        description: "アクセシブルな headless UI 部品一覧から始めます。",
        icon_path_d: "M4 4h16v6H4zM4 13h16M4 17h10",
    },
    Tile {
        href: "../../themes/",
        title: "Themes 一覧",
        description: "スタイル済みの UI 部品一覧から始めます。",
        icon_path_d: "M4 4h6v16H4zM12 4h8v16h-8zM14 8h4M14 12h4",
    },
    Tile {
        href: "../",
        title: "Blocks 一覧",
        description: "既存部品を組み合わせた合成例一覧から始めます。",
        icon_path_d: "M12 3l1.2 2.4 2.6.4-1.9 1.9.5 2.6-2.4-1.3-2.4 1.3.5-2.6-1.9-1.9 2.6-.4z",
    },
    Tile {
        href: "../../api/",
        title: "API カタログ",
        description: "エンドポイント一覧を備えた参照ページから始めます。",
        icon_path_d: "M4 4h16v16H4zM4 9h16",
    },
];

/// 自作の線画（stroke）アイコンを組み立てる（`grid_list_action_tiles::
/// geo_icon` と同型のパターン。装飾用途のため `IconProps::label` は付けない）。
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
                ("stroke-width", "1.5"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// タイル 1 件分を組み立てる（モジュール doc「構造」節。`item::root` へ
/// `position: relative`（[`LAYOUT_CSS`]）を持たせ、`open_link` の
/// stretched-link（`::after { inset: 0 }`）でタイル全面をクリック可能に
/// する）。
fn starter_tile(tile: &Tile) -> Node {
    item::root(
        ItemRootProps::default(),
        vec![(TILE_ATTR, "")],
        vec![
            item::media(
                ItemMediaVariant::Icon,
                vec![(MEDIA_ATTR, "")],
                vec![geo_icon(tile.icon_path_d)],
            ),
            item::content(
                vec![],
                vec![
                    heading(
                        HeadingLevel::H4,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(tile.title)],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(tile.description)],
                    ),
                ],
            ),
            item::actions(
                vec![],
                vec![link::root(
                    tile.href,
                    &LinkProps::default(),
                    vec![
                        (OPEN_ATTR, ""),
                        ("aria-label", &format!("{} を開く", tile.title)),
                    ],
                    vec![text("開く")],
                )],
            ),
        ],
    )
}

/// 見出し + 説明文 + タイルグリッド + 末尾導線の 1 インスタンス分を
/// 組み立てる。`columns` は `[data-columns]` の値（`@container` 側で
/// 参照、[`LAYOUT_CSS`]）。
fn starter_instance(
    heading_text: &'static str,
    description_text: &'static str,
    columns: &'static str,
    tiles: &[Tile],
) -> Node {
    div(
        vec![("class", "blocks-empty-state-starter-grid-instance")],
        vec![
            div(
                vec![("class", "blocks-empty-state-starter-grid-header")],
                vec![
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(heading_text)],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(description_text)],
                    ),
                ],
            ),
            div(
                vec![
                    ("class", "blocks-empty-state-starter-grid-grid"),
                    ("data-columns", columns),
                ],
                tiles.iter().map(starter_tile).collect(),
            ),
            div(
                vec![("class", "blocks-empty-state-starter-grid-footer")],
                vec![
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("テンプレートが合わない場合は")],
                    ),
                    link::root(
                        "../",
                        &LinkProps {
                            variant: LinkVariant::Underline,
                            ..LinkProps::default()
                        },
                        vec![],
                        vec![text("Blocks 一覧に戻る")],
                    ),
                ],
            ),
        ],
    )
}

/// 版ラベル（`empty_state_invite_team::variant_label` と同型）。
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

/// `empty-state-starter-grid` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数（モジュール doc「2 版と集約元の対応」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-empty-state-starter-grid-stack")],
        vec![
            variant_label("代表構成・3 列（R0463）"),
            starter_instance(
                "どこから見ますか",
                "気になる入り口を選ぶと、対応する一覧ページへ移動します。",
                "3",
                &TILES,
            ),
            variant_label("テンプレート選択・2 列（R1396）"),
            starter_instance(
                "調べたい分野を選ぶ",
                "気になる分野を選ぶと、そのページへ移動します。",
                "2",
                &TILES[..4],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/empty-state-starter-grid/",
    title: "empty-state-starter-grid",
    category: BlockCategory::EmptyState,
    rust_source: "crates/docs-site/src/blocks/application/empty_state/empty_state_starter_grid.rs",
    demo_class: "blocks-empty-state-starter-grid",
    parts: &[
        Part {
            label: "Item",
            path: "/themes/item/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
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
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `empty_state_starter_grid` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節と同型）。色リテラル
/// （`#`/`white` 等）は使わず可読性の確保はすべて `--fandhe-*` トークンで
/// 行う。
const LAYOUT_CSS: &str = "\
.blocks-empty-state-starter-grid-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  container-type: inline-size;\n  container-name: blocks-empty-state-starter-grid;\n}\n\
.blocks-empty-state-starter-grid-instance {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-empty-state-starter-grid-header {\n  text-align: center;\n  max-width: 32rem;\n  margin-inline: auto;\n}\n\
.blocks-empty-state-starter-grid-grid {\n  display: grid;\n  gap: var(--fandhe-space-4);\n  grid-template-columns: minmax(0, 1fr);\n}\n\
[data-scope=\"item\"][data-part=\"root\"][data-blocks-empty-state-starter-grid-tile] {\n  position: relative;\n  flex-direction: column;\n  align-items: flex-start;\n  gap: var(--fandhe-space-4);\n}\n\
[data-scope=\"item\"][data-part=\"media\"][data-blocks-empty-state-starter-grid-media] {\n  width: 2.5rem;\n  height: 2.5rem;\n  border-radius: var(--fandhe-radius-md);\n}\n\
[data-blocks-empty-state-starter-grid-tile] > [data-scope=\"item\"][data-part=\"content\"] {\n  align-self: stretch;\n}\n\
[data-scope=\"link\"][data-part=\"root\"][data-blocks-empty-state-starter-grid-open] {\n  display: inline-flex;\n  align-items: center;\n  padding: var(--fandhe-space-2) var(--fandhe-space-4);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n  color: var(--fandhe-color-fg);\n}\n\
[data-scope=\"link\"][data-part=\"root\"][data-blocks-empty-state-starter-grid-open]::after {\n  content: \"\";\n  position: absolute;\n  inset: 0;\n}\n\
.blocks-empty-state-starter-grid-footer {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-2);\n  align-items: baseline;\n  justify-content: center;\n}\n\
@container blocks-empty-state-starter-grid (min-width: 36rem) {\n  \
.blocks-empty-state-starter-grid-grid[data-columns=\"2\"] {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n  \
.blocks-empty-state-starter-grid-grid[data-columns=\"3\"] {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n\
}\n\
@container blocks-empty-state-starter-grid (min-width: 56rem) {\n  \
.blocks-empty-state-starter-grid-grid[data-columns=\"3\"] {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（item/icon/heading/text/link）の anatomy をすべて
    /// 実際に出力していることを固定する。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"item\"",
            "data-scope=\"icon\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// 「開く」リンク（6 + 4 = 10 件）と、それぞれの `aria-label`・末尾
    /// 導線リンク 2 件を固定する。
    #[test]
    fn demo_renders_ten_tile_links_with_aria_label() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-empty-state-starter-grid-open")
                .count(),
            10
        );
        assert_eq!(html.matches("を開く\"").count(), 10);
        assert_eq!(html.matches("Blocks 一覧に戻る").count(), 2);
    }

    /// 非対話・XSS の不変条件（`crate::blocks` モジュール doc）と、
    /// `<button>` を使わない設計判断（モジュール doc参照）を固定する。
    #[test]
    fn demo_has_no_form_dead_links_or_unsafe_output() {
        let html = render(&demo());
        for absent in [
            "<form",
            "href=\"#\"",
            "src=\"data:",
            " id=\"",
            "<script",
            "<button",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
        for href in [
            "../../guides/",
            "../../examples/",
            "../../primitives/",
            "../../themes/",
            "../../api/",
        ] {
            assert!(
                html.contains(&format!(r#"href="{href}""#)),
                "demo should link to {href}"
            );
        }
        assert!(
            html.contains(r#"href="../""#),
            "demo should link back to blocks index"
        );
    }

    /// [`LAYOUT_CSS`] が想定する `@container`・stretched-link を持ち、
    /// 色リテラルを含まないことを固定する。
    #[test]
    fn layout_css_declares_container_queries_and_stretched_link() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("white"));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(
            LAYOUT_CSS.contains("@container blocks-empty-state-starter-grid (min-width: 36rem)")
        );
        assert!(
            LAYOUT_CSS.contains("@container blocks-empty-state-starter-grid (min-width: 56rem)")
        );
        assert!(LAYOUT_CSS.contains("repeat(3, minmax(0, 1fr))"));
        assert!(LAYOUT_CSS.contains("inset: 0;"));
    }

    /// タイルセレクタが `item` recipe の詳細度（3 属性セレクタ）と同格で
    /// あることを固定する（`grid_list_action_tiles` と同型の判断）。
    #[test]
    fn layout_css_tile_selector_matches_item_recipe_specificity() {
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"item\"][data-part=\"root\"][data-blocks-empty-state-starter-grid-tile] {"
        ));
    }
}
