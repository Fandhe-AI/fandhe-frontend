//! Clipboard（Interactive）/ Skip Nav（Utilities）の Themes 部品ページ原稿
//! （イシュー #1155。#1154〔link / link-overlay / nav-list〕の兄弟イシュー・
//! 同型構成）。
//!
//! # 背景・呼び出し文脈
//!
//! `/themes/clipboard/` と `/themes/skip-nav/` は
//! [`crate::showcase::COMPONENT_PAGES`] に未登録（Demo 節を
//! `showcase.rs` から供給されない）2 部品である。[`crate::component_page_specs_948`]
//! モジュール doc はかつて「この 2 ページへ `ComponentPageSpec` を登録して
//! もデッドコードになる」と記していたが、これは #979 が
//! [`crate::component_page::ComponentPageSpec::demo`]（Demo フォールバック
//! 供給口）を導入する**前**の記述であり、angle-slider / image-cropper /
//! pin-input / signature-pad の 4 部品（[`crate::component_specs::forms`]
//! 参照）が同機構で既に充填済みである。本モジュールはこの機構の最後の
//! 未適用 2 件を埋める（`showcase.rs` 自体は Demo 節目的では変更しない、
//! #945 の受け入れ条件を踏襲）。
//!
//! [`crate::component_page::generated_content`] が `page_path` から
//! [`SPECS`] を線形探索し、Features / API Reference の引数表 / Examples /
//! Accessibility の各節を合成する。Demo 節は本モジュール末尾の `demo_*`
//! 関数が [`ComponentPageSpec::demo`] 経由で直接供給する。
//!
//! # 一次情報・非捏造の方針
//!
//! - Clipboard: `crates/headless-ui/src/clipboard.rs` モジュール doc・
//!   関数シグネチャ、`crates/pre-styled-ui/src/clipboard.rs` モジュール doc
//!   （styled 薄ラッパーの選択的 re-export 方針）から採る。
//! - Skip Nav: `crates/headless-ui/src/skip_nav.rs` モジュール doc・
//!   関数シグネチャ、`crates/pre-styled-ui/src/skip_nav.rs` モジュール doc
//!   （純 CSS `:focus-visible` 表現、`DEFAULT_ID`）から採る。
//! - Keyboard: 本 docs サイトは `crate::script`（テーマトグル・目次
//!   スクロールスパイ・検索 UI）以外の JS を出力せず、headless-ui も状態
//!   機械のみで JS を配線しない。JS 前提のキー操作は「対応済み」と書かず、
//!   ネイティブ要素（`<button>`/`<a>`/`tabindex`）由来のブラウザ標準操作
//!   のみを記載する（`component_specs/forms.rs` と同じ方針）。
//! - 責務境界（`docs/policy/intentional-non-adoption.md` §3.25）: 実際の
//!   `navigator.clipboard.writeText` 書き込み・コピー完了後の自動リセット
//!   はクライアント配線層（`fandhe-frontend-wasm-full`）の責務であり、本
//!   ファイルはこれらを部品の機能として記述しない。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! 本ファイルはノード木 API（[`fandhe_frontend_core`] /
//! [`fandhe_frontend_pre_styled_ui`]）のみで Demo・Examples を組み立て、
//! `raw_html()` および HTML 文字列の直接組み立てを一切使わない
//! （`component_specs/` 配下を再帰走査する
//! `tests/component_pages.rs::component_page_source_does_not_use_raw_html`
//! が本ファイルも対象に含める）。Clipboard の `value`（コピー対象値）は
//! パスワード等の機微情報を含みうる契約のため、Demo・Examples ではダミー
//! URL のみを使う（`crates/pre-styled-ui/src/clipboard.rs` の「`value` を
//! CSS・ログへ出力しない」不変条件を利用者向けドキュメントとして継承）。
//! Skip Nav の Demo は [`fandhe_frontend_pre_styled_ui::skip_nav::DEFAULT_ID`]
//! を使わずカスタム id を使う（全ページ共通のレイアウト実適用分
//! `id="fandhe-skip-nav"` との重複回避。`tests/component_specs_1155.rs` が
//! 回帰を固定する）。

use fandhe_frontend_core::{el, p, text, Node};
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::skip_nav;

use crate::component_page::{ArgRow, ComponentPageSpec, ExampleEntry, KeyRow};

/// 部品 1 件分の Demo 節を組み立てる（`component_specs/forms.rs::demo_section`
/// と同型。`crate::component_page::strip_demo_heading` が先頭 `h2` を 1 個
/// だけ剥がす前提の `div > section > [h2, p, …]` 構造に合わせる）。
fn demo_section(heading: &str, description: &str, demo: Node) -> Node {
    el(
        "div",
        vec![],
        vec![el(
            "section",
            vec![],
            vec![
                el("h2", vec![], vec![text(heading)]),
                p(vec![], vec![text(description)]),
                demo,
            ],
        )],
    )
}

