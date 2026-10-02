# docs-layout-page-header

`fandhe-frontend-pre-styled-ui` の `breadcrumb` / `heading` / `text` /
`button` / `clipboard` / `badge` / `code` 部品を合成した、ドキュメント
ページ上部のヘッダーの実例です。Blocks セクションは新規部品を追加する
ものではなく、既存の Themes/Primitives 部品を組み合わせた実例集である
ことに注意してください（主参照は対応表 ID R0080、集約元は R0091・
R0079。出典の固有名・ファイル名は記載しません）。

上段にパンくず（または小見出し）、見出し行（右側に編集・ソース表示
などの操作ボタン）、説明文を縦に並べた代表構成（例 A）に加え、小見出し
と「ページをコピー」操作を組み合わせた形（例 B）、見出しの下に HTTP
メソッドバッジとエンドポイント文字列の行を置いた API ページ版（例 C）
の 3 通りを並べています。狭い幅（`40rem` 未満）では操作ボタンが見出し
の下へ回ります。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・コピーを行いません。ボタンは `type="button"` のまま送信先を持たず、
クリップボードは未コピー（idle）状態の固定表示です（実際のコピー動作
には `fandhe-frontend-wasm-full` の JS 配線が必要で、docs サイトは JS
ハイドレーションを行いません）。文言・エンドポイントはすべて独自に
書いた架空のものであり、実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// パンくず（「Blocks」→ 中間項目 → 現在ページ）。
fn breadcrumb_row(middle_label: &'static str, current_label: &'static str) -> Node {
    breadcrumb::root(
        Size::Md,
        BreadcrumbVariant::default(),
        Some("パンくずリスト"),
        vec![],
        vec![breadcrumb::list(
            vec![],
            vec![
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::link("../", vec![], vec![text("Blocks")])],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::link("../", vec![], vec![text(middle_label)])],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::current_link(vec![], vec![text(current_label)])],
                ),
            ],
        )],
    )
}

/// A: 代表構成（R0080）。パンくず → 見出し行（編集・ソース操作）→ 説明文。
fn instance_a() -> Node {
    let top_row = div(
        vec![("data-blocks-docs-layout-page-header-top-row", "")],
        vec![breadcrumb_row("ガイド", "埋め込みガイド")],
    );
    let header = div(
        vec![("data-blocks-docs-layout-page-header-header", "")],
        vec![
            div(
                vec![("data-blocks-docs-layout-page-header-title-group", "")],
                vec![heading(
                    HeadingLevel::H2,
                    &HeadingProps {
                        size: HeadingSize::Xl2,
                        ..HeadingProps::default()
                    },
                    vec![],
                    vec![text("埋め込みガイド")],
                )],
            ),
            div(
                vec![("data-blocks-docs-layout-page-header-actions", "")],
                vec![
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("編集する")],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("ソースを表示")],
                    ),
                ],
            ),
        ],
    );
    let description = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(
            "最小構成の HTML への部分埋め込み手順をまとめた案内ページです。",
        )],
    );
    div(
        vec![
            ("data-blocks-docs-layout-page-header-instance", ""),
            ("data-blocks-docs-layout-page-header-variant", "a"),
        ],
        vec![top_row, header, description],
    )
}

/// B: 小見出し + コピー（R0091）。小見出し → 見出し行（ページをコピー）→
/// 説明文。
fn instance_b() -> Node {
    const COPY_VALUE: &str = "/guides/npm-asset-build";
    const COPY_INPUT_ID: &str = "blocks-docs-layout-page-header-copy-b";
    let top_row = div(
        vec![("data-blocks-docs-layout-page-header-top-row", "")],
        vec![span(
            vec![("data-blocks-docs-layout-page-header-eyebrow", "")],
            vec![text("ガイド")],
        )],
    );
    let copy = clipboard::root(
        COPY_VALUE,
        false,
        vec![("data-blocks-docs-layout-page-header-copy", "")],
        vec![
            clipboard::label(
                false,
                Some(COPY_INPUT_ID),
                vec![],
                vec![text("ページをコピー")],
            ),
            clipboard::control(
                false,
                vec![],
                vec![
                    clipboard::input(COPY_VALUE, false, vec![("id", COPY_INPUT_ID)]),
                    clipboard::trigger(
                        false,
                        vec![],
                        vec![
                            clipboard::indicator(false, false, vec![], vec![text("コピー")]),
                            clipboard::indicator(true, false, vec![], vec![text("コピー済み")]),
                        ],
                    ),
                ],
            ),
        ],
    );
    let header = div(
        vec![("data-blocks-docs-layout-page-header-header", "")],
        vec![
            div(
                vec![("data-blocks-docs-layout-page-header-title-group", "")],
                vec![heading(
                    HeadingLevel::H2,
                    &HeadingProps {
                        size: HeadingSize::Xl2,
                        ..HeadingProps::default()
                    },
                    vec![],
                    vec![text("NPM アセットビルド")],
                )],
            ),
            div(
                vec![("data-blocks-docs-layout-page-header-actions", "")],
                vec![copy],
            ),
        ],
    );
    let description = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(
            "NPM 互換の静的アセットビルドを `--ignore-scripts` 前提で組み込む手順です。",
        )],
    );
    div(
        vec![
            ("data-blocks-docs-layout-page-header-instance", ""),
            ("data-blocks-docs-layout-page-header-variant", "b"),
        ],
        vec![top_row, header, description],
    )
}

