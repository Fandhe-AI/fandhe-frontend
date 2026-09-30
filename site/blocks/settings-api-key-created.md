# settings-api-key-created

API キーを発行した直後に、「一度しか表示しない」注意とともにキー値を提示し
コピーできるカードです。`card` / `callout` / `clipboard` / `input-group` /
`button` の 5 部品を合成します。Blocks は既存部品の合成例であり、新しい UI
部品は追加しません。

主参照は対応表 ID R0241（代表構成: 単一キー）で、R0242（複数キー行表示）を
集約しています。キー値はすべて `fd_demo_` 接頭辞の架空データであり、実在
サービスの秘密情報ではありません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。

## Rust コード

```rust
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

/// B の 1 行分（`label`・キー値）を組み立てる。行ごとに `id` を分ける
/// 理由（実アプリで独立させるには行ごとに別々の `mount`/`hydrate` が
/// 必要であること）はモジュール doc「行ごとの `clipboard` root と実
/// アプリでの独立方法」節参照。`row_index` は `id` 一意性のための連番
/// （0 始まり）。
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
```

## 原案差分メモ

- **版 A（代表構成、R0241）**: 単一キーを `clipboard` 1 個で提示します。
- **版 B（複数キー行表示、R0242）**: 3 個のキー（本番用・ステージング用・
  読み取り専用）をそれぞれ別々の `clipboard` root（行ごとに一意な `id`）で
  行表示します。`headless_clipboard` 配線は「1 root : 1 状態機械契約」
  （`Runtime::mount`/`hydrate` に渡されたマウントルート配下の全
  `clipboard` パーツの表示が連動する簡略化）を持つため、この Demo 全体を
  1 回でマウントする限り行ごとの表示は連動します。行ごとに `id` を分けて
  あるのは、実アプリへ組み込む際にこの `id` を持つ要素それぞれへ個別に
  `mount`/`hydrate` を呼ぶことで初めて独立させられるようにするためです。
- 版 B は `input-group` でキー入力欄とコピーボタンを 1 行に整列しますが、
  ボタン自体は `input_group::button` ではなく `clipboard` scope 内の
  `clipboard::trigger` を使います。これにより実アプリへ組み込んだ際、
  ボタン押下で実際にコピー機能が働きます。
- 「完了」ボタンは遷移先を持たない合成例のボタンです（押下は無効化して
  いません）。

関連情報: [Card](../themes/card.md) / [Callout](../themes/callout.md) /
[Clipboard](../themes/clipboard.md) / [Input Group](../themes/input-group.md) /
[Button](../themes/button.md)
