//! Link / Link Overlay / Nav List 3 ページの原稿データ（イシュー #1154、
//! 親 #928）。
//!
//! `site/themes/link.md` / `link-overlay.md` / `nav-list.md` は Themes
//! 107 部品の中で唯一プレースホルダ（`SPEC_TABLES` 未登録・
//! `showcase::COMPONENT_PAGES` 未登録のスタブ）のまま残っていた 3 ページで
//! あり、本モジュールが最後の充填対象を消化する。台帳上は Link/Link
//! Overlay/Nav List の 3 部品は Typography・Utilities・Interactive とカテゴリ
//! が分かれるが、`crate::component_specs::mod` は「カテゴリ 1 個につき 1
//! モジュール」を原則としつつ、本イシューはスタブ一括充填という単発の
//! 作業単位であるため専用の 1 モジュール（`navigation`、ナビゲーション系
//! リンク部品という共通点で束ねる）へまとめる。
//!
//! # 責務境界・呼び出し文脈
//!
//! [`crate::component_page::generated_content`] が `page_path` から
//! [`SPECS`] を線形探索し、Features / API Reference の引数表 / Examples /
//! Accessibility の各節を合成する（[`crate::component_page::ComponentPageSpec`]
//! 参照）。Demo 節は [`crate::showcase::COMPONENT_PAGES`]（正）の
//! `link_section`/`link_overlay_section`/`nav_list_section` から供給される
//! ため、本モジュールの `demo` フィールドはすべて `None`（イシュー #996
//! Tab Nav・#980 Toggle と同じ正経路移設パターン）。
//!
//! # 一次情報・非捏造の方針
//!
//! - Features / Arguments: `crates/pre-styled-ui/src/{link,link_overlay,
//!   nav_list}.rs` と `crates/headless-ui/src/` 同名 mod の rustdoc・
//!   シグネチャから採り、各行に `file:line` を併記する（未確認の挙動は
//!   書かない、`.claude/rules/out-of-scope-tracking.md` の方針）。
//! - Keyboard: 3 部品とも JS ハイドレーション前提の操作を持たず、素の
//!   `<a>`/`<nav>` のブラウザ標準操作のみのため、Tab Nav（#996）と同じ
//!   粒度で `Tab / Shift+Tab` の 1 行のみを記載する。
//! - Anatomy・`data-*` 属性表・CSS 変数表は手書きしない（機械導出が正、
//!   `docs/design/docs-site-component-pages.md` §7b.3）。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! 本ファイルはノード木 API のみで組み立て、`raw_html()` を使わない
//! （`crates/docs-site/tests/component_pages.rs::component_page_source_does_not_use_raw_html`
//! が `component_specs/` 配下を再帰走査してこれを固定する）。

use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::link_overlay;
use fandhe_frontend_pre_styled_ui::nav_list;

use crate::component_page::{ArgRow, AriaRow, ComponentPageSpec, ExampleEntry, KeyRow};

/// 横並びのデモ列（`showcase.rs::row` / `component_specs/typography.rs::row`
/// と同型、`SHOWCASE_LAYOUT_CSS` の `.showcase-row` を使う）。
fn row(children: Vec<Node>) -> Node {
    div(vec![("class", "showcase-row")], children)
}

fn link_example() -> Node {
    row(vec![
        link::root(
            "",
            &LinkProps {
                current: true,
                variant: LinkVariant::Underline,
                ..LinkProps::default()
            },
            vec![],
            vec![text("Guides")],
        ),
        link::root(
            "",
            &LinkProps {
                variant: LinkVariant::Underline,
                ..LinkProps::default()
            },
            vec![],
            vec![text("API Reference")],
        ),
    ])
}

