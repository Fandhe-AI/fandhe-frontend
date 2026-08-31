//! Typography カテゴリ追補（`site/nav.toml` の `title = "Typography"`）の
//! Quote / Strong 2 ページ分の原稿データ（イシュー #995、親 #928）。
//!
//! 既存 6 部品（Blockquote/Code/Em/Heading/Highlight/Kbd/Link/List/Mark/
//! Text 相当）は [`crate::component_page_specs_948`] が供給済みのため触れず、
//! 本モジュールは新規 2 ページのみを追加する（`crate::component_page::mod`
//! doc の「後続 issue が `typography` 等を追加する想定」を実装する初例）。
//!
//! # 責務境界・呼び出し文脈
//!
//! [`crate::component_page::generated_content`] が `page_path` から
//! [`SPECS`] を線形探索し、Features / API Reference の引数表 / Examples /
//! Accessibility の各節を合成する（[`crate::component_page::ComponentPageSpec`]
//! 参照）。Demo 節は [`crate::showcase::COMPONENT_PAGES`]（正）の
//! `quote_section`/`strong_section` から供給されるため、本モジュールの
//! `demo` フィールドは両方とも `None`。
//!
//! # 一次情報・非捏造の方針
//!
//! - Features: `fandhe-frontend-pre-styled-ui` の `quote::quote` /
//!   `strong::strong` のシグネチャ・module doc から採る。
//! - Arguments: 両部品とも variant 軸を持たないため空（`em`/`link_overlay`
//!   と同型の判断）。
//! - Accessibility（`aria`/`keyboard`）: 両部品とも `role`/`aria-*` を
//!   付与しない素の HTML 意味論のみのため空のままとする
//!   （`docs/design/docs-site-component-pages.md` §7、Accessibility 節は
//!   自動省略される）。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! 本ファイルはノード木 API のみで組み立て、`raw_html()` を使わない
//! （`crates/docs-site/tests/component_pages.rs::component_page_source_does_not_use_raw_html`
//! が `component_specs/` 配下を再帰走査してこれを固定する）。

use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::quote::quote;
use fandhe_frontend_pre_styled_ui::strong::strong;

use crate::component_page::{ComponentPageSpec, ExampleEntry};

/// 横並びのデモ列（`showcase.rs::row` / `component_page_specs_948::row` と
/// 同型、`SHOWCASE_LAYOUT_CSS` の `.showcase-row` を使う）。
fn row(children: Vec<Node>) -> Node {
    div(vec![("class", "showcase-row")], children)
}

fn quote_example() -> Node {
    row(vec![el(
        "p",
        vec![],
        vec![
            text("彼はこう言った、"),
            quote(vec![], vec![text("為せば成る")]),
            text("と。"),
        ],
    )])
}

const QUOTE: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "素の <q> 要素をそのまま styled 化した短いインライン引用部品",
        "variant 軸を持たない最小部品（em/link_overlay と同型）",
        "ブラウザ既定の引用符生成コンテンツ（q::before/q::after）は上書きしない",
    ],
    arguments: &[],
    examples: &[ExampleEntry {
        title: "文中の短い引用",
        description: "地の文の一部を <q> で短いインライン引用として囲みます。",
        render: quote_example,
    }],
    keyboard: &[],
    aria: &[],
    demo: None,
};

fn strong_example() -> Node {
    row(vec![el(
        "p",
        vec![],
        vec![
            text("この操作は"),
            strong(vec![], vec![text("元に戻せません")]),
            text("。"),
        ],
    )])
}

const STRONG: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "素の <strong> 要素をそのまま styled 化した重要性の強調テキスト部品",
        "variant 軸を持たない最小部品（em/link_overlay と同型）",
        "font-weight: bold（em は font-style: italic のみで weight は継承）と見た目を区別する",
    ],
    arguments: &[],
    examples: &[ExampleEntry {
        title: "重要性の強調",
        description: "地の文の一部を <strong> で重要性の強調として囲みます。",
        render: strong_example,
    }],
    keyboard: &[],
    aria: &[],
    demo: None,
};

/// Quote / Strong 2 ページ（イシュー #995）の `path -> ComponentPageSpec`
/// テーブル。[`crate::component_page::SPEC_TABLES`] が集約する。
pub const SPECS: &[(&str, ComponentPageSpec)] =
    &[("/themes/quote/", QUOTE), ("/themes/strong/", STRONG)];
