# docs-layout-toc

ページ内目次（サイドバー用の基本形）。`link`/`nav-list`/`heading`/`text`
の 4 部品のみで合成する例で、集約元は 2 件です（対応表 ID R0364・R0365）。

目次ラベルの下にページ内の節へのリンクを縦に並べ、階層は字下げで、
現在の節は強調で示します。インスタンス A（縦線あり、R0364 主参照）は
各階層の左に縦線（rail）を引いて階層と現在位置を線でも示し、インスタ
ンス B（R0365）は縦線を持たない簡素版です。広い幅では本文列の右側に
目次列を置く 2 列構成（`@container` で切り替え）にし、狭い幅でも目次は
隠しません。現在位置は Demo 内で固定した静的表示で、scroll spy のよう
な実行時更新は行いません（docs サイトは無 JS）。`<form>` は出しません。
文言はすべて独自に書いた架空のものであり、実企業名・実クレデンシャル・
PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::nav_list;
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};

/// 目次項目 1 件（節 1 つ）。`children` で第 3 階層まで表現できるが、本
/// Demo は第 2 階層までを使う。`id`/`href` は単一の真実源（モジュール doc
/// 「リンク先・`id`/`href` の単一情報源」参照）。
struct TocEntry {
    id: &'static str,
    href: &'static str,
    title: &'static str,
    children: &'static [TocEntry],
}

/// インスタンス A（rail 版）の目次項目 4 件（うち 2 件が子節 2 件ずつを
/// 持つ）。
const TOC_A: &[TocEntry] = &[
    TocEntry {
        id: "blocks-docs-layout-toc-a-overview",
        href: "#blocks-docs-layout-toc-a-overview",
        title: "概要",
        children: &[],
    },
    TocEntry {
        id: "blocks-docs-layout-toc-a-configuration",
        href: "#blocks-docs-layout-toc-a-configuration",
        title: "設定",
        children: &[
            TocEntry {
                id: "blocks-docs-layout-toc-a-configuration-env",
                href: "#blocks-docs-layout-toc-a-configuration-env",
                title: "環境変数",
                children: &[],
            },
            TocEntry {
                id: "blocks-docs-layout-toc-a-configuration-auth",
                href: "#blocks-docs-layout-toc-a-configuration-auth",
                title: "認証",
                children: &[],
            },
        ],
    },
    TocEntry {
        id: "blocks-docs-layout-toc-a-usage",
        href: "#blocks-docs-layout-toc-a-usage",
        title: "使い方",
        children: &[
            TocEntry {
                id: "blocks-docs-layout-toc-a-usage-basic",
                href: "#blocks-docs-layout-toc-a-usage-basic",
                title: "基本操作",
                children: &[],
            },
            TocEntry {
                id: "blocks-docs-layout-toc-a-usage-advanced",
                href: "#blocks-docs-layout-toc-a-usage-advanced",
                title: "応用操作",
                children: &[],
            },
        ],
    },
    TocEntry {
        id: "blocks-docs-layout-toc-a-faq",
        href: "#blocks-docs-layout-toc-a-faq",
        title: "よくある質問",
        children: &[],
    },
];

/// インスタンス B（簡素版）の目次項目（[`TOC_A`] と同型、`id`/`href` の
/// 接頭辞のみ `-b-` に差し替えた独立テーブル）。
const TOC_B: &[TocEntry] = &[
    TocEntry {
        id: "blocks-docs-layout-toc-b-overview",
        href: "#blocks-docs-layout-toc-b-overview",
        title: "概要",
        children: &[],
    },
    TocEntry {
        id: "blocks-docs-layout-toc-b-configuration",
        href: "#blocks-docs-layout-toc-b-configuration",
        title: "設定",
        children: &[
            TocEntry {
                id: "blocks-docs-layout-toc-b-configuration-env",
                href: "#blocks-docs-layout-toc-b-configuration-env",
                title: "環境変数",
                children: &[],
            },
            TocEntry {
                id: "blocks-docs-layout-toc-b-configuration-auth",
                href: "#blocks-docs-layout-toc-b-configuration-auth",
                title: "認証",
                children: &[],
            },
        ],
    },
    TocEntry {
        id: "blocks-docs-layout-toc-b-usage",
        href: "#blocks-docs-layout-toc-b-usage",
        title: "使い方",
        children: &[
            TocEntry {
                id: "blocks-docs-layout-toc-b-usage-basic",
                href: "#blocks-docs-layout-toc-b-usage-basic",
                title: "基本操作",
                children: &[],
            },
            TocEntry {
                id: "blocks-docs-layout-toc-b-usage-advanced",
                href: "#blocks-docs-layout-toc-b-usage-advanced",
                title: "応用操作",
                children: &[],
            },
        ],
    },
    TocEntry {
        id: "blocks-docs-layout-toc-b-faq",
        href: "#blocks-docs-layout-toc-b-faq",
        title: "よくある質問",
        children: &[],
    },
];

/// インスタンス A の現在位置（「設定 › 環境変数」）。
const CURRENT_A_ID: &str = "blocks-docs-layout-toc-a-configuration-env";

/// インスタンス B の現在位置（「設定 › 環境変数」、[`CURRENT_A_ID`] と同型）。
const CURRENT_B_ID: &str = "blocks-docs-layout-toc-b-configuration-env";

/// 本文プレースホルダー段落の共通文言。
const PLACEHOLDER_BODY: &str =
    "本文はプレースホルダーです。実際のページでは、ここに節ごとの解説が入ります。";

