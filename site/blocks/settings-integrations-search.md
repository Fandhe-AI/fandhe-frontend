# settings-integrations-search

`fandhe-frontend-pre-styled-ui` の `field` / `input-group` / `input` /
`button` / `card` / `link` 部品を合成した、検索付き連携アプリ一覧です。
Blocks セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください（主参照・集約元とも
対応表 ID R0248、1 件の代表構成。出典の固有名・ファイル名は記載しません）。

上部に検索欄とカテゴリ絞り込みボタン群を配置し、下部へ連携アプリの
2 列カードグリッドを並べます。選択中のカテゴリはボタンの塗りつぶし
（`aria-pressed="true"`）で強調します。絞り込み行は `40rem` 未満の狭幅で
折り返さず横スクロールにし、`40rem` 以上で通常の折り返しへ切り替えます。
カードグリッドも `40rem` を境に 1 列 / 2 列を切り替えます。

本 Demo は結果あり（例 A）・該当なし（例 B）の 2 パネルを縦積みで併記した
静的な表示例です。無 JS のため検索・絞り込みは初期状態のまま固定で、実際の
絞り込み処理は行いません。`<form>` 要素は一切持たず、データの取得・送信・
状態管理も行いません。連携アプリ名・説明文はすべて独自に書いた架空のもの
であり、実企業名・実クレデンシャル・PII を含みません。ダミーリンクは
`href="#"` ではなく実在の外部 URL（本リポジトリの GitHub ページ）を指し、
リンク文言も遷移先と一致する「GitHub で見る」を使います。接続ボタンは
全カード共通の可視ラベル（「接続する」/「接続済み」）のため、支援技術が
どのアプリへの操作か区別できるよう `aria-label` へアプリ名を含めます。

## Rust コード

```rust
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
                        vec![text("GitHub で見る")],
                    ),
                    // 接続ボタンの可視ラベルは全カード共通（「接続する」/「接続済み」）
                    // のため、支援技術がどのアプリへの操作か区別できるよう
                    // `aria-label` へアプリ名を埋め込む（settings_integrations_list
                    // と同じ回避、PR #3441/#3443 レビュー指摘対応）。
                    if integration.connected {
                        button::button(
                            &ButtonProps {
                                variant: ButtonVariant::Outline,
                                disabled: true,
                                ..ButtonProps::default()
                            },
                            vec![("aria-label", &format!("{} と接続済み", integration.name))],
                            vec![text("接続済み")],
                        )
                    } else {
                        button::button(
                            &ButtonProps::default(),
                            vec![("aria-label", &format!("{} と接続する", integration.name))],
                            vec![text("接続する")],
                        )
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
```

## 集約元との差分メモ

- 主参照・集約元ともに対応表 ID R0248 の 1 件のみで、複数版を束ねる構成
  ではありません。空状態（例 B）は参照元の複数状態を機械的に踏襲した
  ものではなく、本 Demo が独自に追加した並記です。
- 三点メニュー・並べ替え・ページネーション等、原案に無い操作は追加して
  いません。
- 配色・余白・角丸は既存のテーマトークンに従っています。

関連情報: [Field](../themes/field.md) / [Input Group](../themes/input-group.md) /
[Input](../themes/input.md) / [Button](../themes/button.md) /
[Card](../themes/card.md) / [リンク部品](../themes/link.md)
