//! `auth-recovery-codes` block（イシュー #3827）。2 要素認証の復旧コードを
//! 登録直後に一度だけ表示し、一括コピーとダウンロードの導線を示す静的な合成例。
//!
//! # 使用部品
//!
//! `card`（骨格）/ `alert`（一度だけ表示の注意）/ `clipboard`（全コード一括
//! コピー）/ `download_trigger`（装飾扱い）/ `button`（完了）の 5 部品を
//! 合成する（`crate::blocks::Block::parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # `clipboard` root は 1 個に限る
//!
//! `headless_clipboard` は「1 root : 1 状態機械」を前提とし、マウントルート
//! 配下の全 `clipboard` パーツへコピー済み表示を及ぼす。このため root は
//! 1 個とし、`value` に全コードを改行連結した文字列を渡す。コード自体は
//! `ul`/`li` で表示するので `clipboard::input` は使わない。無 JS の docs
//! サイトでは押しても何も起きないが、実アプリで hydrate すれば機能する。
//!
//! # ダウンロードは装飾扱い
//!
//! core の `is_safe_url` は `data:`/`blob:` を拒否し href ごと出力しないため、
//! クライアント側で Blob や data URL を作る構成は取らない。実アプリでは
//! `Content-Disposition: attachment` と `Cache-Control: no-store` を付けた
//! 同一オリジン URL をアプリ側が用意して渡す。本 Demo は `href=""` のまま
//! `aria-disabled="true"` と `tabindex="-1"` を付け、CSS でも操作不能を示す
//! （空 href のままだと現在のページ HTML が保存されてしまうため）。
//!
//! # `role="alert"` の扱い
//!
//! `alert::root` は `role="alert"` を固定で出す（表示時に読み上げられる）。
//! 文言は短くし、`clipboard` の外に置く。
//!
//! # `<form>` を使わない・認証処理を持たない
//!
//! 復旧コードの生成・保存・検証はアプリの責務であり、本 block は画面構成
//! だけを示す（`docs/policy/intentional-non-adoption.md` §3.25）。値は
//! すべて明白な架空パターン（`demo-NNNN-code`）で、実在形式を模さない。
//!
//! # CSS フックの選び方
//!
//! 部品は `class` を除去するため `data-blocks-auth-recovery-codes-*` 属性を、
//! 素の `div`/`ul` には `blocks-auth-recovery-codes-*` class を付ける。
use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{code, div, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::alert::{self, AlertProps, AlertStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::download_trigger::{self, DownloadTriggerProps};

/// 表示するダミー復旧コード（明白な架空パターン）。
const CODES: [&str; 8] = [
    "demo-0001-code",
    "demo-0002-code",
    "demo-0003-code",
    "demo-0004-code",
    "demo-0005-code",
    "demo-0006-code",
    "demo-0007-code",
    "demo-0008-code",
];

/// 一括コピー対象（改行連結。`CODES` と同順）。
const ALL_CODES: &str = "demo-0001-code\ndemo-0002-code\ndemo-0003-code\ndemo-0004-code\ndemo-0005-code\ndemo-0006-code\ndemo-0007-code\ndemo-0008-code";

/// `auth-recovery-codes` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let items: Vec<Node> = CODES
        .iter()
        .map(|c| li(vec![], vec![code(vec![], vec![text(*c)])]))
        .collect();

    card::root(
        CardProps::default(),
        vec![("data-blocks-auth-recovery-codes-card", "")],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("Save your recovery codes")]),
                    card::description(
                        vec![],
                        vec![text(
                            "Use one of these codes to sign in if you lose access to your authenticator. Each code works once.",
                        )],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![div(
                    vec![("class", "blocks-auth-recovery-codes-stack")],
                    vec![
                        alert::root(
                            &AlertProps {
                                status: AlertStatus::Warning,
                                ..AlertProps::default()
                            },
                            vec![],
                            vec![alert::content(
                                vec![],
                                vec![
                                    alert::title(vec![], vec![text("These codes are shown only once")]),
                                    alert::description(
                                        vec![],
                                        vec![text("Store them somewhere safe before leaving this page.")],
                                    ),
                                ],
                            )],
                        ),
                        ul(vec![("class", "blocks-auth-recovery-codes-list")], items),
                        div(
                            vec![("class", "blocks-auth-recovery-codes-actions")],
                            vec![
                                clipboard::root(
                                    ALL_CODES,
                                    false,
                                    vec![("data-blocks-auth-recovery-codes-copy", "")],
                                    vec![clipboard::control(
                                        false,
                                        vec![],
                                        vec![clipboard::trigger(
                                            false,
                                            vec![("aria-label", "Copy all recovery codes")],
                                            vec![
                                                clipboard::indicator(false, false, vec![], vec![text("Copy all")]),
                                                clipboard::indicator(true, false, vec![], vec![text("Copied")]),
                                            ],
                                        )],
                                    )],
                                ),
                                download_trigger::root(
                                    &DownloadTriggerProps {
                                        variant: ButtonVariant::Outline,
                                        ..DownloadTriggerProps::default()
                                    },
                                    "",
                                    Some("recovery-codes.txt"),
                                    vec![
                                        ("aria-disabled", "true"),
                                        ("tabindex", "-1"),
                                        ("data-blocks-auth-recovery-codes-download", ""),
                                    ],
                                    vec![text("Download")],
                                ),
                            ],
                        ),
                    ],
                )],
            ),
            card::footer(
                vec![],
                vec![button::button(
                    &ButtonProps::default(),
                    vec![("data-blocks-auth-recovery-codes-done", "")],
                    vec![text("I've saved my codes")],
                )],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/auth-recovery-codes/",
    title: "auth-recovery-codes",
    category: BlockCategory::Auth,
    rust_source: "crates/docs-site/src/blocks/application/auth/auth_recovery_codes.rs",
    demo_class: "blocks-auth-recovery-codes",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Alert",
            path: "/themes/alert/",
        },
        Part {
            label: "Clipboard",
            path: "/themes/clipboard/",
        },
        Part {
            label: "Download Trigger",
            path: "/themes/download-trigger/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `auth_recovery_codes` 固有のレイアウト規則（`crate::blocks` モジュール doc
/// 「CSS の置き場」節。`super::stylesheet` から連結される）。
const LAYOUT_CSS: &str = "\
.blocks-auth-recovery-codes {\n  display: flex;\n  justify-content: center;\n  align-items: center;\n  min-height: 20rem;\n}\n\
[data-blocks-auth-recovery-codes-card] {\n  width: 100%;\n  max-width: 26rem;\n}\n\
.blocks-auth-recovery-codes-stack {\n  display: flex;\n  flex-direction: column;\n  gap: 1rem;\n}\n\
.blocks-auth-recovery-codes-list {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: 0.5rem 1rem;\n  margin: 0;\n  padding: 0;\n  list-style: none;\n  font-family: var(--fandhe-font-font-mono);\n}\n\
.blocks-auth-recovery-codes-actions {\n  display: flex;\n  flex-wrap: wrap;\n  gap: 0.5rem;\n}\n\
[data-blocks-auth-recovery-codes-download] {\n  pointer-events: none;\n  opacity: 0.5;\n}\n\
@media (max-width: 39.99rem) {\n  [data-blocks-auth-recovery-codes-card] {\n    max-width: 100%;\n  }\n  .blocks-auth-recovery-codes-list {\n    grid-template-columns: minmax(0, 1fr);\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    /// 使用部品の anatomy と注意表示が出力されることを固定する。
    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for s in [
            "data-scope=\"card\"",
            "data-scope=\"alert\"",
            "data-scope=\"clipboard\"",
            "data-scope=\"download-trigger\"",
            "data-scope=\"button\"",
            "role=\"alert\"",
        ] {
            assert!(html.contains(s), "demo should contain {s}");
        }
    }

    /// `data:` URL・`<form`・送信系・死んだリンクを出さないことを固定する。
    #[test]
    fn no_data_url_form_or_dead_links() {
        let html = demo_html();
        for f in [
            "data:",
            "blob:",
            "<form",
            "type=\"submit\"",
            "href=\"#\"",
            "<script",
            " name=\"",
        ] {
            assert!(!html.contains(f), "demo must not contain {f}");
        }
    }

    /// 全コードが一括コピー値に含まれ、ダミー値が一貫することを固定する。
    #[test]
    fn all_codes_are_copied_together() {
        let html = demo_html();
        assert_eq!(ALL_CODES.lines().count(), CODES.len());
        for (i, c) in CODES.iter().enumerate() {
            assert_eq!(ALL_CODES.lines().nth(i), Some(*c));
            assert!(html.contains(c));
        }
    }

    /// ダウンロードが装飾扱い（空 href・無効化）であることを固定する。
    #[test]
    fn download_trigger_is_decorative() {
        let html = demo_html();
        assert_eq!(html.matches("download=\"recovery-codes.txt\"").count(), 1);
        assert!(html.contains("href=\"\""));
        assert!(html.contains("aria-disabled=\"true\""));
        assert!(html.contains("tabindex=\"-1\""));
    }

    /// 全ボタンが `type="button"` であることを固定する。
    #[test]
    fn all_buttons_are_type_button() {
        let html = demo_html();
        let buttons = html.matches("<button").count();
        assert!(buttons >= 2);
        assert_eq!(html.matches("type=\"button\"").count(), buttons);
    }

    /// ダミーコードが明白な架空パターンであることを固定する。
    #[test]
    fn dummy_codes_are_obviously_fake() {
        assert_eq!(CODES.len(), 8);
        assert!(CODES.iter().all(|c| c.starts_with("demo-")));
    }

    /// 外部参照を作る CSS 構文を含まないことを固定する。
    #[test]
    fn layout_css_is_safe() {
        for f in ["url(", "@import", "expression", "<"] {
            assert!(!LAYOUT_CSS.contains(f));
        }
    }
}
