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
//! [`StyleSheet`]/[`Theme`]）が揃ったため統合済み。Switch / RadioGroup /
//! Avatar の styled ラッパーも v0.4.0（#682/#683/#684、公開 #686）で出揃い、
//! 全部品が pre-styled-ui 経由の styled 提供となったため
//! `fandhe-frontend-headless-ui` への直接依存を撤去した（イシュー #689）。
//! さらに PR #719（pre-styled-ui 0.5.0、イシュー #728 で公開・追随）で
//! Switch / RadioGroup の `root` が Avatar と同じ「styled root（`size`/
//! `palette` variant 付与）」形へ変更されたため、現在の層別内訳は次のとおり:
//!
//! - **pre-styled-ui の headless ラッパー**: Tabs / Accordion / Dialog /
//!   Menu / Select / Popover / Tooltip（headless 層のパーツ関数を
//!   `pub use` 再エクスポートし、`data-scope`/`data-part` セレクタへの
//!   既定 CSS を `stylesheet()` で追加提供する薄い委譲層。Menu / Select /
//!   Popover / Tooltip はラッパー第 1 弾（#551）・第 2 弾（#664、PR #672）
//!   で追加された 4 部品で、いずれも `positioner` が `position: absolute`
//!   のオーバーレイ型のため、既存 Dialog 節と同じ「SSR 初期状態は closed、
//!   全 anatomy を DOM に掲載（`hidden` 付き）」方針で掲示する）
//! - **pre-styled-ui の styled root（variant 付与）**: Avatar（headless の
//!   自由関数 `root` とは別に、`size`/`shape` variant クラスを付与する
//!   styled `root` を提供する。`image`/`fallback` は再エクスポート、#684）・
//!   Switch / RadioGroup（`size`/`palette` variant クラスを付与する styled
//!   `root`。`control`/`thumb`/`label`/`item` 等の子パーツは引き続き
//!   headless 層由来の自由関数、ラッパー第 3 弾 #682/#683 → PR #719 で
//!   `root` のみ variant 付与化）
//! - **pre-styled-ui の単純 styled 部品**: Button / Badge / Card / Alert /
//!   Spinner（variant/size/colorPalette を Rust enum で型安全に指定する）
//!
//! headless-ui 直接依存を撤去しても headless 層の anatomy・`data-*`・
//! WAI-ARIA 属性付与の実演は失われない。styled ラッパーは薄い委譲層で
//! マークアップ自体は headless 層の自由関数がそのまま生成するため、本
//! ページの各 `*_section` はこれまでどおり anatomy 実演を兼ねる（headless
//! 素材 + 手書き CSS の対比節をあえて設けない判断理由も同じ: 対比節を
//! 残すと手書き CSS が残存し「全部品 styled 化」の趣旨と矛盾するため）。
//!
//! CSS は [`StyleSheet`] へテーマトークン（[`Theme::default`]）・使用
//! コンポーネントの recipe CSS・ページ骨格のみの手書き CSS を集約し、
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
//!   WAI-ARIA 属性付与の実演（Tabs / Accordion / Dialog / Menu / Select /
//!   Popover / Tooltip / Switch / RadioGroup / Avatar）
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
use fandhe_frontend_pre_styled_ui::accordion;
use fandhe_frontend_pre_styled_ui::alert::{self, AlertStatus};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarShape, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardVariant};
use fandhe_frontend_pre_styled_ui::dialog::{self, ContentIds, DialogRole};
use fandhe_frontend_pre_styled_ui::menu;
use fandhe_frontend_pre_styled_ui::menubar;
use fandhe_frontend_pre_styled_ui::navigation_menu;
use fandhe_frontend_pre_styled_ui::popover;
use fandhe_frontend_pre_styled_ui::radio_group;
use fandhe_frontend_pre_styled_ui::select;
use fandhe_frontend_pre_styled_ui::spinner::{spinner, SpinnerProps};
use fandhe_frontend_pre_styled_ui::stylesheet::StyleSheet;
use fandhe_frontend_pre_styled_ui::switch;
use fandhe_frontend_pre_styled_ui::tabs::{self, ActivationMode, TabItem, TabsProps};
use fandhe_frontend_pre_styled_ui::theme::Theme;
use fandhe_frontend_pre_styled_ui::tooltip;
use fandhe_frontend_pre_styled_ui::{ColorPalette, OpenState, Orientation, Size};
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
        Size::Md,
        ColorPalette::Accent,
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
                        value,
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
        vec![accordion::root(Size::Md, vec![], root_children)],
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
                Size::Md,
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
                                // イシュー #1693/#1795: close-trigger は
                                // content 右上のゴーストボタン化に伴い
                                // アイコン専用契約になった。支援技術向け
                                // ラベルは aria-label で維持する
                                // （crates/pre-styled-ui/src/dialog.rs の
                                // rustdoc「close-trigger はアイコン専用
                                // 契約」参照）。
                                dialog::close_trigger(
                                    vec![("aria-label", "Close")],
                                    vec![text("×")],
                                ),
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

