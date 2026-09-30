//! `settings-api-key-created` block（イシュー #2982。Application / Settings
//! カテゴリ、最初の block）。API キーを発行した直後に「一度しか表示しない」
//! 注意とともにキー値・コピー操作を提示するカードを合成する。主参照 R0241
//! （代表構成、単一キー）を軸に、R0242（複数キー行表示）を集約する。
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`profile_detail_datalist`〔イシュー #2937〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `card` / `callout` / `clipboard` / `input-group` / `button` の 5 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 2 版と集約元の対応（原稿「原案差分メモ」節と対になる索引）
//!
//! - **A（代表構成）**: R0241。単一キーを 1 個の
//!   [`fandhe_frontend_pre_styled_ui::clipboard`] で提示する。
//! - **B（複数キー行表示）**: R0242。3 個のキーをそれぞれ独立した
//!   `clipboard` root で行表示する（下記「行ごとに独立した clipboard root
//!   にする理由」節参照）。`clipboard::input` を
//!   [`fandhe_frontend_pre_styled_ui::input_group`] の `root`/`addon` で
//!   包み、trigger を addon 側へ配置する。
//!
//! # 行ごとに独立した `clipboard` root にする理由（1 root : 1 状態機械契約）
//!
//! `fandhe-frontend-wasm-full` の `headless_clipboard` 配線は「1 root : 1
//! 状態機械契約」という簡略化を持ち、同一マウントルート配下の**全**
//! `clipboard` パーツへ `data-copied` の反映を及ぼす
//! （`hero_install_command` モジュール doc「コピー配線の範囲」節と同型の
//! 制約）。版 B の 3 行を 1 個の `clipboard` root にまとめると、1 行を
//! コピーしただけで他の 2 行の表示も連動して変わってしまうため、行ごとに
//! 独立した `id` 付き `clipboard::root`（`blocks-settings-api-key-created-b-
//! <n>`）に分離した。`headless_clipboard` 自体の「1 root : 1 状態機械契約」
//! を変更する提案は本 block のスコープ外である
//! （`.claude/rules/coding-rust.md` の意図的非採用機能の再評価基準と同様、
//! cross-cutting な変更は個別 Issue で評価する）。
//!
//! # `input_group` を使う理由（版 B のみ、版 A は `clipboard::control` 直接）
//!
//! [`fandhe_frontend_pre_styled_ui::input_group::root`] は `role="group"`
//! の配置コンテナで内包する `<input>` の種類を限定しない（headless
//! rustdoc）ため、[`fandhe_frontend_pre_styled_ui::clipboard::input`]
//! （`flex: 1 1 0%` として振る舞う）を子に置いても構造上問題ない。trigger
//! は [`fandhe_frontend_pre_styled_ui::input_group::button`] ではなく
//! `clipboard::trigger` を `input_group::addon` の子として配置する（trigger
//! を `clipboard` scope 内に保つことで、実アプリでは `headless_clipboard`
//! 配線が有効に働く。`hero_install_command` の版 B〔addon ボタンを
//! `clipboard` scope の外側へ置き `disabled: true` にする〕とは逆の判断で
//! あり、本 block では複数行の視覚整列に `input_group` の flex レイアウトを
//! 借りつつコピー機能自体は保つ）。
//!
//! # コピー配線の範囲（実アプリに組み込めば A/B とも機能する）
//!
//! A・B ともキー値は [`fandhe_frontend_pre_styled_ui::clipboard`] の
//! `root`/`control`/`input`/`trigger` を組み合わせているため、
//! `headless_clipboard` 配線（`crates/wasm-full/src/headless_clipboard.rs`）
//! が `mount`/`hydrate` 時に自動で `navigator.clipboard.writeText` を配線
//! する。無 JS の docs サイト自体では他の全部品と同じく静的表示に留まる
//! （`site/primitives/clipboard.md` が明記する既存の site 全体の制約）が、
//! この block を実アプリへ組み込めば A/B のコピー操作は実際に機能する。
//! 各 `clipboard::root` は `copied: false`（idle）で初期化する。
//!
//! # `<form>` を使わない・完了ボタンは静的表示
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。「完了」
//! ボタン（[`fandhe_frontend_pre_styled_ui::button::button`]）は遷移先・
//! クリック処理を持たない合成例のボタンであり、既定の `type="button"` の
//! まま用いる（`disabled` にはしない。`profile_detail_datalist` の操作
//! ボタンと同型の扱い）。
//!
//! # ダミー値は明白な架空パターン
//!
//! キー値は `fd_demo_` 接頭辞 + 規則的な 16 進風の並びとし、実在サービスの
//! シークレット形式（`sk_live_`/`ghp_`/`AKIA`/`xoxb` 等）を模さない。
//! gitleaks・GitHub push protection の誤検知を避けつつ、値自体が架空である
//! ことを一目で示す（`crates/pre-styled-ui/src/clipboard.rs` モジュール doc
//! 「セキュリティ不変条件」節: コピー対象値は機微情報を含みうるため CSS・
//! ログへは出力しない、という headless 層の既存不変条件を継承するのみで、
//! 本 block 自身はダミー文字列を扱う）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `card::root`/`callout::root`/`clipboard::root`/`input_group::root`/
//! `button::button` はいずれも `drop_class_attr` により呼び出し側 `attrs`
//! の `class` を黙って除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-settings-api-key-created-*` 属性で渡す。素の `div` は
//! `class` がそのまま効くため `.blocks-settings-api-key-created-*` クラス
//! セレクタを使う（`hero_install_command` と同型の判断）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps};
use fandhe_frontend_pre_styled_ui::callout::{self, CalloutProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::visually_hidden;

/// 版 A（単一キー、R0241）で提示するダミーキー値。実在サービスの
/// シークレット形式を避けた明白な架空パターン（モジュール doc「ダミー値は
/// 明白な架空パターン」節参照）。
const KEY_A: &str = "fd_demo_0000-1111-2222-3333";

/// 版 B（複数キー、R0242）で提示するダミーキー値 3 件（用途ラベルと対）。
const KEYS_B: &[(&str, &str)] = &[
    ("本番用", "fd_demo_4444-5555-6666-7777"),
    ("ステージング用", "fd_demo_8888-9999-aaaa-bbbb"),
    ("読み取り専用", "fd_demo_cccc-dddd-eeee-ffff"),
];

/// 「発行完了」見出し + 注意 + コピー欄 + 完了ボタンを束ねるカード骨格。
/// `key_area` は版ごとに異なるキー表示領域（A: 単一 clipboard、B: 複数行）。
fn card_with(key_area: Node) -> Node {
    card::root(
        CardProps::from(CardVariant::Outline),
        vec![],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("API キーを発行しました")]),
                    card::description(
                        vec![],
                        vec![text(
                            "この API キーはアプリケーションが外部サービスへ認証するために使います。",
                        )],
                    ),
                ],
            ),
            card::body(
                vec![("class", "blocks-settings-api-key-created-body")],
                vec![
                    callout::root(
                        &CalloutProps::default(),
                        vec![],
                        vec![callout::text(
                            vec![],
                            vec![text(
                                "このキーは今回のみ表示されます。閉じる前に安全な場所へ保管してください。",
                            )],
                        )],
                    ),
                    key_area,
                ],
            ),
            card::footer(
                vec![("class", "blocks-settings-api-key-created-footer")],
                vec![button(&ButtonProps::default(), vec![], vec![text("完了")])],
            ),
        ],
    )
}

