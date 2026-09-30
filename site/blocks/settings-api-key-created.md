# settings-api-key-created

API キーを発行した直後に、「一度しか表示しない」注意とともにキー値を提示し
コピーできるカードです。`card` / `callout` / `clipboard` / `input-group` /
`field` / `input` / `button` の 7 部品を合成します。Blocks は既存部品の合成例であり、新しい UI
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
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
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

/// B の 1 行分（`label`・キー値）を組み立てる。`clipboard` root を使わない
/// 理由はモジュール doc「Demo 内の `clipboard` root は 1 個に限る」節参照。
/// `row_index` は `id` 一意性のための連番（0 始まり）。
fn key_row(row_index: usize, label: &'static str, value: &'static str) -> Node {
    let input_id = format!("blocks-settings-api-key-created-b-{row_index}-input");
    let field_props = FieldProps {
        id: input_id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: true,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };
    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &field_props,
        vec![],
        vec![
            field::label(&field_props, vec![], vec![text(label)]),
            input_group::root(
                &group_props,
                vec![("data-blocks-settings-api-key-created-key", "")],
                vec![
                    input::input(&InputProps::default(), &field_props, vec![("value", value)]),
                    input_group::addon(
                        InputGroupAlign::InlineEnd,
                        &group_props,
                        vec![],
                        vec![input_group::button(
                            // `clipboard` scope の外側にあり `headless_clipboard`
                            // 配線が届かないため、押しても何も起きないことを
                            // `disabled: true` で明示する（`hero_install_command`
                            // の版 B と同型の判断）。
                            &InputGroupProps {
                                disabled: true,
                                ..group_props
                            },
                            vec![],
                            vec![text("コピー")],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// B: 複数キー行表示版（R0242）。3 行のキー欄を縦に積む（コピー状態は
/// 持たない）。
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
  読み取り専用）を行ごとに `field` + `input-group` + `readonly` の `input`
  で行表示します（`hero-install-command` の版 B と同じ構成）。キー値は選択
  して手動でコピーできます。
- 版 B の各行のコピーボタンは `clipboard` scope の外側にある
  `input_group::button` で、`disabled` にしています。`headless_clipboard`
  配線は「1 root : 1 状態機械契約」（`Runtime::mount`/`hydrate` に渡された
  マウントルート配下の全 `clipboard` パーツの表示が連動する簡略化）を持つ
  ため、Demo 内に `clipboard` root を複数置くと 1 つのキーをコピーした
  だけで他のキーまで「コピーしました」表示になります。これを避けるため
  `clipboard` root は版 A の 1 個に限っています。実アプリで行ごとにコピー
  させる場合は、版 A の `clipboard` を行ごとに別々のマウントルートへ置き、
  個別に `mount`/`hydrate` してください。
- 「完了」ボタンは遷移先を持たない合成例のボタンです（押下は無効化して
  いません）。

関連情報: [Card](../themes/card.md) / [Callout](../themes/callout.md) /
[Clipboard](../themes/clipboard.md) / [Input Group](../themes/input-group.md) /
[Field](../themes/field.md) / [Input](../themes/input.md) /
[Button](../themes/button.md)
