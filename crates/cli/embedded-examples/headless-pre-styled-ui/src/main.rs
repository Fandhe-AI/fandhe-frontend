//! `fandhe-frontend-example-headless-pre-styled-ui`: headless-ui /
//! pre-styled-ui コンポーネントショーケースの正本サンプル（イシュー #552、
//! examples 規約 #499 準拠。親トラッキング #520 の Phase 4）。
//!
//! # 役割・呼び出し文脈
//!
//! `fandhe-frontend-headless-ui`（ark-ui 相当の headless UI 層、#520/#522）が
//! 提供する代表的なコンポーネント（Tabs / Accordion / Dialog / Switch /
//! RadioGroup / Avatar）を、`fandhe_frontend_core::Node` を返す通常の Rust
//! 関数として組み立て、静的 HTML 1 ページへレンダリングする。
//!
//! **スコープ調整（イシュー #552 §3 シナリオ B）**: 本サンプル作成時点
//! （2026-07-22）で `fandhe-frontend-pre-styled-ui`（chakra-ui 相当の上層、
//! #520/#546）はクレート骨格のみで公開 API を持たない（テーマトークン #547・
//! variant API #548・styled 部品 #550/#551 が並列進行中で未マージ）。本サンプル
//! は headless-ui のみを依存に持ち、pre-styled-ui は
//! `static/ui.css`（headless-ui が出力する `data-scope`/`data-part`/
//! `data-state` セレクタへ手書きで当てる最小 CSS）で代替する。pre-styled-ui
//! の公開 API が揃い次第、本サンプルへの統合をフォローアップする
//! （README.md「pre-styled-ui 統合について」節・PR 本文参照）。
//!
//! headless-ui の各コンポーネントは SSR 静的マークアップ（クリック等の
//! 実挙動・dispatch 状態遷移は wasm 層の責務、各モジュールの rustdoc
//! 参照）を組み立てる自由関数のみを使用する。`fandhe_frontend_app::page_shell`
//! （`String` を返す）は使わず、`examples/ssg-blog` と同様に `Node` を返す
//! 自作の [`layout`] でページ骨格を組み立てる。
//!
//! # 学べること
//!
//! - `fandhe-frontend-headless-ui` の anatomy（`data-scope`/`data-part`）・
//!   `data-*` 状態属性・WAI-ARIA 属性付与の実演（Tabs / Accordion / Dialog /
//!   Switch / RadioGroup / Avatar）
//! - 既定エスケープ（REQ-1）: 動的に見える値も含めすべて `text()` 経由でノード
//!   木へ載せ、`raw_html()` や `format!` によるタグ文字列の直接組み立ては
//!   使わない
//! - `@view-transition { navigation: auto; }`（`fandhe_frontend_app::page_shell`
//!   と同一の固定リテラル）による Cross-Document View Transitions の有効化
//!
//! # セキュリティ不変条件（REQ-1・OWASP A01）
//!
//! - HTML はすべて `fandhe_frontend_core`/`fandhe_frontend_headless_ui` の
//!   ノード木 API で組み立てる。`format!` は属性値のプレーン文字列整形
//!   （id の組み立て等）にのみ使い、タグ文字列の直接組み立てには使わない。
//! - 出力先パスは `dist/index.html`・`dist/assets/ui.css` の固定リテラルのみ
//!   （外部入力由来のパスを使わない）。

#![forbid(unsafe_code)]

use fandhe_frontend_core::{el, render, text, Node};
use fandhe_frontend_headless_ui::accordion;
use fandhe_frontend_headless_ui::avatar::{self, ImageStatus};
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::dialog::{self, ContentIds, DialogRole};
use fandhe_frontend_headless_ui::radio_group;
use fandhe_frontend_headless_ui::switch;
use fandhe_frontend_headless_ui::{tabs, OpenState, TabItem, TabsProps};
use std::path::Path;

/// ページ共通の骨格（`<html>` 全体）を組み立てる。
///
/// `examples/ssg-blog::layout` と同じ方針: `fandhe_frontend_app::page_shell`
/// は `String` を返すため `Node` 木のみを扱う本サンプルには使えず、自作の
/// `Node` 版として存在する。`static/ui.css`（本関数の呼び出し元が
/// `dist/assets/ui.css` へコピー済み）を `<link>` で参照する。
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