const LINK: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "素の <a> 要素 1 パーツ（anatomy root）のみで構成する最小部品（crates/headless-ui/src/link.rs:16-19）",
        "external=true のとき target=\"_blank\" と rel=\"noopener noreferrer\" を不可分に付与する（片方のみを付与できる API は持たない、reverse tabnabbing 対策。crates/headless-ui/src/link.rs:14-19, 84-88）",
        "current=true のとき aria-current=\"page\" + data-current を付与する（crates/headless-ui/src/link.rs:21-24, 89-92）",
        "Plain（既定・下線なし）/ Underline（常時下線）の 2 variant。current 状態の装飾（フォント太字化）は aria-current=\"page\" を条件にした CSS 状態セレクタで表現し、追加の bool 引数は持たない（crates/pre-styled-ui/src/link.rs 参照）",
        "ColorPalette 軸（6 値、既定 Accent）。hover 時に文字色を emphasized 段へ強調し、focus-visible 時にフォーカスリングを表示、color の transition を伴う（イシュー #1437、crates/pre-styled-ui/src/link.rs 参照）",
        "href の URL スキーム検証（javascript: 等の拒否）は core の render() が担う deny-by-default（crates/headless-ui/src/link.rs:41-45）",
    ],
    arguments: &[
        ArgRow {
            name: "href",
            kind: "&str",
            default: "",
            description: "a 要素の href（crates/pre-styled-ui/src/link.rs）。",
        },
        ArgRow {
            name: "props.external",
            kind: "bool",
            default: "false",
            description: "true のとき target=\"_blank\" + rel=\"noopener noreferrer\" を付与する（crates/headless-ui/src/link.rs:70-73）。",
        },
        ArgRow {
            name: "props.current",
            kind: "bool",
            default: "false",
            description: "true のとき aria-current=\"page\" + data-current を付与する（crates/headless-ui/src/link.rs:74-77）。",
        },
        ArgRow {
            name: "props.variant",
            kind: "LinkVariant",
            default: "LinkVariant::Plain",
            description: "root の見た目（Plain/Underline、crates/pre-styled-ui/src/link.rs）。",
        },
        ArgRow {
            name: "props.palette",
            kind: "ColorPalette",
            default: "ColorPalette::Accent",
            description: "colorPalette 軸（イシュー #1437、crates/pre-styled-ui/src/link.rs）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "Current page と通常リンク",
        description: "Guides を現在ページ（aria-current=\"page\"）とした Underline variant の 2 リンクです。",
        render: link_example,
    }],
    keyboard: &[KeyRow {
        key: "Tab / Shift+Tab",
        description: "素の <a> のブラウザ標準操作でフォーカス移動する。JS ハイドレーション前提の操作は持たない。",
    }],
    aria: &[
        AriaRow {
            attribute: "aria-current=\"page\"（root、current=true のとき）",
            description: "現在ページを示す（crates/headless-ui/src/link.rs:74-77）。",
        },
        AriaRow {
            attribute: "rel=\"noopener noreferrer\"（root、external=true のとき）",
            description: "target=\"_blank\" と不可分に付与し reverse tabnabbing を防ぐ（crates/headless-ui/src/link.rs:14-19）。",
        },
    ],
    demo: None,
};

fn link_overlay_example() -> Node {
    link_overlay::root(
        vec![],
        vec![
            el("h3", vec![], vec![text("Getting started")]),
            el(
                "p",
                vec![],
                vec![text(
                    "プロジェクトの作成から最初のページ公開までの手順です。",
                )],
            ),
            link_overlay::overlay("", vec![("aria-label", "Getting started を開く")], vec![]),
        ],
    )
}

const LINK_OVERLAY: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "root（div、位置決めコンテキスト）/ overlay（a、カード全面へ拡張されるリンク）の 2 anatomy パーツ構成（crates/headless-ui/src/link_overlay.rs:16-19, 70-83）",
        "::before 疑似要素を使わず overlay 自身を position: absolute; inset: 0 で root 全面へ展開する（recipe が疑似要素セレクタを表現できないため。crates/pre-styled-ui/src/link_overlay.rs:14-22, 56-64）",
        "root の高さは overlay 以外の子ノード（見出し・画像等の通常フロー要素）が確立する契約（crates/headless-ui/src/link_overlay.rs:22-30）",
        "href の URL スキーム検証は link と同じく core の render() が担う（crates/headless-ui/src/link_overlay.rs:55-58）",
    ],
    arguments: &[
        ArgRow {
            name: "attrs / children（root）",
            kind: "Vec<(&str, &str)>, Vec<Node>",
            default: "",
            description: "位置決めコンテキストとなる div の属性・子ノード。overlay 以外の子ノードで高さを確立する（crates/pre-styled-ui/src/link_overlay.rs:75-78）。",
        },
        ArgRow {
            name: "href（overlay）",
            kind: "&str",
            default: "",
            description: "root 全面へ展開されるリンク先（crates/headless-ui/src/link_overlay.rs:70-73、fandhe_frontend_headless_ui::link_overlay::overlay の再エクスポート）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "カード全面クリック",
        description: "見出しと説明文で高さを確立し、overlay（aria-label 付き）がカード全面へ展開されるリンクとして重なります。",
        render: link_overlay_example,
    }],
    keyboard: &[KeyRow {
        key: "Tab / Shift+Tab",
        description: "overlay（素の <a>）のブラウザ標準操作でフォーカス移動する。",
    }],
    aria: &[AriaRow {
        attribute: "aria-label（overlay、呼び出し側が付与）",
        description: "overlay に可視テキストを持たせない構成では呼び出し側が aria-label でアクセシブルネームを与える運用を推奨する（crates/headless-ui/src/link_overlay.rs:24-31）。",
    }],
    demo: None,
};

