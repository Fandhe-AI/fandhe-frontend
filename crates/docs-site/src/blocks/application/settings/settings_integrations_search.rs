//! `settings-integrations-search` block（イシュー #2995。親トラッキング
//! #2951「Blocks アプリケーション B」配下、Application / Settings
//! カテゴリ）。検索欄 + カテゴリ絞り込みボタン群の下へ、連携アプリの
//! 2 列カードグリッドを並べる合成例。主参照・集約元とも R0248（1 件、
//! 代表構成）。`_/blocks-intake/` の対応ファイルは本イシュー着手時点で
//! 本 worktree に存在しないため、原稿・本コメントには対応表 ID のみを
//! 記す（`settings_billing_overview` と同じ扱い）。
//!
//! # 使用部品
//!
//! `field` / `input-group` / `input` / `button` / `card` / `link` の 6 部品
//! を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない（`empty_state` は使用部品に含めないため、
//! 空状態は `card` で合成する）。
//!
//! # 2 パネルで「結果あり」「該当なし」を併記する（無 JS のため静的表示）
//!
//! 1 block・2 パネル縦積みの構成で、検索欄・絞り込み行の構造は同一のまま、
//! グリッド部分だけを次の 2 通りで併記する。
//!
//! - `results` variant: 選択中カテゴリ「チャット」+ 連携アプリカード 4 件。
//! - `empty` variant: 検索語を固定入力した状態 + 該当なしの空状態カード
//!   1 件。
//!
//! # 狭幅では絞り込み行を横スクロールにする
//!
//! 絞り込みボタン群は `40rem` 未満で `overflow-x: auto`（折り返さず横
//! スクロール）、`40rem` 以上で通常の折り返しへ切り替える。カードグリッドは
//! `40rem` 未満で 1 列、以上で 2 列にする（[`LAYOUT_CSS`] 参照）。判定は
//! ビューポート幅ではなくルート（`.blocks-settings-integrations-search-layout`）
//! を基準にした `@container`（コンテナクエリ）で行う。`.docs-content`/
//! `.blocks-demo` の幅制限フレーム内で表示されるため、ビューポート基準の
//! `@media` では切替がフレーム幅とずれる（兄弟 block の
//! `settings_billing_usage` と同じ回避）。空状態カード（[`empty_card`]）は
//! グリッド内で唯一のアイテムのため `grid-column: 1 / -1;` で全幅化し、
//! 2 列時にも半幅タイルへ収まらないようにする。
//!
//! # ダミーリンクは外部 URL に限定する（`linkcheck` 対応）
//!
//! 内部ダミーリンク（`href="#"` 等）は `crate::linkcheck::check_links` の
//! 検証対象になり壊れるため、`link::root` の href は固定の外部 URL
//! （[`REPO`]）を使い `external: true` を指定する
//! （`docs/design/docs-site-blocks-section.md` の既存 block と同じ回避）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。ボタンは `button::button` の既定 `type="button"` のまま送信先を
//! 持たない（`docs/policy/intentional-non-adoption.md` §3.25：バリデーション・
//! 送信処理は UI コンポーネント層の責務外）。文言はすべて独自の架空の
//! 日本語/英語ダミー（実企業名・実クレデンシャル・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldIds, FieldProps, FieldRootProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};

/// ダミーリンクの遷移先（内部固定リンク `href="#"` は `linkcheck` で壊れる
/// ため、実在する外部 URL を使う。モジュール doc「ダミーリンクは外部 URL に
/// 限定する」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 連携アプリ 1 件分のダミーデータ（実ブランド名を避けた架空の名称）。
struct Integration {
    name: &'static str,
    description: &'static str,
    connected: bool,
}

/// 例 A（`results` パネル）で表示する連携アプリ 4 件。
const RESULT_INTEGRATIONS: &[Integration] = &[
    Integration {
        name: "Hinoki Chat",
        description: "チームチャットへ通知を転送します。",
        connected: true,
    },
    Integration {
        name: "Kasumi Talk",
        description: "音声・ビデオ通話を予定に連携します。",
        connected: false,
    },
    Integration {
        name: "Sumire Bot",
        description: "問い合わせ対応を自動化するチャットボットです。",
        connected: false,
    },
    Integration {
        name: "Ao Messenger",
        description: "外部パートナーとのやり取りを一元化します。",
        connected: true,
    },
];

