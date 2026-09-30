# settings-integrations-list

連携アプリを枠付きの行リストで並べる設定画面向けブロックです。ロゴ・名前・
接続状態バッジ・説明・詳細リンク・接続/解除操作を各行に配置します。
`badge` / `button` / `separator` / `link` / `image` / `switch` / `clipboard` /
`field` / `input-group` / `input` / `empty-state` / `heading` の 12 部品を
合成します。`image` は使用部品一覧に無いロゴ表示のために追加しました
（`page-heading-avatar` と同じ判断）。Blocks は既存部品の合成例であり、
新しい UI 部品は追加しません。

集約元は R0251（要望一覧 + 送信フォーム）/ R0252（個人用・組織用 2 グループの
代表構成）/ R0254（末尾の空状態）/ R0255（スイッチ展開 + API キー欄）の 4 件
です（`_/blocks-intake/` の対応ファイルは本 worktree に存在しないため、対応表
ID のみを記載）。単一 Demo 内に版 A〜D として縦に並記し、差分を読み取れるよう
にしています。アプリ名・説明文はすべて架空であり、実在の企業・製品・商標・
PII を含みません。API キーのデモ値は `fd_demo_` 接頭辞の明白な架空パターンで
実クレデンシャル形式と衝突しません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。詳細リンクは架空
アプリのため `href="#"` ではなく、本リポジトリへの外部リンク
（`rel="noopener noreferrer"` 付き）を使っています。版 B のスイッチは
readonly + disabled で静的固定し、展開/折りたたみの両状態を並記します。

## Rust コード