// ---------------------------------------------------------------------
// Clipboard（/themes/clipboard/）
// ---------------------------------------------------------------------

/// Demo: 未コピー状態。Root/Label/Control/Input/Trigger/Indicator/ValueText
/// の 7 anatomy パーツすべてを描画する（Anatomy 表・`data-*` 属性表の機械
/// 導出元がデモの部分集合になるため、7 パーツすべてを描画して表を完全に
/// する。`crate::primitive_showcase::forms_c_date_status::clipboard_section`
/// と同型構成を styled API へ移植）。
fn demo_clipboard() -> Node {
    demo_section(
        "Clipboard",
        "共有 URL 等のコピー対象値を表示し、ボタン押下でコピー状態を切り替える部品。コピー実処理・自動リセットはクライアント配線層（wasm-full）の責務です。",
        clipboard::root(
            "https://example.com/share/abc",
            false,
            vec![],
            vec![
                clipboard::label(vec![], vec![text("Share link")]),
                clipboard::control(
                    false,
                    vec![],
                    vec![
                        clipboard::input("https://example.com/share/abc", false, vec![]),
                        clipboard::trigger(
                            false,
                            vec![],
                            vec![
                                clipboard::indicator(false, false, vec![], vec![text("Copy")]),
                                clipboard::indicator(
                                    true,
                                    false,
                                    vec![],
                                    vec![text("Copied")],
                                ),
                            ],
                        ),
                    ],
                ),
                clipboard::value_text(vec![], vec![text("https://example.com/share/abc")]),
            ],
        ),
    )
}

/// Examples: コピー済み状態（`copied = true`）。Demo は未コピー状態のみを
/// 描画するため、`indicator` の可視性が入れ替わる変種を補完する。
fn clipboard_copied_state_example() -> Node {
    clipboard::root(
        "https://example.com/share/abc",
        true,
        vec![],
        vec![
            clipboard::label(vec![], vec![text("Share link")]),
            clipboard::control(
                true,
                vec![],
                vec![
                    clipboard::input("https://example.com/share/abc", true, vec![]),
                    clipboard::trigger(
                        true,
                        vec![],
                        vec![
                            clipboard::indicator(false, true, vec![], vec![text("Copy")]),
                            clipboard::indicator(true, true, vec![], vec![text("Copied")]),
                        ],
                    ),
                ],
            ),
            clipboard::value_text(vec![], vec![text("https://example.com/share/abc")]),
        ],
    )
}

const CLIPBOARD: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "headless-ui の Root/Label/Control/Input/Trigger/Indicator/ValueText 7 anatomy パーツを styled 薄ラッパーとして再エクスポートし、styled `root` のみ `class` 属性を除去して再定義する。",
        "`size`/`colorPalette` variant は提供しない（hover-card / toggle-tip と同じ判断。variant 展開は別途一括検討する）。",
        "コピー状態は値語彙ではなく `data-copied`（存在属性）で表現し、`indicator` は copied 用/idle 用の 2 変種を SSR で両方描画したうえで、現在状態と不一致の側へ `hidden` を付与する（子孫セレクタを使わない表示切り替え）。",
        "`input` パーツはコピー元テキストの表示専用（`readonly`）であり `name` を持たず、フォーム送信を目的としない。",
        "実際の `navigator.clipboard.writeText` 書き込み・コピー完了後の自動リセットはクライアント配線層の責務であり、本コンポーネントは状態遷移の表現のみを提供する（`docs/policy/intentional-non-adoption.md` §3.25）。コピー対象値（`value`）はパスワード等の機微情報を含みうるため、CSS・ログのいずれにも出力しない。",
    ],
    arguments: &[
        ArgRow {
            name: "value",
            kind: "&str",
            default: "（必須）",
            description: "コピー対象値。`root` の `data-value` としてそのまま出力される（既定エスケープ経由）。クライアント側はこの `data-value` を読み取って実際の書き込みを行う契約。",
        },
        ArgRow {
            name: "copied",
            kind: "bool",
            default: "false",
            description: "コピー済みかどうか。`data-copied`・`indicator` の可視性・`input`/`control`/`trigger` の `data-copied` へ反映される。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "vec![]",
            description: "各パーツへ渡す追加属性。styled `root` は呼び出し側 `class` を除去してから合成する。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "vec![]",
            description: "パーツ配下に描画する子ノード。",
        },
    ],
    examples: &[ExampleEntry {
        title: "コピー済み状態",
        description: "`copied = true` を渡すと `data-copied` が付与され、`indicator` の可視性が入れ替わります（コピー用 → コピー済み用）。",
        render: clipboard_copied_state_example,
    }],
    keyboard: &[KeyRow {
        key: "Tab / Space / Enter",
        description: "`trigger` はネイティブ `<button type=\"button\">` であり、フォーカス・押下操作はブラウザ既定動作で成立する。",
    }],
    aria: &[],
    demo: Some(demo_clipboard),
};