/// Tabs コンポーネント節（`data-scope="tabs"`）。
fn tabs_section() -> Node {
    let node = tabs::tabs(
        &TabsProps {
            id: "showcase-tabs",
            selected: "profile",
            orientation: Orientation::Horizontal,
            activation_mode: fandhe_frontend_headless_ui::ActivationMode::Automatic,
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
        "WAI-ARIA APG の Tabs パターンに準拠したマークアップ（fandhe_frontend_headless_ui::tabs）。",
        vec![node],
    )
}

/// Accordion コンポーネント節（`data-scope="accordion"`、single モード想定）。
///
/// 状態機械（`Accordion`/`SingleSelect`、dispatch 連携）は wasm 層の責務
/// （モジュール doc 参照）のため、本サンプルは自由関数のみを直接呼び、
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
            "Does it depend on pre-styled-ui?",
            "いいえ。headless-ui は外部依存が fandhe-frontend-core / -interactive のみで、スタイルは呼び出し側（pre-styled-ui 等）の責務です。",
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
        "高々 1 項目が開く single モードの Accordion（fandhe_frontend_headless_ui::accordion）。",
        vec![accordion::root(vec![], root_children)],
    )
}

/// Dialog コンポーネント節（`data-scope="dialog"`）。SSR 初期状態は常に
/// closed（`OpenState::Closed`）。開閉の実挙動は wasm 層の責務。
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
        "モーダルダイアログ（fandhe_frontend_headless_ui::dialog）。SSR 初期状態は closed。",
        vec![node],
    )
}

/// Switch コンポーネント節（`data-scope="switch"`）。
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
        "WAI-ARIA APG の Switch パターン（fandhe_frontend_headless_ui::switch）。",
        vec![node],
    )
}

/// RadioGroup コンポーネント節（`data-scope="radio-group"`）。
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
        "択一選択の RadioGroup（fandhe_frontend_headless_ui::radio_group）。",
        vec![node],
    )
}

/// Avatar コンポーネント節（`data-scope="avatar"`）。画像読み込み失敗
/// （[`ImageStatus::Error`]）状態を実演し、フォールバック（イニシャル）が
/// 表示されることを示す（実画像を同梱しない静的サンプルのため）。
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
        "画像読み込み状態（loading/loaded/error）に応じて表示を切り替える Avatar（fandhe_frontend_headless_ui::avatar）。",
        vec![node],
    )
}

/// 既定エスケープ（REQ-1）の実演節。`<script>` を含む固定文字列を
/// [`accordion::item_trigger`] の子ノードへ渡し、出力が実体参照化される
/// ことを示す（`tests/xss_regression.rs` が同じ入力で回帰を固定する）。
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
                    "fandhe-frontend-headless-ui の代表的なコンポーネントを静的 SSR マークアップとして実演します。",
                )],
            ),
            tabs_section(),
            accordion_section(),
            dialog_section(),
            switch_section(),
            radio_group_section(),
            avatar_section(),
            xss_probe_section(),
        ],
    );
    layout("headless-ui / pre-styled-ui showcase", main)
}

/// `dist/` へ書き出す。出力先は固定リテラルのみ（外部入力由来のパスは
/// 扱わない）。`static/ui.css` は `include_str!` でバイナリへ埋め込み、
/// 実行時ファイルシステム探索を行わない（配布バイナリ単体でも動作する）。
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
    let css = include_str!("../static/ui.css");
    if let Err(err) = std::fs::write(assets.join("ui.css"), css) {
        eprintln!("failed to write dist/assets/ui.css: {err}");
        std::process::exit(1);
    }

    println!("dist/index.html");
    println!("dist/assets/ui.css");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ページ生成関数の純関数スモーク: panic せず組み立てられ、各コンポーネ
    /// ントの anatomy セレクタ（`data-scope`）が出力に含まれることを固定する。
    #[test]
    fn build_page_includes_all_component_scopes() {
        let html = render(&build_page());
        for scope in [
            "data-scope=\"tabs\"",
            "data-scope=\"accordion\"",
            "data-scope=\"dialog\"",
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