```rust
use crate::blocks::dummy_assets::LOGO_SRC;
use fandhe_frontend_core::{div, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::empty_state::{self, EmptyStateProps, EmptyStateVariant};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{image, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::switch::{self, SwitchProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 詳細リンクの遷移先（架空アプリのため本リポジトリへの外部リンクで代替、
/// モジュール doc「詳細リンクの遷移先」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 版 B 展開行の API キーデモ値（モジュール doc「API キー値・ダミー素材に
/// ついて」節参照。`fd_demo_` 接頭辞の明白な架空パターンで実クレデンシャル
/// 形式と衝突しない）。
const API_KEY_DEMO: &str = "fd_demo_7f0c1a2e9b3d4f5a8c6e";

/// 連携アプリ 1 件分のデータ（名前・説明・接続状態）。
struct Integration {
    name: &'static str,
    description: &'static str,
    connected: bool,
}

/// グループ 1 件分（見出し + 所属アプリ一覧）。
struct Group {
    title: &'static str,
    items: &'static [Integration],
}

/// 個人用・組織用の 2 グループ（各 3 件、接続済み/未接続を混在させ両分岐を
/// Demo に出す）。版 B〜D も本配列の一部を再利用する（モジュール doc
/// 「版 A〜D と集約元の対応」節参照）。
const GROUPS: &[Group] = &[
    Group {
        title: "個人用",
        items: &[
            Integration {
                name: "Lattice Notes",
                description: "メモとドキュメントを同期するノートアプリ。",
                connected: true,
            },
            Integration {
                name: "Harbor Calendar",
                description: "予定を共有カレンダーへ反映する。",
                connected: false,
            },
            Integration {
                name: "Pulse Alerts",
                description: "重要な通知をモバイルへ転送する。",
                connected: true,
            },
        ],
    },
    Group {
        title: "組織用",
        items: &[
            Integration {
                name: "Ledger Sync",
                description: "経費データを会計システムへ同期する。",
                connected: false,
            },
            Integration {
                name: "Beacon Chat",
                description: "チームチャットへ更新情報を投稿する。",
                connected: true,
            },
            Integration {
                name: "Quarry Storage",
                description: "添付ファイルをクラウドストレージへ保管する。",
                connected: false,
            },
        ],
    },
];

/// 連携アプリのロゴ画像（共通ダミー画像、`data-*` で CSS フックを渡す）。
/// `image` recipe の base（`[data-scope="image"][data-part="root"]`、詳細度
/// (0,2,0)）に `height: auto`/`max-width: 100%` が乗るため、[`LAYOUT_CSS`]
/// 側は `img[data-scope="image"][data-blocks-settings-integrations-list-logo]`
/// （詳細度 (0,2,1)）で上回る（`page_heading_avatar::logo_image` と同じ
/// 判断、イシュー #2931 Bugbot 指摘の再発形）。
fn logo(name: &str) -> Node {
    image(
        &ImageProps {
            shape: ImageShape::Rounded,
            fit: ImageFit::Contain,
            ..ImageProps::new(LOGO_SRC, &format!("{name} のロゴ"))
        },
        vec![("data-blocks-settings-integrations-list-logo", "")],
    )
}

/// 接続状態バッジ（本文の `name_row` に埋め込む）。
fn status_badge(item: &Integration) -> Node {
    if item.connected {
        badge(
            &BadgeProps {
                variant: BadgeVariant::Subtle,
                ..BadgeProps::default()
            },
            vec![("data-blocks-settings-integrations-list-status", "")],
            vec![text("接続済み")],
        )
    } else {
        badge(
            &BadgeProps {
                variant: BadgeVariant::Outline,
                ..BadgeProps::default()
            },
            vec![("data-blocks-settings-integrations-list-status", "")],
            vec![text("未接続")],
        )
    }
}

/// 接続/解除操作ボタン（版 A/C/D の操作領域）。
///
/// 接続/解除ボタンの可視ラベルは全行共通（「接続」/「解除」）のため、
/// 支援技術がどのアプリへの操作か区別できるよう `aria-label` へアプリ名を
/// 埋め込む（PR #3441 Codex 指摘）。
fn action_button(item: &Integration) -> Node {
    if item.connected {
        button(
            &ButtonProps {
                variant: ButtonVariant::Outline,
                ..ButtonProps::default()
            },
            vec![
                ("data-blocks-settings-integrations-list-action", ""),
                ("aria-label", &format!("{} の連携を解除", item.name)),
            ],
            vec![text("解除")],
        )
    } else {
        button(
            &ButtonProps {
                variant: ButtonVariant::Solid,
                ..ButtonProps::default()
            },
            vec![
                ("data-blocks-settings-integrations-list-action", ""),
                ("aria-label", &format!("{} と連携", item.name)),
            ],
            vec![text("接続")],
        )
    }
}

/// 枠付きリストの 1 行（ロゴ・本文（名前 + 状態バッジ + 説明 + 詳細
/// リンク）・操作領域・任意の展開領域）。
///
/// `actions` は操作領域の中身（版 A/C/D は [`action_button`]、版 B は
/// `switch::root`）、`expanded` は版 B の展開行のみが持つ API キー欄
/// （`grid-area: key`）。
fn row_with(item: &Integration, actions: Node, expanded: Option<Node>) -> Node {
    let name_row = div(
        vec![("class", "blocks-settings-integrations-list-name")],
        vec![text(item.name), status_badge(item)],
    );

    let description_row = div(
        vec![("class", "blocks-settings-integrations-list-description")],
        vec![
            text(item.description),
            text(" "),
            link::root(
                REPO,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![("data-blocks-settings-integrations-list-detail", "")],
                vec![text("GitHub で見る")],
            ),
        ],
    );

    let body = div(
        vec![("class", "blocks-settings-integrations-list-body")],
        vec![name_row, description_row],
    );

    let actions_wrap = div(
        vec![("class", "blocks-settings-integrations-list-actions")],
        vec![actions],
    );

    let mut children = vec![logo(item.name), body, actions_wrap];
    let has_expanded = expanded.is_some();
    if let Some(expanded) = expanded {
        children.push(expanded);
    }

    // 展開行（版 B の先頭行）のみ `expanded-row` 修飾クラスを併せて
    // 付与し、CSS 側で key トラックを持つ grid-template-areas を限定
    // 適用する（PR #3447 Codex/Bugbot 指摘: 全行が key トラックを持つと
    // 展開領域のない行にも空段の `gap` が乗り余分な余白が生じるため）。
    let class = if has_expanded {
        "blocks-settings-integrations-list-row blocks-settings-integrations-list-expanded-row"
    } else {
        "blocks-settings-integrations-list-row"
    };

    li(vec![("class", class)], children)
}

/// 版 A/C/D 共通の行（接続/解除ボタン、展開領域なし）。[`row_with`] の薄い
/// ラッパー。
fn row(item: &Integration) -> Node {
    row_with(item, action_button(item), None)
}

/// 1 グループ分（見出し + `ul` 行リスト）。版 A のみが使う（版 B〜D は単一
/// リストのため本関数を経由しない）。グループ見出しは版見出し（H3）の
/// 配下にあるため H4 とし、見出し階層を版見出しと一致させる（#3447 レビュー
/// 指摘対応）。
fn group_section(group: &Group) -> Node {
    div(
        vec![("class", "blocks-settings-integrations-list-group")],
        vec![
            heading(
                HeadingLevel::H4,
                &HeadingProps {
                    size: HeadingSize::Sm,
                    ..HeadingProps::default()
                },
                vec![("data-blocks-settings-integrations-list-group-title", "")],
                vec![text(group.title)],
            ),
            ul(
                vec![("class", "blocks-settings-integrations-list-list")],
                group.items.iter().map(row).collect(),
            ),
        ],
    )
}

/// 版見出し（H3、`heading` 部品。モジュール doc「グループ/版見出しは
/// `heading` を使う」節参照）。
fn version_title(label: &str) -> Node {
    heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Sm,
            ..HeadingProps::default()
        },
        vec![("data-blocks-settings-integrations-list-version-title", "")],
        vec![text(label)],
    )
}

/// 版 B 展開行の API キー欄（`clipboard`、Demo 全体で唯一の
/// `clipboard::root` 呼び出し。モジュール doc「`clipboard` root は版 B の
/// 展開行 1 個に限る」節参照）。
///
/// `clipboard::trigger` に `aria-label` を明示し、可視ラベル「コピー」と
/// 一致させる（既定値 `Copy to clipboard` は英語で可視ラベルと不一致の
/// ため、PR #3447 Codex 指摘）。
fn api_key_clipboard() -> Node {
    const INPUT_ID: &str = "blocks-settings-integrations-list-api-key";
    div(
        vec![("class", "blocks-settings-integrations-list-key")],
        vec![clipboard::root(
            API_KEY_DEMO,
            false,
            vec![("data-blocks-settings-integrations-list-clipboard", "")],
            vec![
                clipboard::label(false, Some(INPUT_ID), vec![], vec![text("API キー")]),
                clipboard::control(
                    false,
                    vec![],
                    vec![
                        clipboard::input(API_KEY_DEMO, false, vec![("id", INPUT_ID)]),
                        clipboard::trigger(
                            false,
                            vec![("aria-label", "API キーをコピー")],
                            vec![
                                clipboard::indicator(false, false, vec![], vec![text("コピー")]),
                                clipboard::indicator(true, false, vec![], vec![text("コピー済み")]),
                            ],
                        ),
                    ],
                ),
            ],
        )],
    )
}

/// 版 B（R0255）: 有効化スイッチで行を展開し API キー欄を表示する版。
///
/// 先頭行のみ `checked: true` で展開し [`api_key_clipboard`] を表示する。
/// 残り 2 行は `checked: false` で折りたたみのまま（展開/折りたたみの両
/// 状態を同一リスト内に並記する）。スイッチは `readonly` + `disabled` で
/// 静的固定する（モジュール doc「スイッチは readonly + disabled で静的
/// 固定」節参照）。
///
/// # 接続済みバッジは `checked` と同じ値を参照する（PR #3447 Codex 指摘）
///
/// `GROUPS[0].items` をそのまま [`row_with`] へ渡すと、`status_badge` が
/// 参照する `item.connected`（版 A/C/D 用の接続状態）と本関数のスイッチ
/// `checked`（i == 0 のみ true）が独立した値になり、3 行目（Pulse
/// Alerts、`connected: true`）のように「接続済み」バッジとオフのスイッチ
/// が同一行に矛盾して並ぶ状態が生じ得た。版 B は元データを複製した
/// `Integration` を作り `connected` フィールドへ `checked` を代入するこ
/// とで、バッジとスイッチが必ず同じ状態を指すようにする（`GROUPS` 自体
/// は版 A/C/D が引き続き参照するため変更しない）。
fn version_switch_keys() -> Node {
    let items = GROUPS[0].items;
    let switch_props = SwitchProps {
        readonly: true,
        disabled: true,
        ..SwitchProps::default()
    };
    let rows: Vec<Node> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let checked = i == 0;
            // バッジ（status_badge）とスイッチ（checked）を同じ状態源に
            // 揃えるための表示用コピー。GROUPS[0].items の connected は
            // 版 A の接続/解除ボタン用の意味づけであり、版 B の「有効化」
            // とは独立の状態のため、ここでのみ checked へ上書きする。
            let display_item = Integration {
                name: item.name,
                description: item.description,
                connected: checked,
            };
            let hidden_input_name = format!("blocks-settings-integrations-list-enabled-{i}");
            let switch_node = switch::root(
                Size::Md,
                ColorPalette::Accent,
                checked,
                &switch_props,
                vec![("data-blocks-settings-integrations-list-switch", "")],
                vec![
                    switch::label(
                        checked,
                        &switch_props,
                        vec![],
                        vec![text(format!("{} を有効化", item.name))],
                    ),
                    switch::hidden_input(&hidden_input_name, "on", checked, &switch_props, vec![]),
                    switch::control(
                        checked,
                        &switch_props,
                        vec![],
                        vec![switch::thumb(checked, &switch_props, vec![], vec![])],
                    ),
                ],
            );
            let expanded = checked.then(api_key_clipboard);
            row_with(&display_item, switch_node, expanded)
        })
        .collect();
    ul(
        vec![("class", "blocks-settings-integrations-list-list")],
        rows,
    )
}

/// 要望済みアプリ 2 件（名前 + 検討状況バッジ）。
const REQUESTED_APPS: &[(&str, &str, BadgeVariant)] = &[
    ("Flow Board", "検討中", BadgeVariant::Subtle),
    ("Signal Mail", "予定", BadgeVariant::Outline),
];

/// 要望済みアプリの小リスト（版 C 要望領域の一部）。
fn requested_apps_list() -> Node {
    ul(
        vec![("class", "blocks-settings-integrations-list-requested-list")],
        REQUESTED_APPS
            .iter()
            .map(|(name, status, variant)| {
                li(
                    vec![],
                    vec![
                        text(*name),
                        badge(
                            &BadgeProps {
                                variant: *variant,
                                ..BadgeProps::default()
                            },
                            vec![(
                                "data-blocks-settings-integrations-list-requested-status",
                                "",
                            )],
                            vec![text(*status)],
                        ),
                    ],
                )
            })
            .collect(),
    )
}

/// 版 C（R0251）: 連携要望の一覧 + 要望送信フォームを末尾に持つ版。
///
/// `<form>` は使わず送信先も持たない静的表示（モジュール doc「要望フォーム
/// は `<form>` を持たず送信先も持たない」節参照）。
fn version_request_form() -> Node {
    const FIELD_ID: &str = "blocks-settings-integrations-list-request-app";
    let field_props = FieldProps {
        id: FIELD_ID,
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

    let request_area = li(
        vec![("class", "blocks-settings-integrations-list-request")],
        vec![
            requested_apps_list(),
            field::root(
                &FieldRootProps {
                    orientation: FieldOrientation::Vertical,
                },
                &field_props,
                vec![],
                vec![
                    field::label(&field_props, vec![], vec![text("連携してほしいアプリ")]),
                    input_group::root(
                        &group_props,
                        vec![],
                        vec![
                            input::input(
                                &InputProps::default(),
                                &field_props,
                                vec![("type", "text"), ("placeholder", "例: Slack, Notion")],
                            ),
                            input_group::addon(
                                InputGroupAlign::InlineEnd,
                                &group_props,
                                vec![],
                                vec![input_group::button(
                                    &group_props,
                                    vec![],
                                    vec![text("要望を送る")],
                                )],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    );

    let mut rows: Vec<Node> = GROUPS[1].items[0..2].iter().map(row).collect();
    rows.push(request_area);
    ul(
        vec![("class", "blocks-settings-integrations-list-list")],
        rows,
    )
}

/// 版 D（R0254）: 末尾に空状態の枠を持つ版。
fn version_empty_state() -> Node {
    let empty_area = li(
        vec![("class", "blocks-settings-integrations-list-empty")],
        vec![empty_state::root(
            &EmptyStateProps {
                variant: EmptyStateVariant::Outline,
                ..EmptyStateProps::default()
            },
            vec![],
            vec![empty_state::content(
                vec![],
                vec![
                    empty_state::title(vec![], vec![text("ほかの連携アプリを探す")]),
                    empty_state::description(
                        vec![],
                        vec![text("カタログから追加の連携アプリを探して接続できます。")],
                    ),
                    empty_state::actions(
                        vec![],
                        vec![button(
                            &ButtonProps {
                                variant: ButtonVariant::Outline,
                                ..ButtonProps::default()
                            },
                            vec![],
                            vec![text("アプリを追加")],
                        )],
                    ),
                ],
            )],
        )],
    );

    let mut rows: Vec<Node> = GROUPS[1].items[1..3].iter().map(row).collect();
    rows.push(empty_area);
    ul(
        vec![("class", "blocks-settings-integrations-list-list")],
        rows,
    )
}

/// `settings-integrations-list` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数。版 A〜D を縦に並記する（モジュール doc「版 A〜D と集約元の
/// 対応」節参照）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-integrations-list-stack")],
        vec![
            version_title("版 A: 個人用・組織用グループ（R0252）"),
            group_section(&GROUPS[0]),
            separator(
                &SeparatorProps::default(),
                vec![("data-blocks-settings-integrations-list-divider", "")],
            ),
            group_section(&GROUPS[1]),
            separator(
                &SeparatorProps::default(),
                vec![("data-blocks-settings-integrations-list-divider", "")],
            ),
            version_title("版 B: 有効化スイッチと API キー欄（R0255）"),
            version_switch_keys(),
            separator(
                &SeparatorProps::default(),
                vec![("data-blocks-settings-integrations-list-divider", "")],
            ),
            version_title("版 C: 連携要望フォーム付き（R0251）"),
            version_request_form(),
            separator(
                &SeparatorProps::default(),
                vec![("data-blocks-settings-integrations-list-divider", "")],
            ),
            version_title("版 D: 空状態付き（R0254）"),
            version_empty_state(),
        ],
    )
}
```

