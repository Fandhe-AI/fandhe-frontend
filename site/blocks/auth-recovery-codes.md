# auth-recovery-codes

`fandhe-frontend-pre-styled-ui` の `card` / `alert` / `clipboard` / `download_trigger` /
`button` 部品を合成した、2 要素認証の復旧コードを一度だけ表示する画面の合成例です。
注意の `alert`、コード一覧、全コードの一括コピー、ダウンロードの導線を並べます。
Blocks セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください。

本 Demo は静的な表示例であり、`<form>` 要素を持ちません。復旧コードの生成・保存・
検証はアプリケーション側の責務です（`docs/policy/intentional-non-adoption.md` §3.25）。
一括コピーは `clipboard` を 1 個だけ置き、`fandhe-frontend-wasm-full` で hydrate した
実アプリで機能します（無 JS の docs サイトでは静的表示です）。

## Rust コード

```rust
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
```

## ダウンロードの実装方針

この Block の「Download」は装飾です。core の `is_safe_url` は `data:` / `blob:` を
拒否し、その場合 href 属性ごと出力されないため、クライアント側で Blob や data URL を
作る構成は取りません。実アプリでは、アプリ側が `Content-Disposition: attachment` と
`Cache-Control: no-store` を付けた同一オリジンの URL を用意し、`download_trigger::root` の
`href` に渡してください。復旧コードは再表示しないでください。

## 原案差分メモ

- **`clipboard` は 1 個**: `headless_clipboard` の 1 root : 1 状態機械の前提に合わせ、
  全コードを改行連結した値を 1 つの root に渡します。
- **値はすべてダミー**: `demo-NNNN-code` の明白な架空パターンで、実在形式を模しません。
- **ロゴ・商標は持ち込まない**: 認証器やサービスのロゴは使いません。

関連情報: [Card](../themes/card.md) / [Alert](../themes/alert.md) /
[Clipboard](../themes/clipboard.md) / [Download Trigger](../themes/download-trigger.md) /
[Button](../themes/button.md)