fn nav_list_example() -> Node {
    nav_list::root(
        "Documentation",
        vec![],
        vec![
            nav_list::heading(vec![], vec![text("Guides")]),
            nav_list::list(
                vec![],
                vec![
                    nav_list::item(
                        vec![],
                        vec![nav_list::link("", true, vec![], vec![text("Overview")])],
                    ),
                    nav_list::item(
                        vec![],
                        vec![nav_list::link(
                            "",
                            false,
                            vec![],
                            vec![text("Installation")],
                        )],
                    ),
                ],
            ),
        ],
    )
}

const NAV_LIST: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "root（nav）/ heading（h2）/ list（ul）/ item（li）/ link（a）の 5 anatomy パーツ構成（crates/headless-ui/src/nav_list.rs:41-45, 62-97）",
        "role を一切付与しない。素の nav/h2/ul/li/a の暗黙 ARIA ロール（navigation/heading/list/listitem/link）のみを使う（pre-styled-ui の menu が WAI-ARIA menu ロールを文書ナビへ転用してしまう意味論不整合を避けるための専用部品。crates/headless-ui/src/nav_list.rs:16-24）",
        "link の current=true のとき aria-current=\"page\" + data-current を付与する（crates/headless-ui/src/nav_list.rs:88-101）",
        "fandhe-frontend-docs-site 自身のサイドバー（src/nav.rs::sidebar）が本部品の headless 再エクスポートで組み立てられている（crates/pre-styled-ui/src/nav_list.rs:24-38）",
    ],
    arguments: &[
        ArgRow {
            name: "label（root）",
            kind: "&str",
            default: "",
            description: "root の aria-label（必須引数。複数の nav ランドマークを区別可能にする。crates/headless-ui/src/nav_list.rs:47-55, 61-68）。",
        },
        ArgRow {
            name: "href（link）",
            kind: "&str",
            default: "",
            description: "link の href（crates/headless-ui/src/nav_list.rs:90-93）。",
        },
        ArgRow {
            name: "current（link）",
            kind: "bool",
            default: "false",
            description: "true のとき aria-current=\"page\" + data-current を付与する（crates/headless-ui/src/nav_list.rs:88-101）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "見出し付きナビリスト",
        description: "Guides 見出し配下に Overview（現在ページ）・Installation の 2 リンクを掲示します。",
        render: nav_list_example,
    }],
    keyboard: &[KeyRow {
        key: "Tab / Shift+Tab",
        description: "各 link（素の <a>）のブラウザ標準操作でフォーカス移動する。矢印キーでの項目間移動は提供しない（crates/headless-ui/src/nav_list.rs:97-102 スコープ外節）。",
    }],
    aria: &[
        AriaRow {
            attribute: "aria-label（root、必須引数）",
            description: "landmark のアクセシブルネームを常に持つ（crates/headless-ui/src/nav_list.rs:61-68）。",
        },
        AriaRow {
            attribute: "aria-current=\"page\"（link、current=true のとき）",
            description: "現在ページを示す（crates/headless-ui/src/nav_list.rs:88-101）。",
        },
        AriaRow {
            attribute: "role",
            description: "一切出力しない（crates/headless-ui/src/nav_list.rs:16-24）。",
        },
    ],
    demo: None,
};

/// Link / Link Overlay / Nav List 3 ページ（イシュー #1154）の
/// `path -> ComponentPageSpec` テーブル。[`crate::component_page::SPEC_TABLES`]
/// が集約する。
pub const SPECS: &[(&str, ComponentPageSpec)] = &[
    ("/themes/link/", LINK),
    ("/themes/link-overlay/", LINK_OVERLAY),
    ("/themes/nav-list/", NAV_LIST),
];