/// 検索欄（`field` + `input-group` + `input`）。
fn search_field(suffix: &'static str, value: Option<&'static str>) -> Node {
    let control_id = format!("blocks-settings-integrations-search-query-{suffix}");
    let field = FieldProps {
        id: &control_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };
    let mut input_attrs = vec![("type", "search"), ("placeholder", "アプリ名で検索")];
    if let Some(value) = value {
        input_attrs.push(("value", value));
    }
    field::root(
        &FieldRootProps::default(),
        &field,
        vec![],
        vec![
            field::label(&field, vec![], vec![text("連携アプリを検索")]),
            input_group::root(
                &group_props,
                vec![],
                vec![
                    input::input(&InputProps::default(), &field, input_attrs),
                    input_group::addon(
                        InputGroupAlign::InlineEnd,
                        &group_props,
                        vec![],
                        vec![input_group::button(
                            &group_props,
                            vec![],
                            vec![text("検索")],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// カテゴリ絞り込みボタン 1 件（選択中は `Solid` + `aria-pressed="true"`、
/// 他は `Outline` + `aria-pressed="false"`）。
fn filter_button(label: &'static str, selected: bool) -> Node {
    button::button(
        &ButtonProps {
            variant: if selected {
                ButtonVariant::Solid
            } else {
                ButtonVariant::Outline
            },
            ..ButtonProps::default()
        },
        vec![
            ("data-blocks-settings-integrations-search-filter", ""),
            ("aria-pressed", if selected { "true" } else { "false" }),
        ],
        vec![text(label)],
    )
}

/// カテゴリ絞り込み行（`role="group"` で 1 グループとして関連付ける）。
fn filters(selected: &'static str) -> Node {
    div(
        vec![
            ("data-blocks-settings-integrations-search-filters", ""),
            ("role", "group"),
            ("aria-label", "カテゴリで絞り込む"),
        ],
        ["すべて", "カレンダー", "チャット", "ストレージ", "分析"]
            .into_iter()
            .map(|label| filter_button(label, label == selected))
            .collect(),
    )
}

/// 連携アプリカード 1 件。
fn integration_card(integration: &Integration) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-integrations-search-card", "")],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text(integration.name)]),
                    card::description(vec![], vec![text(integration.description)]),
                ],
            ),
            card::footer(
                vec![],
                vec![
                    link::root(
                        REPO,
                        &LinkProps {
                            external: true,
                            ..LinkProps::default()
                        },
                        vec![],
                        vec![text("詳細を見る")],
                    ),
                    if integration.connected {
                        button::button(
                            &ButtonProps {
                                variant: ButtonVariant::Outline,
                                disabled: true,
                                ..ButtonProps::default()
                            },
                            vec![],
                            vec![text("接続済み")],
                        )
                    } else {
                        button::button(&ButtonProps::default(), vec![], vec![text("接続する")])
                    },
                ],
            ),
        ],
    )
}

/// 該当なしの空状態カード（`empty-state` 部品は使わず `card` で合成する。
/// モジュール doc「使用部品」節参照）。
fn empty_card() -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-integrations-search-empty", "")],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("該当する連携アプリがありません")]),
                    card::description(
                        vec![],
                        vec![text("検索語や絞り込みを変えて再度お試しください。")],
                    ),
                ],
            ),
            card::footer(
                vec![],
                vec![button::button(
                    &ButtonProps {
                        variant: ButtonVariant::Outline,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text("絞り込みを解除")],
                )],
            ),
        ],
    )
}

/// パネル 1 件（`variant` で検索欄の値・選択中カテゴリ・グリッドの内容を
/// 切り替える）。
fn panel(
    variant: &'static str,
    selected_filter: &'static str,
    search_value: Option<&'static str>,
) -> Node {
    let grid: Vec<Node> = if variant == "results" {
        RESULT_INTEGRATIONS.iter().map(integration_card).collect()
    } else {
        vec![empty_card()]
    };
    div(
        vec![
            ("data-blocks-settings-integrations-search-panel", ""),
            ("data-blocks-settings-integrations-search-variant", variant),
        ],
        vec![
            search_field(variant, search_value),
            filters(selected_filter),
            div(
                vec![("data-blocks-settings-integrations-search-grid", "")],
                grid,
            ),
        ],
    )
}