/// A: 代表構成（R0241）。単一キーを `clipboard` で提示する。
fn version_single_key() -> Node {
    let input_id = "blocks-settings-api-key-created-a-input";
    let key_area = clipboard::root(
        KEY_A,
        false,
        vec![
            ("id", "blocks-settings-api-key-created-a"),
            ("data-blocks-settings-api-key-created-clipboard", ""),
        ],
        vec![
            visually_hidden::root(
                vec![],
                vec![clipboard::label(
                    false,
                    Some(input_id),
                    vec![],
                    vec![text("API キー")],
                )],
            ),
            clipboard::control(
                false,
                vec![],
                vec![
                    clipboard::input(KEY_A, false, vec![("id", input_id)]),
                    clipboard::trigger(
                        false,
                        vec![],
                        vec![
                            clipboard::indicator(false, false, vec![], vec![text("コピー")]),
                            clipboard::indicator(true, false, vec![], vec![text("コピーしました")]),
                        ],
                    ),
                ],
            ),
        ],
    );
    card_with(key_area)
}

/// B の 1 行分（`label`・キー値）を組み立てる。行ごとに独立した
/// `clipboard::root` にする理由はモジュール doc 参照。`row_index` は
/// `id` 一意性のための連番（0 始まり）。
fn key_row(row_index: usize, label: &'static str, value: &'static str) -> Node {
    let root_id = format!("blocks-settings-api-key-created-b-{row_index}");
    let input_id = format!("blocks-settings-api-key-created-b-{row_index}-input");
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };
    clipboard::root(
        value,
        false,
        vec![
            ("id", root_id.as_str()),
            ("data-blocks-settings-api-key-created-clipboard", ""),
        ],
        vec![
            clipboard::label(false, Some(input_id.as_str()), vec![], vec![text(label)]),
            clipboard::control(
                false,
                vec![],
                vec![input_group::root(
                    &group_props,
                    vec![],
                    vec![
                        clipboard::input(value, false, vec![("id", input_id.as_str())]),
                        input_group::addon(
                            InputGroupAlign::InlineEnd,
                            &group_props,
                            vec![],
                            vec![clipboard::trigger(
                                false,
                                vec![],
                                vec![
                                    clipboard::indicator(
                                        false,
                                        false,
                                        vec![],
                                        vec![text("コピー")],
                                    ),
                                    clipboard::indicator(
                                        true,
                                        false,
                                        vec![],
                                        vec![text("コピーしました")],
                                    ),
                                ],
                            )],
                        ),
                    ],
                )],
            ),
        ],
    )
}

