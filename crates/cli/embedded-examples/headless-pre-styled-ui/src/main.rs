//! `fandhe-frontend-example-headless-pre-styled-ui`: headless-ui /
//! pre-styled-ui コンポーネントショーケースの正本サンプル（イシュー #552、
//! examples 規約 #499 準拠。親トラッキング #520 の Phase 4）。
//!
//! # 役割・呼び出し文脈
//!
//! `fandhe-frontend-headless-ui`（ark-ui 相当の headless UI 層、#520/#522）と
//! `fandhe-frontend-pre-styled-ui`（chakra-ui 相当の pre-styled 上層、
//! #520/#546）の 2 層構成を 1 ページで実演する。各コンポーネントは
//! `fandhe_frontend_core::Node` を返す通常の Rust 関数として組み立て、
//! 静的 HTML 1 ページへレンダリングする。
//!
//! **2 層の使い分け（#552 当時の「シナリオ B」暫定構成からの更新）**:
//! サンプル作成時点（2026-07-22）では pre-styled-ui がクレート骨格のみ
//! だったため headless-ui + 手書き CSS（`static/ui.css`）で代替していたが、
//! pre-styled-ui v0.3.1 で公開 API（styled 部品・headless ラッパー・
//! [`StyleSheet`]/[`Theme`]）が揃ったため統合済み。現在の層別内訳:
//!
//! - **pre-styled-ui の headless ラッパー**: Tabs / Accordion / Dialog
//!   （headless 層のパーツ関数を `pub use` 再エクスポートし、
//!   `data-scope`/`data-part` セレクタへの既定 CSS を `stylesheet()` で追加
//!   提供する薄い委譲層）
//! - **pre-styled-ui の単純 styled 部品**: Button / Badge / Card / Alert /
//!   Spinner（variant/size/colorPalette を Rust enum で型安全に指定する）
//! - **headless-ui + 手書き CSS（残存）**: Switch / RadioGroup / Avatar
//!   （pre-styled-ui にラッパー未提供のため。`static/ui.css` が
//!   `data-scope`/`data-part`/`data-state` セレクタへ直接スタイルを当てる）
//!
//! CSS は [`StyleSheet`] へテーマトークン（[`Theme::default`]）・使用
//! コンポーネントの recipe CSS・headless 残存分の手書き CSS を集約し、
//! `dist/assets/ui.css` 1 ファイルへ書き出す（SSG 向け
//! [`StyleSheet::write_css_file`] 経路の実演）。
//!
//! headless 系コンポーネントは SSR 静的マークアップ（クリック等の実挙動・
//! dispatch 状態遷移は wasm 層の責務、各モジュールの rustdoc 参照）を
//! 組み立てる自由関数のみを使用する。`fandhe_frontend_app::page_shell`
//! （`String` を返す）は使わず、`examples/ssg-blog` と同様に `Node` を返す
//! 自作の [`layout`] でページ骨格を組み立てる。
//!
//! # 学べること
//!
//! - headless-ui の anatomy（`data-scope`/`data-part`）・`data-*` 状態属性・
//!   WAI-ARIA 属性付与の実演（Tabs / Accordion / Dialog / Switch /
//!   RadioGroup / Avatar）
//! - pre-styled-ui の variant API（`ButtonVariant`/`Size`/`ColorPalette` 等の
//!   Rust enum によるクラス切り替え）と [`StyleSheet`] による静的 CSS 集約
//! - 既定エスケープ（REQ-1）: 動的に見える値も含めすべて `text()` 経由でノード
//!   木へ載せ、`raw_html()` や `format!` によるタグ文字列の直接組み立ては
//!   使わない
//! - `@view-transition { navigation: auto; }`（`fandhe_frontend_app::page_shell`
//!   と同一の固定リテラル）による Cross-Document View Transitions の有効化
//!
//! # セキュリティ不変条件（REQ-1・OWASP A01）
//!
//! - HTML はすべて `fandhe_frontend_core` / `fandhe_frontend_headless_ui` /
//!   `fandhe_frontend_pre_styled_ui` のノード木 API で組み立てる。`format!` は
//!   属性値のプレーン文字列整形（id の組み立て等）にのみ使い、タグ文字列の
//!   直接組み立てには使わない。
//! - CSS は [`StyleSheet`]（検証済み CSS のみを保持し `<` を fail-closed で
//!   拒否する型）経由でのみ書き出す。
//! - 出力先パスは `dist/index.html`・`dist/assets/ui.css` の固定リテラルのみ
//!   （外部入力由来のパスを使わない）。