## 原案差分メモ

- **版 A（R0252）**: 個人用・組織用の 2 グループへ分け、見出し + 区切り線
  （`separator`）で分離。接続状態を状態バッジ（接続済み: Subtle / 未接続:
  Outline）と接続/解除ボタンの variant 切り替えで表現し、両分岐を Demo に出す
- **版 B（R0255）**: 単一グループ 3 行。操作領域を接続ボタンではなく `switch`
  （有効化トグル）にし、先頭行のみ展開して `clipboard`（API キー欄）を表示
  する。残り 2 行は折りたたみのまま同一リスト内に並記する
- **版 C（R0251）**: 単一グループ 2 行の末尾に「連携の要望」領域（要望済み
  一覧 + `field`/`input-group`/`input` による要望送信欄）。`<form>` は使わず
  送信処理・送信先も持たない静的表示
- **版 D（R0254）**: 単一グループ 2 行の末尾に `empty-state`（末尾の空状態枠）

静的固定の理由:

- **スイッチが disabled**: `readonly` は `data-readonly` を出すのみで native
  トグル操作自体を止めないため、`disabled: true` も併用して実際に操作を止める
  （`pricing-seats-split` と同じ判断）
- **`clipboard` root は 1 個のみ**: headless clipboard の「1 root : 1 状態機械
  契約」に従い、Demo 全体で版 B の展開行 1 箇所のみが `clipboard::root` を呼ぶ
- **要望フォームが `<form>` を持たない**: `crate::blocks` モジュール doc の
  不変条件どおり、送信処理・送信先を持たない静的表示に限る

コンテナ幅が 40rem 未満のとき、操作領域を説明の下（2 行目）へ、版 B の展開
行（API キー欄）をさらにその下（3 行目）へ回します。

関連情報: [Badge](../themes/badge.md) / [Button](../themes/button.md) /
[Separator](../themes/separator.md) / [Link](../themes/link.md) /
[Image](../themes/image.md) / [Switch](../themes/switch.md) /
[Clipboard](../themes/clipboard.md) / [Field](../themes/field.md) /
[Input Group](../themes/input-group.md) / [Input](../themes/input.md) /
[Empty State](../themes/empty-state.md) / [Heading](../themes/heading.md)