/// B: 複数キー行表示版（R0242）。3 行の独立した `clipboard` root を
/// 縦に積む。
fn version_multiple_keys() -> Node {
    let rows: Vec<Node> = KEYS_B
        .iter()
        .enumerate()
        .map(|(i, (label, value))| key_row(i, label, value))
        .collect();
    let key_area = div(
        vec![("class", "blocks-settings-api-key-created-rows")],
        rows,
    );
    card_with(key_area)
}

/// `settings-api-key-created` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。版 A・B を縦に並記する。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-api-key-created-stack")],
        vec![version_single_key(), version_multiple_keys()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-api-key-created/",
    title: "settings-api-key-created",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_api_key_created.rs",
    demo_class: "blocks-settings-api-key-created",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Callout",
            path: "/themes/callout/",
        },
        Part {
            label: "Clipboard",
            path: "/themes/clipboard/",
        },
        Part {
            label: "Input Group",
            path: "/themes/input-group/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_api_key_created` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-settings-api-key-created-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-settings-api-key-created-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-api-key-created-rows {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-settings-api-key-created-clipboard] {\n  display: flex;\n  flex-direction: column;\n  width: 100%;\n}\n\
[data-blocks-settings-api-key-created-clipboard] [data-scope=\"clipboard\"][data-part=\"control\"] {\n  width: 100%;\n}\n\
[data-blocks-settings-api-key-created-clipboard] [data-scope=\"input-group\"][data-part=\"root\"] {\n  width: 100%;\n}\n\
.blocks-settings-api-key-created-footer {\n  display: flex;\n  justify-content: flex-end;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"card\"",
            "data-scope=\"callout\"",
            "data-scope=\"clipboard\"",
            "data-scope=\"input-group\"",
            "data-scope=\"button\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        // trigger: A 1 + B 3 = 4、完了ボタン: A + B = 2、合計 6 個の <button>。
        assert_eq!(html.matches("<button").count(), 6);
        assert_eq!(
            html.matches(r#"data-scope="input-group" data-part="root""#)
                .count(),
            3
        );
        assert_eq!(
            html.matches("data-blocks-settings-api-key-created-clipboard=\"\"")
                .count(),
            4
        );
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("type=\"submit\""));
    }

    #[test]
    fn inputs_are_readonly_and_ids_are_unique() {
        let html = demo_html();
        assert_eq!(html.matches(" readonly=\"\"").count(), 4);
        let mut ids = std::collections::HashSet::new();
        let mut rest = html.as_str();
        while let Some(pos) = rest.find("id=\"") {
            rest = &rest[pos + 4..];
            let end = rest.find('"').expect("id attribute must be closed");
            let id = &rest[..end];
            assert!(ids.insert(id.to_string()), "duplicate id: {id}");
            rest = &rest[end..];
        }
    }

    #[test]
    fn layout_css_is_safe() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("--fandhe-space-"));
    }

    #[test]
    fn dummy_keys_are_obviously_fake() {
        let html = demo_html();
        assert!(html.contains("fd_demo_"));
        for forbidden in ["sk_live_", "ghp_", "AKIA", "xoxb"] {
            assert!(
                !html.contains(forbidden),
                "must not resemble real secret: {forbidden}"
            );
        }
    }
}