#![forbid(unsafe_code)]

use fandhe_frontend_core::{el, render, text, Node};
use fandhe_frontend_headless_ui::avatar::{self, ImageStatus};
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::radio_group;
use fandhe_frontend_headless_ui::switch;
use fandhe_frontend_headless_ui::OpenState;
use fandhe_frontend_pre_styled_ui::accordion;
use fandhe_frontend_pre_styled_ui::alert::{self, AlertStatus};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardVariant};
use fandhe_frontend_pre_styled_ui::dialog::{self, ContentIds, DialogRole};
use fandhe_frontend_pre_styled_ui::spinner::{spinner, SpinnerProps};
use fandhe_frontend_pre_styled_ui::stylesheet::StyleSheet;
use fandhe_frontend_pre_styled_ui::tabs::{self, ActivationMode, TabItem, TabsProps};
use fandhe_frontend_pre_styled_ui::theme::Theme;
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};
use std::path::Path;

/// ページ共通の骨格（`<html>` 全体）を組み立てる。
///
/// `examples/ssg-blog::layout` と同じ方針: `fandhe_frontend_app::page_shell`
/// は `String` を返すため `Node` 木のみを扱う本サンプルには使えず、自作の
/// `Node` 版として存在する。[`build_stylesheet`] が `dist/assets/ui.css` へ
/// 書き出した CSS を `<link>` で参照する。
fn layout(title: &str, main: Node) -> Node {
    let head = el(
        "head",
        vec![],
        vec![
            el("meta", vec![("charset", "utf-8")], vec![]),
            el(
                "meta",
                vec![
                    ("name", "viewport"),
                    ("content", "width=device-width, initial-scale=1"),
                ],
                vec![],
            ),
            el(
                "style",
                vec![],
                vec![text("@view-transition { navigation: auto; }")],
            ),
            el(
                "link",
                vec![("rel", "stylesheet"), ("href", "/assets/ui.css")],
                vec![],
            ),
            el("title", vec![], vec![text(title)]),
        ],
    );
    let document_body = el("body", vec![], vec![main]);
    el("html", vec![("lang", "ja")], vec![head, document_body])
}

/// 見出し + 説明文の節を組み立てる小さなヘルパー（ページ内の反復を避ける）。
///
/// `description` はショーケースの解説文であり、XSS ペイロードを含む固定
/// 文字列（[`xss_probe`]）を差し込む呼び出し（[`xss_probe_section`]）も
/// 経由するため、`text()` を通した既定エスケープの回帰をここで一元的に
/// 保証する。
fn section(heading: &str, description: &str, body: Vec<Node>) -> Node {
    let mut children = vec![
        el("h2", vec![], vec![text(heading)]),
        el("p", vec![], vec![text(description)]),
    ];
    children.extend(body);
    el("section", vec![], children)
}

/// 横並びのショーケース行（`static/ui.css` の `.showcase-row` が flex 配置を
/// 当てる）。styled 部品を variant 別に並べる節で共有する。
fn showcase_row(children: Vec<Node>) -> Node {
    el("div", vec![("class", "showcase-row")], children)
}

