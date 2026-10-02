//! `docs-layout-toc` block（イシュー #3110。親トラッキング #3099
//! 「Blocks docs（phase:6）」配下、Docs Layout カテゴリ）。サイドバー用の
//! ページ内目次（最も基本的な形。開閉式の兄弟 [`super::docs_layout_toc_collapsible`]・
//! 番号付き進捗表示の兄弟 [`super::docs_layout_toc_progress`] に対する基本形）。
//! `_/blocks-intake/` の対応ファイルは本 worktree に存在しないため、
//! 原稿・本コメントには対応表 ID（R0364/R0365）のみを記す（兄弟 block と
//! 同じ扱い）。
//!
//! # 使用部品
//!
//! `link` / `nav_list` / `heading` / `text` の 4 部品を合成する（[`BLOCK`]
//! の `parts` に一致させる契約）。目次の階層構造自体（`nav` > `ul` > `li`
//! の入れ子）は [`fandhe_frontend_pre_styled_ui::nav_list`] の
//! `root`/`list`/`item` を使う。
//!
//! # 2 版を併記する（対応表 ID R0364・R0365）
//!
//! - **インスタンス A**（`variant="rail"`、R0364 主参照）: 各階層の `ul` に
//!   縦線を引き、階層と現在位置をその線で示す。
//! - **インスタンス B**（`variant="plain"`、R0365）: 縦線を持たない簡素版。
//!   字下げのみで階層を示す。
//!
//! # `nav_list::link` ではなく styled `link::root` を使う理由
//!
//! `fandhe_frontend_headless_ui::nav_list::link` は `aria-current`/
//! `data-current` を予約キーとして無条件に除去する（`current` 引数経由の
//! `aria-current="page"` のみ出力可能）。本 block の目次リンクは**ページ内
//! の節**を指すため、ページ単位の現在地を表す `"page"` ではなく
//! `"location"` を付けたい。`docs_layout_toc_collapsible`/
//! `docs_layout_toc_progress` と同じ理由で、`nav_list::link` は使わず
//! styled [`link::root`] へ `attrs` 経由で直接 `aria-current="location"` を
//! 渡す。
//!
//! # 目次ラベルに `nav_list::heading`（固定 `h2`）を使わない理由
//!
//! `nav_list::heading` は headless 側で `h2` 固定であり、`## Demo` の `h2`
//! とアウトライン階層が衝突する（`content_article_toc` と同じ判断）。
//! 目次ラベル「このページの内容」は [`styled_text::text`]（`Sm`/`Muted`）
//! で表す。
//!
//! # リンク先・`id`/`href` の単一情報源
//!
//! 目次の遷移先は本文プレースホルダー列（[`body`]）に置き、節見出しへ
//! [`heading::heading`] の `attrs` で `("id", …)` を付与する
//! （`linkcheck::check_links` が `#fragment` を fail-closed で検証するため、
//! 実在する遷移先が必須。`heading` の root は `data-scope="heading"` を持ち
//! `crate::layout::with_heading_anchors` の走査対象外のため、docs サイト
//! 右目次への誤収集は起きない）。
//!
//! `id`/`href` はインスタンスごとに独立した `'static` 定数テーブル
//! （[`TOC_A`]/[`TOC_B`]）を単一の真実源とし、`format!` では組み立てない
//! （インスタンス間・[`crate::layout::RESERVED_LAYOUT_IDS`]・Markdown 見出し
//! の自動採番 slug・兄弟 block の `blocks-docs-layout-toc-collapsible-*`/
//! `blocks-docs-layout-toc-progress-*` のいずれとも衝突しない接頭辞
//! `blocks-docs-layout-toc-{a,b}-` を持つ）。
//!
//! # 現在位置の表現（静的固定）
//!
//! 各インスタンスは第 2 階層の 1 件（「設定」節の子「環境変数」）を現在
//! 位置として固定する（[`CURRENT_A_ID`]/[`CURRENT_B_ID`]）。階層と現在
//! 位置を同時に示す構成を実演するための選択であり、実際のスクロール追従
//! （scroll spy）は行わない（無 JS の docs サイトの制約、スコープ外節
//! 参照）。
//!
//! # レスポンシブ（本文列 + 目次列）
//!
//! `.blocks-docs-layout-toc-frame`（コンテナクエリの祖先コンテナ）と
//! `.blocks-docs-layout-toc-layout`（列数を切り替える対象）を別要素に
//! 分離する（`docs_layout_toc_progress` と同じ理由。CSS Containment の
//! 仕様上、要素は自身が確立するサイズコンテナに対しては `@container` で
//! 選択されないため、同一要素へ両方を重ねると 2 列化が永久に発火しない）。
//! 既定は 1 列（目次も隠さず本文の下に表示する。目次がこの block の主題の
//! ため、狭い幅でも非表示にはしない）、`@container (min-width: 40rem)` で
//! `minmax(0,1fr) 12rem` の 2 列へ切り替える。
//!
//! # `position: sticky` を使わない
//!
//! `.blocks-demo`（`crate::blocks::LAYOUT_CSS`）が `overflow-x: auto` を
//! 持ち Demo 枠自体がスクロールコンテナになるため `sticky` は意図どおりに
//! 機能しない（`content_article_toc`/`docs_layout_toc_progress` と同じ
//! 判断）。実アプリでは目次に `sticky` を付けられる旨を原稿の「原案差分
//! メモ」へ注記する。
//!
//! # 名前空間
//!
//! class・データ属性フックはすべて `blocks-docs-layout-toc-*`/
//! `data-blocks-docs-layout-toc-*` に収め、`docs-toc`（docs サイト本体の
//! scroll spy が唯一のセレクタとして専有する class）・`-collapsible-`/
//! `-progress-` で始まる名前は一切使わない。
//!
//! # 既存部品へのフックは data 属性で付ける
//!
//! `nav_list::root`/`link::root`/`heading::heading` はいずれも呼び出し側
//! `class` を `drop_class_attr` で除去する（それぞれの rustdoc 参照）ため、
//! これらへのフックは data 属性（`data-blocks-docs-layout-toc-*`）で渡す。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「セキュリティ不変条件」節・
//! `docs/policy/intentional-non-adoption.md` §3.25 に従い、本 Demo は
//! フォーム・送信処理・状態機械を持たない静的な合成例である。文言はすべて
//! 独自の架空の日本語ダミー（実企業名・実クレデンシャル・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::nav_list;
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

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
                    size: TextSize::Sm,
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/docs-layout-toc/",
    title: "docs-layout-toc",
    category: BlockCategory::DocsLayout,
    rust_source: "crates/docs-site/src/blocks/docs/docs_layout/docs_layout_toc.rs",
    demo_class: "blocks-docs-layout-toc",
    parts: &[
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "NavList",
            path: "/themes/nav-list/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `docs_layout_toc` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
///
/// セレクタは `.blocks-docs-layout-toc-*` と `[data-blocks-docs-layout-toc-*]`
/// のみを用いる。
///
/// # 現在項目の強調ルールが当たるためのセレクタ（前回試行からの是正）
///
/// ルート `nav`（[`nav_list::root`]）は `data-blocks-docs-layout-toc-nav`
/// を持つが、素の `class` ではない（`nav_list::root` が呼び出し側 `class`
/// を除去するため）。このため現在項目の強調は
/// `[data-blocks-docs-layout-toc-nav] [data-scope="link"][data-part="root"][aria-current="location"]`
/// （属性セレクタ 4 個、詳細度 `(0,4,0)`）とし、`link` recipe の palette
/// 規則（`(0,3,0)`）を上回るようにする。
///
/// # `link::root` を `display: block` にする
///
/// 既定の `link::root` は inline のため、目次内では行全体をクリック/
/// フォーカスできるよう `display: block` + 縦余白 + `position: relative`
/// （rail 版の現在マーカー `::before` の基準）を与える。
///
/// # rail 版の縦線とマーカーの位置合わせ
///
/// 各階層の `ul`（[`nav_list::list`]。ネストしても同じ
/// `data-scope="nav-list" data-part="list"` を持つ）に
/// `border-inline-start: 1px` + `padding-inline-start: var(--fandhe-space-3)`
/// を揃え、入れ子（子節の `ul`）の字下げは `margin-inline-start` で別途
/// 付ける。現在マーカー（`::before`、`left: calc(-1 * space-3 - 1px)`）は
/// こうすることで常に直近の `ul` の縦線に重なる（前回試行はネストした
/// `ul` に別途 `padding-inline-start: 1rem` が強く当たり、字下げの基準が
/// ずれてマーカーが縦線から外れていた）。plain 版は縦線を持たず、字下げは
/// `padding-inline-start: var(--fandhe-space-4)` のみで表す。
const LAYOUT_CSS: &str = "\
.blocks-docs-layout-toc-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-docs-layout-toc-caption {\n  margin: 0 0 var(--fandhe-space-3);\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
.blocks-docs-layout-toc-frame {\n  container-type: inline-size;\n  container-name: blocks-docs-layout-toc;\n}\n\
.blocks-docs-layout-toc-layout {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-docs-layout-toc-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  min-width: 0;\n}\n\
.blocks-docs-layout-toc-body h3,\n.blocks-docs-layout-toc-body h4 {\n  margin: 0;\n}\n\
.blocks-docs-layout-toc-body p {\n  margin: 0;\n}\n\
[data-blocks-docs-layout-toc-nav] [data-scope=\"link\"][data-part=\"root\"] {\n  display: block;\n  position: relative;\n  padding-block: var(--fandhe-space-1, 0.25rem);\n}\n\
[data-blocks-docs-layout-toc-nav][data-blocks-docs-layout-toc-variant=\"plain\"] [data-scope=\"nav-list\"][data-part=\"list\"] {\n  padding-inline-start: var(--fandhe-space-4);\n}\n\
[data-blocks-docs-layout-toc-nav][data-blocks-docs-layout-toc-variant=\"rail\"] [data-scope=\"nav-list\"][data-part=\"list\"] {\n  border-inline-start: 1px solid var(--fandhe-color-border);\n  padding-inline-start: var(--fandhe-space-3);\n}\n\
[data-blocks-docs-layout-toc-nav][data-blocks-docs-layout-toc-variant=\"rail\"] [data-scope=\"nav-list\"][data-part=\"item\"] [data-scope=\"nav-list\"][data-part=\"list\"] {\n  margin-inline-start: var(--fandhe-space-3);\n}\n\
[data-blocks-docs-layout-toc-nav][data-blocks-docs-layout-toc-variant=\"rail\"] [data-scope=\"link\"][data-part=\"root\"][aria-current=\"location\"]::before {\n  content: \"\";\n  position: absolute;\n  top: 0;\n  bottom: 0;\n  left: calc(-1 * var(--fandhe-space-3) - 1px);\n  width: 2px;\n  background: var(--fandhe-color-accent);\n}\n\
[data-blocks-docs-layout-toc-nav] [data-scope=\"link\"][data-part=\"root\"][aria-current=\"location\"] {\n  color: var(--fandhe-color-accent);\n  font-weight: var(--fandhe-font-font-weight-semibold);\n}\n\
@container blocks-docs-layout-toc (min-width: 40rem) {\n  .blocks-docs-layout-toc-layout {\n    grid-template-columns: minmax(0, 1fr) 12rem;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, CURRENT_A_ID, CURRENT_B_ID, LAYOUT_CSS, TOC_A, TOC_B};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（link/nav-list/heading/text）の anatomy をすべて
    /// 実際に出力していること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"link\"",
            "data-scope=\"nav-list\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// 目次の全 `href` に対応する `id` がちょうど 1 件ずつ存在する
    /// （両インスタンス合計 10 件 x 2）。
    #[test]
    fn every_toc_link_href_resolves_to_exactly_one_id() {
        let html = render(&demo());
        fn assert_entries(html: &str, entries: &[super::TocEntry]) {
            for entry in entries {
                let href = format!("href=\"{}\"", entry.href);
                let id = format!("id=\"{}\"", entry.id);
                assert_eq!(
                    html.matches(&href).count(),
                    1,
                    "missing href for {}",
                    entry.id
                );
                assert_eq!(
                    html.matches(&id).count(),
                    1,
                    "missing/duplicate id for {}",
                    entry.id
                );
                assert_entries(html, entry.children);
            }
        }
        assert_entries(&html, TOC_A);
        assert_entries(&html, TOC_B);
    }

    /// `aria-current="location"` がちょうど 2 件（両インスタンス 1 件ずつ）
    /// で、その `<a>` の `href` が現在項目のものと一致する。
    /// `aria-current="page"` は出力しない。
    #[test]
    fn aria_current_location_targets_the_fixed_current_item() {
        let html = render(&demo());
        assert_eq!(html.matches("aria-current=\"location\"").count(), 2);
        assert_eq!(html.matches("aria-current=\"page\"").count(), 0);

        for current_id in [CURRENT_A_ID, CURRENT_B_ID] {
            let href_attr = format!("href=\"#{current_id}\"");
            let marker_pos = html
                .find(&href_attr)
                .unwrap_or_else(|| panic!("href for current item {current_id} should exist"));
            let tag_start = html[..marker_pos]
                .rfind("<a")
                .expect("anchor open tag should precede its href attribute");
            let tag_end = html[tag_start..].find('>').unwrap() + tag_start;
            let tag = &html[tag_start..tag_end];
            assert!(
                tag.contains("aria-current=\"location\""),
                "current item {current_id} should carry aria-current=\"location\" on its own anchor: {tag}"
            );
        }
    }

    /// `<form>`・死リンク・`data:` URI を出力しない。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in ["<form", "href=\"#\"", "src=\"data:"] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// docs サイト本体の scroll spy class（`docs-toc`）・兄弟 block の
    /// 名前空間（`-collapsible-`/`-progress-`）を一切出力しない。
    #[test]
    fn never_emits_sibling_or_scroll_spy_tokens() {
        let html = render(&demo());
        for absent in [
            "docs-toc",
            "blocks-docs-layout-toc-collapsible",
            "blocks-docs-layout-toc-progress",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
            assert!(!LAYOUT_CSS.contains(absent));
        }
    }

    /// [`LAYOUT_CSS`] が参照する現在項目フック
    /// （`[data-blocks-docs-layout-toc-nav]`）が Demo の出力に実在する
    /// （前回試行の「強調ルールが一度も当たらない」バグの回帰防止）。
    #[test]
    fn layout_css_nav_hook_exists_in_demo_output() {
        let html = render(&demo());
        assert!(html.contains("data-blocks-docs-layout-toc-nav=\"\""));
        assert!(LAYOUT_CSS.contains("[data-blocks-docs-layout-toc-nav]"));
    }

    /// [`LAYOUT_CSS`] は `<` を含まず（REQ-1: `</style>` 脱出防止）、既定
    /// 1 列の規則が `@container` より前にある（モバイルファースト）。
    #[test]
    fn layout_css_is_safe_and_mobile_first() {
        assert!(!LAYOUT_CSS.contains('<'));
        let default_pos = LAYOUT_CSS
            .find(".blocks-docs-layout-toc-layout {")
            .expect("default rule should exist");
        let container_pos = LAYOUT_CSS
            .find("@container blocks-docs-layout-toc")
            .expect("container query should exist");
        assert!(default_pos < container_pos);
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-docs-layout-toc-stack\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-docs-layout-toc-stack");
    }
}