/// Menu コンポーネント節（`data-scope="menu"`）。SSR 初期状態は常に closed
/// （`OpenState::Closed`）。開閉・項目選択の実挙動は wasm 層の責務。
///
/// pre-styled-ui の headless ラッパー（`fandhe_frontend_pre_styled_ui::menu`、
/// ラッパー第 1 弾 #551）を使う。`positioner` が `position: absolute` の
/// オーバーレイ型のため、[`dialog_section`] と同じ「closed のまま全 anatomy
/// を DOM に掲載」方針で掲示する。1 件目の項目は `highlighted: true` を渡し、
/// virtual focus パターン（実 DOM フォーカスは trigger に留まり、選択候補は
/// `data-highlighted` で示す、イシュー #581/#643）を実演する。
fn menu_section() -> Node {
    let state = OpenState::Closed;
    let items = vec![
        menu::item_group(
            Some("showcase-menu-group-label"),
            vec![],
            vec![
                menu::item_group_label(
                    Some("showcase-menu-group-label"),
                    vec![],
                    vec![text("Actions")],
                ),
                menu::item("duplicate", false, true, vec![], vec![text("Duplicate")]),
                menu::item("rename", false, false, vec![], vec![text("Rename")]),
            ],
        ),
        menu::separator(vec![], vec![]),
        menu::item("delete", true, false, vec![], vec![text("Delete")]),
    ];
    let node = menu::root(
        Size::Md,
        state,
        vec![],
        vec![
            menu::trigger(
                state,
                false,
                Some("showcase-menu-content"),
                vec![("id", "showcase-menu-trigger")],
                vec![text("Open menu")],
            ),
            menu::indicator(state, vec![], vec![]),
            menu::positioner(
                state,
                vec![],
                vec![menu::content(
                    state,
                    Some("showcase-menu-content"),
                    Some("showcase-menu-trigger"),
                    vec![],
                    items,
                )],
            ),
        ],
    );
    section(
        "Menu",
        "WAI-ARIA APG の Menu パターン。SSR 初期状態は closed。既定 CSS は fandhe_frontend_pre_styled_ui::menu::stylesheet() が提供します。",
        vec![node],
    )
}

/// Select コンポーネント節（`data-scope="select"`）。listbox（`positioner`/
/// `content`）は closed のまま掲示しつつ、選択済み値（`value_text`/
/// `aria-selected`/`hidden_select` の `selected` option）を実演する。
///
/// pre-styled-ui の headless ラッパー（`fandhe_frontend_pre_styled_ui::select`、
/// ラッパー第 1 弾 #551）を使う。`name="deploy-target"` は
/// [`radio_group_section`] の `name="render-mode"` と衝突しない値を選ぶ。
fn select_section() -> Node {
    let state = OpenState::Closed;
    let options = [
        ("ssr", "SSR", false),
        ("ssg", "SSG", true),
        ("csr", "CSR", false),
    ];
    let mut items = Vec::new();
    for (value, label_text, selected) in options {
        let selected_state = if selected {
            OpenState::Open
        } else {
            OpenState::Closed
        };
        let item_id = format!("showcase-select-item-{value}");
        items.push(select::item(
            selected_state,
            false,
            false,
            value,
            Some(item_id.as_str()),
            vec![],
            vec![
                select::item_text(None, vec![], vec![text(label_text)]),
                select::item_indicator(selected_state, vec![], vec![text("✓")]),
            ],
        ));
    }
    let node = select::root(
        Size::Md,
        state,
        vec![],
        vec![
            select::label(
                Some("showcase-select-label"),
                vec![],
                vec![text("Deploy target")],
            ),
            select::control(
                state,
                vec![],
                vec![select::trigger(
                    state,
                    false,
                    Some("showcase-select-content"),
                    Some("showcase-select-label"),
                    vec![],
                    vec![
                        select::value_text(false, vec![], vec![text("SSG")]),
                        select::indicator(state, vec![], vec![]),
                    ],
                )],
            ),
            select::positioner(
                state,
                vec![],
                vec![select::content(
                    state,
                    Some("showcase-select-content"),
                    Some("showcase-select-label"),
                    None,
                    vec![],
                    items,
                )],
            ),
            select::hidden_select(
                Some("ssg"),
                Some("deploy-target"),
                false,
                vec![],
                vec![("ssr", "SSR"), ("ssg", "SSG"), ("csr", "CSR")],
            ),
        ],
    );
    section(
        "Select",
        "WAI-ARIA APG の Listbox パターンに基づく Select。SSG が選択済みの状態を実演します。既定 CSS は fandhe_frontend_pre_styled_ui::select::stylesheet() が提供します。",
        vec![node],
    )
}