/// 節 1 件分の本文（見出し + プレースホルダー段落）を深さ優先で積む。
fn push_body_section(entry: &TocEntry, level: HeadingLevel, out: &mut Vec<Node>) {
    let size = match level {
        HeadingLevel::H3 => HeadingSize::Sm,
        _ => HeadingSize::Xs,
    };
    out.push(heading::heading(
        level,
        &HeadingProps {
            size,
            ..HeadingProps::default()
        },
        vec![("id", entry.id)],
        vec![text(entry.title)],
    ));
    out.push(p(vec![], vec![text(PLACEHOLDER_BODY)]));
    for child in entry.children {
        push_body_section(child, HeadingLevel::H4, out);
    }
}

/// 本文プレースホルダー列（目次の全 `href` に対応する `id` を提供する）。
fn body(entries: &[TocEntry]) -> Node {
    let mut children: Vec<Node> = Vec::new();
    for entry in entries {
        push_body_section(entry, HeadingLevel::H3, &mut children);
    }
    div(vec![("class", "blocks-docs-layout-toc-body")], children)
}

/// 目次リンク 1 件（現在項目にのみ `aria-current="location"` を付ける。
/// モジュール doc「`nav_list::link` ではなく styled `link::root` を使う
/// 理由」参照）。
fn toc_link(entry: &TocEntry, current_id: &str) -> Node {
    let mut attrs: Vec<(&str, &str)> = Vec::new();
    if entry.id == current_id {
        attrs.push(("aria-current", "location"));
    }
    link::root(
        entry.href,
        &LinkProps {
            palette: ColorPalette::Neutral,
            ..LinkProps::default()
        },
        attrs,
        vec![text(entry.title)],
    )
}

/// 目次項目 1 件（`li`。子節があれば入れ子の `nav_list::list` を併設する）。
fn toc_item(entry: &TocEntry, current_id: &str) -> Node {
    let mut children: Vec<Node> = vec![toc_link(entry, current_id)];
    if !entry.children.is_empty() {
        children.push(toc_list(entry.children, current_id));
    }
    nav_list::item(vec![], children)
}

/// 目次リスト（`ul`。再帰呼び出しで入れ子の階層を表す）。
fn toc_list(entries: &[TocEntry], current_id: &str) -> Node {
    nav_list::list(
        vec![],
        entries
            .iter()
            .map(|entry| toc_item(entry, current_id))
            .collect(),
    )
}

/// 目次（`nav`）。`variant` は `"rail"`/`"plain"` のいずれかで、
/// [`LAYOUT_CSS`] のセレクタがこれを読んで見た目を切り替える
/// （`data-blocks-docs-layout-toc-variant`）。
fn toc(variant: &'static str, nav_label: &str, entries: &[TocEntry], current_id: &str) -> Node {
    nav_list::root(
        nav_label,
        vec![
            ("data-blocks-docs-layout-toc-nav", ""),
            ("data-blocks-docs-layout-toc-variant", variant),
        ],
        vec![
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("このページの内容")],
            ),
            toc_list(entries, current_id),
        ],
    )
}

/// インスタンス 1 件（キャプション + 本文列 + 目次列の 2 列レイアウト）。
fn instance(
    variant: &'static str,
    nav_label: &str,
    caption: &str,
    entries: &'static [TocEntry],
    current_id: &str,
) -> Node {
    div(
        vec![("class", "blocks-docs-layout-toc-instance")],
        vec![
            p(
                vec![("class", "blocks-docs-layout-toc-caption")],
                vec![text(caption)],
            ),
            div(
                vec![("class", "blocks-docs-layout-toc-frame")],
                vec![div(
                    vec![("class", "blocks-docs-layout-toc-layout")],
                    vec![body(entries), toc(variant, nav_label, entries, current_id)],
                )],
            ),
        ],
    )
}

/// `docs-layout-toc` の Demo 本体（インスタンス A・B を縦に並べる。呼び
/// 出しごとに同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-docs-layout-toc-stack")],
        vec![
            instance(
                "rail",
                "このページの目次（縦線あり）",
                "縦線（rail）で階層と現在位置を示す版です。",
                TOC_A,
                CURRENT_A_ID,
            ),
            instance(
                "plain",
                "このページの目次（簡素版）",
                "縦線を持たない簡素版です。字下げのみで階層を示します。",
                TOC_B,
                CURRENT_B_ID,
            ),
        ],
    )
}
```

## 原案差分メモ

- 目次リンクの現在地強調には `link` の `current`（`aria-current="page"`）
  ではなく、`attrs` で直接渡した `aria-current="location"` を使って
  います。`current` はページ単位の現在地を表すため、ページ内の節には
  意味論上そぐわないと判断しました。
- 目次ラベルには `nav-list` の `heading`（`h2` 固定）ではなく `text`
  （小さめ・ミュート色）を使っています。`## Demo` の `h2` とアウトライン
  階層が衝突するためです。
- 目次に `position: sticky` は付けていません。Demo 枠が横スクロール
  コンテナ（`overflow-x: auto`）になるため、Demo 内では意図どおりに
  効かないためです。実際のページでは目次に `sticky` を付けられます。
- インスタンス A・B はそれぞれ独立した `id`/`href` の名前空間
  （`blocks-docs-layout-toc-a-*`/`-b-*`）を持ちます。
- docs サイト本体のスクロールスパイが使う `class="docs-toc"` は、本
  block のクラス名と衝突しないよう使っていません。