/// Tabs コンポーネント節（`data-scope="tabs"`）。
///
/// pre-styled-ui の headless ラッパー（`fandhe_frontend_pre_styled_ui::tabs` は
/// headless 層の `tabs`/`TabsProps`/`TabItem` を再エクスポートし、既定 CSS を
/// `stylesheet()` で追加提供する）を使う。
fn tabs_section() -> Node {
    let node = tabs::tabs(
        &TabsProps {
            id: "showcase-tabs",
            selected: "profile",
            orientation: Orientation::Horizontal,
            activation_mode: ActivationMode::Automatic,
            loop_focus: true,
            indicator: false,
        },
        vec![
            TabItem {
                value: "profile",
                trigger: vec![text("Profile")],
                content: vec![el(
                    "p",
                    vec![],
                    vec![text("プロフィール情報のパネルです。")],
                )],
                disabled: false,
            },
            TabItem {
                value: "settings",
                trigger: vec![text("Settings")],
                content: vec![el("p", vec![], vec![text("設定パネルです。")])],
                disabled: false,
            },
        ],
    );
    section(
        "Tabs",
        "WAI-ARIA APG の Tabs パターン。マークアップは headless 層、既定 CSS は fandhe_frontend_pre_styled_ui::tabs::stylesheet() が提供します。",
        vec![node],
    )
}

/// Accordion コンポーネント節（`data-scope="accordion"`、single モード想定）。
///
/// pre-styled-ui の headless ラッパー（`fandhe_frontend_pre_styled_ui::accordion`）
/// を使う。状態機械（`Accordion`/`SingleSelect`、dispatch 連携）は wasm 層の
/// 責務（モジュール doc 参照）のため、本サンプルは自由関数のみを直接呼び、
/// 1 項目目が開いた静的な SSR マークアップを実演する。
fn accordion_section() -> Node {
    let items: Vec<(&str, &str, &str, OpenState)> = vec![
        (
            "item-1",
            "What is fandhe-frontend-headless-ui?",
            "ark-ui 相当の headless UI コンポーネント層です。anatomy・data-* 属性・WAI-ARIA 属性付与のための共通 API を提供します。",
            OpenState::Open,
        ),
        (
            "item-2",
            "What does pre-styled-ui add on top?",
            "chakra-ui 相当の pre-styled 上層です。headless 層のパーツ関数を再エクスポートし、テーマトークンと recipe による既定 CSS を追加提供します。",
            OpenState::Closed,
        ),
    ];
    let mut root_children = Vec::new();
    for (value, question, answer, state) in items {
        let trigger_id = format!("{value}-trigger");
        let content_id = format!("{value}-content");
        let item_node = accordion::item(
            state,
            false,
            vec![],
            vec![
                el(
                    "h3",
                    vec![],
                    vec![accordion::item_trigger(
                        state,
                        false,
                        Some(trigger_id.as_str()),
                        Some(content_id.as_str()),
                        vec![],
                        vec![text(question)],
                    )],
                ),
                accordion::item_content(
                    state,
                    Some(content_id.as_str()),
                    Some(trigger_id.as_str()),
                    vec![],
                    vec![el("p", vec![], vec![text(answer)])],
                ),
            ],
        );
        root_children.push(item_node);
    }
    section(
        "Accordion",
        "高々 1 項目が開く single モードの Accordion。既定 CSS は fandhe_frontend_pre_styled_ui::accordion::stylesheet() が提供します。",
        vec![accordion::root(vec![], root_children)],
    )
}