/// C: API ページ版（R0079）。パンくず → 見出し行（ソース表示）→
/// エンドポイント行（メソッドバッジ + コピー可能なパス）→ 説明文。
fn instance_c() -> Node {
    const ENDPOINT: &str = "/v1/projects/{id}";
    const ENDPOINT_INPUT_ID: &str = "blocks-docs-layout-page-header-endpoint-c";
    let top_row = div(
        vec![("data-blocks-docs-layout-page-header-top-row", "")],
        vec![breadcrumb_row("API Reference", "プロジェクト取得")],
    );
    let header = div(
        vec![("data-blocks-docs-layout-page-header-header", "")],
        vec![
            div(
                vec![("data-blocks-docs-layout-page-header-title-group", "")],
                vec![heading(
                    HeadingLevel::H2,
                    &HeadingProps {
                        size: HeadingSize::Xl2,
                        ..HeadingProps::default()
                    },
                    vec![],
                    vec![text("プロジェクトを取得する")],
                )],
            ),
            div(
                vec![("data-blocks-docs-layout-page-header-actions", "")],
                vec![button::button(
                    &ButtonProps {
                        variant: ButtonVariant::Outline,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text("ソースを表示")],
                )],
            ),
        ],
    );
    let endpoint_row = div(
        vec![("data-blocks-docs-layout-page-header-endpoint", "")],
        vec![
            badge::badge(
                &BadgeProps {
                    palette: ColorPalette::Success,
                    ..BadgeProps::default()
                },
                vec![],
                vec![text("GET")],
            ),
            clipboard::root(
                ENDPOINT,
                false,
                vec![],
                vec![clipboard::control(
                    false,
                    vec![],
                    vec![
                        clipboard::value_text(
                            vec![],
                            vec![code::code(
                                &CodeProps::default(),
                                vec![("id", ENDPOINT_INPUT_ID)],
                                vec![text(ENDPOINT)],
                            )],
                        ),
                        clipboard::trigger(
                            false,
                            vec![],
                            vec![
                                clipboard::indicator(false, false, vec![], vec![text("コピー")]),
                                clipboard::indicator(true, false, vec![], vec![text("コピー済み")]),
                            ],
                        ),
                    ],
                )],
            ),
        ],
    );
    let description = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text("指定した ID のプロジェクトを取得します。")],
    );
    div(
        vec![
            ("data-blocks-docs-layout-page-header-instance", ""),
            ("data-blocks-docs-layout-page-header-variant", "c"),
        ],
        vec![top_row, header, endpoint_row, description],
    )
}

/// `docs-layout-page-header` の Demo 本体（3 インスタンスを縦積みで並記
/// する。呼び出しごとに同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-docs-layout-page-header-layout")],
        vec![instance_a(), instance_b(), instance_c()],
    )
}
```

## 原案差分メモ

- 例 A（代表構成）は主参照（対応表 ID R0080）を軸に、パンくず・見出し・
  編集/ソース表示の操作ボタン・説明文という組み合わせを表します。
- 例 B（小見出し + コピー）は R0091 に対応し、パンくずの代わりに小さな
  小見出し（eyebrow）を置き、操作列を `clipboard`（ページをコピー）へ
  差し替えた形の差分を表します。
- 例 C（API ページ版）は R0079 に対応し、見出しの下に `badge`（HTTP
  メソッド）と `clipboard` + `code`（エンドポイントのコピー可能な表示）
  の行を追加した形の差分を表します。
- 操作ボタンが見出しの下へ回る切り替え（`40rem` 未満）は無 JS のため
  CSS のみで表現しています。実際のブラウザでの表示切り替え・クリップ
  ボードのコピー動作確認は本 Demo では行っていません。
- 文言・エンドポイントは既存のテーマトークンに従い、独自に書いた架空
  のものです。

関連情報: [Breadcrumb](../themes/breadcrumb.md) / [Heading](../themes/heading.md) /
[Text](../themes/text.md) / [Button](../themes/button.md) /
[Clipboard](../themes/clipboard.md) / [Badge](../themes/badge.md) /
[Code](../themes/code.md)