/// `settings-integrations-search` の Demo 本体（結果あり版・該当なし版の
/// 2 パネルを縦積みで並記する。呼び出しごとに同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-integrations-search-layout")],
        vec![
            panel("results", "チャット", None),
            panel("empty", "すべて", Some("請求書")),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-integrations-search/",
    title: "settings-integrations-search",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_integrations_search.rs",
    demo_class: "blocks-settings-integrations-search",
    parts: &[
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Input Group",
            path: "/themes/input-group/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_integrations_search` 固有のレイアウト規則
/// （`crate::blocks::LAYOUT_CSS` doc「block 固有 CSS の置き場」節と同型で、
/// 本ファイル内 private 定数として `super::stylesheet` 経由の `push_css` で
/// 連結される）。
///
/// セレクタは `.blocks-settings-integrations-search-*` と
/// `[data-blocks-settings-integrations-search-*]` のみを用いる。
///
/// # ルート class を `demo_class` と別名にする理由
///
/// [`Block::demo_class`] は `blocks-settings-integrations-search` だが、
/// `demo()` が返すルート `div` の class は
/// `blocks-settings-integrations-search-layout` という別名にする
/// （`card_heading_toolbar` 等と同じ Bugbot 教訓の回避）。
const LAYOUT_CSS: &str = "\
.blocks-settings-integrations-search-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n  container-type: inline-size;\n  container-name: blocks-settings-integrations-search;\n}\n\
[data-blocks-settings-integrations-search-panel] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  padding: var(--fandhe-space-6);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n}\n\
[data-blocks-settings-integrations-search-panel] [data-scope=\"input-group\"][data-part=\"root\"] {\n  max-width: 28rem;\n}\n\
[data-blocks-settings-integrations-search-filters] {\n  display: flex;\n  gap: var(--fandhe-space-2);\n  flex-wrap: nowrap;\n  overflow-x: auto;\n  padding-bottom: var(--fandhe-space-1);\n}\n\
[data-blocks-settings-integrations-search-grid] {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-settings-integrations-search-card] [data-scope=\"card\"][data-part=\"footer\"] {\n  display: flex;\n  justify-content: space-between;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-settings-integrations-search-empty] {\n  grid-column: 1 / -1;\n  text-align: center;\n}\n\
[data-blocks-settings-integrations-search-empty] [data-scope=\"card\"][data-part=\"footer\"] {\n  justify-content: center;\n}\n\
@container blocks-settings-integrations-search (min-width: 40rem) {\n  [data-blocks-settings-integrations-search-filters] {\n    flex-wrap: wrap;\n    overflow-x: visible;\n  }\n  [data-blocks-settings-integrations-search-grid] {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（field/input-group/input/button/card/link）の
    /// anatomy をすべて実際に出力していること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"field\"",
            "data-scope=\"input-group\"",
            "data-scope=\"field\" data-part=\"input\"",
            "data-scope=\"button\"",
            "data-scope=\"card\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// パネルはちょうど 2 件、空状態カードは 1 件、連携アプリカードは
    /// 4 件出力される。
    #[test]
    fn demo_has_two_panels_and_one_empty_card() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-settings-integrations-search-panel")
                .count(),
            2
        );
        assert_eq!(
            html.matches("data-blocks-settings-integrations-search-empty")
                .count(),
            1
        );
        assert_eq!(
            html.matches("data-blocks-settings-integrations-search-card")
                .count(),
            4
        );
    }

    /// 各パネルにつき選択中フィルタが 1 個だけ（合計 2 件）、非選択が
    /// 8 件（4 個 × 2 パネル）。
    #[test]
    fn exactly_one_filter_selected_per_panel() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"aria-pressed="true""#).count(), 2);
        assert_eq!(html.matches(r#"aria-pressed="false""#).count(), 8);
    }

    /// `<form>` を出力しない・XSS 回帰の不変条件を固定する
    /// （`crate::blocks` モジュール doc）。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in ["<form", "type=\"submit\"", "href=\"#\"", "src=\"data:"] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// 検索欄の id はパネルごとに一意（`results`/`empty` suffix、
    /// `field::input` の派生 id 規則「`{id}-control`」に従う）。
    #[test]
    fn field_ids_are_unique_per_panel() {
        let html = render(&demo());
        assert!(html.contains(r#"id="blocks-settings-integrations-search-query-results-control""#));
        assert!(html.contains(r#"id="blocks-settings-integrations-search-query-empty-control""#));
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイントを持ち、`<` を含まない
    /// （REQ-1: `</style>` によるスタイル脱出を防ぐ）。判定はビューポート
    /// 幅ではなくルート要素基準の `@container`（`.docs-content`/
    /// `.blocks-demo` の幅制限フレーム内対応、兄弟 block
    /// `settings_billing_usage` と同型）であること。
    #[test]
    fn layout_css_declares_breakpoint_and_no_angle_bracket() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(!LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS
            .contains("@container blocks-settings-integrations-search (min-width: 40rem)"));
    }

    /// 空状態カード（[`empty_card`]）はグリッド内で `grid-column: 1 / -1;`
    /// により全幅化され、2 列レイアウトでも半幅タイルへ収まらないこと。
    #[test]
    fn empty_card_spans_full_grid_width() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-settings-integrations-search-empty] {\n  grid-column: 1 / -1;"
        ));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-settings-integrations-search-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-settings-integrations-search-layout"
        );
    }
}