/// Dialog コンポーネント節（`data-scope="dialog"`）。SSR 初期状態は常に
/// closed（`OpenState::Closed`）。開閉の実挙動は wasm 層の責務。
///
/// pre-styled-ui の headless ラッパー（`fandhe_frontend_pre_styled_ui::dialog`）
/// を使う。トリガーは pre-styled-ui の [`button`]（Outline variant）で包まず、
/// headless 層の `dialog::trigger`（`aria-haspopup` 等を持つ専用パーツ）を
/// そのまま使う（anatomy の実演を優先する）。
fn dialog_section() -> Node {
    let state = OpenState::Closed;
    let node = el(
        "div",
        vec![],
        vec![
            dialog::trigger(
                state,
                Some("showcase-dialog-content"),
                vec![],
                vec![text("Open dialog")],
            ),
            dialog::root(
                state,
                vec![],
                vec![
                    dialog::backdrop(state, vec![], vec![]),
                    dialog::positioner(
                        state,
                        vec![],
                        vec![dialog::content(
                            state,
                            DialogRole::Dialog,
                            true,
                            ContentIds {
                                id: Some("showcase-dialog-content"),
                                labelledby: Some("showcase-dialog-title"),
                                describedby: Some("showcase-dialog-description"),
                            },
                            vec![],
                            vec![
                                dialog::title(
                                    Some("showcase-dialog-title"),
                                    vec![],
                                    vec![text("Confirm action")],
                                ),
                                dialog::description(
                                    Some("showcase-dialog-description"),
                                    vec![],
                                    vec![text("この操作は取り消せません。続行しますか？")],
                                ),
                                dialog::close_trigger(vec![], vec![text("Close")]),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    );
    section(
        "Dialog",
        "モーダルダイアログ。SSR 初期状態は closed。既定 CSS は fandhe_frontend_pre_styled_ui::dialog::stylesheet() が提供します。",
        vec![node],
    )
}

/// Button コンポーネント節（`data-scope="button"`、pre-styled-ui の単純
/// styled 部品）。variant / colorPalette / disabled / loading の代表的な
/// 組み合わせを実演する。
fn button_section() -> Node {
    let row = showcase_row(vec![
        button(&ButtonProps::default(), vec![], vec![text("Solid")]),
        button(
            &ButtonProps {
                variant: ButtonVariant::Outline,
                ..ButtonProps::default()
            },
            vec![],
            vec![text("Outline")],
        ),
        button(
            &ButtonProps {
                variant: ButtonVariant::Ghost,
                ..ButtonProps::default()
            },
            vec![],
            vec![text("Ghost")],
        ),
        button(
            &ButtonProps {
                variant: ButtonVariant::Subtle,
                ..ButtonProps::default()
            },
            vec![],
            vec![text("Subtle")],
        ),
        button(
            &ButtonProps {
                palette: ColorPalette::Danger,
                ..ButtonProps::default()
            },
            vec![],
            vec![text("Danger")],
        ),
        button(
            &ButtonProps {
                disabled: true,
                ..ButtonProps::default()
            },
            vec![],
            vec![text("Disabled")],
        ),
        button(
            &ButtonProps {
                loading: true,
                ..ButtonProps::default()
            },
            vec![],
            vec![text("Loading")],
        ),
    ]);
    section(
        "Button",
        "pre-styled-ui の単純 styled 部品。variant/size/colorPalette を Rust enum で型安全に指定します（fandhe_frontend_pre_styled_ui::button）。",
        vec![row],
    )
}

/// Badge コンポーネント節（`data-scope="badge"`、pre-styled-ui の単純
/// styled 部品）。
fn badge_section() -> Node {
    let row = showcase_row(vec![
        badge(&BadgeProps::default(), vec![], vec![text("Subtle")]),
        badge(
            &BadgeProps {
                variant: BadgeVariant::Solid,
                ..BadgeProps::default()
            },
            vec![],
            vec![text("Solid")],
        ),
        badge(
            &BadgeProps {
                variant: BadgeVariant::Outline,
                ..BadgeProps::default()
            },
            vec![],
            vec![text("Outline")],
        ),
        badge(
            &BadgeProps {
                palette: ColorPalette::Success,
                ..BadgeProps::default()
            },
            vec![],
            vec![text("Success")],
        ),
    ]);
    section(
        "Badge",
        "短いステータス表示のための styled 部品（fandhe_frontend_pre_styled_ui::badge）。",
        vec![row],
    )
}

/// Card コンポーネント節（`data-scope="card"`、pre-styled-ui の slot recipe
/// styled 部品）。全部入りコンビニ関数はなく、各パーツ関数を個別に呼び出して
/// 組み立てる契約（`crates/pre-styled-ui/src/card.rs` の rustdoc 参照）。
fn card_section() -> Node {
    let node = card::root(
        CardVariant::Elevated,
        vec![],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("2 層 UI コンポーネント構成")]),
                    card::description(
                        vec![],
                        vec![text("headless-ui の上に pre-styled-ui を重ねる構成の紹介")],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![el(
                    "p",
                    vec![],
                    vec![text(
                        "スタイルが不要なら headless-ui 単体、既定スタイル込みなら pre-styled-ui を使います。",
                    )],
                )],
            ),
            card::footer(
                vec![],
                vec![button(
                    &ButtonProps {
                        variant: ButtonVariant::Ghost,
                        size: Size::Sm,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text("Learn more")],
                )],
            ),
        ],
    );
    section(
        "Card",
        "root/header/body/footer/title/description の 6 パーツで構成する装飾的コンテナ（fandhe_frontend_pre_styled_ui::card）。",
        vec![node],
    )
}

/// Alert コンポーネント節（`data-scope="alert"`、pre-styled-ui の slot recipe
/// styled 部品）。`root` に `role="alert"` が固定付与される。
fn alert_section() -> Node {
    let make = |status: AlertStatus, title_text: &str, description_text: &str| {
        alert::root(
            status,
            vec![],
            vec![
                alert::indicator(vec![], vec![text("!")]),
                alert::content(
                    vec![],
                    vec![
                        alert::title(vec![], vec![text(title_text)]),
                        alert::description(vec![], vec![text(description_text)]),
                    ],
                ),
            ],
        )
    };
    section(
        "Alert",
        "ステータス付き通知バナー（fandhe_frontend_pre_styled_ui::alert）。status ごとに colorPalette が内部で切り替わります。",
        vec![
            make(
                AlertStatus::Info,
                "Info",
                "pre-styled-ui v0.3.1 を統合済みです。",
            ),
            make(
                AlertStatus::Warning,
                "Warning",
                "Switch / RadioGroup / Avatar は headless-ui + 手書き CSS のままです。",
            ),
        ],
    )
}

/// Spinner コンポーネント節（`data-scope="spinner"`、pre-styled-ui の単純
/// styled 部品）。`role="status"` + `aria-label` でスクリーンリーダーへ状態を
/// 伝える。
fn spinner_section() -> Node {
    let row = showcase_row(vec![
        spinner(&SpinnerProps::default()),
        spinner(&SpinnerProps {
            size: Size::Sm,
            ..SpinnerProps::default()
        }),
        spinner(&SpinnerProps {
            palette: ColorPalette::Danger,
            ..SpinnerProps::default()
        }),
    ]);
    section(
        "Spinner",
        "読み込み中表示のための styled 部品（fandhe_frontend_pre_styled_ui::spinner）。",
        vec![row],
    )
}

/// Switch コンポーネント節（`data-scope="switch"`）。
///
/// pre-styled-ui に styled ラッパー未提供のため、headless-ui の自由関数 +
/// 手書き CSS（`static/ui.css`）を維持する（モジュール doc の層別内訳参照）。
fn switch_section() -> Node {
    let checked = true;
    let node = switch::root(
        checked,
        false,
        vec![],
        vec![
            switch::control(
                checked,
                false,
                vec![],
                vec![switch::thumb(checked, vec![], vec![])],
            ),
            switch::label(checked, vec![], vec![text("Enable notifications")]),
            switch::hidden_input("notifications", "on", checked, false, false, vec![]),
        ],
    );
    section(
        "Switch",
        "WAI-ARIA APG の Switch パターン（fandhe_frontend_headless_ui::switch + 手書き CSS。pre-styled-ui 未提供）。",
        vec![node],
    )
}

/// RadioGroup コンポーネント節（`data-scope="radio-group"`）。
///
/// pre-styled-ui に styled ラッパー未提供のため、headless-ui の自由関数 +
/// 手書き CSS（`static/ui.css`）を維持する。
fn radio_group_section() -> Node {
    let options = [
        ("ssr", "SSR", true),
        ("ssg", "SSG", false),
        ("csr", "CSR", false),
    ];
    let mut items = Vec::new();
    for (value, label_text, checked) in options {
        items.push(radio_group::item(
            checked,
            false,
            value,
            vec![],
            vec![
                radio_group::item_hidden_input(checked, false, Some("render-mode"), value, vec![]),
                radio_group::item_control(checked, false, vec![]),
                radio_group::item_text(checked, false, vec![], vec![text(label_text)]),
            ],
        ));
    }
    let node = radio_group::root(
        false,
        Some(Orientation::Vertical),
        Some("render-mode-label"),
        vec![],
        std::iter::once(radio_group::label(
            Some("render-mode-label"),
            vec![],
            vec![text("Render mode")],
        ))
        .chain(items)
        .collect(),
    );
    section(
        "RadioGroup",
        "択一選択の RadioGroup（fandhe_frontend_headless_ui::radio_group + 手書き CSS。pre-styled-ui 未提供）。",
        vec![node],
    )
}

/// Avatar コンポーネント節（`data-scope="avatar"`）。画像読み込み失敗
/// （[`ImageStatus::Error`]）状態を実演し、フォールバック（イニシャル）が
/// 表示されることを示す（実画像を同梱しない静的サンプルのため）。
///
/// pre-styled-ui に styled ラッパー未提供のため、headless-ui の自由関数 +
/// 手書き CSS（`static/ui.css`）を維持する。
fn avatar_section() -> Node {
    let status = ImageStatus::Error;
    let node = avatar::root(
        vec![],
        vec![
            avatar::image(status, "/nonexistent.png", "User avatar", vec![]),
            avatar::fallback(status, vec![], vec![text("FT")]),
        ],
    );
    section(
        "Avatar",
        "画像読み込み状態（loading/loaded/error）に応じて表示を切り替える Avatar（fandhe_frontend_headless_ui::avatar + 手書き CSS。pre-styled-ui 未提供）。",
        vec![node],
    )
}

/// 既定エスケープ（REQ-1）の実演節。`<script>` を含む固定文字列を
/// [`accordion::item_trigger`] の子ノードへ渡し、出力が実体参照化される
/// ことを示す（`tests/cli_output.rs` が同じ入力で回帰を固定する）。
fn xss_probe() -> &'static str {
    "<script>alert('xss')</script>"
}

fn xss_probe_section() -> Node {
    let state = OpenState::Closed;
    let node = accordion::root(
        vec![],
        vec![accordion::item(
            state,
            false,
            vec![],
            vec![
                el(
                    "h3",
                    vec![],
                    vec![accordion::item_trigger(
                        state,
                        false,
                        None,
                        None,
                        vec![],
                        vec![text(xss_probe())],
                    )],
                ),
                accordion::item_content(state, None, None, vec![], vec![]),
            ],
        )],
    );
    section(
        "既定エスケープの実演",
        "コンポーネントの子ノードへ渡した文字列は常にエスケープされます（raw_html() は使用していません）。",
        vec![node],
    )
}

/// ショーケースページ全体を組み立てる。
fn build_page() -> Node {
    let main = el(
        "main",
        vec![],
        vec![
            el("h1", vec![], vec![text("headless-ui / pre-styled-ui showcase")]),
            el(
                "p",
                vec![],
                vec![text(
                    "fandhe-frontend-headless-ui / fandhe-frontend-pre-styled-ui の 2 層 UI コンポーネント構成を静的 SSR マークアップとして実演します。",
                )],
            ),
            tabs_section(),
            accordion_section(),
            dialog_section(),
            button_section(),
            badge_section(),
            card_section(),
            alert_section(),
            spinner_section(),
            switch_section(),
            radio_group_section(),
            avatar_section(),
            xss_probe_section(),
        ],
    );
    layout("headless-ui / pre-styled-ui showcase", main)
}

/// `dist/assets/ui.css` へ書き出す CSS 全量を [`StyleSheet`] へ集約する。
///
/// 取り込み順（前段のトークン定義を後段の recipe が `var(--fandhe-...)` で
/// 参照する）:
///
/// 1. テーマトークン（[`Theme::default`]。ライト/ダーク両対応の
///    `--fandhe-color-*` 等）
/// 2. ページ骨格 + headless 残存分（Switch / RadioGroup / Avatar）の手書き
///    CSS（`static/ui.css`、`include_str!` でバイナリへ埋め込み）
/// 3. 本ページで使用する pre-styled-ui コンポーネントの recipe CSS
///
/// 手書き CSS は [`StyleSheet::push_css`] の fail-closed 検証（`<` 拒否）を
/// 経由させるため、検証エラーを `Err` として呼び出し元（[`main`]）へ返す
/// （pre-styled-ui 生成分は検証済み契約のため infallible な
/// `push_recipe`/`push_theme` 相当の扱いで `push_css` が常に `Ok` になる。
/// `crates/pre-styled-ui/src/stylesheet.rs` の
/// `push_recipe_is_infallible_for_all_styled_components` 参照）。
fn build_stylesheet() -> Result<StyleSheet, fandhe_frontend_pre_styled_ui::StylesheetError> {
    let mut sheet = StyleSheet::new();
    sheet.push_theme(&Theme::default());
    sheet.push_css(include_str!("../static/ui.css"))?;
    for css in [
        fandhe_frontend_pre_styled_ui::tabs::stylesheet(),
        fandhe_frontend_pre_styled_ui::accordion::stylesheet(),
        fandhe_frontend_pre_styled_ui::dialog::stylesheet(),
        fandhe_frontend_pre_styled_ui::button::css(),
        fandhe_frontend_pre_styled_ui::badge::css(),
        fandhe_frontend_pre_styled_ui::card::css(),
        fandhe_frontend_pre_styled_ui::alert::css(),
        fandhe_frontend_pre_styled_ui::spinner::css(),
    ] {
        sheet.push_css(&css)?;
    }
    Ok(sheet)
}

/// `dist/` へ書き出す。出力先は固定リテラルのみ（外部入力由来のパスは
/// 扱わない）。CSS は [`build_stylesheet`] が集約した [`StyleSheet`] を
/// [`StyleSheet::write_css_file`]（SSG 向け経路）で書き出す。
fn main() {
    let page = build_page();
    // `<!DOCTYPE html>` はユーザー入力を一切含まない固定リテラルとして
    // `render()` 済みの既定エスケープ済み HTML の前に結合するのみであり、
    // 新たなエスケープ迂回経路ではない（`crates/app/src/lib.rs` の
    // `page_shell` と同じ方針）。
    let html = format!("<!DOCTYPE html>\n{}", render(&page));

    let dist = Path::new("dist");
    let assets = dist.join("assets");
    if let Err(err) = std::fs::create_dir_all(&assets) {
        eprintln!("failed to create dist/assets: {err}");
        std::process::exit(1);
    }
    if let Err(err) = std::fs::write(dist.join("index.html"), html) {
        eprintln!("failed to write dist/index.html: {err}");
        std::process::exit(1);
    }
    let sheet = match build_stylesheet() {
        Ok(sheet) => sheet,
        Err(err) => {
            eprintln!("failed to assemble ui.css: {err}");
            std::process::exit(1);
        }
    };
    if let Err(err) = sheet.write_css_file(&assets.join("ui.css")) {
        eprintln!("failed to write dist/assets/ui.css: {err}");
        std::process::exit(1);
    }

    println!("dist/index.html");
    println!("dist/assets/ui.css");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ページ生成関数の純関数スモーク: panic せず組み立てられ、headless 系・
    /// pre-styled 系両層の各コンポーネントの anatomy セレクタ（`data-scope`）
    /// が出力に含まれることを固定する。
    #[test]
    fn build_page_includes_all_component_scopes() {
        let html = render(&build_page());
        for scope in [
            // pre-styled-ui の headless ラッパー経由（マークアップは headless 層）
            "data-scope=\"tabs\"",
            "data-scope=\"accordion\"",
            "data-scope=\"dialog\"",
            // pre-styled-ui の単純 styled 部品
            "data-scope=\"button\"",
            "data-scope=\"badge\"",
            "data-scope=\"card\"",
            "data-scope=\"alert\"",
            "data-scope=\"spinner\"",
            // headless-ui + 手書き CSS（pre-styled-ui 未提供）
            "data-scope=\"switch\"",
            "data-scope=\"radio-group\"",
            "data-scope=\"avatar\"",
        ] {
            assert!(html.contains(scope), "missing {scope} in rendered page");
        }
    }

    /// anatomy・`data-state`・ARIA 属性の検証（受け入れ条件(a)）。
    #[test]
    fn tabs_section_renders_aria_and_data_state() {
        let html = render(&tabs_section());
        assert!(html.contains(r#"role="tablist""#));
        assert!(html.contains(r#"aria-selected="true""#));
        assert!(html.contains(r#"data-state="active""#));
    }

    #[test]
    fn dialog_section_renders_closed_state_with_aria_modal() {
        let html = render(&dialog_section());
        assert!(html.contains(r#"data-state="closed""#));
        assert!(html.contains(r#"aria-modal="true""#));
        assert!(html.contains("hidden"));
    }

    #[test]
    fn switch_section_renders_checked_state() {
        let html = render(&switch_section());
        assert!(html.contains(r#"data-state="checked""#));
        assert!(html.contains(r#"role="switch""#));
    }

    /// pre-styled-ui の variant API（受け入れ条件: 統合後の styled 部品実演）:
    /// Button の recipe 生成クラスが variant/size/colorPalette の enum 指定
    /// どおりに出力へ現れることを固定する。
    #[test]
    fn button_section_renders_recipe_variant_classes() {
        let html = render(&button_section());
        assert!(html.contains("fd-button--variant-solid"));
        assert!(html.contains("fd-button--variant-outline"));
        assert!(html.contains("fd-button--color-palette-danger"));
        assert!(html.contains(r#"type="button""#));
        // loading ボタンは disabled + aria-busy + 装飾 Spinner を伴う。
        assert!(html.contains(r#"aria-busy="true""#));
        assert!(html.contains(r#"data-scope="spinner""#));
    }

    /// Alert は全ステータス共通で `role="alert"` を固定付与する契約
    /// （`crates/pre-styled-ui/src/alert.rs`）。
    #[test]
    fn alert_section_renders_role_alert_with_status_classes() {
        let html = render(&alert_section());
        assert!(html.contains(r#"role="alert""#));
        assert!(html.contains("fd-alert--status-info"));
        assert!(html.contains("fd-alert--status-warning"));
    }

    /// [`build_stylesheet`] がテーマトークン・pre-styled recipe・手書き残存
    /// CSS の 3 系統すべてを集約し、`<` を含まない（`<style>` 文脈でも安全な）
    /// CSS を返すことを固定する。
    #[test]
    fn build_stylesheet_aggregates_theme_recipes_and_manual_css() {
        let sheet = build_stylesheet().expect("all CSS sources should pass validation");
        let css = sheet.as_css();
        // 1. テーマトークン（Theme::default）
        assert!(css.contains("--fandhe-color-"));
        // 2. 手書き残存分（Switch は headless + 手書き CSS）
        assert!(css.contains(r#"[data-scope="switch"][data-part="control"]"#));
        // 3. pre-styled recipe（ラッパー分 + 単純 styled 部品分）
        assert!(css.contains(r#"[data-scope="tabs"][data-part="trigger"]"#));
        assert!(css.contains(".fd-button--variant-solid"));
        assert!(!css.contains('<'));
    }

    /// 既定エスケープ回帰（REQ-1、受け入れ条件(b)）: `<script>` を含む
    /// トリガーラベルが実体参照化されて出力され、生の `<script>` タグとして
    /// 現れないことを固定する。
    #[test]
    fn xss_probe_section_escapes_script_payload() {
        let html = render(&xss_probe_section());
        assert!(!html.contains("<script>alert"));
        assert!(html.contains("&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"));
    }

    /// 既定エスケープ回帰: ページ全体の出力にも生の `<script>` が現れない
    /// ことを固定する（`xss_probe_section` がページに含まれているため）。
    #[test]
    fn build_page_output_contains_no_raw_script_payload() {
        let html = render(&build_page());
        assert!(!html.contains("<script>alert"));
    }
}