// ---------------------------------------------------------------------
// Skip Nav（/themes/skip-nav/）
// ---------------------------------------------------------------------

/// Demo 専用のカスタム id。docs サイト自身がページ骨格
/// （`crate::layout::docs_page_with_assets`）で
/// [`fandhe_frontend_pre_styled_ui::skip_nav::DEFAULT_ID`]（`"fandhe-skip-nav"`）
/// を全ページへ実適用しているため、部品ページ内 Demo で同じ id を使うと
/// `id` 重複（HTML 仕様違反かつ `href="#…"` 関連付け破壊）を起こす。個別
/// ページの Demo はカスタム id を使うことでレイアウト実適用分と衝突なく
/// 成立させる（Primitives 側 `/primitives/skip-nav/` の先例、イシュー #1022）。
const DEMO_ID: &str = "themes-skip-nav-demo";

/// Demo: link はキーボードフォーカス時のみ視覚的に表示される（純 CSS
/// `:focus-visible`）ため、見た目上はほぼ空になる。フォーカス操作
/// （`Tab` キー）で挙動を確認できる旨を説明文に明記する。
fn demo_skip_nav() -> Node {
    demo_section(
        "Skip Nav",
        "本文へのスキップリンク（WCAG 2.1 SC 2.4.1 Bypass Blocks）。link はキーボードフォーカス時のみ視覚的に表示されるため、Tab キーでフォーカスするまで見た目には現れません（このデモ枠内を Tab フォーカスして確認してください）。",
        el(
            "div",
            vec![],
            vec![
                skip_nav::link(DEMO_ID, vec![], vec![text("Skip to demo content")]),
                skip_nav::content(DEMO_ID, vec![], vec![text("Demo content starts here.")]),
            ],
        ),
    )
}

/// Examples: カスタム id の使用例。docs サイト自身が `DEFAULT_ID` を全
/// ページで使用済みのため、部品利用側が別 id を選ぶ実例として意味を持つ。
fn skip_nav_custom_id_example() -> Node {
    el(
        "div",
        vec![],
        vec![
            skip_nav::link(
                "themes-skip-nav-example",
                vec![],
                vec![text("Skip to example content")],
            ),
            skip_nav::content(
                "themes-skip-nav-example",
                vec![],
                vec![text("Example content starts here.")],
            ),
        ],
    )
}

const SKIP_NAV: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "Link / Content の 2 anatomy パーツで WCAG 2.1 SC 2.4.1 Bypass Blocks を実現する。styled `link`/`content` は呼び出し側 `class` を除去してから headless-ui へ委譲する薄ラッパー。",
        "`link` は任意の URL を呼び出し側から受け取らず、常に `#<id>`（フラグメントのみ）を内部で組み立てるためスキーム注入経路を持たない。",
        "focus していないときは視覚的に隠し（clip 手法）、キーボードフォーカス時（`:focus-visible`）のみ表示する。docs サイトは hydration を持たないため、この表示切り替えは純 CSS のみで成立する（クライアント配線不要）。",
        "`DEFAULT_ID` 定数（`\"fandhe-skip-nav\"`）を提供し、ページ全体に 1 個だけ配置する典型利用を想定する。docs サイト自身もページ骨格へこの id で全ページ 1 個ずつ実適用している。",
    ],
    arguments: &[
        ArgRow {
            name: "link: id",
            kind: "&str",
            default: "skip_nav::DEFAULT_ID",
            description: "スキップ先 id。href=\"#<id>\" として出力する。",
        },
        ArgRow {
            name: "content: id",
            kind: "&str",
            default: "skip_nav::DEFAULT_ID",
            description: "id 属性値。link の href と対にする。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "vec![]",
            description: "各パーツへ渡す追加属性。`class` は除去してから合成する。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "vec![]",
            description: "パーツ配下に描画する子ノード。",
        },
    ],
    examples: &[ExampleEntry {
        title: "カスタム id",
        description: "`DEFAULT_ID` ではなく呼び出し側指定の id を使う例です。1 ページに複数の skip-nav リンクを配置する場合、それぞれ別 id を割り当てます。",
        render: skip_nav_custom_id_example,
    }],
    keyboard: &[KeyRow {
        key: "Tab",
        description: "`link` はページ内で最初にフォーカス可能な要素として配置する運用を想定し、フォーカス時のみ視覚的に表示される。",
    }],
    aria: &[],
    demo: Some(demo_skip_nav),
};

/// `path -> ComponentPageSpec` テーブル
/// （[`crate::component_page::SPEC_TABLES`] へ 1 行追記される）。
pub const SPECS: &[(&str, ComponentPageSpec)] = &[
    ("/themes/clipboard/", CLIPBOARD),
    ("/themes/skip-nav/", SKIP_NAV),
];