/// Popover コンポーネント節（`data-scope="popover"`）。SSR 初期状態は常に
/// closed。開閉の実挙動は wasm 層の責務。
///
/// pre-styled-ui の headless ラッパー（`fandhe_frontend_pre_styled_ui::popover`、
/// ラッパー第 2 弾 #664）を使う。[`dialog_section`] と異なり `close_trigger`
/// にはテキスト内容がないため、アクセシブルネームを `aria-label`（`attrs`
/// 経由）で明示する（headless 層の契約、`crates/headless-ui/src/popover.rs`
/// 参照）。
fn popover_section() -> Node {
    let state = OpenState::Closed;
    let node = popover::root(
        state,
        vec![],
        vec![
            popover::trigger(
                state,
                false,
                Some("showcase-popover-content"),
                vec![],
                vec![text("Open popover")],
            ),
            popover::positioner(
                state,
                vec![],
                vec![
                    popover::arrow(vec![], vec![popover::arrow_tip(vec![], vec![])]),
                    popover::content(
                        state,
                        Some("showcase-popover-content"),
                        Some("showcase-popover-title"),
                        Some("showcase-popover-description"),
                        vec![],
                        vec![
                            popover::title(
                                Some("showcase-popover-title"),
                                vec![],
                                vec![text("Share this page")],
                            ),
                            popover::description(
                                Some("showcase-popover-description"),
                                vec![],
                                vec![text("リンクをコピーして共有できます。")],
                            ),
                            popover::close_trigger(
                                vec![("aria-label", "Close popover")],
                                vec![text("×")],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    );
    section(
        "Popover",
        "アンカー配置のオーバーレイ。SSR 初期状態は closed。既定 CSS は fandhe_frontend_pre_styled_ui::popover::stylesheet() が提供します。",
        vec![node],
    )
}

/// Tooltip コンポーネント節（`data-scope="tooltip"`）。SSR 初期状態は常に
/// closed。開閉の実挙動は wasm 層の責務。
///
/// pre-styled-ui の headless ラッパー（`fandhe_frontend_pre_styled_ui::tooltip`、
/// ラッパー第 2 弾 #664）を使う。WAI-ARIA tooltip パターンに従い
/// `aria-expanded`/`aria-controls` は使わず `aria-describedby` のみで
/// trigger と content を関連付ける（`crates/headless-ui/src/tooltip.rs`
/// 参照）。
fn tooltip_section() -> Node {
    let state = OpenState::Closed;
    let node = tooltip::root(
        state,
        vec![],
        vec![
            tooltip::trigger(
                state,
                false,
                Some("showcase-tooltip-content"),
                vec![],
                vec![text("Hover me")],
            ),
            tooltip::positioner(
                state,
                vec![],
                vec![
                    tooltip::content(
                        state,
                        Some("showcase-tooltip-content"),
                        vec![],
                        vec![text("既定 CSS 変数によるダーク/ライト両対応です。")],
                    ),
                    tooltip::arrow(vec![], vec![tooltip::arrow_tip(vec![], vec![])]),
                ],
            ),
        ],
    );
    section(
        "Tooltip",
        "WAI-ARIA tooltip パターン（aria-describedby のみで関連付け）。SSR 初期状態は closed。既定 CSS は fandhe_frontend_pre_styled_ui::tooltip::stylesheet() が提供します。",
        vec![node],
    )
}

/// Navigation Menu コンポーネント節（`data-scope="navigation-menu"`）。SSR
/// 初期状態は常に closed（`OpenState::Closed`）。トリガー起点で開閉する
/// ディスクロージャの実挙動は wasm 層の責務。
///
/// pre-styled-ui の headless ラッパー（`fandhe_frontend_pre_styled_ui::navigation_menu`）
/// を使う。`root` の暗黙 ARIA role（`navigation`）に依拠し
/// `role="navigation"` を明示付与しない点、`role="menu"`/`role="menuitem"`
/// を一切付与しない点（文書ナビを操作メニューと誤伝達しないための判断）は
/// `crates/headless-ui/src/navigation_menu.rs` の rustdoc 参照。[`menu_section`]
/// と同じ「closed のまま全 anatomy を DOM に掲載」方針で、トリガー付き項目
/// 1 件と現在ページを示すリンク単独項目 1 件を実演する。
fn navigation_menu_section() -> Node {
    let state = OpenState::Closed;
    let node = navigation_menu::root(
        "Product navigation",
        vec![],
        vec![navigation_menu::list(
            vec![],
            vec![
                navigation_menu::item(
                    state,
                    false,
                    vec![],
                    vec![
                        navigation_menu::trigger(
                            state,
                            false,
                            "docs",
                            Some("showcase-nav-menu-docs-trigger"),
                            Some("showcase-nav-menu-docs-content"),
                            vec![],
                            vec![text("Docs")],
                        ),
                        navigation_menu::content(
                            state,
                            Some("showcase-nav-menu-docs-content"),
                            Some("showcase-nav-menu-docs-trigger"),
                            vec![],
                            vec![navigation_menu::link(
                                "/docs/getting-started",
                                false,
                                vec![],
                                vec![text("Getting Started")],
                            )],
                        ),
                    ],
                ),
                navigation_menu::item(
                    state,
                    false,
                    vec![],
                    vec![navigation_menu::link(
                        "/pricing",
                        true,
                        vec![],
                        vec![text("Pricing")],
                    )],
                ),
            ],
        )],
    );
    section(
        "Navigation Menu",
        "トリガー起点で開閉するナビゲーションパネル。SSR 初期状態は closed。既定 CSS は fandhe_frontend_pre_styled_ui::navigation_menu::stylesheet() が提供します。",
        vec![node],
    )
}

/// Menubar コンポーネント節（`data-scope="menubar"`）。SSR 初期状態は常に
/// closed。roving tabindex（1 件目のトリガーのみ `tabindex="0"`）・複数
/// Menu の水平配置の実演。実際のフォーカス移動・開閉は wasm 層の責務
/// （`crates/headless-ui/src/menubar.rs` rustdoc 参照）。
///
/// pre-styled-ui の headless ラッパー（`fandhe_frontend_pre_styled_ui::menubar`）
/// を使う。[`menu_section`] と異なり `menu` パーツ自身は `role="none"` を
/// 固定付与し、実際の `role="menubar"`/`role="menuitem"` は `root`/`trigger`
/// が担う（同モジュール rustdoc「`role="none"` の根拠と制約」参照）。
fn menubar_section() -> Node {
    let closed = OpenState::Closed;
    let node = menubar::root(
        Orientation::Horizontal,
        "Application menu",
        vec![],
        vec![
            menubar::menu(
                closed,
                vec![],
                vec![
                    menubar::trigger(
                        true,
                        closed,
                        false,
                        false,
                        0,
                        Some("showcase-menubar-file-content"),
                        vec![("id", "showcase-menubar-file-trigger")],
                        vec![text("File")],
                    ),
                    menubar::positioner(
                        closed,
                        vec![],
                        vec![menubar::content(
                            closed,
                            Some("showcase-menubar-file-content"),
                            Some("showcase-menubar-file-trigger"),
                            vec![],
                            vec![
                                menubar::item("new", false, true, vec![], vec![text("New")]),
                                menubar::item("open", false, false, vec![], vec![text("Open")]),
                                menubar::separator(vec![], vec![]),
                                menubar::item("exit", false, false, vec![], vec![text("Exit")]),
                            ],
                        )],
                    ),
                ],
            ),
            menubar::menu(
                closed,
                vec![],
                vec![
                    menubar::trigger(
                        false,
                        closed,
                        false,
                        false,
                        1,
                        Some("showcase-menubar-edit-content"),
                        vec![("id", "showcase-menubar-edit-trigger")],
                        vec![text("Edit")],
                    ),
                    menubar::positioner(
                        closed,
                        vec![],
                        vec![menubar::content(
                            closed,
                            Some("showcase-menubar-edit-content"),
                            Some("showcase-menubar-edit-trigger"),
                            vec![],
                            vec![
                                menubar::item("undo", false, false, vec![], vec![text("Undo")]),
                                menubar::item("redo", true, false, vec![], vec![text("Redo")]),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    );
    section(
        "Menubar",
        "複数 Menu の水平配置と roving tabindex。SSR 初期状態は closed。既定 CSS は fandhe_frontend_pre_styled_ui::menubar::stylesheet() が提供します。",
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
                "この操作は取り消せません（デモ用の警告文です）。",
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
/// pre-styled-ui の styled `root`（`fandhe_frontend_pre_styled_ui::switch::root`、
/// PR #719 で size/palette variant 引数を追加。イシュー #728 で 0.5.0 へ追随）
/// を使う。headless の自由関数（`checked`/`disabled`/`attrs`/`children` のみ）
/// とは異なり `size`/`palette` variant 引数を取り、recipe 生成クラス
/// （`fd-switch--size-*`/`fd-switch--color-palette-*`）を付与する。既定 CSS は
/// `switch::stylesheet()` が提供する（モジュール doc の層別内訳参照）。
fn switch_section() -> Node {
    let checked = true;
    let node = switch::root(
        Size::Md,
        ColorPalette::Accent,
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
        "WAI-ARIA APG の Switch パターン。既定 CSS は fandhe_frontend_pre_styled_ui::switch::stylesheet() が提供します。",
        vec![node],
    )
}

/// RadioGroup コンポーネント節（`data-scope="radio-group"`）。
///
/// pre-styled-ui の styled `root`（`fandhe_frontend_pre_styled_ui::radio_group::root`、
/// PR #719 で size/palette variant 引数を追加。イシュー #728 で 0.5.0 へ追随）
/// を使う。`size`/`palette` variant 引数により recipe 生成クラス
/// （`fd-radio-group--size-*`/`fd-radio-group--color-palette-*`）を付与する。
fn radio_group_section() -> Node {
    let options = [
        ("ssr", "SSR", true),
        ("ssg", "SSG", false),
        ("csr", "CSR", false),
    ];
    let radio_group_props = radio_group::RadioGroupProps::default();
    let mut items = Vec::new();
    for (value, label_text, checked) in options {
        items.push(radio_group::item(
            checked,
            &radio_group_props,
            value,
            vec![],
            vec![
                radio_group::item_hidden_input(
                    checked,
                    &radio_group_props,
                    Some("render-mode"),
                    value,
                    vec![],
                ),
                radio_group::item_control(checked, &radio_group_props, vec![]),
                radio_group::item_text(checked, &radio_group_props, vec![], vec![text(label_text)]),
            ],
        ));
    }
    let node = radio_group::root(
        Size::Md,
        ColorPalette::Accent,
        false,
        Some(Orientation::Vertical),
        Some("render-mode-label"),
        vec![],
        std::iter::once(radio_group::label(
            &radio_group_props,
            Some("render-mode-label"),
            vec![],
            vec![text("Render mode")],
        ))
        .chain(items)
        .collect(),
    );
    section(
        "RadioGroup",
        "択一選択の RadioGroup。既定 CSS は fandhe_frontend_pre_styled_ui::radio_group::stylesheet() が提供します。",
        vec![node],
    )
}

/// Avatar コンポーネント節（`data-scope="avatar"`）。画像読み込み失敗
/// （[`ImageStatus::Error`]）状態を実演し、フォールバック（イニシャル）が
/// 表示されることを示す（実画像を同梱しない静的サンプルのため）。
///
/// pre-styled-ui の styled `root`（`fandhe_frontend_pre_styled_ui::avatar::root`、
/// #684）を使う。headless の自由関数 `root(attrs, children)` とは異なり
/// `size`/`shape` variant 引数を取り、recipe 生成クラス（`fd-avatar--size-*`/
/// `fd-avatar--shape-*`）を付与する。
fn avatar_section() -> Node {
    let status = ImageStatus::Error;
    let node = avatar::root(
        Size::Md,
        AvatarShape::Circle,
        vec![],
        vec![
            avatar::image(status, "/nonexistent.png", "User avatar", vec![]),
            avatar::fallback(status, vec![], vec![text("FT")]),
        ],
    );
    section(
        "Avatar",
        "画像読み込み状態（loading/loaded/error）に応じて表示を切り替える Avatar。既定 CSS は fandhe_frontend_pre_styled_ui::avatar::stylesheet() が提供します。",
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
        Size::Md,
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
                        "xss-probe",
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
            menu_section(),
            select_section(),
            popover_section(),
            tooltip_section(),
            navigation_menu_section(),
            menubar_section(),
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
/// 1. テーマトークン（[`Theme::default`] を [`Theme::upsert_color`] /
///    [`Theme::upsert_space`] で上書き・拡張したもの。ライト/ダーク両対応の
///    `--fandhe-color-*` 等）
/// 2. ページ骨格のみの手書き CSS（`static/ui.css`、`include_str!` で
///    バイナリへ埋め込み。コンポーネント CSS は v0.4.0 で全部品 recipe
///    提供となったため撤去済み、#689）
/// 3. 本ページで使用する pre-styled-ui コンポーネントの recipe CSS
///
/// 手書き CSS は [`StyleSheet::push_css`] の fail-closed 検証（`<` 拒否）を
/// 経由させるため、検証エラーを `Err` として呼び出し元（[`main`]）へ返す
/// （pre-styled-ui 生成分は検証済み契約のため infallible な
/// `push_recipe`/`push_theme` 相当の扱いで `push_css` が常に `Ok` になる。
/// `crates/pre-styled-ui/src/stylesheet.rs` の
/// `push_recipe_is_infallible_for_all_styled_components` 参照）。
fn build_stylesheet() -> Result<StyleSheet, fandhe_frontend_pre_styled_ui::StylesheetError> {
    let mut theme = Theme::default();
    // `Theme::push_color` は同名トークンを `ThemeError::DuplicateTokenName`
    // で fail-closed 拒否するため、既定パレット（`accent`）の上書きには
    // `upsert_color`（イシュー #1118/#1138）が正規経路（イシュー #1175 実演）。
    // 挿入順＝出力順は upsert でも保たれる（`Theme::to_css` rustdoc 参照）。
    // 値は静的リテラルのみを渡す（呼び出し元入力を経由しない）。
    theme
        .upsert_color("accent", "#0f766e", "#2dd4bf")
        .expect("\"accent\"/\"#0f766e\"/\"#2dd4bf\" are statically valid theme tokens");
    // `upsert_space` は不在トークンに対しては `push_space` と同じ挿入動作
    // （末尾追加）になる。本サンプルの手書き CSS（`static/ui.css`）が
    // `var(--fandhe-space-showcase-gap)` を参照し、追加トークンが実際に
    // 使われることを示す（死にトークン化を防ぐ）。
    theme
        .upsert_space("showcase-gap", "1.25rem")
        .expect("\"showcase-gap\"/\"1.25rem\" are statically valid theme tokens");

    let mut sheet = StyleSheet::new();
    sheet.push_theme(&theme);
    sheet.push_css(include_str!("../static/ui.css"))?;
    for css in [
        fandhe_frontend_pre_styled_ui::tabs::stylesheet(),
        fandhe_frontend_pre_styled_ui::accordion::stylesheet(),
        fandhe_frontend_pre_styled_ui::dialog::stylesheet(),
        fandhe_frontend_pre_styled_ui::menu::stylesheet(),
        fandhe_frontend_pre_styled_ui::select::stylesheet(),
        fandhe_frontend_pre_styled_ui::popover::stylesheet(),
        fandhe_frontend_pre_styled_ui::tooltip::stylesheet(),
        fandhe_frontend_pre_styled_ui::navigation_menu::stylesheet(),
        fandhe_frontend_pre_styled_ui::menubar::stylesheet(),
        fandhe_frontend_pre_styled_ui::switch::stylesheet(),
        fandhe_frontend_pre_styled_ui::radio_group::stylesheet(),
        fandhe_frontend_pre_styled_ui::avatar::stylesheet(),
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
            "data-scope=\"menu\"",
            "data-scope=\"select\"",
            "data-scope=\"popover\"",
            "data-scope=\"tooltip\"",
            "data-scope=\"navigation-menu\"",
            "data-scope=\"menubar\"",
            // pre-styled-ui の単純 styled 部品
            "data-scope=\"button\"",
            "data-scope=\"badge\"",
            "data-scope=\"card\"",
            "data-scope=\"alert\"",
            "data-scope=\"spinner\"",
            // pre-styled-ui の styled root（variant 付与、#684・PR #719）
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

    /// anatomy・ARIA・`data-highlighted` の検証（closed 状態のまま virtual
    /// focus の highlight 表示を実演することを固定する）。
    #[test]
    fn menu_section_renders_closed_state_with_menu_roles() {
        let html = render(&menu_section());
        assert!(html.contains(r#"role="menu""#));
        assert!(html.contains(r#"role="menuitem""#));
        assert!(html.contains(r#"aria-haspopup="menu""#));
        assert!(html.contains(r#"data-state="closed""#));
        assert!(html.contains("data-highlighted"));
        assert!(html.contains(r#"role="separator""#));
        assert!(html.contains("hidden"));
    }

    /// anatomy・ARIA・選択済み値の検証（listbox は closed のまま、SSG が
    /// 選択済みであることを `aria-selected`/`hidden_select` で実演する）。
    #[test]
    fn select_section_renders_selected_value_and_listbox() {
        let html = render(&select_section());
        assert!(html.contains(r#"role="listbox""#));
        assert!(html.contains(r#"role="option""#));
        assert!(html.contains(r#"aria-selected="true""#));
        assert!(html.contains(r#"aria-haspopup="listbox""#));
        assert!(html.contains(r#"data-value="ssg""#));
        assert!(html.contains(r#"value="ssg" selected="""#));
    }

    /// anatomy・ARIA の検証（Dialog 節と同じ closed 掲示方針を固定する）。
    #[test]
    fn popover_section_renders_closed_state_with_dialog_role() {
        let html = render(&popover_section());
        assert!(html.contains(r#"role="dialog""#));
        assert!(html.contains(r#"aria-haspopup="dialog""#));
        assert!(html.contains(r#"aria-expanded="false""#));
        assert!(html.contains("hidden"));
    }

    /// anatomy・WAI-ARIA tooltip パターン（`aria-describedby` のみで
    /// 関連付け、`aria-expanded`/`aria-controls` は使わない）の検証。
    #[test]
    fn tooltip_section_renders_closed_state_with_tooltip_role() {
        let html = render(&tooltip_section());
        assert!(html.contains(r#"role="tooltip""#));
        assert!(html.contains(r#"aria-describedby="showcase-tooltip-content""#));
        assert!(html.contains("hidden"));
    }

    /// anatomy・ARIA の検証（`role="menu"`/`role="menuitem"` を一切付与
    /// しないこと・`nav` の暗黙 role に依拠して `role="navigation"` を
    /// 明示付与しないことを固定する。`aria-current="page"` は現在ページを
    /// 示す 2 件目のリンクのみに付与される）。
    #[test]
    fn navigation_menu_section_renders_closed_state_without_menu_roles() {
        let html = render(&navigation_menu_section());
        assert!(html.contains("<nav"));
        assert!(html.contains(r#"aria-label="Product navigation""#));
        assert!(html.contains(r#"data-state="closed""#));
        assert!(html.contains(r#"aria-expanded="false""#));
        assert!(html.contains(r#"aria-current="page""#));
        assert!(html.contains("hidden"));
        assert!(!html.contains(r#"role="menu""#));
        assert!(!html.contains(r#"role="menuitem""#));
        assert!(!html.contains(r#"role="navigation""#));
    }

    /// anatomy・ARIA の検証（`role="menubar"`/`role="menuitem"`・roving
    /// tabindex（1 件目のみ `tabindex="0"`、2 件目は `tabindex="-1"`）・
    /// `menu` パーツの `role="none"` を固定する）。
    #[test]
    fn menubar_section_renders_closed_state_with_menubar_roles() {
        let html = render(&menubar_section());
        assert!(html.contains(r#"role="menubar""#));
        assert!(html.contains(r#"role="menuitem""#));
        assert!(html.contains(r#"role="none""#));
        assert!(html.contains(r#"aria-haspopup="menu""#));
        assert!(html.contains(r#"tabindex="0""#));
        assert!(html.contains(r#"tabindex="-1""#));
        assert!(html.contains(r#"data-state="closed""#));
        assert!(html.contains(r#"role="separator""#));
        assert!(html.contains("hidden"));
    }

    #[test]
    fn switch_section_renders_checked_state() {
        let html = render(&switch_section());
        assert!(html.contains(r#"data-state="checked""#));
        assert!(html.contains(r#"role="switch""#));
    }

    /// styled Switch の `root` が size/palette variant クラス
    /// （`fd-switch--size-*`/`fd-switch--color-palette-*`）を出力することを
    /// 固定する（pre-styled-ui 0.5.0・PR #719 で `root` が
    /// `size`/`palette` variant 引数を取るようになった破壊的変更、
    /// イシュー #728 で 0.5.0 へ追随）。
    #[test]
    fn switch_section_renders_size_and_palette_variant_classes() {
        let html = render(&switch_section());
        assert!(html.contains("fd-switch--size-md"));
        assert!(html.contains("fd-switch--color-palette-accent"));
    }

    /// styled RadioGroup の `root` が size/palette variant クラス
    /// （`fd-radio-group--size-*`/`fd-radio-group--color-palette-*`）を
    /// 出力することを固定する（switch と同じ PR #719 破壊的変更・#728 追随）。
    #[test]
    fn radio_group_section_renders_size_and_palette_variant_classes() {
        let html = render(&radio_group_section());
        assert!(html.contains("fd-radio-group--size-md"));
        assert!(html.contains("fd-radio-group--color-palette-accent"));
    }

    /// styled Avatar の `root` が size/shape variant クラス（`fd-avatar--size-*`/
    /// `fd-avatar--shape-*`）を出力することを固定する（イシュー #689、headless
    /// 自由関数 `root` にはない styled 層固有の振る舞い）。
    #[test]
    fn avatar_section_renders_size_and_shape_variant_classes() {
        let html = render(&avatar_section());
        assert!(html.contains("fd-avatar--size-md"));
        assert!(html.contains("fd-avatar--shape-circle"));
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

    /// [`build_stylesheet`] がテーマトークン・pre-styled recipe・ページ骨格
    /// のみの手書き CSS の 3 系統すべてを集約し、`<` を含まない（`<style>`
    /// 文脈でも安全な）CSS を返すことを固定する。
    #[test]
    fn build_stylesheet_aggregates_theme_recipes_and_manual_css() {
        let sheet = build_stylesheet().expect("all CSS sources should pass validation");
        let css = sheet.as_css();
        // 1. テーマトークン（Theme::default）
        assert!(css.contains("--fandhe-color-"));
        // 2. ページ骨格のみの手書き CSS（コンポーネント CSS は v0.4.0 で
        //    全部品 recipe 提供となったため撤去済み、#689）
        assert!(css.contains(".showcase-row"));
        // 3. pre-styled recipe（ラッパー分 + styled root 分 + 単純 styled 部品分）
        assert!(css.contains(r#"[data-scope="tabs"][data-part="trigger"]"#));
        assert!(css.contains(r#"[data-scope="menu"][data-part="content"]"#));
        assert!(css.contains(r#"[data-scope="select"][data-part="trigger"]"#));
        assert!(css.contains(r#"[data-scope="popover"][data-part="content"]"#));
        assert!(css.contains(r#"[data-scope="tooltip"][data-part="content"]"#));
        assert!(css.contains(r#"[data-scope="navigation-menu"][data-part="trigger"]"#));
        assert!(css.contains(r#"[data-scope="menubar"][data-part="trigger"]"#));
        assert!(css.contains(r#"[data-scope="switch"][data-part="control"]"#));
        assert!(css.contains(r#"[data-scope="radio-group"][data-part="item-control"]"#));
        assert!(css.contains(r#"[data-scope="avatar"][data-part="root"]"#));
        assert!(css.contains(".fd-button--variant-solid"));
        assert!(!css.contains('<'));
    }

    /// `Theme::upsert_color`（イシュー #1138/#1175）の実演回帰: 既定パレット
    /// の `accent`（`#3182ce`/`#4299e1`、`crates/pre-styled-ui/src/theme.rs`
    /// `DEFAULT_COLORS`）が [`build_stylesheet`] の上書き後の値
    /// （`#0f766e`/`#2dd4bf`）へ置き換わり、既定値は出力に残らないことを
    /// 固定する。
    #[test]
    fn build_stylesheet_overrides_default_accent_color_via_upsert() {
        let sheet = build_stylesheet().expect("all CSS sources should pass validation");
        let css = sheet.as_css();
        assert!(css.contains("--fandhe-color-accent: #0f766e;"));
        assert!(css.contains("#2dd4bf"));
        // `#3182ce` は既定 `accent` のライト値だが、`info` トークン
        // （`DEFAULT_COLORS`）も同じライト値 `#3182ce` を持つため、CSS 全体
        // からの単純な文字列不在ではなく `--fandhe-color-accent:` 宣言行
        // そのものが上書き後の値を指すことのみを断定する（`info` 側は
        // upsert 対象外のため変更されないのが正しい挙動）。`#4299e1`
        // （既定 `accent` のダーク値）は `DEFAULT_COLORS` 中で他トークンと
        // 衝突しない一意な値のため、単純な文字列不在で断定できる。
        assert!(
            !css.contains("--fandhe-color-accent: #3182ce;"),
            "default accent light value should be overridden"
        );
        assert!(
            !css.contains("#4299e1"),
            "default accent dark value should be overridden"
        );
    }

    /// `Theme::upsert_space`（イシュー #1138/#1175）の実演回帰: 既定テーマに
    /// 存在しない `showcase-gap` トークンが末尾追加され、`static/ui.css` の
    /// `.showcase-row` から `var(--fandhe-space-showcase-gap)` として実際に
    /// 参照されている（死にトークンでない）ことを固定する。
    #[test]
    fn build_stylesheet_adds_showcase_gap_space_token_via_upsert() {
        let sheet = build_stylesheet().expect("all CSS sources should pass validation");
        let css = sheet.as_css();
        assert!(css.contains("--fandhe-space-showcase-gap: 1.25rem;"));
        assert!(css.contains("gap: var(--fandhe-space-showcase-gap);"));
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
